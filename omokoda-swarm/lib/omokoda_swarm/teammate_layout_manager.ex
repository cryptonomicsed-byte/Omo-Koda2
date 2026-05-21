defmodule OmokodaSwarm.TeammateLayoutManager do
  @moduledoc """
  Manages the logical layout and grouping of teammates in a swarm session.

  Ports Claw's layout management pattern: tracks teammate positions,
  roles, and group assignments for multi-agent sessions.
  """

  use GenServer

  @type layout_entry :: %{
          id: String.t(),
          role: atom(),
          position: non_neg_integer(),
          group: String.t() | nil,
          visible: boolean()
        }

  # Client API

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, [], name: Keyword.get(opts, :name, __MODULE__))
  end

  def add_teammate(server \\ __MODULE__, id, opts \\ []) do
    GenServer.call(server, {:add, id, opts})
  end

  def remove_teammate(server \\ __MODULE__, id) do
    GenServer.call(server, {:remove, id})
  end

  def update_teammate(server \\ __MODULE__, id, updates) do
    GenServer.call(server, {:update, id, updates})
  end

  def get_layout(server \\ __MODULE__) do
    GenServer.call(server, :get_layout)
  end

  def list_teammates(server \\ __MODULE__) do
    GenServer.call(server, :list)
  end

  def list_by_group(server \\ __MODULE__, group) do
    GenServer.call(server, {:by_group, group})
  end

  def list_by_role(server \\ __MODULE__, role) do
    GenServer.call(server, {:by_role, role})
  end

  def reorder(server \\ __MODULE__, id, new_position) do
    GenServer.call(server, {:reorder, id, new_position})
  end

  # GenServer callbacks

  @impl true
  def init([]) do
    {:ok, %{entries: %{}, next_position: 0}}
  end

  @impl true
  def handle_call({:add, id, opts}, _from, state) do
    if Map.has_key?(state.entries, id) do
      {:reply, {:error, :already_exists}, state}
    else
      entry = %{
        id: id,
        role: Keyword.get(opts, :role, :worker),
        position: state.next_position,
        group: Keyword.get(opts, :group),
        visible: Keyword.get(opts, :visible, true)
      }

      new_state = %{
        state
        | entries: Map.put(state.entries, id, entry),
          next_position: state.next_position + 1
      }

      {:reply, :ok, new_state}
    end
  end

  @impl true
  def handle_call({:remove, id}, _from, state) do
    {:reply, :ok, %{state | entries: Map.delete(state.entries, id)}}
  end

  @impl true
  def handle_call({:update, id, updates}, _from, state) do
    case Map.get(state.entries, id) do
      nil ->
        {:reply, {:error, :not_found}, state}

      entry ->
        updated = Map.merge(entry, Map.new(updates))
        {:reply, :ok, %{state | entries: Map.put(state.entries, id, updated)}}
    end
  end

  @impl true
  def handle_call(:get_layout, _from, state) do
    layout =
      state.entries
      |> Map.values()
      |> Enum.sort_by(& &1.position)

    {:reply, layout, state}
  end

  @impl true
  def handle_call(:list, _from, state) do
    {:reply, Map.keys(state.entries), state}
  end

  @impl true
  def handle_call({:by_group, group}, _from, state) do
    matches =
      state.entries
      |> Map.values()
      |> Enum.filter(&(&1.group == group))
      |> Enum.sort_by(& &1.position)

    {:reply, matches, state}
  end

  @impl true
  def handle_call({:by_role, role}, _from, state) do
    matches =
      state.entries
      |> Map.values()
      |> Enum.filter(&(&1.role == role))
      |> Enum.sort_by(& &1.position)

    {:reply, matches, state}
  end

  @impl true
  def handle_call({:reorder, id, new_position}, _from, state) do
    case Map.get(state.entries, id) do
      nil ->
        {:reply, {:error, :not_found}, state}

      entry ->
        updated = %{entry | position: new_position}
        {:reply, :ok, %{state | entries: Map.put(state.entries, id, updated)}}
    end
  end
end
