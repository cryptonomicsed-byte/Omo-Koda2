defmodule Yemoja do
  @moduledoc """
  YEMỌJA — Production-grade swarm coordination layer.
  OTP Application with DynamicSupervisor, Registry, and agent workers.
  """

  def version, do: "0.1.0"
end

defmodule Yemoja.Application do
  use Application

  @impl true
  def start(_type, _args) do
    http_port = System.get_env("YEMOJA_HTTP_PORT", "4001") |> String.to_integer()

    children = [
      {Registry, keys: :unique, name: Yemoja.Registry},
      {DynamicSupervisor, strategy: :one_for_one, name: Yemoja.AgentSupervisor},
      Yemoja.HiveAggregator,
      {Plug.Cowboy, scheme: :http, plug: Yemoja.Router, port: http_port},
    ]
    opts = [strategy: :one_for_one, name: Yemoja.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
