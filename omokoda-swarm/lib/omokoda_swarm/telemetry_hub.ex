defmodule OmokodaSwarm.TelemetryHub do
  @moduledoc """
  Real-time device telemetry pub/sub built on OTP's Registry (dispatch mode).

  Channels are arbitrary strings — typically device IDs or event categories.
  Any process can subscribe to a channel and receive `{:telemetry, channel, event}`
  messages. Multiple subscribers per channel are supported.

  Usage:

      # Subscribe the current process to a device channel
      OmokodaSwarm.TelemetryHub.subscribe("dev-abc123")

      # Publish an event to all subscribers of that channel
      OmokodaSwarm.TelemetryHub.publish("dev-abc123", %{status: :active, temp: 42.1})

      # In a receive/handle_info block:
      def handle_info({:telemetry, channel, event}, state) do
        ...
      end

  The hub is supervised via the application supervisor and requires no
  external dependencies.
  """

  @registry OmokodaSwarm.TelemetryRegistry

  # ---------------------------------------------------------------------------
  # Child spec — returns the Registry child spec for the supervisor.
  # ---------------------------------------------------------------------------

  def child_spec(_opts) do
    # Override id so it doesn't conflict with the unique Registry in the same supervisor.
    %{Registry.child_spec(keys: :duplicate, name: @registry) | id: __MODULE__}
  end

  # ---------------------------------------------------------------------------
  # Pub/sub API
  # ---------------------------------------------------------------------------

  @doc "Subscribe the calling process to `channel`."
  def subscribe(channel) when is_binary(channel) do
    {:ok, _} = Registry.register(@registry, channel, [])
    :ok
  end

  @doc "Unsubscribe the calling process from `channel`."
  def unsubscribe(channel) when is_binary(channel) do
    Registry.unregister(@registry, channel)
  end

  @doc "Publish `event` to all subscribers of `channel`."
  def publish(channel, event) when is_binary(channel) do
    Registry.dispatch(@registry, channel, fn entries ->
      for {pid, _} <- entries, do: send(pid, {:telemetry, channel, event})
    end)
  end

  @doc "List all channels that currently have at least one subscriber."
  def channels do
    Registry.select(@registry, [{{:"$1", :_, :_}, [], [:"$1"]}])
    |> Enum.uniq()
  end

  @doc "Number of active subscribers across all channels."
  def subscriber_count do
    Registry.select(@registry, [{{:_, :_, :_}, [], [true]}]) |> length()
  end
end
