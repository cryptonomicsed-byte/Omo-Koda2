defmodule OmokodaSwarm.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Registry, keys: :unique, name: OmokodaSwarm.Registry},
      OmokodaSwarm.TelemetryHub,
      OmokodaSwarm.SwarmSupervisor,
      OmokodaSwarm.Coordinator
    ]

    opts = [strategy: :one_for_one, name: OmokodaSwarm.Supervisor]
    Supervisor.start_link(children, opts)
  end
end