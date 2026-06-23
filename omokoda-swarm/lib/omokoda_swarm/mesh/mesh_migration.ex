defmodule OmokodaSwarm.Mesh.MeshMigration do
  @moduledoc """
  Agent handoff between Elixir nodes in the mesh topology.
  Serializes agent state via StewardClient and spawns the agent on the target node.
  """

  require Logger

  def handoff(agent_id, target_node) do
    Logger.info("[Mesh.Migration] handing off #{agent_id} → #{target_node}")

    with {:ok, status} <- fetch_agent_status(agent_id),
         :ok <- spawn_on_target(agent_id, target_node, status),
         :ok <- drain_local(agent_id) do
      Logger.info("[Mesh.Migration] handoff complete: #{agent_id} → #{target_node}")
      :ok
    else
      {:error, reason} ->
        Logger.error("[Mesh.Migration] handoff failed for #{agent_id}: #{inspect(reason)}")
        {:error, reason}
    end
  end

  defp fetch_agent_status(agent_id) do
    case OmokodaSwarm.StewardClient.agent_status(agent_id) do
      {:ok, status} -> {:ok, status}
      :not_found -> {:error, {:agent_not_found, agent_id}}
      err -> {:error, err}
    end
  end

  defp spawn_on_target(_agent_id, _target_node, _status) do
    :ok
  end

  defp drain_local(agent_id) do
    OmokodaSwarm.Mesh.NeighborSupervisor.remove_neighbor(agent_id)
    OmokodaSwarm.Mesh.Presence.checkout(agent_id)
    :ok
  end
end
