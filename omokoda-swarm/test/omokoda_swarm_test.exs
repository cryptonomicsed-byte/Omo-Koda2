defmodule OmokodaSwarmTest do
  use ExUnit.Case, async: false

  # ---------------------------------------------------------------------------
  # Swarm integration tests (require the application to be running)
  # ---------------------------------------------------------------------------

  setup do
    Application.ensure_all_started(:omokoda_swarm)
    :ok
  end

  test "submits a task to the swarm" do
    task = %{type: :think, prompt: "Test task"}
    assert {:ok, task_id} = OmokodaSwarm.submit_task(task)
    assert is_binary(task_id)
  end

  test "gets swarm status" do
    status = OmokodaSwarm.status()
    assert is_map(status)
    assert Map.has_key?(status, :active_agents)
    assert Map.has_key?(status, :active_tasks)
    assert Map.has_key?(status, :agent_statuses)
  end

  test "lists agents" do
    agents = OmokodaSwarm.list_agents()
    assert is_list(agents)
    assert length(agents) >= 3
  end

  test "scales the swarm" do
    initial_count = length(OmokodaSwarm.list_agents())

    :ok = OmokodaSwarm.scale_to(initial_count + 2)
    assert length(OmokodaSwarm.list_agents()) == initial_count + 2

    :ok = OmokodaSwarm.scale_to(initial_count)
    assert length(OmokodaSwarm.list_agents()) == initial_count
  end

  test "delegates task to specific agent" do
    [agent_id | _] = OmokodaSwarm.list_agents()
    task = %{type: :act, tool: "echo", params: "{}"}
    assert :ok = OmokodaSwarm.delegate_to_agent(agent_id, task)
  end

  test "gets agent state" do
    [agent_id | _] = OmokodaSwarm.list_agents()
    assert {:ok, state} = OmokodaSwarm.agent_state(agent_id)
    assert Map.has_key?(state, :id)
    assert Map.has_key?(state, :state)
    assert Map.has_key?(state, :tasks)
  end

  # ---------------------------------------------------------------------------
  # Witness consensus — pure unit tests, no live agents required
  # ---------------------------------------------------------------------------

  describe "Witness.requires_quorum?" do
    test "true for tier 4 and above" do
      assert OmokodaSwarm.Witness.requires_quorum?(4)
      assert OmokodaSwarm.Witness.requires_quorum?(5)
      assert OmokodaSwarm.Witness.requires_quorum?(10)
    end

    test "false for tier below 4" do
      refute OmokodaSwarm.Witness.requires_quorum?(3)
      refute OmokodaSwarm.Witness.requires_quorum?(0)
    end
  end

  describe "Witness.consensus" do
    test "returns insufficient_witnesses when fewer than 5 alive for tier 4+" do
      assert {:error, :insufficient_witnesses} =
               OmokodaSwarm.Witness.consensus(%{type: :act}, ["ghost_1", "ghost_2"], 4)
    end

    test "advisory consensus succeeds with one live witness for tier 0" do
      agent_id = "witness_test_#{System.unique_integer([:positive])}"
      OmokodaSwarm.SwarmSupervisor.start_agent(agent_id)
      on_exit(fn -> OmokodaSwarm.SwarmSupervisor.stop_agent(agent_id) end)

      assert {:ok, result} =
               OmokodaSwarm.Witness.consensus(%{type: :think}, [agent_id], 0)

      assert is_boolean(result.approved)
      assert is_integer(result.total_votes)
    end
  end

  describe "Witness.cleared?" do
    test "tier 4+ requires both approved and quorum_met" do
      assert OmokodaSwarm.Witness.cleared?(%{approved: true, quorum_met: true}, 4)
      refute OmokodaSwarm.Witness.cleared?(%{approved: true, quorum_met: false}, 4)
      refute OmokodaSwarm.Witness.cleared?(%{approved: false, quorum_met: true}, 4)
    end

    test "tier < 4 only requires approved" do
      assert OmokodaSwarm.Witness.cleared?(%{approved: true, quorum_met: false}, 3)
      assert OmokodaSwarm.Witness.cleared?(%{approved: true, quorum_met: false}, 0)
      refute OmokodaSwarm.Witness.cleared?(%{approved: false, quorum_met: true}, 3)
    end
  end

  # ---------------------------------------------------------------------------
  # TelemetryHub — pub/sub unit tests
  # ---------------------------------------------------------------------------

  describe "TelemetryHub" do
    test "subscribe then publish delivers event to subscriber" do
      channel = "test_dev_#{System.unique_integer([:positive])}"
      OmokodaSwarm.TelemetryHub.subscribe(channel)
      OmokodaSwarm.TelemetryHub.publish(channel, %{temp: 42.1})
      assert_receive {:telemetry, ^channel, %{temp: 42.1}}, 500
      OmokodaSwarm.TelemetryHub.unsubscribe(channel)
    end

    test "subscriber_count increments on subscribe" do
      channel = "test_cnt_#{System.unique_integer([:positive])}"
      before = OmokodaSwarm.TelemetryHub.subscriber_count()
      OmokodaSwarm.TelemetryHub.subscribe(channel)
      assert OmokodaSwarm.TelemetryHub.subscriber_count() >= before + 1
      OmokodaSwarm.TelemetryHub.unsubscribe(channel)
    end

    test "channels includes active subscription" do
      channel = "test_ch_#{System.unique_integer([:positive])}"
      OmokodaSwarm.TelemetryHub.subscribe(channel)
      assert channel in OmokodaSwarm.TelemetryHub.channels()
      OmokodaSwarm.TelemetryHub.unsubscribe(channel)
    end

    test "no message received after unsubscribe" do
      channel = "test_unsub_#{System.unique_integer([:positive])}"
      OmokodaSwarm.TelemetryHub.subscribe(channel)
      OmokodaSwarm.TelemetryHub.unsubscribe(channel)
      OmokodaSwarm.TelemetryHub.publish(channel, %{val: 1})
      refute_receive {:telemetry, ^channel, _}, 100
    end
  end

  # ---------------------------------------------------------------------------
  # StewardClient — graceful degradation when Rust Steward is offline
  # ---------------------------------------------------------------------------

  describe "StewardClient" do
    test "health returns error when Steward is not running" do
      assert {:error, _} = OmokodaSwarm.StewardClient.health()
    end

    test "birth returns steward_unavailable error when offline" do
      assert {:error, {:http_error, _}} = OmokodaSwarm.StewardClient.birth("test_agent")
    end
  end
end
