defmodule OmokodaSwarm.Memory.RemCycleTest do
  use ExUnit.Case, async: true

  alias OmokodaSwarm.Memory.RemCycle

  describe "sabbath?/1" do
    test "Saturday is the Sabbath, matching the Rust rhythm gate" do
      # 1970-01-03 was the first Saturday after the unix epoch — the same
      # anchor the Rust dream engine tests use.
      assert RemCycle.sabbath?(~D[1970-01-03])
      assert RemCycle.sabbath?(~D[2026-07-11])
      refute RemCycle.sabbath?(~D[1970-01-02]), "Friday is not"
      refute RemCycle.sabbath?(~D[1970-01-04]), "Sunday is not"
      refute RemCycle.sabbath?(~D[2026-07-10])
    end
  end

  describe "due?/2" do
    test "due on the Sabbath when not yet run today" do
      assert RemCycle.due?(~D[2026-07-11], nil)
      assert RemCycle.due?(~D[2026-07-11], ~D[2026-07-04])
    end

    test "at most once per Sabbath" do
      refute RemCycle.due?(~D[2026-07-11], ~D[2026-07-11])
    end

    test "never due off-Sabbath" do
      refute RemCycle.due?(~D[2026-07-10], nil)
      refute RemCycle.due?(~D[2026-07-13], ~D[2026-07-11])
    end
  end

  describe "daily_state_to_nodes/1" do
    test "maps notes to noise-tier and decisions to scaled importance" do
      started = ~U[2026-07-06 09:00:00Z]

      state = %{
        agent_id: "luna",
        date: ~D[2026-07-06],
        started_at: started,
        notes: ["hello", "weather chat"],
        decisions: [
          %{text: "adopt larql", importance: 5, at: started},
          %{text: "minor tweak", importance: 1, at: started}
        ],
        carry_forward: []
      }

      nodes = RemCycle.daily_state_to_nodes(state)
      assert length(nodes) == 4

      path = "daily/luna/2026-07-06"
      assert Enum.all?(nodes, &(&1.path == path)), "folds stay within one agent's day"
      assert Enum.all?(nodes, &(&1.created_at == DateTime.to_unix(started)))

      notes = Enum.filter(nodes, &String.contains?(&1.id, "note"))
      assert Enum.all?(notes, &(&1.importance == 0.2)), "notes are noise-tier"

      importances =
        nodes
        |> Enum.filter(&String.contains?(&1.id, "decision"))
        |> Enum.map(& &1.importance)
        |> Enum.sort()

      assert importances == [0.2, 1.0], "1–5 scale maps into [0.2, 1.0]"
    end

    test "empty day yields no nodes" do
      state = %{agent_id: "luna", date: ~D[2026-07-06], notes: [], decisions: []}
      assert RemCycle.daily_state_to_nodes(state) == []
    end
  end
end
