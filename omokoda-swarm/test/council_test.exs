defmodule OmokodaSwarm.CouncilTest do
  use ExUnit.Case, async: false

  setup do
    Application.ensure_started(:omokoda_swarm)
    Process.sleep(100)
    :ok
  end

  describe "Council.start_link/1" do
    test "starts successfully with valid councillors" do
      councillors = [
        %{id: "test_bug_agent_1", role: :bugs, weight: 1.0},
        %{id: "test_compliance_agent_1", role: :compliance, weight: 0.9}
      ]

      assert {:ok, pid} = OmokodaSwarm.Council.start_link(
        task: "review test code",
        councillors: councillors,
        timeout_ms: 5_000
      )

      assert Process.alive?(pid)
    end

    test "tally shows pending councillors" do
      councillors = [%{id: "tally_agent_1", role: :bugs, weight: 1.0}]
      {:ok, pid} = OmokodaSwarm.Council.start_link(
        task: "some task",
        councillors: councillors,
        timeout_ms: 200
      )

      tally = OmokodaSwarm.Council.tally(pid)
      assert is_map(tally)
      assert Map.has_key?(tally, :task)
      assert Map.has_key?(tally, :opinions_received)
      assert Map.has_key?(tally, :pending)
      assert Map.has_key?(tally, :status)
    end

    test "await returns verdict map after councillors respond" do
      councillors = [
        %{id: "fast_agent_#{System.unique_integer()}", role: :bugs, weight: 1.0}
      ]

      {:ok, pid} = OmokodaSwarm.Council.start_link(
        task: "quick review",
        councillors: councillors,
        timeout_ms: 3_000
      )

      {:ok, verdict} = OmokodaSwarm.Council.await(pid, 10_000)
      assert is_map(verdict)
      assert Map.has_key?(verdict, :summary)
      assert Map.has_key?(verdict, :findings)
      assert Map.has_key?(verdict, :highest_severity)
      assert Map.has_key?(verdict, :confidence_score)
    end

    test "handles empty task gracefully" do
      councillors = [%{id: "empty_agent_#{System.unique_integer()}", role: :comments, weight: 0.5}]
      assert {:ok, _pid} = OmokodaSwarm.Council.start_link(
        task: "",
        councillors: councillors,
        timeout_ms: 2_000
      )
    end
  end
end
