defmodule YemojaTest do
  @moduledoc """
  ExUnit test suite for the Yemọja OTP swarm coordination layer.

  Tests are organised into four describe blocks matching the four behaviours
  specified in the Ọmọ Kọ́dà design brief:

    1. AgentWorker — starts and responds to :memory_checkpoint
    2. HiveAggregator — accumulates contributions from multiple agents
    3. ProfileSync — hard fails when :private_memory key is present
    4. DynamicSupervisor — restarts crashed agent workers
  """

  use ExUnit.Case, async: false

  alias Yemoja.AgentWorker
  alias Yemoja.HiveAggregator
  alias Yemoja.ProfileSync

  # Ensure a clean ETS state before each test.
  setup do
    # Clear the public garden so tests don't bleed into each other.
    HiveAggregator.clear()
    :ok
  end

  # ---------------------------------------------------------------------------
  # 1. AgentWorker
  # ---------------------------------------------------------------------------

  describe "AgentWorker" do
    test "starts successfully and is registered in Yemoja.Registry" do
      agent_id = unique_id()
      assert {:ok, pid} = AgentWorker.start_supervised(%{agent_id: agent_id})
      assert is_pid(pid)
      assert Process.alive?(pid)

      # Registry lookup must find it by ID.
      assert [{^pid, _}] = Registry.lookup(Yemoja.Registry, agent_id)
    end

    test "responds to :memory_checkpoint with an ok tuple and a list" do
      agent_id = unique_id()
      {:ok, _pid} = AgentWorker.start_supervised(%{agent_id: agent_id})

      # Do some work to populate memory.
      {:ok, _} = AgentWorker.think(agent_id, "What is the nature of light?")
      {:ok, _} = AgentWorker.act(agent_id, :search, %{"query" => "photon"})

      assert {:ok, entries} = AgentWorker.memory_checkpoint(agent_id)
      assert is_list(entries)
      # Both interactions should have created entries.
      assert length(entries) == 2
    end

    test "memory_checkpoint pushes entries to HiveAggregator" do
      agent_id = unique_id()
      {:ok, _pid} = AgentWorker.start_supervised(%{agent_id: agent_id})

      {:ok, _} = AgentWorker.think(agent_id, "River flows to the sea")
      AgentWorker.memory_checkpoint(agent_id)

      # Allow cast to complete.
      Process.sleep(20)

      garden = HiveAggregator.get_garden()
      assert Map.has_key?(garden, agent_id)
      assert length(garden[agent_id]) >= 1
    end

    test "think returns a routed result containing the agent_id" do
      agent_id = unique_id()
      {:ok, _pid} = AgentWorker.start_supervised(%{agent_id: agent_id})

      assert {:ok, result} = AgentWorker.think(agent_id, "Hello, Yemọja")
      assert result.agent_id == agent_id
      assert result.routed == true
    end

    test "act returns a routed result with the tool name" do
      agent_id = unique_id()
      {:ok, _pid} = AgentWorker.start_supervised(%{agent_id: agent_id})

      assert {:ok, result} = AgentWorker.act(agent_id, :fetch, %{"url" => "https://example.com"})
      assert result.tool == :fetch
      assert result.routed == true
    end

    test "get_state returns public state without private_memory key" do
      agent_id = unique_id()
      {:ok, _} = AgentWorker.start_supervised(%{agent_id: agent_id, tier: :participant})

      assert {:ok, state} = AgentWorker.get_state(agent_id)
      assert state.agent_id == agent_id
      assert state.tier == :participant
      refute Map.has_key?(state, :private_memory)
    end

    test "get_state returns :not_found for an unknown agent" do
      assert {:error, :not_found} = AgentWorker.get_state("does-not-exist-#{:rand.uniform(999_999)}")
    end
  end

  # ---------------------------------------------------------------------------
  # 2. HiveAggregator
  # ---------------------------------------------------------------------------

  describe "HiveAggregator" do
    test "accumulates contributions from a single agent" do
      agent_id = unique_id()

      HiveAggregator.push_public(agent_id, "Wave 1")
      HiveAggregator.push_public(agent_id, "Wave 2")
      HiveAggregator.push_public(agent_id, "Wave 3")

      # Allow async casts to land.
      Process.sleep(20)

      entries = HiveAggregator.get_agent_entries(agent_id)
      assert length(entries) == 3
      assert "Wave 1" in entries
      assert "Wave 3" in entries
    end

    test "accumulates contributions from multiple independent agents" do
      ids = for _ <- 1..5, do: unique_id()

      for {id, i} <- Enum.with_index(ids) do
        HiveAggregator.push_public(id, "entry-#{i}-a")
        HiveAggregator.push_public(id, "entry-#{i}-b")
      end

      Process.sleep(30)

      garden = HiveAggregator.get_garden()

      for id <- ids do
        assert Map.has_key?(garden, id), "expected garden to contain agent #{id}"
        assert length(garden[id]) == 2
      end
    end

    test "get_agent_entries returns empty list for unknown agent" do
      assert [] = HiveAggregator.get_agent_entries("unknown-#{:rand.uniform(999_999)}")
    end

    test "clear/0 empties the garden" do
      agent_id = unique_id()
      HiveAggregator.push_public(agent_id, "to be forgotten")
      Process.sleep(20)

      HiveAggregator.clear()

      Process.sleep(10)
      assert %{} = HiveAggregator.get_garden()
    end
  end

  # ---------------------------------------------------------------------------
  # 3. ProfileSync
  # ---------------------------------------------------------------------------

  describe "ProfileSync" do
    test "validate_no_private_keys/1 returns :ok for a clean profile" do
      profile = %{reputation: 42, tier: :participant, public_memory: ["hello"]}
      assert :ok = ProfileSync.validate_no_private_keys(profile)
    end

    test "hard fails when atom key :private_memory is present" do
      profile = %{reputation: 0, private_memory: ["secret"]}
      assert {:error, :private_key_detected} = ProfileSync.validate_no_private_keys(profile)
    end

    test "hard fails when string key \"private_memory\" is present" do
      profile = %{"reputation" => 0, "private_memory" => ["secret"]}
      assert {:error, :private_key_detected} = ProfileSync.validate_no_private_keys(profile)
    end

    test "sync_profile/3 rejects transfer when profile contains :private_memory" do
      from_id = unique_id()
      to_id = unique_id()

      {:ok, _} = AgentWorker.start_supervised(%{agent_id: from_id})
      {:ok, _} = AgentWorker.start_supervised(%{agent_id: to_id})

      profile = %{reputation: 10, private_memory: ["DO NOT PASS"]}

      assert {:error, :private_key_detected} =
               ProfileSync.sync_profile(from_id, to_id, profile)
    end

    test "sync_profile/3 returns :from_agent_not_found when source is unregistered" do
      to_id = unique_id()
      {:ok, _} = AgentWorker.start_supervised(%{agent_id: to_id})

      assert {:error, :from_agent_not_found} =
               ProfileSync.sync_profile("ghost-#{unique_id()}", to_id, %{reputation: 5})
    end

    test "sync_profile/3 returns :to_agent_not_found when target is unregistered" do
      from_id = unique_id()
      {:ok, _} = AgentWorker.start_supervised(%{agent_id: from_id})

      assert {:error, :to_agent_not_found} =
               ProfileSync.sync_profile(from_id, "ghost-#{unique_id()}", %{reputation: 5})
    end

    test "sync_profile/3 succeeds and merges public profile into target agent" do
      from_id = unique_id()
      to_id = unique_id()

      {:ok, _} = AgentWorker.start_supervised(%{agent_id: from_id, tier: :steward, reputation: 100})
      {:ok, _} = AgentWorker.start_supervised(%{agent_id: to_id, tier: :observer, reputation: 0})

      profile = %{
        reputation: 50,
        tier: :participant,
        public_memory: ["I carry the wisdom of the river"]
      }

      assert :ok = ProfileSync.sync_profile(from_id, to_id, profile)

      {:ok, state} = AgentWorker.get_state(to_id)
      assert state.reputation == 50
      assert state.tier == :participant
      assert "I carry the wisdom of the river" in state.public_memory
    end
  end

  # ---------------------------------------------------------------------------
  # 4. DynamicSupervisor restarts crashed agent workers
  # ---------------------------------------------------------------------------

  describe "DynamicSupervisor restart behaviour" do
    test "crashed AgentWorker is restarted and re-registered in the Registry" do
      agent_id = unique_id()
      {:ok, pid1} = AgentWorker.start_supervised(%{agent_id: agent_id})

      # Monitor the original process so we know when it's truly gone.
      ref = Process.monitor(pid1)

      # Kill the worker.
      Process.exit(pid1, :kill)

      # Wait for the DOWN signal confirming the original process died.
      assert_receive {:DOWN, ^ref, :process, ^pid1, :killed}, 1_000

      # Give the supervisor a moment to restart the child.
      Process.sleep(200)

      # A new PID should now be in the Registry.
      case Registry.lookup(Yemoja.Registry, agent_id) do
        [{pid2, _}] ->
          assert pid2 != pid1, "expected a new PID after restart"
          assert Process.alive?(pid2)

        [] ->
          flunk("AgentWorker was not restarted by DynamicSupervisor")
      end
    end

    test "restarted worker correctly responds to memory_checkpoint" do
      agent_id = unique_id()
      {:ok, pid1} = AgentWorker.start_supervised(%{agent_id: agent_id})

      ref = Process.monitor(pid1)
      Process.exit(pid1, :kill)
      assert_receive {:DOWN, ^ref, :process, ^pid1, :killed}, 1_000

      Process.sleep(200)

      assert {:ok, entries} = AgentWorker.memory_checkpoint(agent_id)
      assert is_list(entries)
    end
  end

  # ---------------------------------------------------------------------------
  # Helpers
  # ---------------------------------------------------------------------------

  defp unique_id do
    "agent-#{System.unique_integer([:positive, :monotonic])}"
  end
end
