defmodule OmokodaSwarm.Delegation do
  @moduledoc """
  Handles task delegation between agents in the swarm.
  """

  @doc """
  Delegates a task to an available agent.
  """
  def delegate_task(task, options \\ []) do
    # Find available agents
    available_agents = find_available_agents()

    case available_agents do
      [] ->
        {:error, :no_available_agents}

      agents ->
        # Select agent based on strategy (round-robin, load balancing, etc.)
        strategy = Keyword.get(options, :strategy, :round_robin)
        agent_id = select_agent(agents, strategy)

        # Delegate the task
        OmokodaSwarm.Agent.delegate_task(agent_id, task)
    end
  end

  @doc """
  Delegates a task to multiple agents for consensus/witness.
  """
  def delegate_for_consensus(task, witness_count) do
    available_agents = find_available_agents()

    if length(available_agents) < witness_count do
      {:error, :insufficient_agents_for_consensus}
    else
      # Select witnesses
      witnesses = Enum.take_random(available_agents, witness_count)

      # Delegate to each witness
      results =
        Enum.map(witnesses, fn agent_id ->
          Task.async(fn -> OmokodaSwarm.Agent.delegate_task(agent_id, task) end)
        end)

      # Wait for all results
      Task.await_many(results, 5000)
    end
  end

  @doc """
  Finds agents that are currently idle, with fallback to all agents.
  """
  def find_available_agents do
    all_agents = OmokodaSwarm.SwarmSupervisor.list_agents()

    idle_agents =
      Enum.filter(all_agents, fn agent_id ->
        case OmokodaSwarm.Agent.get_state(agent_id) do
          {:ok, %{state: :idle}} -> true
          _ -> false
        end
      end)

    if idle_agents == [], do: all_agents, else: idle_agents
  end

  @doc """
  Selects an agent based on the given strategy.
  """
  def select_agent(agents, :round_robin) do
    # Simple round-robin selection
    Enum.random(agents)
  end

  def select_agent(agents, :least_loaded) do
    # Select agent with fewest tasks
    agents
    |> Enum.map(fn agent_id ->
      case OmokodaSwarm.Agent.get_state(agent_id) do
        {:ok, state} -> {agent_id, length(state.tasks)}
        _ -> {agent_id, 999}
      end
    end)
    |> Enum.min_by(fn {_id, load} -> load end)
    |> elem(0)
  end
end
