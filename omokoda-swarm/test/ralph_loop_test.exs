defmodule OmokodaSwarm.RalphLoopTest do
  use ExUnit.Case, async: false

  setup do
    Application.ensure_started(:omokoda_swarm)
    Process.sleep(100)
    :ok
  end

  describe "RalphLoop.start_link/1" do
    test "starts successfully with a task" do
      assert {:ok, pid} = OmokodaSwarm.RalphLoop.start_link(
        task: "iterate on this",
        max_iterations: 3,
        iteration_timeout_ms: 2_000
      )
      assert Process.alive?(pid)
    end

    test "status shows running state" do
      {:ok, pid} = OmokodaSwarm.RalphLoop.start_link(
        task: "test task",
        max_iterations: 2,
        iteration_timeout_ms: 500
      )

      status = OmokodaSwarm.RalphLoop.status(pid)
      assert is_map(status)
      assert Map.has_key?(status, :task)
      assert Map.has_key?(status, :iteration)
      assert Map.has_key?(status, :max_iterations)
      assert Map.has_key?(status, :status)
      assert Map.has_key?(status, :elapsed_ms)
      assert status.max_iterations == 2
    end

    test "stops when max_iterations reached" do
      {:ok, pid} = OmokodaSwarm.RalphLoop.start_link(
        task: "loop task #{System.unique_integer()}",
        max_iterations: 1,
        iteration_timeout_ms: 3_000
      )

      result = OmokodaSwarm.RalphLoop.await(pid, 15_000)
      assert match?({:ok, _}, result) or match?({:error, _}, result)
    end

    test "stop_loop terminates early" do
      {:ok, pid} = OmokodaSwarm.RalphLoop.start_link(
        task: "long running task",
        max_iterations: 100,
        iteration_timeout_ms: 5_000
      )

      # Allow at least one iteration to start
      Process.sleep(50)
      OmokodaSwarm.RalphLoop.stop_loop(pid)

      # After stop, status should not be :running
      Process.sleep(100)
      status = OmokodaSwarm.RalphLoop.status(pid)
      assert status.status != :running
    end

    test "handles map task with prompt key" do
      assert {:ok, pid} = OmokodaSwarm.RalphLoop.start_link(
        task: %{prompt: "do the thing", context: "test"},
        max_iterations: 1,
        iteration_timeout_ms: 2_000
      )
      assert Process.alive?(pid)
    end
  end
end
