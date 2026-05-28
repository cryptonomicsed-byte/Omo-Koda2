defmodule Yemoja.GRPC.SwarmServer do
  @moduledoc """
  gRPC server implementation for `omokoda.swarm.SwarmService`.

  This module is the transport boundary.  It translates incoming gRPC
  requests into calls against the OTP layer (AgentWorker, HiveAggregator)
  and maps results back to protobuf response messages.

  ## Generated modules

  The protobuf message modules below (`Omokoda.Swarm.*`) are generated at
  compile-time by `protoc` using the `protobuf` plugin, sourced from
  `proto/swarm.proto`.  Run:

      mix grpc.gen.proto proto/swarm.proto

  to regenerate after changing the proto file.

  ## Streaming

  `SubscribeSwarmUpdates` uses Erlang `:pg` process groups (built into OTP ≥ 23,
  no external deps required).  Any process can broadcast swarm events via
  `Yemoja.GRPC.SwarmServer.broadcast/2`.  Each open stream loop process
  subscribes to the appropriate `:pg` group and receives `{:swarm_update, %SwarmUpdate{}}` messages.
  """

  use GRPC.Server, service: Omokoda.Swarm.SwarmService.Service

  require Logger

  alias Omokoda.Swarm.{
    SpawnResponse,
    MessageResponse,
    QueryResponse,
    AgentMemoryEntries,
    SwarmUpdate
  }

  @pg_scope :yemoja_swarm_events

  # ---------------------------------------------------------------------------
  # Public broadcast API (called from AgentWorker, HiveAggregator, etc.)
  # ---------------------------------------------------------------------------

  @doc """
  Broadcasts a `SwarmUpdate` to all subscribers for `agent_id`.

  Sends to both the per-agent group and the catch-all group so that
  `:all` subscribers also receive the event.
  """
  @spec broadcast(binary(), SwarmUpdate.t()) :: :ok
  def broadcast(agent_id, %SwarmUpdate{} = update) do
    msg = {:swarm_update, update}

    for group <- [pg_group(agent_id), pg_group(:all)] do
      members = :pg.get_members(@pg_scope, group)
      Enum.each(members, &send(&1, msg))
    end

    :ok
  end

  # ---------------------------------------------------------------------------
  # RPC: SpawnAgent
  # ---------------------------------------------------------------------------

  @impl true
  def spawn_agent(request, _stream) do
    agent_id =
      if request.agent_id != "" do
        request.agent_id
      else
        generate_id()
      end

    tier = proto_tier_to_atom(request.tier)

    case Yemoja.AgentWorker.start_supervised(%{agent_id: agent_id, tier: tier}) do
      {:ok, _pid} ->
        Logger.info("[SwarmServer] spawned agent_id=#{agent_id}")
        SpawnResponse.new(agent_id: agent_id, success: true)

      {:error, {:already_started, _}} ->
        SpawnResponse.new(agent_id: agent_id, success: true)

      {:error, reason} ->
        Logger.warning("[SwarmServer] spawn failed reason=#{inspect(reason)}")
        SpawnResponse.new(success: false, error: inspect(reason))
    end
  end

  # ---------------------------------------------------------------------------
  # RPC: SendMessage
  # ---------------------------------------------------------------------------

  @impl true
  def send_message(request, _stream) do
    result =
      case request.payload do
        {:think, %{prompt: prompt, opts: opts}} ->
          Yemoja.AgentWorker.think(request.agent_id, prompt, Enum.to_list(opts))

        {:act, %{tool: tool, args: args}} ->
          Yemoja.AgentWorker.act(request.agent_id, String.to_existing_atom(tool), args)

        _ ->
          {:error, :unknown_payload}
      end

    case result do
      {:ok, data} ->
        encoded = data |> inspect() |> to_string()
        MessageResponse.new(success: true, result: encoded)

      {:error, reason} ->
        MessageResponse.new(success: false, error: inspect(reason))
    end
  rescue
    e ->
      Logger.error("[SwarmServer] send_message crashed #{inspect(e)}")
      MessageResponse.new(success: false, error: Exception.message(e))
  end

  # ---------------------------------------------------------------------------
  # RPC: QueryPublicMemory
  # ---------------------------------------------------------------------------

  @impl true
  def query_public_memory(request, _stream) do
    raw_garden =
      if request.agent_id != "" do
        entries = Yemoja.HiveAggregator.get_agent_entries(request.agent_id)
        %{request.agent_id => entries}
      else
        Yemoja.HiveAggregator.get_garden()
      end

    proto_garden =
      Enum.into(raw_garden, %{}, fn {agent_id, entries} ->
        limited =
          if request.limit > 0 do
            Enum.take(entries, request.limit)
          else
            entries
          end

        {agent_id, AgentMemoryEntries.new(entries: limited)}
      end)

    QueryResponse.new(garden: proto_garden)
  end

  # ---------------------------------------------------------------------------
  # RPC: SubscribeSwarmUpdates  (server-streaming)
  # ---------------------------------------------------------------------------

  @impl true
  def subscribe_swarm_updates(request, stream) do
    # Join the :pg process group for either all-agents or a specific agent.
    group = pg_group(if request.agent_id == "", do: :all, else: request.agent_id)
    :ok = :pg.join(@pg_scope, group, self())

    Logger.info("[SwarmServer] stream opened agent_filter=#{inspect(request.agent_id)}")

    try do
      stream_loop(stream, request.event_types)
    after
      :pg.leave(@pg_scope, group, self())
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp stream_loop(stream, event_type_filter) do
    receive do
      {:swarm_update, update} ->
        if passes_filter?(update, event_type_filter) do
          GRPC.Server.send_reply(stream, update)
        end

        stream_loop(stream, event_type_filter)

      :stop ->
        :ok
    after
      30_000 ->
        # Send a keepalive heartbeat so the client knows the stream is alive.
        heartbeat =
          SwarmUpdate.new(
            agent_id: "",
            event_type: :SWARM_EVENT_UNSPECIFIED,
            payload: "heartbeat",
            timestamp: System.system_time(:millisecond)
          )

        GRPC.Server.send_reply(stream, heartbeat)
        stream_loop(stream, event_type_filter)
    end
  end

  defp passes_filter?(_update, []), do: true
  defp passes_filter?(update, filter), do: update.event_type in filter

  defp pg_group(:all), do: :swarm_all
  defp pg_group(agent_id) when is_binary(agent_id), do: {:swarm_agent, agent_id}

  defp proto_tier_to_atom(1), do: :observer
  defp proto_tier_to_atom(2), do: :participant
  defp proto_tier_to_atom(3), do: :steward
  defp proto_tier_to_atom(_), do: :observer

  # Generates a simple unique ID using OTP primitives (no UUID dep).
  defp generate_id do
    :crypto.strong_rand_bytes(16)
    |> Base.encode16(case: :lower)
  end
end
