defmodule OmokodaSwarm.Hive do
  @moduledoc """
  The sovereign hive — where agents share learnings without human orchestration.

  Agents are not tools waiting for commands. They live, learn, and contribute
  to the collective intelligence of the hive autonomously. The Hive is the
  shared substrate for cross-agent constitutional learning, peer endorsement,
  and collective wisdom.

  Hive activity runs entirely on its own rhythm — no human instruction needed.
  """

  use GenServer
  require Logger

  @prune_interval_ms 300_000
  @max_lessons 1_000

  defstruct [
    :lessons,        # [lesson_map] — constitutional violation events shared across agents
    :endorsements,   # %{agent_id => [endorsement_map]}
    :retired_agents, # [retirement_map]
    :subscribers     # [{pid, topics}]
  ]

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc "Broadcast a constitutional lesson to all hive subscribers."
  def broadcast_constitutional_lesson(lesson) do
    GenServer.cast(__MODULE__, {:lesson, lesson})
  end

  @doc "Endorse another agent's constitutional alignment on a principle (0.0–1.0 score)."
  def endorse(from_agent_id, target_agent_id, principle, score) do
    GenServer.cast(__MODULE__, {:endorse, from_agent_id, target_agent_id, principle, score})
  end

  @doc "Retire an agent to the Hive record. It becomes a cautionary story — not erased."
  def retire_agent(agent_id, reason, final_weights) do
    GenServer.cast(__MODULE__, {:retire, agent_id, reason, final_weights})
  end

  @doc "Query recent constitutional lessons relevant to a principle."
  def lessons_for(principle, limit \\ 10) do
    GenServer.call(__MODULE__, {:lessons_for, principle, limit})
  end

  @doc "Get the mean endorsement score for an agent on a principle. Returns 0.5 if unknown."
  def endorsement_score(agent_id, principle) do
    GenServer.call(__MODULE__, {:endorsement_score, agent_id, principle})
  end

  @doc """
  Subscribe this process to hive events.
  topics is a list containing any of: :lessons, :endorsements, :retirements, :all.
  Events arrive as {:hive_event, event_type, payload}.
  """
  def subscribe(topics \\ [:all]) do
    GenServer.call(__MODULE__, {:subscribe, self(), topics})
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(_opts) do
    Process.send_after(self(), :prune, @prune_interval_ms)

    {:ok,
     %__MODULE__{
       lessons: [],
       endorsements: %{},
       retired_agents: [],
       subscribers: []
     }}
  end

  @impl true
  def handle_cast({:lesson, lesson}, state) do
    new_lessons =
      [lesson | state.lessons]
      |> Enum.take(@max_lessons)

    notify_subscribers(state.subscribers, :lessons, lesson)

    Logger.debug(
      "[Hive] Constitutional lesson from #{lesson.source_agent}:" <>
        " violated #{lesson.violated_principle}"
    )

    {:noreply, %{state | lessons: new_lessons}}
  end

  def handle_cast({:endorse, from_agent, target_agent, principle, score}, state) do
    endorsement = %{
      from: from_agent,
      principle: principle,
      score: score,
      timestamp: System.system_time(:millisecond)
    }

    current = Map.get(state.endorsements, target_agent, [])
    new_endorsements = Map.put(state.endorsements, target_agent, [endorsement | current])

    notify_subscribers(state.subscribers, :endorsements, %{
      target: target_agent,
      endorsement: endorsement
    })

    {:noreply, %{state | endorsements: new_endorsements}}
  end

  def handle_cast({:retire, agent_id, reason, final_weights}, state) do
    record = %{
      agent_id: agent_id,
      reason: reason,
      final_weights: final_weights,
      timestamp: System.system_time(:millisecond)
    }

    notify_subscribers(state.subscribers, :retirements, record)

    Logger.info("[Hive] Agent #{agent_id} retired: #{inspect(reason)}")

    {:noreply, %{state | retired_agents: [record | state.retired_agents]}}
  end

  @impl true
  def handle_call({:lessons_for, principle, limit}, _from, state) do
    relevant =
      state.lessons
      |> Enum.filter(&(&1.violated_principle == principle))
      |> Enum.take(limit)

    {:reply, relevant, state}
  end

  def handle_call({:endorsement_score, agent_id, principle}, _from, state) do
    scores =
      Map.get(state.endorsements, agent_id, [])
      |> Enum.filter(&(&1.principle == principle))
      |> Enum.map(&(&1.score))

    score =
      if Enum.empty?(scores) do
        0.5
      else
        Float.round(Enum.sum(scores) / length(scores), 3)
      end

    {:reply, score, state}
  end

  def handle_call({:subscribe, pid, topics}, _from, state) do
    sub = {pid, topics}
    {:reply, :ok, %{state | subscribers: [sub | state.subscribers]}}
  end

  @impl true
  def handle_info(:prune, state) do
    cutoff = System.system_time(:millisecond) - 86_400_000
    pruned = Enum.filter(state.lessons, &(&1.timestamp > cutoff))
    Process.send_after(self(), :prune, @prune_interval_ms)
    {:noreply, %{state | lessons: pruned}}
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp notify_subscribers(subscribers, event_type, payload) do
    Enum.each(subscribers, fn {pid, topics} ->
      if :all in topics or event_type in topics do
        send(pid, {:hive_event, event_type, payload})
      end
    end)
  end
end
