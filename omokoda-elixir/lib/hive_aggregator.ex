defmodule Yemoja.HiveAggregator do
  @moduledoc """
  Aggregates public memory contributions across agents.
  Private memory is never sent here — stays Rust-side only.
  """
  use GenServer
  require Logger

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def contribute(agent_id, memory_entry) do
    GenServer.cast(__MODULE__, {:contribute, agent_id, memory_entry})
  end

  def query_public_memory(filter \\ %{}) do
    GenServer.call(__MODULE__, {:query, filter})
  end

  @impl true
  def init([]) do
    {:ok, %{entries: []}}
  end

  @impl true
  def handle_cast({:contribute, agent_id, entry}, state) do
    tagged = Map.merge(entry, %{agent_id: agent_id, timestamp: System.system_time(:second)})
    Logger.debug("[HiveAggregator] New public memory from #{agent_id}")
    {:noreply, %{state | entries: [tagged | state.entries]}}
  end

  @impl true
  def handle_call({:query, _filter}, _from, state) do
    {:reply, state.entries, state}
  end
end
