defmodule OmokodaSwarm.Application do
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      {Registry, keys: :unique, name: OmokodaSwarm.Registry},
      OmokodaSwarm.SwarmSupervisor,
      OmokodaSwarm.Coordinator,
      OmokodaSwarm.BackendRegistry,
      OmokodaSwarm.TeammateLayoutManager,
      OmokodaSwarm.PermissionSync
    ]

    opts = [strategy: :one_for_one, name: OmokodaSwarm.Supervisor]

    case Supervisor.start_link(children, opts) do
      {:ok, pid} ->
        OmokodaSwarm.SwarmSupervisor.ensure_initial_agents()
        {:ok, pid}

      other ->
        other
    end
  end
end
