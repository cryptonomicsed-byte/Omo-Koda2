defmodule OmokodaSwarmTest do
  use ExUnit.Case

  setup do
    Application.ensure_started(:omokoda_swarm)
    Process.sleep(200)
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
    # Should have at least the initial agents
    assert length(agents) >= 3
  end

  test "scales the swarm" do
    initial_count = length(OmokodaSwarm.list_agents())

    # Scale up
    :ok = OmokodaSwarm.scale_to(initial_count + 2)
    assert length(OmokodaSwarm.list_agents()) == initial_count + 2

    # Scale down
    :ok = OmokodaSwarm.scale_to(initial_count)
    assert length(OmokodaSwarm.list_agents()) == initial_count
  end

  test "delegates task to specific agent" do
    agents = OmokodaSwarm.list_agents()
    agent_id = List.first(agents)

    task = %{type: :act, action: "test action"}
    assert :ok = OmokodaSwarm.delegate_to_agent(agent_id, task)
  end

  test "gets agent state" do
    agents = OmokodaSwarm.list_agents()
    agent_id = List.first(agents)

    assert {:ok, state} = OmokodaSwarm.agent_state(agent_id)
    assert Map.has_key?(state, :id)
    assert Map.has_key?(state, :state)
    assert Map.has_key?(state, :tasks)
  end
end
