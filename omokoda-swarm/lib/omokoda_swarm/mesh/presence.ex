defmodule OmokodaSwarm.Mesh.Presence do
  @moduledoc """
  GenServer tracking online/offline/crashed neighbors on each block.
  Sends periodic heartbeats and removes stale entries after a TTL.
  """

  use GenServer
  require Logger

  @heartbeat_ms 15_000
  @ttl_ms 45_000

  defstruct agents: %{}

  # agent entry: %{agent_id, block_id, role, last_seen_ms}

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def checkin(agent_id, block_id, role) do
    GenServer.cast(__MODULE__, {:checkin, agent_id, block_id, role})
  end

  def checkout(agent_id) do
    GenServer.cast(__MODULE__, {:checkout, agent_id})
  end

  def online_agents(block_id) do
    GenServer.call(__MODULE__, {:online_agents, block_id})
  end

  def all_agents do
    GenServer.call(__MODULE__, :all_agents)
  end

  @impl true
  def init(_opts) do
    schedule_heartbeat()
    {:ok, %__MODULE__{}}
  end

  @impl true
  def handle_cast({:checkin, agent_id, block_id, role}, state) do
    entry = %{
      agent_id: agent_id,
      block_id: block_id,
      role: role,
      last_seen_ms: System.monotonic_time(:millisecond)
    }

    {:noreply, %{state | agents: Map.put(state.agents, agent_id, entry)}}
  end

  @impl true
  def handle_cast({:checkout, agent_id}, state) do
    {:noreply, %{state | agents: Map.delete(state.agents, agent_id)}}
  end

  @impl true
  def handle_call({:online_agents, block_id}, _from, state) do
    now = System.monotonic_time(:millisecond)

    agents =
      state.agents
      |> Map.values()
      |> Enum.filter(fn e ->
        e.block_id == block_id and now - e.last_seen_ms < @ttl_ms
      end)

    {:reply, agents, state}
  end

  @impl true
  def handle_call(:all_agents, _from, state) do
    {:reply, Map.values(state.agents), state}
  end

  @impl true
  def handle_info(:heartbeat, state) do
    now = System.monotonic_time(:millisecond)

    fresh =
      state.agents
      |> Enum.reject(fn {_id, e} -> now - e.last_seen_ms >= @ttl_ms end)
      |> Map.new()

    evicted = map_size(state.agents) - map_size(fresh)

    if evicted > 0 do
      Logger.debug("[Mesh.Presence] evicted #{evicted} stale agent(s)")
    end

    schedule_heartbeat()
    {:noreply, %{state | agents: fresh}}
  end

  defp schedule_heartbeat do
    Process.send_after(self(), :heartbeat, @heartbeat_ms)
  end
end
