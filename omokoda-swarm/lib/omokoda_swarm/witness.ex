defmodule OmokodaSwarm.Witness do
  @moduledoc """
  Handles witness consensus for swarm operations.
  """

  @doc """
  Performs witness consensus on a task result.
  """
  def consensus(task, witnesses, threshold \\ 0.66) do
    # Get results from witnesses
    results =
      Enum.map(witnesses, fn witness_id ->
        case OmokodaSwarm.Agent.get_state(witness_id) do
          {:ok, state} ->
            # Simulate witness evaluation
            evaluate_task(task, state)

          _ ->
            nil
        end
      end)

    # Filter out nil results
    valid_results = Enum.reject(results, &is_nil/1)

    if length(valid_results) < length(witnesses) * threshold do
      {:error, :insufficient_consensus}
    else
      # Calculate consensus result
      consensus_result = calculate_consensus(valid_results)
      {:ok, consensus_result}
    end
  end

  @doc """
  Evaluates a task by a witness agent.
  """
  def evaluate_task(task, _agent_state) do
    # Simulate task evaluation
    # In a real implementation, this would involve the agent processing the task
    # and returning a result with confidence score

    confidence = :rand.uniform()
    result = %{task: task, confidence: confidence, timestamp: DateTime.utc_now()}

    result
  end

  @doc """
  Calculates consensus from multiple witness results.
  """
  def calculate_consensus(results) do
    # Simple majority/consensus calculation
    avg_confidence = Enum.reduce(results, 0, &(&1.confidence + &2)) / length(results)

    %{
      consensus_reached: avg_confidence > 0.5,
      average_confidence: avg_confidence,
      witness_count: length(results),
      timestamp: DateTime.utc_now()
    }
  end

  @doc """
  Validates a consensus result.
  """
  def validate_consensus(consensus_result, required_threshold \\ 0.66) do
    consensus_result.average_confidence >= required_threshold
  end
end
