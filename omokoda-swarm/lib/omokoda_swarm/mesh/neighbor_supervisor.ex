defmodule OmokodaSwarm.Mesh.NeighborSupervisor do
  @moduledoc """
  DynamicSupervisor managing one lightweight process per known neighbor agent.
  Neighbors are tracked by agent_id. Restarted on crash with :transient strategy.
  """

  use DynamicSupervisor
  require Logger

  def start_link(opts \\ []) do
    DynamicSupervisor.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @impl true
  def init(_opts) do
    DynamicSupervisor.init(strategy: :one_for_one)
  end

  def add_neighbor(agent_id, block_id, role \\ :unknown) do
    spec = {OmokodaSwarm.Mesh.NeighborWorker, {agent_id, block_id, role}}

    case DynamicSupervisor.start_child(__MODULE__, spec) do
      {:ok, pid} ->
        Logger.debug("[Mesh.NeighborSupervisor] started worker for #{agent_id}")
        {:ok, pid}

      {:error, {:already_started, pid}} ->
        {:ok, pid}

      err ->
        err
    end
  end

  def remove_neighbor(agent_id) do
    case Registry.lookup(OmokodaSwarm.Registry, {:mesh_neighbor, agent_id}) do
      [{pid, _}] ->
        DynamicSupervisor.terminate_child(__MODULE__, pid)

      [] ->
        :ok
    end
  end

  def list_neighbors do
    DynamicSupervisor.which_children(__MODULE__)
    |> Enum.filter(fn {_, pid, _, _} -> is_pid(pid) end)
    |> Enum.map(fn {_, pid, _, _} -> pid end)
  end
end

defmodule OmokodaSwarm.Mesh.NeighborWorker do
  @moduledoc false

  use GenServer, restart: :transient

  def start_link({agent_id, block_id, role}) do
    GenServer.start_link(
      __MODULE__,
      %{agent_id: agent_id, block_id: block_id, role: role},
      name: {:via, Registry, {OmokodaSwarm.Registry, {:mesh_neighbor, agent_id}}}
    )
  end

  @impl true
  def init(state) do
    OmokodaSwarm.Mesh.Presence.checkin(state.agent_id, state.block_id, state.role)
    {:ok, state}
  end

  @impl true
  def terminate(_reason, state) do
    OmokodaSwarm.Mesh.Presence.checkout(state.agent_id)
  end
end
