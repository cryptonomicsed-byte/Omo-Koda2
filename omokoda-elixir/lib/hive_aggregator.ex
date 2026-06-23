defmodule Yemoja.HiveAggregator do
  @moduledoc """
  Singleton GenServer that maintains the **public memory garden** for the
  Yemọja swarm.

  Agents push individual public memory strings via `push_public/2`.  Any
  caller can read the full aggregated garden via `get_garden/0`.

  ## Storage

  Contributions are stored in an ETS table (`Yemoja.HiveETS`) for O(1)
  concurrent reads.  The table is owned by this GenServer; if the GenServer
  crashes the table is lost (a deliberate trade-off — public memory is
  reconstructed by a fresh `memory_checkpoint` from each agent on restart).

  ## Table schema

      {agent_id :: binary(), entries :: [String.t()]}

  One row per agent; entries are prepended so the most-recent item is first.
  """

  use GenServer

  require Logger

  @table Yemoja.HiveETS

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc "Starts the HiveAggregator singleton."
  def start_link(_opts \\ []) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc """
  Pushes a single public memory `entry` from `agent_id` into the garden.

  This is a cast (fire-and-forget) to avoid blocking agent workers.
  """
  @spec push_public(binary(), String.t()) :: :ok
  def push_public(agent_id, entry) when is_binary(agent_id) and is_binary(entry) do
    GenServer.cast(__MODULE__, {:push, agent_id, entry})
  end

  @doc """
  Returns the aggregated public garden as a map of `agent_id => [entries]`.
  """
  @spec get_garden() :: %{binary() => [String.t()]}
  def get_garden do
    GenServer.call(__MODULE__, :get_garden)
  end

  @doc """
  Returns all entries for a specific agent, or `[]` if unknown.
  """
  @spec get_agent_entries(binary()) :: [String.t()]
  def get_agent_entries(agent_id) when is_binary(agent_id) do
    case :ets.lookup(@table, agent_id) do
      [{^agent_id, entries}] -> entries
      [] -> []
    end
  end

  @doc "Clears all entries in the garden.  Intended for tests only."
  @spec clear() :: :ok
  def clear do
    GenServer.call(__MODULE__, :clear)
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init([]) do
    table = :ets.new(@table, [:named_table, :set, :public, read_concurrency: true])
    Logger.info("[HiveAggregator] ETS table #{@table} created (#{table})")
    {:ok, %{table: table}}
  end

  @impl true
  def handle_cast({:push, agent_id, entry}, state) do
    existing =
      case :ets.lookup(@table, agent_id) do
        [{^agent_id, entries}] -> entries
        [] -> []
      end

    :ets.insert(@table, {agent_id, [entry | existing]})
    {:noreply, state}
  end

  @impl true
  def handle_call(:get_garden, _from, state) do
    garden =
      :ets.tab2list(@table)
      |> Enum.into(%{}, fn {agent_id, entries} -> {agent_id, entries} end)

    {:reply, garden, state}
  end

  @impl true
  def handle_call(:clear, _from, state) do
    :ets.delete_all_objects(@table)
    Logger.info("[HiveAggregator] garden cleared")
    {:reply, :ok, state}
  end
end
