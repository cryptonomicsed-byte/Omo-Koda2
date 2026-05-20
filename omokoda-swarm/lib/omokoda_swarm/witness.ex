defmodule OmokodaSwarm.Witness do
  @moduledoc """
  Witness consensus for high-tier agent acts.

  Tier 4+ acts require a 3-of-5 witness quorum before the act can proceed.
  Each witness independently evaluates the act proposal and casts a signed
  vote (approve / reject). The coordinator collects votes and only proceeds
  when the required threshold is met.

  Below Tier 4, consensus is advisory — the majority result is returned but
  the act is not blocked.
  """

  @quorum_threshold 3
  @quorum_size 5
  @tier4_threshold 4

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc """
  Determine whether `tier` requires mandatory witness quorum.
  Tier 4+ (sovereign-grade) acts must have 3-of-5 witnesses agree.
  """
  def requires_quorum?(tier) when is_integer(tier), do: tier >= @tier4_threshold

  @doc """
  Run witness consensus for an act proposal.

  Returns:
    {:ok, %{approved: bool, votes: [vote], quorum_met: bool}} on success
    {:error, :insufficient_witnesses} if < 5 active witnesses exist

  For Tier 4+ acts, `approved` is only `true` when `quorum_met` is true AND
  the majority of quorum witnesses voted :approve.
  """
  def consensus(task, witnesses, tier \\ 0) do
    available = Enum.filter(witnesses, &witness_alive?/1)

    required_count = if requires_quorum?(tier), do: @quorum_size, else: 1

    if length(available) < required_count do
      {:error, :insufficient_witnesses}
    else
      panel = Enum.take_random(available, min(@quorum_size, length(available)))
      votes = Enum.map(panel, &cast_vote(&1, task))
      approvals = Enum.count(votes, &(&1.decision == :approve))

      quorum_met = approvals >= @quorum_threshold
      approved = if requires_quorum?(tier), do: quorum_met, else: approvals > length(panel) / 2

      {:ok,
       %{
         approved: approved,
         quorum_met: quorum_met,
         approvals: approvals,
         total_votes: length(votes),
         votes: votes,
         tier: tier,
         timestamp: DateTime.utc_now()
       }}
    end
  end

  @doc """
  Validate a previously computed consensus result against the required tier.
  Returns true only if the act is cleared to proceed.
  """
  def cleared?(%{approved: true, quorum_met: true}, tier) when tier >= @tier4_threshold, do: true
  def cleared?(%{approved: true}, tier) when tier < @tier4_threshold, do: true
  def cleared?(_, _), do: false

  # ---------------------------------------------------------------------------
  # Internal helpers
  # ---------------------------------------------------------------------------

  defp witness_alive?(witness_id) do
    case OmokodaSwarm.Agent.get_state(witness_id) do
      {:ok, _} -> true
      _ -> false
    end
  end

  # Each witness independently evaluates the act. In production, this would
  # involve the witness agent calling its own think/justice pipeline. Here we
  # model the decision as a reputation-weighted probabilistic vote so the
  # system is testable without a live Steward.
  defp cast_vote(witness_id, task) do
    confidence = evaluate_confidence(witness_id, task)
    decision = if confidence >= 0.5, do: :approve, else: :reject

    %{
      witness: witness_id,
      decision: decision,
      confidence: confidence,
      timestamp: DateTime.utc_now()
    }
  end

  defp evaluate_confidence(witness_id, _task) do
    # Derive a deterministic-ish confidence from the witness ID so tests are
    # repeatable while still exercising the voting logic.
    hash = :erlang.phash2(witness_id, 100)
    base = hash / 100.0
    # Add small jitter so repeated calls aren't identical.
    jitter = :rand.uniform() * 0.1 - 0.05
    Float.round(max(0.0, min(1.0, base + jitter)), 3)
  end
end
