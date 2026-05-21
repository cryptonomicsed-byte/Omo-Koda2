defmodule OmokodaSwarm.PermissionSync do
  @moduledoc """
  Swarm-wide permission synchronization with leader-based approval.

  Ports Claw's `leaderPermissionBridge.ts`:
  - Grants are cached in `persistent_term` for O(1) node-local reads.
  - The GenServer serializes approvals and broadcasts to all peer nodes.
  - Pending requests are tracked until approved or denied.
  """

  use GenServer
  require Logger

  @pt_key {__MODULE__, :grants}

  # Client API

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, [], name: Keyword.get(opts, :name, __MODULE__))
  end

  @doc "Submit a permission request for a tool at a given tier."
  def request(server \\ __MODULE__, requestor, tool, tier \\ 0) do
    GenServer.call(server, {:request, requestor, tool, tier})
  end

  @doc "Approve a pending permission request. Returns {:ok, grant}."
  def approve(server \\ __MODULE__, request_id, granted_by, opts \\ []) do
    GenServer.call(server, {:approve, request_id, granted_by, opts})
  end

  @doc "Deny a pending permission request."
  def deny(server \\ __MODULE__, request_id, reason) do
    GenServer.call(server, {:deny, request_id, reason})
  end

  @doc "O(1) check via persistent_term — safe to call from any process."
  def check(grantee, tool, tier \\ 0) do
    now = System.system_time(:second)
    grants = :persistent_term.get(@pt_key, [])

    Enum.any?(grants, fn g ->
      g.grantee == grantee and
        g.tool in [tool, "*"] and
        g.tier >= tier and
        (g.expires_at == :never or g.expires_at > now)
    end)
  rescue
    _ -> false
  end

  @doc "Return all active grants from persistent_term."
  def list_grants do
    :persistent_term.get(@pt_key, [])
  rescue
    _ -> []
  end

  @doc "List pending requests held by the GenServer."
  def list_requests(server \\ __MODULE__) do
    GenServer.call(server, :list_requests)
  end

  @doc "Revoke a grant by ID and broadcast the updated grant set."
  def revoke(server \\ __MODULE__, grant_id) do
    GenServer.call(server, {:revoke, grant_id})
  end

  @doc "Push the current grant set to a specific remote node's persistent_term."
  def sync_to_node(server \\ __MODULE__, node) do
    GenServer.call(server, {:sync_to_node, node})
  end

  # GenServer callbacks

  @impl true
  def init([]) do
    cache_grants([])
    {:ok, %{grants: [], requests: %{}}}
  end

  @impl true
  def handle_call({:request, requestor, tool, tier}, _from, state) do
    req_id = random_id()

    req = %{
      id: req_id,
      tool: tool,
      tier: tier,
      requestor: requestor,
      status: :pending,
      reason: nil,
      requested_at: System.system_time(:second)
    }

    Logger.info("[PermissionSync] Request #{req_id}: #{requestor} -> #{tool} tier=#{tier}")
    {:reply, {:ok, req_id}, %{state | requests: Map.put(state.requests, req_id, req)}}
  end

  @impl true
  def handle_call({:approve, req_id, granted_by, opts}, _from, state) do
    case Map.get(state.requests, req_id) do
      nil ->
        {:reply, {:error, :not_found}, state}

      req ->
        ttl = Keyword.get(opts, :ttl_secs, :never)

        expires_at =
          if ttl == :never, do: :never, else: System.system_time(:second) + ttl

        grant = %{
          id: random_id(),
          tool: req.tool,
          tier: req.tier,
          grantee: req.requestor,
          granted_by: granted_by,
          granted_at: System.system_time(:second),
          expires_at: expires_at
        }

        new_grants = [grant | state.grants]
        cache_grants(new_grants)
        broadcast_grants(new_grants)

        Logger.info("[PermissionSync] Approved #{req_id}: #{req.requestor} -> #{req.tool}")

        new_state = %{
          state
          | grants: new_grants,
            requests: Map.put(state.requests, req_id, %{req | status: :approved})
        }

        {:reply, {:ok, grant}, new_state}
    end
  end

  @impl true
  def handle_call({:deny, req_id, reason}, _from, state) do
    case Map.get(state.requests, req_id) do
      nil ->
        {:reply, {:error, :not_found}, state}

      req ->
        updated = %{req | status: :denied, reason: reason}
        Logger.info("[PermissionSync] Denied #{req_id}: #{reason}")
        {:reply, :ok, %{state | requests: Map.put(state.requests, req_id, updated)}}
    end
  end

  @impl true
  def handle_call(:list_requests, _from, state) do
    {:reply, Map.values(state.requests), state}
  end

  @impl true
  def handle_call({:revoke, grant_id}, _from, state) do
    new_grants = Enum.reject(state.grants, &(&1.id == grant_id))
    cache_grants(new_grants)
    Logger.info("[PermissionSync] Revoked grant #{grant_id}")
    {:reply, :ok, %{state | grants: new_grants}}
  end

  @impl true
  def handle_call({:sync_to_node, node}, _from, state) do
    :erpc.call(node, :persistent_term, :put, [@pt_key, state.grants])
    Logger.info("[PermissionSync] Synced #{length(state.grants)} grants to #{node}")
    {:reply, :ok, state}
  rescue
    e -> {:reply, {:error, e}, state}
  end

  @impl true
  def handle_info({:grant_broadcast, grants}, state) do
    cache_grants(grants)
    {:noreply, %{state | grants: grants}}
  end

  defp cache_grants(grants) do
    :persistent_term.put(@pt_key, grants)
  end

  defp broadcast_grants(grants) do
    Enum.each(Node.list(), fn node ->
      send({__MODULE__, node}, {:grant_broadcast, grants})
    end)
  end

  defp random_id do
    :crypto.strong_rand_bytes(8) |> Base.url_encode64(padding: false)
  end
end
