defmodule OmokodaSwarm.SwarmSupervisor do
  @moduledoc """
  Supervisor for managing the swarm of agents.
  """

  use Supervisor

  def start_link(init_arg) do
    Supervisor.start_link(__MODULE__, init_arg, name: __MODULE__)
  end

  @impl true
  def init(_init_arg) do
    children = [
      # Dynamic supervisor for agents
      {DynamicSupervisor, strategy: :one_for_one, name: OmokodaSwarm.AgentSupervisor}
    ]

    Supervisor.init(children, strategy: :one_for_one)
  end

  @doc """
  Starts a new agent in the swarm.
  """
  def start_agent(agent_id, config \\ %{}) do
    spec = %{
      id: {OmokodaSwarm.Agent, agent_id},
      start: {OmokodaSwarm.Agent, :start_link, [agent_id, config]},
      restart: :transient
    }

    DynamicSupervisor.start_child(OmokodaSwarm.AgentSupervisor, spec)
  end

  @doc """
  Ensures the default boot agents are running.
  """
  def ensure_initial_agents do
    for {agent_id, role} <- [{"agent_1", :planner}, {"agent_2", :builder}, {"agent_3", :witness}] do
      case start_agent(agent_id, %{role: role}) do
        {:ok, _pid} -> :ok
        {:error, {:already_started, _pid}} -> :ok
        {:error, {:already_present, _pid}} -> :ok
        {:error, _reason} -> :ok
      end
    end

    :ok
  end

  @doc """
  Stops an agent in the swarm.
  """
  def stop_agent(agent_id) do
    case GenServer.whereis(OmokodaSwarm.Agent.process_name(agent_id)) do
      nil ->
        {:error, :not_found}

      pid ->
        DynamicSupervisor.terminate_child(OmokodaSwarm.AgentSupervisor, pid)
        :ok
    end
  end

  @doc """
  Lists all active agents.
  """
  def list_agents do
    DynamicSupervisor.which_children(OmokodaSwarm.AgentSupervisor)
    |> Enum.map(fn {_, pid, _, _} -> pid end)
    |> Enum.map(&OmokodaSwarm.Agent.get_id/1)
  end
end
