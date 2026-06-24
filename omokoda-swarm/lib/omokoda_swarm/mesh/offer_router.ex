defmodule OmokodaSwarm.Mesh.OfferRouter do
  @moduledoc """
  Routes incoming BlockMesh offers (proposals) to the correct handler.

  Handlers are registered with a required capability string and a minimum trust
  score threshold. When an offer arrives, all handlers whose capability matches
  and whose trust threshold is satisfied receive the offer via their callback.

  Callbacks are synchronous within the router process; long-running work should
  be dispatched to a Task or separate process inside the callback.

  This is a singleton GenServer started by the mesh supervisor.
  """

  use GenServer
  require Logger

  @type callback :: (map() -> :ok | {:error, term()})

  @type handler :: %{
    agent_id: String.t(),
    capability: String.t(),
    min_trust: float(),
    callback: callback()
  }

  ## Public API

  def start_link(_opts \\ []) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc """
  Register an offer handler for `capability`.

  `min_trust` (default 0.3) is the minimum `:trust_score` an offer must carry
  for this handler to be invoked. Use `""` as capability to match all offers.
  """
  def register_handler(agent_id, capability, min_trust \\ 0.3, callback) do
    GenServer.call(__MODULE__, {:register, agent_id, capability, min_trust, callback})
  end

  @doc """
  Route `offer` to all matching handlers.

  Returns `{:ok, [{agent_id, result}]}` where result is the return value of
  each handler callback (or `{:error, message}` if the callback raised).
  """
  def route_offer(offer) do
    GenServer.call(__MODULE__, {:route, offer})
  end

  @doc "Return all currently registered handlers (callbacks omitted for safety)."
  def handlers do
    GenServer.call(__MODULE__, :list)
  end

  ## GenServer callbacks

  @impl true
  def init(_) do
    {:ok, %{handlers: []}}
  end

  @impl true
  def handle_call({:register, agent_id, capability, min_trust, callback}, _from, state) do
    handler = %{
      agent_id: agent_id,
      capability: capability,
      min_trust: min_trust,
      callback: callback
    }
    Logger.debug("[Mesh.OfferRouter] registered handler agent=#{agent_id} capability=#{inspect(capability)} min_trust=#{min_trust}")
    {:reply, :ok, %{state | handlers: [handler | state.handlers]}}
  end

  def handle_call({:route, offer}, _from, state) do
    required_capability = Map.get(offer, :capability, "")
    trust_score = Map.get(offer, :trust_score, 0.0)

    matching =
      Enum.filter(state.handlers, fn h ->
        (h.capability == "" or h.capability == required_capability) and
          trust_score >= h.min_trust
      end)

    Logger.debug(
      "[Mesh.OfferRouter] routing offer capability=#{inspect(required_capability)} " <>
        "trust=#{trust_score} matched_handlers=#{length(matching)}"
    )

    results =
      Enum.map(matching, fn h ->
        try do
          {h.agent_id, h.callback.(offer)}
        rescue
          e -> {h.agent_id, {:error, Exception.message(e)}}
        end
      end)

    {:reply, {:ok, results}, state}
  end

  def handle_call(:list, _from, state) do
    # Strip callbacks before returning to avoid leaking function references
    safe = Enum.map(state.handlers, &Map.delete(&1, :callback))
    {:reply, safe, state}
  end
end
