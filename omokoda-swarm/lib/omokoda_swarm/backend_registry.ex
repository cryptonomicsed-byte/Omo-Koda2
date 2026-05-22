defmodule OmokodaSwarm.BackendRegistry do
  @moduledoc """
  GenServer registry of execution backends.

  Pre-registers Local, Remote, and Container backends on startup.
  `select/2` returns the highest-priority available backend matching constraints.
  """

  use GenServer

  @name __MODULE__

  @default_backends [
    OmokodaSwarm.Backends.LocalBackend,
    OmokodaSwarm.Backends.RemoteBackend,
    OmokodaSwarm.Backends.ContainerBackend
  ]

  # Client API

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, [], name: Keyword.get(opts, :name, @name))
  end

  def register(server \\ @name, backend_module) do
    GenServer.call(server, {:register, backend_module})
  end

  def unregister(server \\ @name, name) do
    GenServer.call(server, {:unregister, name})
  end

  def get(server \\ @name, name) do
    GenServer.call(server, {:get, name})
  end

  def list(server \\ @name) do
    GenServer.call(server, :list)
  end

  @doc """
  Select the best available backend.

  Options:
  - `:prefer` — atom name of preferred backend (tried first)
  - `:available` — if true (default), only consider available backends
  """
  def select(server \\ @name, constraints \\ []) do
    GenServer.call(server, {:select, constraints})
  end

  # GenServer callbacks

  @impl true
  def init([]) do
    registered =
      @default_backends
      |> Enum.map(fn mod -> {mod.name(), mod} end)
      |> Map.new()

    {:ok, registered}
  end

  @impl true
  def handle_call({:register, backend_module}, _from, state) do
    {:reply, :ok, Map.put(state, backend_module.name(), backend_module)}
  end

  @impl true
  def handle_call({:unregister, name}, _from, state) do
    {:reply, :ok, Map.delete(state, name)}
  end

  @impl true
  def handle_call({:get, name}, _from, state) do
    {:reply, Map.get(state, name), state}
  end

  @impl true
  def handle_call(:list, _from, state) do
    entries =
      Enum.map(state, fn {name, mod} ->
        %{name: name, module: mod, available: mod.available?()}
      end)

    {:reply, entries, state}
  end

  @impl true
  def handle_call({:select, constraints}, _from, state) do
    prefer = Keyword.get(constraints, :prefer)
    require_available = Keyword.get(constraints, :available, true)

    candidates =
      state
      |> Map.values()
      |> then(fn mods ->
        if require_available, do: Enum.filter(mods, & &1.available?()), else: mods
      end)

    result =
      if prefer do
        Enum.find(candidates, &(&1.name() == prefer)) || List.first(candidates)
      else
        List.first(candidates)
      end

    {:reply, result, state}
  end
end
