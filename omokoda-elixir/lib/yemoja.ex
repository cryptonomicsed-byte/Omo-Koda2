defmodule Yemoja do
  @moduledoc """
  Yemọja — the OTP swarm coordination layer for Ọmọ Kọ́dà.

  Yemọja is the goddess of the ocean and the mother of rivers.
  In Ọmọ Kọ́dà, she coordinates the public-facing swarm intelligence:
  agent lifecycle, message routing, public memory aggregation, and gRPC ingress.

  ## Sovereignty model

  Private memory NEVER flows through Elixir.  It lives and dies inside the Rust
  core per each agent's sovereignty.  Everything Yemọja touches is either
  public or ephemeral coordination metadata.
  """

  @grpc_port 50051

  @doc "Returns the configured gRPC port (default 50051)."
  def grpc_port, do: Application.get_env(:yemoja, :grpc_port, @grpc_port)
end

defmodule Yemoja.Application do
  @moduledoc """
  OTP Application entry-point for the Yemọja swarm layer.

  Supervision tree:

      Yemoja.Application
      ├── Yemoja.Registry          — ETS-backed Registry keyed by agent_id
      ├── Yemoja.DynamicSupervisor — one AgentWorker per live agent
      ├── Yemoja.HiveAggregator    — public memory garden (ETS)
      └── Yemoja.GRPC.Endpoint     — gRPC server on port 50051
  """

  use Application

  @impl true
  def start(_type, _args) do
    port = Yemoja.grpc_port()

    children = [
      # Erlang :pg scope for gRPC stream fan-out (built-in OTP ≥ 23, no extra dep).
      %{
        id: :yemoja_pg,
        start: {:pg, :start_link, [:yemoja_swarm_events]},
        type: :worker
      },

      # Registry for O(1) agent lookup by string ID.
      {Registry, keys: :unique, name: Yemoja.Registry},

      # DynamicSupervisor that owns all AgentWorker children.
      {DynamicSupervisor, strategy: :one_for_one, name: Yemoja.DynamicSupervisor},

      # Singleton aggregator for the public memory garden.
      Yemoja.HiveAggregator,

      # gRPC endpoint — last so that infra is ready before we accept traffic.
      {GRPC.Server.Supervisor, {Yemoja.GRPC.SwarmServer, port}}
    ]

    opts = [strategy: :one_for_one, name: Yemoja.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
