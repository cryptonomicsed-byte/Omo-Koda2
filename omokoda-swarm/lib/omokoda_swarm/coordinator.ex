defmodule OmokodaSwarm.Coordinator do
  @moduledoc """
  Advanced Agent Coordination for the Omokoda Swarm.
  Ports the strategies from Swibe (hierarchical, democratic, competitive, pipeline).

  When started via the Application supervisor the process is registered under
  the atom `:swarm_coordinator` so the swarm-level API (submit_task, get_status,
  scale_swarm) can call it without holding a PID.
  """

  use GenServer
  require Logger

  defstruct [:name, :coordination, :agents, :rounds, :message_log]

  # ---------------------------------------------------------------------------
  # Child spec — supervisor-compatible; uses a fixed atom name so the public
  # API can always find the process.
  # ---------------------------------------------------------------------------

  def child_spec(_opts) do
    %{
      id: __MODULE__,
      start: {__MODULE__, :start_link, [:swarm_coordinator, :hierarchical]},
      restart: :permanent,
      type: :worker
    }
  end

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  def start_link(name, coordination \\ :hierarchical) do
    GenServer.start_link(__MODULE__, {name, coordination}, name: name)
  end

  @doc "Dispatches a task to the team using the configured strategy."
  def coordinate(coordinator, task) do
    GenServer.call(coordinator, {:coordinate, task}, 60_000)
  end

  @doc "Adds an agent to the team."
  def add_agent(coordinator, role, config \\ %{}) do
    GenServer.call(coordinator, {:add_agent, role, config})
  end

  @doc """
  Submit a task to the swarm. Returns `{:ok, task_id}` immediately; the task
  is dispatched asynchronously to an available agent.
  """
  def submit_task(task, options \\ []) do
    GenServer.call(:swarm_coordinator, {:submit_task, task, options}, 60_000)
  end

  @doc "Returns a summary map of swarm state: active_agents, active_tasks, agent_statuses."
  def get_status do
    GenServer.call(:swarm_coordinator, :get_status)
  end

  @doc "Scale the agent pool to exactly `count` agents, starting or stopping as needed."
  def scale_swarm(count) do
    GenServer.call(:swarm_coordinator, {:scale_swarm, count})
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init({name, coordination}) do
    # Seed the pool with 3 initial workers.
    Enum.each(1..3, fn i ->
      OmokodaSwarm.SwarmSupervisor.start_agent("agent_#{i}")
    end)

    state = %__MODULE__{
      name: name,
      coordination: coordination,
      agents: %{},
      rounds: [],
      message_log: []
    }

    {:ok, state}
  end

  @impl true
  def handle_call({:add_agent, role, config}, _from, state) do
    agent_id = "#{state.name}_#{role}"

    case OmokodaSwarm.SwarmSupervisor.start_agent(agent_id, config) do
      {:ok, pid} ->
        entry = %{id: agent_id, pid: pid, role: role, weight: config[:weight] || 1.0}
        {:reply, {:ok, agent_id}, %{state | agents: Map.put(state.agents, role, entry)}}

      {:error, reason} ->
        {:reply, {:error, reason}, state}
    end
  end

  @impl true
  def handle_call({:coordinate, task}, _from, state) do
    agents_list = Map.values(state.agents)

    if Enum.empty?(agents_list) do
      {:reply, {:error, :no_agents}, state}
    else
      result = execute_strategy(state.coordination, state, task, agents_list)
      round = %{task: task, strategy: state.coordination, result: result, timestamp: System.system_time(:millisecond)}
      {:reply, {:ok, result}, %{state | rounds: state.rounds ++ [round]}}
    end
  end

  @impl true
  def handle_call({:submit_task, task, _options}, _from, state) do
    task_id = :crypto.strong_rand_bytes(8) |> Base.encode16(case: :lower)
    agents = OmokodaSwarm.SwarmSupervisor.list_agents()

    case agents do
      [] ->
        {:reply, {:error, :no_agents}, state}

      [agent_id | _] ->
        OmokodaSwarm.Agent.delegate_task(agent_id, task)
        {:reply, {:ok, task_id}, state}
    end
  end

  @impl true
  def handle_call(:get_status, _from, state) do
    agents = OmokodaSwarm.SwarmSupervisor.list_agents()

    agent_statuses =
      Enum.map(agents, fn agent_id ->
        case OmokodaSwarm.Agent.get_state(agent_id) do
          {:ok, s} -> s
          _ -> %{id: agent_id, state: :unknown}
        end
      end)

    active_tasks = Enum.count(agent_statuses, &(&1[:state] == :busy))

    status = %{
      active_agents: length(agents),
      active_tasks: active_tasks,
      agent_statuses: agent_statuses
    }

    {:reply, status, state}
  end

  @impl true
  def handle_call({:scale_swarm, count}, _from, state) do
    current = OmokodaSwarm.SwarmSupervisor.list_agents()
    current_count = length(current)

    cond do
      count > current_count ->
        Enum.each(1..(count - current_count), fn _ ->
          id = "auto_#{:crypto.strong_rand_bytes(4) |> Base.encode16(case: :lower)}"
          OmokodaSwarm.SwarmSupervisor.start_agent(id)
        end)

      count < current_count ->
        current
        |> Enum.take(current_count - count)
        |> Enum.each(&OmokodaSwarm.SwarmSupervisor.stop_agent/1)

      true ->
        :ok
    end

    {:reply, :ok, state}
  end

  # ---------------------------------------------------------------------------
  # Strategy implementations
  # ---------------------------------------------------------------------------

  defp execute_strategy(:hierarchical, _state, task, agents) do
    [lead | workers] = agents
    Logger.info("[COORDINATOR] Hierarchical: #{lead.role} leading for task: #{task}")

    {:ok, plan} = OmokodaSwarm.Agent.delegate_task(lead.id, "Plan task: #{task}")

    worker_results =
      Enum.map(workers, fn w ->
        OmokodaSwarm.Agent.delegate_task(w.id, "Execute subtask from plan for: #{task}")
      end)

    {:ok, synthesis} =
      OmokodaSwarm.Agent.delegate_task(lead.id, "Synthesize results: #{inspect(worker_results)}")

    %{synthesis: synthesis, worker_results: worker_results, plan: plan}
  end

  defp execute_strategy(:democratic, _state, task, agents) do
    Logger.info("[COORDINATOR] Democratic: #{length(agents)} agents voting for: #{task}")

    solutions =
      Enum.map(agents, fn agent ->
        {:ok, solution} = OmokodaSwarm.Agent.delegate_task(agent.id, "Propose solution for: #{task}")
        %{agent: agent, solution: solution}
      end)

    winner = Enum.max_by(solutions, fn s -> s.agent.weight end)
    %{winner: winner.solution, solutions: solutions}
  end

  defp execute_strategy(:competitive, _state, task, agents) do
    Logger.info("[COORDINATOR] Competitive: #{length(agents)} agents racing for: #{task}")

    results =
      Enum.map(agents, fn agent ->
        {:ok, result} = OmokodaSwarm.Agent.delegate_task(agent.id, "Compete to solve: #{task}")
        %{agent: agent, result: result}
      end)

    %{winner: hd(results).result, leaderboard: results}
  end

  defp execute_strategy(:pipeline, _state, task, agents) do
    Logger.info("[COORDINATOR] Pipeline: #{length(agents)} stages for: #{task}")

    final_output =
      Enum.reduce(agents, task, fn agent, acc ->
        {:ok, output} = OmokodaSwarm.Agent.delegate_task(agent.id, "Transform input: #{inspect(acc)}")
        output
      end)

    %{output: final_output}
  end

  defp execute_strategy(:mesh, _state, task, agents) do
    Logger.info("[COORDINATOR] Mesh strategy for task: #{task}")
    block_id = agents |> List.first() |> Map.get(:block_id, "local")
    OmokodaSwarm.Mesh.DiscoveryCoordinator.coordinate(task, agents, block_id)
  end
end
