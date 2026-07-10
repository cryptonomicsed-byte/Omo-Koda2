defmodule OmokodaSwarm.Memory.RemCycle do
  @moduledoc """
  Hive-scale Sabbath REM cycle — the Elixir orchestrator of the dream state.

  Mirrors the per-agent REM cycle in omokoda-core's DreamEngine at swarm
  scale: once per Sabbath (UTC Saturday), this GenServer collects node
  summaries from every active DailyServer, streams them to the Julia
  Memory service (`POST /dream/rem`, see omokoda-memory/src/rem_fractal.jl),
  and holds the returned fractal compression plan for the memory owners to
  apply. The plan is **advisory** — this process never mutates agent memory
  (specs/dream-rem.md: "only the memory owner applies them").

  Rhythm alignment: the Sabbath here is UTC Saturday, the same day
  `RhythmGate::is_sabbath()` observes in the Rust core — outward action
  queues, the hive dreams.

  Ticks hourly; runs at most once per Sabbath date. `run_now/0` forces a
  cycle for testing/operations; `last_plan/0` returns the latest plan.
  """

  use GenServer
  require Logger

  alias OmokodaSwarm.AuguryClient

  @tick_interval :timer.hours(1)
  # Elixir's Date.day_of_week: Monday = 1 … Sunday = 7; Saturday = 6.
  @sabbath_day_of_week 6

  defstruct last_run_date: nil, last_plan: nil, runs: 0

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc "Force a REM cycle now, regardless of weekday. Returns {:ok, plan} | {:error, reason}."
  def run_now, do: GenServer.call(__MODULE__, :run_now, 60_000)

  @doc "The most recent compression plan, or nil."
  def last_plan, do: GenServer.call(__MODULE__, :last_plan)

  # ---------------------------------------------------------------------------
  # Pure helpers (unit-tested without processes)
  # ---------------------------------------------------------------------------

  @doc "True if `date` is the Sabbath (UTC Saturday)."
  def sabbath?(%Date{} = date), do: Date.day_of_week(date) == @sabbath_day_of_week

  @doc """
  True when a REM cycle is due: it is the Sabbath and none has run on this
  date yet.
  """
  def due?(%Date{} = today, last_run_date) do
    sabbath?(today) and last_run_date != today
  end

  @doc """
  Map a DailyServer state into /dream/rem node summaries.

  Notes are noise-tier (importance 0.2); decisions carry their recorded
  importance normalised from the 1–5 scale into [0.2, 1.0]. Paths group by
  agent and date so folds stay within one agent's day — no cross-agent folds.
  """
  def daily_state_to_nodes(%{agent_id: agent_id, date: date} = state) do
    created_at =
      case Map.get(state, :started_at) do
        %DateTime{} = dt -> DateTime.to_unix(dt)
        _ -> 0
      end

    path = "daily/#{agent_id}/#{date}"

    notes =
      state
      |> Map.get(:notes, [])
      |> Enum.with_index()
      |> Enum.map(fn {_note, idx} ->
        %{
          id: "#{path}/note-#{idx}",
          path: path,
          importance: 0.2,
          created_at: created_at
        }
      end)

    decisions =
      state
      |> Map.get(:decisions, [])
      |> Enum.with_index()
      |> Enum.map(fn {decision, idx} ->
        raw = Map.get(decision, :importance, 3)

        %{
          id: "#{path}/decision-#{idx}",
          path: path,
          importance: raw / 5.0,
          created_at: created_at
        }
      end)

    notes ++ decisions
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init([]) do
    Process.send_after(self(), :tick, @tick_interval)
    {:ok, %__MODULE__{}}
  end

  @impl true
  def handle_info(:tick, state) do
    Process.send_after(self(), :tick, @tick_interval)

    if due?(Date.utc_today(), state.last_run_date) do
      {:noreply, run_cycle(state)}
    else
      {:noreply, state}
    end
  end

  @impl true
  def handle_call(:run_now, _from, state) do
    new_state = run_cycle(state)

    case new_state.last_plan do
      nil -> {:reply, {:error, :no_plan}, new_state}
      plan -> {:reply, {:ok, plan}, new_state}
    end
  end

  @impl true
  def handle_call(:last_plan, _from, state) do
    {:reply, state.last_plan, state}
  end

  # ---------------------------------------------------------------------------
  # Internal
  # ---------------------------------------------------------------------------

  defp run_cycle(state) do
    nodes = collect_nodes()

    case AuguryClient.dream_rem(nodes) do
      {:ok, plan} ->
        Logger.info(
          "[REM] Sabbath cycle complete: #{length(nodes)} nodes analysed, " <>
            "fractal_dimension=#{inspect(plan["fractal_dimension"])}, " <>
            "#{length(plan["folds"] || [])} folds, " <>
            "#{length(plan["prune_ids"] || [])} prunes"
        )

        %{state | last_run_date: Date.utc_today(), last_plan: plan, runs: state.runs + 1}

      {:error, reason} ->
        # Fail-open: the Julia service being down must not crash the swarm.
        # last_run_date stays unset so the next tick retries this Sabbath.
        Logger.warning("[REM] cycle skipped — Julia memory service unavailable: #{inspect(reason)}")
        state
    end
  end

  defp collect_nodes do
    OmokodaSwarm.Memory.Supervisor
    |> DynamicSupervisor.which_children()
    |> Enum.flat_map(fn
      {_, pid, :worker, _} when is_pid(pid) ->
        try do
          pid
          |> GenServer.call(:get_state, 5_000)
          |> Map.from_struct()
          |> daily_state_to_nodes()
        catch
          :exit, _ -> []
        end

      _ ->
        []
    end)
  end
end
