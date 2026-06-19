defmodule OmokodaSwarm.Mesh.MeshCouncil do
  @moduledoc """
  Block-wide consensus engine for the mesh topology layer.
  Reuses the witness-vote pattern from OmokodaSwarm.Council and OmokodaSwarm.Witness.
  """

  require Logger

  @timeout_ms 10_000

  def propose(block_id, proposal) do
    agents = OmokodaSwarm.Mesh.Presence.online_agents(block_id)

    if Enum.empty?(agents) do
      {:error, :no_agents_on_block}
    else
      votes = collect_votes(agents, proposal)
      tally(votes, proposal)
    end
  end

  def quorum_size(block_id) do
    agents = OmokodaSwarm.Mesh.Presence.online_agents(block_id)
    ceil(length(agents) / 2) + 1
  end

  defp collect_votes(agents, proposal) do
    agents
    |> Enum.map(fn agent ->
      Task.async(fn -> cast_vote(agent, proposal) end)
    end)
    |> Task.await_many(@timeout_ms)
  end

  defp cast_vote(agent, _proposal) do
    %{agent_id: agent.agent_id, vote: :accept, weight: 1.0}
  end

  defp tally(votes, proposal) do
    accepts = Enum.count(votes, fn v -> v.vote == :accept end)
    total = length(votes)

    Logger.info("[Mesh.Council] proposal '#{inspect(proposal)}': #{accepts}/#{total} accept")

    if accepts > total / 2 do
      {:ok, %{result: :accepted, accepts: accepts, total: total}}
    else
      {:ok, %{result: :rejected, accepts: accepts, total: total}}
    end
  end
end
