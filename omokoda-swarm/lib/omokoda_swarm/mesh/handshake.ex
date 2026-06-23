defmodule OmokodaSwarm.Mesh.Handshake do
  @moduledoc """
  Manages BlockMesh handshake sessions between agents.

  A handshake is a lightweight pre-negotiation agreement on communication terms
  (e.g. protocol version, encryption preferences, capability advertisement).
  Sessions expire after 10 minutes; a periodic cleanup sweeps stale entries.

  This is a singleton GenServer started by the mesh supervisor.
  """

  use GenServer
  require Logger

  @cleanup_interval_ms 60_000
  @session_ttl_s 600  # 10 minutes

  ## Public API

  def start_link(_opts \ []) do
    GenServer.start_link(__MODULE__, %{}, name: __MODULE__)
  end

  @doc "Initiate a handshake from `proposer` to `respondent` with the given `terms` map."
  def initiate(proposer, respondent, terms) do
    GenServer.call(__MODULE__, {:initiate, proposer, respondent, terms})
  end

  @doc "Acknowledge a handshake — `respondent_id` must match the original respondent."
  def acknowledge(session_id, respondent_id) do
    GenServer.call(__MODULE__, {:acknowledge, session_id, respondent_id})
  end

  @doc "Return the session map for `session_id`, or `nil` if not found."
  def session_state(session_id) do
    GenServer.call(__MODULE__, {:get, session_id})
  end

  @doc "Return all active (non-expired) sessions."
  def active_sessions do
    GenServer.call(__MODULE__, :list)
  end

  ## GenServer callbacks

  @impl true
  def init(_) do
    schedule_cleanup()
    {:ok, %{sessions: %{}}}
  end

  @impl true
  def handle_call({:initiate, proposer, respondent, terms}, _from, state) do
    session_id = generate_id()
    session = %{
      id: session_id,
      proposer: proposer,
      respondent: respondent,
      terms: terms,
      status: :pending,
      created_at: System.system_time(:second)
    }
    Logger.debug("[Mesh.Handshake] initiated session=#{session_id} proposer=#{proposer} respondent=#{respondent}")
    {:reply, {:ok, session_id, session}, put_in(state, [:sessions, session_id], session)}
  end

  def handle_call({:acknowledge, session_id, respondent_id}, _from, state) do
    case get_in(state, [:sessions, session_id]) do
      nil ->
        {:reply, {:error, :not_found}, state}

      %{respondent: ^respondent_id} = session ->
        updated = %{session | status: :acknowledged}
        Logger.debug("[Mesh.Handshake] acknowledged session=#{session_id}")
        {:reply, {:ok, updated}, put_in(state, [:sessions, session_id], updated)}

      _ ->
        {:reply, {:error, :unauthorized}, state}
    end
  end

  def handle_call({:get, session_id}, _from, state) do
    {:reply, Map.get(state.sessions, session_id), state}
  end

  def handle_call(:list, _from, state) do
    {:reply, Map.values(state.sessions), state}
  end

  @impl true
  def handle_info(:cleanup, state) do
    cutoff = System.system_time(:second) - @session_ttl_s
    sessions = Map.reject(state.sessions, fn {_, s} -> s.created_at < cutoff end)
    evicted = map_size(state.sessions) - map_size(sessions)
    if evicted > 0 do
      Logger.debug("[Mesh.Handshake] evicted #{evicted} stale session(s)")
    end
    schedule_cleanup()
    {:noreply, %{state | sessions: sessions}}
  end

  defp schedule_cleanup do
    Process.send_after(self(), :cleanup, @cleanup_interval_ms)
  end

  defp generate_id do
    :crypto.strong_rand_bytes(8) |> Base.url_encode64(padding: false)
  end
end
