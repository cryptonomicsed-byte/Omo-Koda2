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

  When `config` carries a `:territory` key (a Waggle scent-territory prefix),
  the agent is started under `OmokodaSwarm.TerritorySupervisor`'s
  per-territory `DynamicSupervisor` instead of the single global
  `AgentSupervisor` -- so a crash/restart only affects agents working the
  same scent-territory (YEMỌJA territory-aligned supervision, Connection
  Map v2 §6.5-6.6). No `:territory` key means the pre-existing global
  behavior, unchanged -- this is purely additive/opt-in.
  """
  def start_agent(agent_id, config \\ %{}) do
    spec = %{
      id: {OmokodaSwarm.Agent, agent_id},
      start: {OmokodaSwarm.Agent, :start_link, [agent_id, config]},
      restart: :transient
    }

    case Map.get(config, :territory) do
      nil -> DynamicSupervisor.start_child(OmokodaSwarm.AgentSupervisor, spec)
      territory -> OmokodaSwarm.TerritorySupervisor.start_worker(territory, spec)
    end
  end

  @doc """
  Lists agent ids running under a specific scent-territory's supervisor
  (agents started via `start_agent/2` with a `:territory` key). Agents
  started without a territory live under the global `AgentSupervisor` and
  are only visible via `list_agents/0`.
  """
  def list_territory_agents(territory) do
    OmokodaSwarm.TerritorySupervisor.list_territory(territory)
    |> Enum.flat_map(fn
      {_, :restarting, _, _} ->
        []

      {_, pid, _, _} ->
        try do
          [OmokodaSwarm.Agent.get_id(pid)]
        catch
          :exit, _ -> []
        end
    end)
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
    |> Enum.flat_map(fn
      {_, :restarting, _, _} ->
        []

      {_, pid, _, _} ->
        try do
          [OmokodaSwarm.Agent.get_id(pid)]
        catch
          :exit, _ -> []
        end
    end)
  end
end
