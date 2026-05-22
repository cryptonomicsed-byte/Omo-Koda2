defmodule Yemoja.AgentWorker do
  @moduledoc """
  GenServer that represents one live agent in the Yemọja swarm.

  Each worker is registered under its `agent_id` in `Yemoja.Registry`
  so that any process can locate it via `Yemoja.AgentWorker.via/1`.

  ## Sovereignty model

  Only public state travels through Elixir.  The `:private_memory` field
  intentionally does NOT exist in this struct.  Private memory is owned
  exclusively by the Rust core and never serialised into OTP messages.

  ## State

      %{
        agent_id:      binary(),        # opaque UUID string
        reputation:    non_neg_integer(),
        tier:          :observer | :participant | :steward,
        public_memory: [String.t()]     # ordered, most-recent first
      }
  """

  use GenServer

  require Logger

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc "Returns a `{:via, Registry, …}` tuple for the given agent_id."
  @spec via(binary()) :: {:via, Registry, {Yemoja.Registry, binary()}}
  def via(agent_id), do: {:via, Registry, {Yemoja.Registry, agent_id}}

  @doc """
  Starts an AgentWorker under the `Yemoja.DynamicSupervisor`.

  Returns `{:ok, pid}` or `{:error, reason}`.
  """
  @spec start_supervised(map()) :: DynamicSupervisor.on_start_child()
  def start_supervised(opts) when is_map(opts) do
    DynamicSupervisor.start_child(
      Yemoja.DynamicSupervisor,
      {__MODULE__, opts}
    )
  end

  @doc "Sends a think prompt to the agent and returns the routed result."
  @spec think(binary(), String.t(), keyword()) :: {:ok, term()} | {:error, term()}
  def think(agent_id, prompt, opts \\ []) do
    GenServer.call(via(agent_id), {:think, prompt, opts})
  end

  @doc "Routes a tool-call action to the agent."
  @spec act(binary(), atom(), map()) :: {:ok, term()} | {:error, term()}
  def act(agent_id, tool, args) do
    GenServer.call(via(agent_id), {:act, tool, args})
  end

  @doc """
  Saves a public memory snapshot and pushes it to the `HiveAggregator`.

  Returns the snapshot list.
  """
  @spec memory_checkpoint(binary()) :: {:ok, [String.t()]}
  def memory_checkpoint(agent_id) do
    GenServer.call(via(agent_id), :memory_checkpoint)
  end

  @doc "Returns the current public state of the agent (no private fields)."
  @spec get_state(binary()) :: {:ok, map()} | {:error, :not_found}
  def get_state(agent_id) do
    case Registry.lookup(Yemoja.Registry, agent_id) do
      [{pid, _}] -> {:ok, GenServer.call(pid, :get_state)}
      [] -> {:error, :not_found}
    end
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(%{agent_id: agent_id} = opts) do
    state = %{
      agent_id: agent_id,
      reputation: Map.get(opts, :reputation, 0),
      tier: Map.get(opts, :tier, :observer),
      public_memory: Map.get(opts, :public_memory, [])
    }

    Logger.info("[AgentWorker] started agent_id=#{agent_id} tier=#{state.tier}")
    {:ok, state}
  end

  @impl true
  def handle_call({:think, prompt, opts}, _from, state) do
    # In production this would delegate to the Rust core via a NIF or port.
    # Here we record the intent in public memory and return an ack.
    entry = "think:#{truncate(prompt, 120)}"
    new_memory = [entry | state.public_memory]
    new_state = %{state | public_memory: new_memory}

    result = %{
      agent_id: state.agent_id,
      routed: true,
      prompt_preview: truncate(prompt, 60),
      opts: opts
    }

    Logger.debug("[AgentWorker] #{state.agent_id} think routed")
    {:reply, {:ok, result}, new_state}
  end

  @impl true
  def handle_call({:act, tool, args}, _from, state) do
    entry = "act:#{tool}"
    new_memory = [entry | state.public_memory]
    new_state = %{state | public_memory: new_memory}

    result = %{
      agent_id: state.agent_id,
      tool: tool,
      args: args,
      routed: true
    }

    Logger.debug("[AgentWorker] #{state.agent_id} act tool=#{tool}")
    {:reply, {:ok, result}, new_state}
  end

  @impl true
  def handle_call(:memory_checkpoint, _from, state) do
    snapshot = state.public_memory

    # Push public contributions to the hive garden.
    Enum.each(snapshot, fn entry ->
      Yemoja.HiveAggregator.push_public(state.agent_id, entry)
    end)

    Logger.info("[AgentWorker] #{state.agent_id} memory_checkpoint entries=#{length(snapshot)}")
    {:reply, {:ok, snapshot}, state}
  end

  @impl true
  def handle_call(:get_state, _from, state) do
    # Strip nothing — state already contains no private fields.
    {:reply, state, state}
  end

  @impl true
  def handle_call({:receive_profile, from_agent_id, profile}, _from, state) do
    # Merge incoming public profile fields into this agent's public state.
    # Only known safe keys are merged; unknown keys are silently ignored to
    # prevent injection of unexpected fields.
    merged_memory =
      case Map.get(profile, :public_memory) do
        nil -> state.public_memory
        incoming when is_list(incoming) -> incoming ++ state.public_memory
        _ -> state.public_memory
      end

    new_reputation =
      case Map.get(profile, :reputation) do
        nil -> state.reputation
        r when is_integer(r) and r >= 0 -> r
        _ -> state.reputation
      end

    new_tier =
      case Map.get(profile, :tier) do
        t when t in [:observer, :participant, :steward] -> t
        _ -> state.tier
      end

    new_state = %{
      state
      | public_memory: merged_memory,
        reputation: new_reputation,
        tier: new_tier
    }

    Logger.info(
      "[AgentWorker] #{state.agent_id} received profile from #{from_agent_id}"
    )

    {:reply, :ok, new_state}
  end

  # ---------------------------------------------------------------------------
  # child_spec so DynamicSupervisor can start us
  # ---------------------------------------------------------------------------

  def child_spec(%{agent_id: agent_id} = opts) do
    %{
      id: {__MODULE__, agent_id},
      start: {GenServer, :start_link, [__MODULE__, opts, [name: via(agent_id)]]},
      restart: :transient,
      shutdown: 5_000,
      type: :worker
    }
  end

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp truncate(str, max) when is_binary(str) and byte_size(str) > max,
    do: binary_part(str, 0, max) <> "…"

  defp truncate(str, _max), do: str
end
