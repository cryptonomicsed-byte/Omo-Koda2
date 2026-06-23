defmodule OmokodaSwarm.Coordinator do
  @moduledoc """
  Advanced Agent Coordination for the Omokoda Swarm.
  Ports the strategies from Swibe (hierarchical, democratic, competitive, pipeline).
  """

  use GenServer
  require Logger

  defstruct [:name, :coordination, :agents, :rounds, :message_log]

  @doc """
  Starts the coordinator.
  """
  def start_link(name \\ __MODULE__, coordination \\ :hierarchical)

  def start_link(args, coordination) when is_list(args) do
    name = Keyword.get(args, :name, __MODULE__)
    coordination = Keyword.get(args, :coordination, coordination)
    start_link(name, coordination)
  end

  def start_link(name, coordination) do
    GenServer.start_link(__MODULE__, {name, coordination}, name: name)
  end

  @doc """
  Submits a task to an available swarm agent.
  """
  def submit_task(task, options \\ []) do
    task_id = "task_#{System.unique_integer([:positive, :monotonic])}"

    case OmokodaSwarm.Delegation.delegate_task(Map.put(task, :id, task_id), options) do
      :ok -> {:ok, task_id}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Returns a snapshot of swarm health.
  """
  def get_status do
    agents = OmokodaSwarm.SwarmSupervisor.list_agents()

    agent_statuses =
      agents
      |> Enum.map(fn agent_id ->
        case OmokodaSwarm.Agent.get_state(agent_id) do
          {:ok, state} -> {agent_id, %{state: state.state, tasks: length(state.tasks)}}
          {:error, reason} -> {agent_id, %{state: :unknown, error: reason}}
        end
      end)
      |> Map.new()

    active_tasks =
      agent_statuses
      |> Map.values()
      |> Enum.map(&Map.get(&1, :tasks, 0))
      |> Enum.sum()

    %{
      active_agents: length(agents),
      active_tasks: active_tasks,
      agent_statuses: agent_statuses
    }
  end

  @doc """
  Scales the dynamic agent pool to the requested size.
  """
  def scale_swarm(count) when is_integer(count) and count >= 0 do
    current = OmokodaSwarm.SwarmSupervisor.list_agents()
    current_count = length(current)

    cond do
      count > current_count ->
        for index <- (current_count + 1)..count do
          OmokodaSwarm.SwarmSupervisor.start_agent("agent_#{index}", %{role: :worker})
        end

      count < current_count ->
        current
        |> Enum.sort()
        |> Enum.drop(count)
        |> Enum.each(&OmokodaSwarm.SwarmSupervisor.stop_agent/1)

      true ->
        :ok
    end

    :ok
  end

  @doc """
  Dispatches a task to the team using the configured strategy.
  """
  def coordinate(coordinator, task) do
    GenServer.call(coordinator, {:coordinate, task}, 60_000)
  end

  @doc """
  Adds an agent to the team.
  """
  def add_agent(coordinator, role, config \\ %{}) do
    GenServer.call(coordinator, {:add_agent, role, config})
  end

  # GenServer callbacks

  @impl true
  def init({name, coordination}) do
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
    # Start the agent process via SwarmSupervisor
    agent_id = "#{state.name}_#{role}"

    case OmokodaSwarm.SwarmSupervisor.start_agent(agent_id, config) do
      {:ok, pid} ->
        new_agents =
          Map.put(state.agents, role, %{
            id: agent_id,
            pid: pid,
            role: role,
            weight: config[:weight] || 1.0
          })

        {:reply, {:ok, agent_id}, %{state | agents: new_agents}}

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

      new_rounds =
        state.rounds ++
          [
            %{
              task: task,
              strategy: state.coordination,
              result: result,
              timestamp: System.system_time(:millisecond)
            }
          ]

      {:reply, {:ok, result}, %{state | rounds: new_rounds}}
    end
  end

  # Strategy Implementations

  defp execute_strategy(:hierarchical, _state, task, agents) do
    [lead | workers] = agents
    Logger.info("[COORDINATOR] Hierarchical strategy: #{lead.role} leading for task: #{task}")

    # 1. Lead plans
    {:ok, _plan} = OmokodaSwarm.Agent.delegate_task(lead.id, "Plan task: #{task}")

    # 2. Delegate to workers
    worker_results =
      Enum.map(workers, fn worker ->
        OmokodaSwarm.Agent.delegate_task(worker.id, "Execute subtask from plan for: #{task}")
      end)

    # 3. Lead synthesizes
    {:ok, synthesis} =
      OmokodaSwarm.Agent.delegate_task(lead.id, "Synthesize results: #{inspect(worker_results)}")

    %{synthesis: synthesis, worker_results: worker_results}
  end

  defp execute_strategy(:democratic, _state, task, agents) do
    Logger.info("[COORDINATOR] Democratic strategy for task: #{task}")

    # 1. Agents propose solutions
    solutions =
      Enum.map(agents, fn agent ->
        {:ok, solution} =
          OmokodaSwarm.Agent.delegate_task(agent.id, "Propose solution for: #{task}")

        %{agent: agent, solution: solution}
      end)

    # 2. Vote (simplified weighted consensus)
    winner = Enum.max_by(solutions, fn s -> s.agent.weight end)

    %{winner: winner.solution, solutions: solutions}
  end

  defp execute_strategy(:competitive, _state, task, agents) do
    Logger.info("[COORDINATOR] Competitive strategy for task: #{task}")

    # Agents race (simplified: first in list for now, in real BEAM would be first to finish)
    results =
      Enum.map(agents, fn agent ->
        {:ok, result} = OmokodaSwarm.Agent.delegate_task(agent.id, "Compete to solve: #{task}")
        %{agent: agent, result: result}
      end)

    %{winner: hd(results).result, leaderboard: results}
  end

  defp execute_strategy(:pipeline, _state, task, agents) do
    Logger.info("[COORDINATOR] Pipeline strategy for task: #{task}")

    final_output =
      Enum.reduce(agents, task, fn agent, acc ->
        {:ok, output} =
          OmokodaSwarm.Agent.delegate_task(agent.id, "Transform input: #{inspect(acc)}")

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
