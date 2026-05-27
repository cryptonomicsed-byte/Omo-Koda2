defmodule Yemoja.YemojaTest do
  use ExUnit.Case, async: false

  setup do
    Application.ensure_started(:yemoja)
    :ok
  end

  describe "AgentWorker" do
    setup do
      id = "test-agent-#{System.unique_integer([:positive])}"
      {:ok, pid} = DynamicSupervisor.start_child(
        Yemoja.AgentSupervisor,
        {Yemoja.AgentWorker, [id: id, model: :haiku]}
      )
      on_exit(fn -> if Process.alive?(pid), do: Yemoja.AgentWorker.stop(id) end)
      %{id: id}
    end

    test "get_state returns idle by default", %{id: id} do
      assert {:ok, state} = Yemoja.AgentWorker.get_state(id)
      assert state.state == :idle
    end

    test "think returns a result", %{id: id} do
      assert {:ok, result} = Yemoja.AgentWorker.think(id, "What is the answer?")
      assert Map.has_key?(result, :thought)
    end

    test "act returns ok status", %{id: id} do
      assert {:ok, result} = Yemoja.AgentWorker.act(id, "read_file", %{path: "README.md"})
      assert result.status == :ok
    end
  end

  describe "HiveAggregator" do
    test "contribute and query public memory" do
      Yemoja.HiveAggregator.contribute("agent-1", %{content: "public insight", topic: "test"})
      Process.sleep(50)
      entries = Yemoja.HiveAggregator.query_public_memory()
      assert length(entries) >= 1
      assert Enum.any?(entries, &(&1.agent_id == "agent-1"))
    end
  end
end
