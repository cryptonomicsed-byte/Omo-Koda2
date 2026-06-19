defmodule OmokodaSwarm.Mesh.DiscoveryCoordinator do
  @moduledoc """
  State machine for block-mesh neighbor discovery.

  Phases: :idle → :broadcasting → :collecting → :exchanging → :complete
  Each phase has a timeout after which the machine advances automatically.
  """

  use GenServer
  require Logger

  @broadcast_timeout_ms 3_000
  @collect_timeout_ms 5_000

  defstruct block_id: nil,
            phase: :idle,
            discovered: [],
            capability_cards: %{}

  def start_link(block_id) do
    GenServer.start_link(__MODULE__, block_id, name: via(block_id))
  end

  def coordinate(task, agents, block_id) do
    ensure_started(block_id)
    GenServer.call(via(block_id), {:coordinate, task, agents}, 30_000)
  end

  def discovered_agents(block_id) do
    ensure_started(block_id)
    GenServer.call(via(block_id), :discovered_agents)
  end

  def record_hello(block_id, agent_id, capability_card) do
    ensure_started(block_id)
    GenServer.cast(via(block_id), {:hello, agent_id, capability_card})
  end

  @impl true
  def init(block_id) do
    {:ok, %__MODULE__{block_id: block_id}}
  end

  @impl true
  def handle_call({:coordinate, task, agents, _block_id}, _from, state) do
    Logger.info("[Mesh.Discovery] coordinating '#{task}' across #{length(agents)} agents on #{state.block_id}")
    {:reply, %{task: task, agents: agents, strategy: :mesh}, state}
  end

  def handle_call({:coordinate, task, agents}, _from, state) do
    Logger.info("[Mesh.Discovery] mesh coordinating '#{task}'")
    {:reply, %{task: task, agents: agents, strategy: :mesh}, state}
  end

  @impl true
  def handle_call(:discovered_agents, _from, state) do
    {:reply, state.discovered, state}
  end

  @impl true
  def handle_cast({:hello, agent_id, card}, state) do
    updated = %{
      state
      | discovered: [agent_id | state.discovered] |> Enum.uniq(),
        capability_cards: Map.put(state.capability_cards, agent_id, card)
    }

    OmokodaSwarm.Mesh.Presence.checkin(agent_id, state.block_id, Map.get(card, "role", "unknown"))
    {:noreply, updated}
  end

  @impl true
  def handle_info({:phase_timeout, :broadcasting}, state) do
    Logger.debug("[Mesh.Discovery] #{state.block_id}: broadcast phase done, collecting...")
    Process.send_after(self(), {:phase_timeout, :collecting}, @collect_timeout_ms)
    {:noreply, %{state | phase: :collecting}}
  end

  @impl true
  def handle_info({:phase_timeout, :collecting}, state) do
    Logger.debug("[Mesh.Discovery] #{state.block_id}: collection done, #{length(state.discovered)} neighbors found")
    {:noreply, %{state | phase: :complete}}
  end

  @impl true
  def handle_info({:phase_timeout, _other}, state) do
    {:noreply, state}
  end

  defp via(block_id) do
    {:via, Registry, {OmokodaSwarm.Registry, {:mesh_discovery, block_id}}}
  end

  defp ensure_started(block_id) do
    case start_link(block_id) do
      {:ok, _} -> :ok
      {:error, {:already_started, _}} -> :ok
      _ -> :ok
    end
  end

  def start_broadcast(block_id) do
    ensure_started(block_id)
    Process.send_after(via(block_id), {:phase_timeout, :broadcasting}, @broadcast_timeout_ms)
  end
end
