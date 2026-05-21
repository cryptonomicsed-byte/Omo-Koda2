defmodule OmokodaSwarm.TeammateContext do
  @moduledoc """
  Shared key-value execution context for a teammate.
  Stores model parameters, tool results, and session variables.
  """

  use GenServer

  # Client API

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, %{})
  end

  def put(pid, key, value) do
    GenServer.call(pid, {:put, key, value})
  end

  def get(pid, key, default \\ nil) do
    GenServer.call(pid, {:get, key, default})
  end

  def delete(pid, key) do
    GenServer.call(pid, {:delete, key})
  end

  def keys(pid) do
    GenServer.call(pid, :keys)
  end

  def to_map(pid) do
    GenServer.call(pid, :to_map)
  end

  def merge(pid, map) when is_map(map) do
    GenServer.call(pid, {:merge, map})
  end

  # GenServer callbacks

  @impl true
  def init(initial), do: {:ok, initial}

  @impl true
  def handle_call({:put, key, value}, _from, state) do
    {:reply, :ok, Map.put(state, key, value)}
  end

  @impl true
  def handle_call({:get, key, default}, _from, state) do
    {:reply, Map.get(state, key, default), state}
  end

  @impl true
  def handle_call({:delete, key}, _from, state) do
    {:reply, :ok, Map.delete(state, key)}
  end

  @impl true
  def handle_call(:keys, _from, state) do
    {:reply, Map.keys(state), state}
  end

  @impl true
  def handle_call(:to_map, _from, state) do
    {:reply, state, state}
  end

  @impl true
  def handle_call({:merge, map}, _from, state) do
    {:reply, :ok, Map.merge(state, map)}
  end
end
