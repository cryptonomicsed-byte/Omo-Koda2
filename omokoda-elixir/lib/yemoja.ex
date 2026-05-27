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
    children = [
      {Registry, keys: :unique, name: Yemoja.Registry},
      {DynamicSupervisor, strategy: :one_for_one, name: Yemoja.AgentSupervisor},
      Yemoja.HiveAggregator,
    ]
    opts = [strategy: :one_for_one, name: Yemoja.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
