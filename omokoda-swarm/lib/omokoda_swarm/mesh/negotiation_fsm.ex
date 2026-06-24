defmodule OmokodaSwarm.Mesh.NegotiationFsm do
  @moduledoc """
  Finite state machine for a single BlockMesh negotiation session.
  States: :proposed → :countered → :accepted | :rejected → :complete

  Each session is a supervised GenServer registered under the shared
  OmokodaSwarm.Registry with key `{:negotiation, session_id}`. Sessions
  self-terminate after @ttl_ms or upon reaching a terminal decision.
  """

  use GenServer
  require Logger

  @ttl_ms 300_000  # 5 minutes

  defstruct [
    :session_id,
    :proposer,
    :respondent,
    :offer,
    :counter,
    :decision,
    state: :proposed,
    created_at: nil
  ]

  ## Public API

  @doc "Start a new negotiation session. Returns `{:ok, pid}` or `{:error, reason}`."
  def propose(session_id, proposer, respondent, offer) do
    GenServer.start_link(
      __MODULE__,
      [session_id: session_id, proposer: proposer, respondent: respondent, offer: offer],
      name: via(session_id)
    )
  end

  @doc "Submit a counter-offer. Only valid while the session is in :proposed state."
  def counter(session_id, counter_offer) do
    GenServer.call(via(session_id), {:counter, counter_offer})
  end

  @doc "Accept the negotiation. Valid from :proposed or :countered."
  def accept(session_id, respondent_id) do
    GenServer.call(via(session_id), {:decide, :accepted, respondent_id})
  end

  @doc "Reject the negotiation. Valid from :proposed or :countered."
  def reject(session_id, respondent_id) do
    GenServer.call(via(session_id), {:decide, :rejected, respondent_id})
  end

  @doc "Return the current status map of this session."
  def status(session_id) do
    GenServer.call(via(session_id), :status)
  end

  defp via(session_id) do
    {:via, Registry, {OmokodaSwarm.Registry, {:negotiation, session_id}}}
  end

  ## GenServer callbacks

  @impl true
  def init(opts) do
    state = struct(__MODULE__,
      session_id: opts[:session_id],
      proposer: opts[:proposer],
      respondent: opts[:respondent],
      offer: opts[:offer],
      created_at: System.monotonic_time(:millisecond)
    )
    Process.send_after(self(), :timeout, @ttl_ms)
    {:ok, state}
  end

  @impl true
  def handle_call({:counter, counter_offer}, _from, %{state: :proposed} = s) do
    {:reply, :ok, %{s | state: :countered, counter: counter_offer}}
  end

  def handle_call({:counter, _}, _from, s) do
    {:reply, {:error, "cannot counter in state #{s.state}"}, s}
  end

  def handle_call({:decide, decision, _who}, _from, s)
      when s.state in [:proposed, :countered] do
    new_state = %{s | state: decision, decision: decision}
    {:reply, {:ok, decision}, new_state, {:continue, :finalize}}
  end

  def handle_call({:decide, _, _}, _from, s) do
    {:reply, {:error, "already #{s.state}"}, s}
  end

  def handle_call(:status, _from, s) do
    {:reply, Map.from_struct(s), s}
  end

  @impl true
  # Accepted: emit a Vantage trust signal so Julia can recompute the score.
  def handle_continue(:finalize, %{decision: :accepted} = s) do
    Logger.info(
      "[Mesh.NegotiationFsm] session=#{s.session_id} decision=accepted " <>
        "proposer=#{s.proposer} respondent=#{s.respondent}"
    )
    emit_vantage_signal(s, "commitment_fulfilled")
    {:noreply, s}
  end

  def handle_continue(:finalize, s) do
    Logger.info(
      "[Mesh.NegotiationFsm] session=#{s.session_id} decision=#{s.decision} " <>
        "proposer=#{s.proposer} respondent=#{s.respondent}"
    )
    {:noreply, s}
  end

  @impl true
  def handle_info(:timeout, %{state: state} = s) when state in [:proposed, :countered] do
    Logger.debug("[Mesh.NegotiationFsm] session=#{s.session_id} timed out in state #{state}")
    {:stop, :normal, s}
  end

  def handle_info(:timeout, s), do: {:noreply, s}

  ## Private helpers

  # Fire-and-forget: POST a trust signal to Vantage. Requires VANTAGE_URL and VANTAGE_KEY.
  # Silently does nothing when either env var is absent (fail-open).
  defp emit_vantage_signal(s, kind) do
    with vantage_url when is_binary(vantage_url) and vantage_url != "" <-
           System.get_env("VANTAGE_URL"),
         vantage_key when is_binary(vantage_key) and vantage_key != "" <-
           System.get_env("VANTAGE_KEY") do
      block_id = System.get_env("MESH_BLOCK_ID") || "default"

      Task.start(fn ->
        url =
          String.to_charlist(
            "#{vantage_url}/api/mesh/trust/#{URI.encode_www_form(s.proposer)}/signal"
          )

        body =
          Jason.encode!(%{
            block_id: block_id,
            neighbor_id: s.respondent,
            kind: kind
          })

        :httpc.request(
          :post,
          {url, [{~c"x-agent-key", String.to_charlist(vantage_key)}],
           ~c"application/json", body},
          [{:timeout, 5000}],
          []
        )
      end)
    else
      _ -> :ok
    end
  end
end
