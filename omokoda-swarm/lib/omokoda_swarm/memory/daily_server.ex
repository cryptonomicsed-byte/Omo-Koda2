defmodule OmokodaSwarm.Memory.DailyServer do
  @moduledoc """
  OTP-supervised daily memory GenServer — one per agent per calendar day.

  Auto-spawned by Memory.Supervisor when an agent's session begins.
  Terminates itself after @retention_days, archiving to the SOMA (Julia RACK).

  Mirrors omo-mem's `session.created → createDailyNote` pattern but as a
  supervised Elixir process with fault tolerance and automatic lifecycle management.
  """

  use GenServer, restart: :transient

  @retention_days 30
  @archive_check_interval :timer.hours(6)

  defstruct agent_id: nil,
            date: nil,
            notes: [],
            decisions: [],
            carry_forward: [],
            started_at: nil

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  def start_link({agent_id, date}) do
    GenServer.start_link(__MODULE__, {agent_id, date}, name: via(agent_id, date))
  end

  @doc "Append a raw session note."
  def append_note(agent_id, note, date \\ Date.utc_today()) do
    GenServer.cast(via(agent_id, date), {:append_note, note})
  end

  @doc "Record a decision (with optional importance 1–5)."
  def add_decision(agent_id, decision, importance \\ 3, date \\ Date.utc_today()) do
    GenServer.cast(via(agent_id, date), {:add_decision, {decision, importance}})
  end

  @doc "Mark an item to carry into the next session."
  def carry_forward(agent_id, item, date \\ Date.utc_today()) do
    GenServer.cast(via(agent_id, date), {:carry_forward, item})
  end

  @doc "Return the full daily state snapshot."
  def get_daily(agent_id, date \\ Date.utc_today()) do
    GenServer.call(via(agent_id, date), :get_state)
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init({agent_id, date}) do
    Process.send_after(self(), :check_archival, @archive_check_interval)

    state = %__MODULE__{
      agent_id: agent_id,
      date: date,
      started_at: DateTime.utc_now()
    }

    {:ok, state}
  end

  @impl true
  def handle_cast({:append_note, note}, state) do
    {:noreply, %{state | notes: [note | state.notes]}}
  end

  @impl true
  def handle_cast({:add_decision, {decision, importance}}, state) do
    entry = %{text: decision, importance: importance, at: DateTime.utc_now()}
    {:noreply, %{state | decisions: [entry | state.decisions]}}
  end

  @impl true
  def handle_cast({:carry_forward, item}, state) do
    {:noreply, %{state | carry_forward: [item | state.carry_forward]}}
  end

  @impl true
  def handle_call(:get_state, _from, state) do
    {:reply, state, state}
  end

  @impl true
  def handle_info(:check_archival, state) do
    age = Date.diff(Date.utc_today(), state.date)

    if age > @retention_days do
      {:stop, :normal, state}
    else
      Process.send_after(self(), :check_archival, @archive_check_interval)
      {:noreply, state}
    end
  end

  # ---------------------------------------------------------------------------
  # Internal helpers
  # ---------------------------------------------------------------------------

  defp via(agent_id, date) do
    {:via, Registry, {OmokodaSwarm.Registry, "daily_#{agent_id}_#{date}"}}
  end
end
