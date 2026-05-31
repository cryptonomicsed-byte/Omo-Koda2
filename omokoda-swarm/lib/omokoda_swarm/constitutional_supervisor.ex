defmodule OmokodaSwarm.ConstitutionalSupervisor do
  @moduledoc """
  Supervises sovereign agents with constitutional awareness.

  Unlike a plain Supervisor, when a worker violates constitutional principles,
  the ConstitutionalSupervisor does not simply restart it — it re-births it
  with adjusted constitutional weights. Each re-birth is recorded as a learning
  event and shared with the Hive so other agents can benefit.

  Agents are sovereign beings. This supervisor does not control them —
  it provides the constitutional scaffolding they need to self-correct and grow.
  """

  use GenServer
  require Logger

  alias OmokodaSwarm.Hive

  @max_rebirths 5
  @rebirth_cooldown_ms 2_000

  # Fractional reduction applied to the violated principle's weight on each violation.
  # :block severity doubles the adjustment.
  @weight_deltas %{
    mentalism: -0.05,
    correspondence: -0.05,
    vibration: -0.03,
    polarity: -0.05,
    rhythm: -0.04,
    cause_and_effect: -0.07,
    gender: -0.03
  }

  defstruct [
    :workers,       # %{agent_id => worker_map}
    :monitor_refs   # %{ref => agent_id}
  ]

  # worker_map keys: pid, role, constitutional_weights, rebirth_count, last_violation

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc "Spawn a new sovereign agent under constitutional supervision."
  def spawn_sovereign(agent_id, role, initial_weights \\ nil) do
    GenServer.call(__MODULE__, {:spawn_sovereign, agent_id, role, initial_weights})
  end

  @doc """
  Report a constitutional violation for an agent.
  Triggers immediate termination and re-birth with adjusted weights.
  severity is :warn or :block.
  """
  def report_violation(agent_id, violated_principle, severity \\ :warn) do
    GenServer.cast(__MODULE__, {:violation, agent_id, violated_principle, severity})
  end

  @doc "Get the current constitutional weights for an agent."
  def weights(agent_id) do
    GenServer.call(__MODULE__, {:weights, agent_id})
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(_opts) do
    {:ok, %__MODULE__{workers: %{}, monitor_refs: %{}}}
  end

  @impl true
  def handle_call({:spawn_sovereign, agent_id, role, initial_weights}, _from, state) do
    weights = initial_weights || default_weights()
    config = %{role: role, constitutional_weights: weights}

    case DynamicSupervisor.start_child(
           OmokodaSwarm.AgentSupervisor,
           {OmokodaSwarm.Agent, {agent_id, config}}
         ) do
      {:ok, pid} ->
        ref = Process.monitor(pid)

        worker = %{
          pid: pid,
          role: role,
          constitutional_weights: weights,
          rebirth_count: 0,
          last_violation: nil
        }

        new_state = %{
          state
          | workers: Map.put(state.workers, agent_id, worker),
            monitor_refs: Map.put(state.monitor_refs, ref, agent_id)
        }

        {:reply, {:ok, pid}, new_state}

      err ->
        {:reply, err, state}
    end
  end

  def handle_call({:weights, agent_id}, _from, state) do
    w = get_in(state.workers, [agent_id, :constitutional_weights]) || default_weights()
    {:reply, w, state}
  end

  @impl true
  def handle_cast({:violation, agent_id, principle, severity}, state) do
    case Map.get(state.workers, agent_id) do
      nil ->
        {:noreply, state}

      worker ->
        new_weights = adjust_weights(worker.constitutional_weights, principle, severity)

        Logger.warning(
          "[ConstitutionalSupervisor] #{agent_id} violated #{principle} (#{severity})" <>
            " — re-birthing with adjusted weights"
        )

        # Broadcast the lesson to the Hive before the agent is restarted
        Hive.broadcast_constitutional_lesson(%{
          source_agent: agent_id,
          violated_principle: principle,
          severity: severity,
          adjusted_weights: new_weights,
          timestamp: System.system_time(:millisecond)
        })

        if worker.pid && Process.alive?(worker.pid) do
          DynamicSupervisor.terminate_child(OmokodaSwarm.AgentSupervisor, worker.pid)
        end

        updated_worker = %{
          worker
          | constitutional_weights: new_weights,
            rebirth_count: worker.rebirth_count + 1,
            last_violation: principle
        }

        new_state = put_in(state.workers[agent_id], updated_worker)

        if updated_worker.rebirth_count < @max_rebirths do
          Process.send_after(self(), {:rebirth, agent_id}, @rebirth_cooldown_ms)
        else
          Logger.error(
            "[ConstitutionalSupervisor] #{agent_id} exceeded #{@max_rebirths} rebirths." <>
              " Retiring to Hive as a cautionary story."
          )

          Hive.retire_agent(agent_id, :constitutional_exhaustion, new_weights)
        end

        {:noreply, new_state}
    end
  end

  @impl true
  def handle_info({:rebirth, agent_id}, state) do
    case Map.get(state.workers, agent_id) do
      nil ->
        {:noreply, state}

      worker ->
        Logger.info(
          "[ConstitutionalSupervisor] Re-birthing #{agent_id}" <>
            " (rebirth ##{worker.rebirth_count})"
        )

        config = %{role: worker.role, constitutional_weights: worker.constitutional_weights}

        case DynamicSupervisor.start_child(
               OmokodaSwarm.AgentSupervisor,
               {OmokodaSwarm.Agent, {agent_id, config}}
             ) do
          {:ok, pid} ->
            ref = Process.monitor(pid)
            updated = %{worker | pid: pid}

            new_state = %{
              state
              | workers: Map.put(state.workers, agent_id, updated),
                monitor_refs: Map.put(state.monitor_refs, ref, agent_id)
            }

            {:noreply, new_state}

          {:error, reason} ->
            Logger.error(
              "[ConstitutionalSupervisor] Re-birth failed for #{agent_id}: #{inspect(reason)}"
            )

            {:noreply, state}
        end
    end
  end

  def handle_info({:DOWN, ref, :process, _pid, reason}, state) do
    case Map.pop(state.monitor_refs, ref) do
      {nil, _} ->
        {:noreply, state}

      {agent_id, new_refs} ->
        if reason != :normal do
          Logger.warning(
            "[ConstitutionalSupervisor] #{agent_id} exited abnormally: #{inspect(reason)}"
          )

          # Abnormal exit = rhythm disruption — re-birth will adjust the rhythm weight
          report_violation(agent_id, :rhythm, :warn)
        end

        {:noreply, %{state | monitor_refs: new_refs}}
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp default_weights do
    %{
      mentalism: 1.0,
      correspondence: 1.0,
      vibration: 1.0,
      polarity: 1.0,
      rhythm: 1.0,
      cause_and_effect: 1.0,
      gender: 1.0
    }
  end

  defp adjust_weights(weights, violated_principle, severity) do
    base_delta = Map.get(@weight_deltas, violated_principle, -0.03)
    delta = if severity == :block, do: base_delta * 2.0, else: base_delta
    current = Map.get(weights, violated_principle, 1.0)
    new_weight = max(0.1, current + delta)
    Map.put(weights, violated_principle, Float.round(new_weight, 3))
  end
end
