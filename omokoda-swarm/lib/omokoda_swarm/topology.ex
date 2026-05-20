defmodule OmokodaSwarm.Topology do
  @moduledoc """
  Swarm topology — backend abstraction, permission synchronization, and
  multi-node agent coordination.

  Ports the `swarm/` backends (iTerm/Tmux/InProcess), permission bridging,
  and teammate coordination patterns.
  """

  require Logger

  # ---------------------------------------------------------------------------
  # Backend abstraction
  # ---------------------------------------------------------------------------

  @type backend :: :in_process | :distributed | :external
  @type node_info :: %{id: String.t(), backend: backend(), pid: pid() | nil, status: atom()}

  @doc """
  Describe the active topology: which backend is in use and how many nodes.
  """
  @spec describe() :: map()
  def describe do
    nodes = discover_nodes()
    backend = detect_backend()

    %{
      backend: backend,
      node_count: length(nodes),
      nodes: nodes,
      coordinator: coordinator_node(nodes),
      permissions: permission_summary()
    }
  end

  @doc """
  Detect the execution backend.
    - `:in_process` — all agents run in the same BEAM VM (default)
    - `:distributed` — agents span multiple Erlang nodes
    - `:external` — agents run in external processes (e.g. via ports)
  """
  @spec detect_backend() :: backend()
  def detect_backend do
    cond do
      Node.list() != [] -> :distributed
      System.get_env("OMOKODA_EXTERNAL_BACKEND") == "true" -> :external
      true -> :in_process
    end
  end

  @doc """
  Discover all active swarm nodes.
  In `:in_process` mode this returns process info for each agent.
  In `:distributed` mode it polls connected Erlang nodes.
  """
  @spec discover_nodes() :: [node_info()]
  def discover_nodes do
    case detect_backend() do
      :in_process ->
        discover_in_process()

      :distributed ->
        discover_distributed()

      :external ->
        []
    end
  end

  @doc """
  Broadcast a topology event to all known nodes/agents.
  """
  @spec broadcast(term()) :: :ok
  def broadcast(message) do
    nodes = discover_nodes()
    Logger.info("[Topology] Broadcasting to #{length(nodes)} nodes: #{inspect(message)}")

    Enum.each(nodes, fn node ->
      case node do
        %{pid: pid} when is_pid(pid) ->
          send(pid, {:topology_event, message})

        %{id: agent_id} ->
          OmokodaSwarm.Agent.delegate_task(agent_id, message)
      end
    end)

    :ok
  end

  # ---------------------------------------------------------------------------
  # Permission synchronization
  # ---------------------------------------------------------------------------

  @type permission_rule :: %{tool: String.t(), action: :allow | :deny, tier: non_neg_integer()}

  @doc """
  Synchronize a permission policy across all nodes.
  All agents adopt the same allow/deny rules.
  """
  @spec sync_permissions([permission_rule()]) :: :ok
  def sync_permissions(rules) when is_list(rules) do
    Logger.info("[Topology] Syncing #{length(rules)} permission rules across swarm")
    :ok = store_permissions(rules)
    broadcast({:permission_sync, rules})
    :ok
  end

  @doc """
  Fetch the current effective permission policy for the swarm.
  """
  @spec get_permissions() :: [permission_rule()]
  def get_permissions do
    fetch_permissions()
  end

  @doc """
  Check whether a given tool action is allowed under the current policy.
  """
  @spec permitted?(String.t(), non_neg_integer()) :: boolean()
  def permitted?(tool, tier \\ 0) do
    rules = get_permissions()

    deny = Enum.any?(rules, fn r ->
      r.tool in [tool, "*"] and r.action == :deny and tier < r.tier
    end)

    not deny
  end

  # ---------------------------------------------------------------------------
  # Coordination helpers
  # ---------------------------------------------------------------------------

  @doc """
  Elect a coordinator from the current node list using a simple priority rule:
  prefers the node with the lowest id (stable leader election).
  """
  @spec elect_coordinator() :: node_info() | nil
  def elect_coordinator do
    case discover_nodes() do
      [] -> nil
      nodes -> Enum.min_by(nodes, & &1.id)
    end
  end

  @doc """
  Reconnect a disconnected node by id. For `:distributed` backend,
  this calls `Node.connect/1`.
  """
  @spec reconnect(String.t()) :: :ok | {:error, term()}
  def reconnect(node_id) do
    case detect_backend() do
      :distributed ->
        node_atom = String.to_atom(node_id)
        if Node.connect(node_atom) do
          Logger.info("[Topology] Reconnected to #{node_id}")
          :ok
        else
          {:error, :connection_failed}
        end

      _ ->
        Logger.warning("[Topology] reconnect/1 is a no-op in #{detect_backend()} backend")
        :ok
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp discover_in_process do
    agents = OmokodaSwarm.SwarmSupervisor.list_agents()

    Enum.map(agents, fn agent_id ->
      pid =
        agent_id
        |> OmokodaSwarm.Agent.process_name()
        |> GenServer.whereis()

      status =
        case OmokodaSwarm.Agent.get_state(agent_id) do
          {:ok, state} -> state.state
          _ -> :unknown
        end

      %{id: agent_id, backend: :in_process, pid: pid, status: status}
    end)
  end

  defp discover_distributed do
    Node.list()
    |> Enum.map(fn node ->
      %{id: to_string(node), backend: :distributed, pid: nil, status: :remote}
    end)
  end

  defp coordinator_node([]), do: nil
  defp coordinator_node(nodes), do: Enum.min_by(nodes, & &1.id)

  defp permission_summary do
    rules = fetch_permissions()
    %{
      rule_count: length(rules),
      denied_tools: rules |> Enum.filter(&(&1.action == :deny)) |> Enum.map(& &1.tool)
    }
  end

  # Permissions are stored in the process dictionary of the Application process
  # for simplicity; a production implementation would use ETS or a GenServer.
  defp store_permissions(rules) do
    :persistent_term.put({__MODULE__, :permissions}, rules)
    :ok
  rescue
    _ -> :ok
  end

  defp fetch_permissions do
    :persistent_term.get({__MODULE__, :permissions}, [])
  rescue
    _ -> []
  end
end
