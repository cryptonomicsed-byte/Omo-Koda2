defmodule OmokodaSwarm.Council do
  @moduledoc """
  Multi-agent parallel review council: fans out a code-review task to N specialised
  sub-agents simultaneously, then aggregates their findings by confidence score.

  Each councillor has a defined role and its output is tagged with a confidence
  weight. After all councillors respond (or timeout), findings are merged: conflicts
  are resolved by taking the highest-confidence opinion per finding category.

  ## Usage

      councillors = [
        %{id: "claude-md-check",   role: :compliance,  weight: 1.0},
        %{id: "bug-detector",      role: :bugs,        weight: 1.2},
        %{id: "history-agent",     role: :context,     weight: 0.8},
        %{id: "pr-history-agent",  role: :pr_history,  weight: 0.7},
        %{id: "comment-agent",     role: :comments,    weight: 0.9},
      ]
      {:ok, pid} = Council.start_link(task: "Review PR #42", councillors: councillors)
      {:ok, verdict} = Council.await(pid)
  """

  use GenServer
  require Logger

  @default_timeout_ms 30_000

  defstruct [
    :task,
    :councillors,
    :timeout_ms,
    :pending,
    :opinions,
    :status,
    :verdict,
    :caller,
    :started_at
  ]

  # ---------------------------------------------------------------------------
  # Councillor opinion types
  # ---------------------------------------------------------------------------

  defmodule Opinion do
    @moduledoc false
    defstruct [:councillor_id, :role, :weight, :finding, :severity, :confidence]
  end

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc """
  Starts a council session for `task`.

  Options:
  - `:task` — task description / map (required)
  - `:councillors` — list of `%{id, role, weight}` maps (required)
  - `:timeout_ms` — per-councillor timeout (default: #{@default_timeout_ms} ms)
  """
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts)
  end

  @doc """
  Blocks until all councillors have responded (or timed out).
  Returns `{:ok, verdict}` where `verdict` is the aggregated finding map.
  """
  def await(pid, timeout_ms \\ :infinity) do
    GenServer.call(pid, :await, timeout_ms)
  end

  @doc """
  Returns the current vote tally without blocking.
  """
  def tally(pid) do
    GenServer.call(pid, :tally)
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(opts) do
    task = Keyword.fetch!(opts, :task)
    councillors = Keyword.fetch!(opts, :councillors)
    timeout_ms = Keyword.get(opts, :timeout_ms, @default_timeout_ms)

    state = %__MODULE__{
      task: task,
      councillors: councillors,
      timeout_ms: timeout_ms,
      pending: MapSet.new(Enum.map(councillors, & &1.id)),
      opinions: [],
      status: :deliberating,
      verdict: nil,
      caller: nil,
      started_at: System.system_time(:millisecond)
    }

    # Fan out to all councillors in parallel.
    Enum.each(councillors, &dispatch_councillor(&1, task, timeout_ms, self()))

    {:ok, state}
  end

  @impl true
  def handle_call(:await, from, %{status: :deliberating} = state) do
    {:noreply, %{state | caller: from}}
  end

  def handle_call(:await, _from, state) do
    {:reply, {:ok, state.verdict}, state}
  end

  @impl true
  def handle_call(:tally, _from, state) do
    {:reply,
     %{
       task: state.task,
       opinions_received: length(state.opinions),
       pending: MapSet.size(state.pending),
       status: state.status,
       elapsed_ms: System.system_time(:millisecond) - state.started_at
     }, state}
  end

  @impl true
  def handle_info({:opinion, councillor_id, opinion}, state) do
    new_opinions = [opinion | state.opinions]
    new_pending = MapSet.delete(state.pending, councillor_id)

    Logger.debug("[COUNCIL] opinion from #{councillor_id}, pending: #{MapSet.size(new_pending)}")

    new_state = %{state | opinions: new_opinions, pending: new_pending}

    if MapSet.size(new_pending) == 0 do
      conclude(new_state)
    else
      {:noreply, new_state}
    end
  end

  def handle_info({:timeout, councillor_id}, state) do
    if MapSet.member?(state.pending, councillor_id) do
      Logger.warn("[COUNCIL] councillor #{councillor_id} timed out")
      new_pending = MapSet.delete(state.pending, councillor_id)
      new_state = %{state | pending: new_pending}

      if MapSet.size(new_pending) == 0 do
        conclude(new_state)
      else
        {:noreply, new_state}
      end
    else
      {:noreply, state}
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp dispatch_councillor(councillor, task, timeout_ms, parent) do
    spawn(fn ->
      ensure_agent(councillor.id)

      prompt = build_review_prompt(councillor.role, task)

      raw =
        case OmokodaSwarm.Agent.delegate_task(councillor.id, prompt) do
          :ok -> "review_complete"
          {:ok, r} -> r
          {:error, r} -> "error: #{inspect(r)}"
        end

      opinion = %Opinion{
        councillor_id: councillor.id,
        role: councillor.role,
        weight: councillor.weight,
        finding: raw,
        severity: infer_severity(raw),
        confidence: councillor.weight
      }

      send(parent, {:opinion, councillor.id, opinion})
    end)

    # Schedule a timeout sentinel
    Process.send_after(parent, {:timeout, councillor.id}, timeout_ms)
  end

  defp ensure_agent(agent_id) do
    case OmokodaSwarm.Agent.get_state(agent_id) do
      {:ok, _} -> :ok
      {:error, :agent_not_found} ->
        OmokodaSwarm.SwarmSupervisor.start_agent(agent_id, %{role: :councillor})
    end
  end

  defp build_review_prompt(:compliance, task),  do: "Review CLAUDE.md compliance for: #{inspect(task)}"
  defp build_review_prompt(:bugs, task),        do: "Identify bugs and logic errors in: #{inspect(task)}"
  defp build_review_prompt(:context, task),     do: "Provide historical context and prior decisions for: #{inspect(task)}"
  defp build_review_prompt(:pr_history, task),  do: "Analyse PR history and related changes for: #{inspect(task)}"
  defp build_review_prompt(:comments, task),    do: "Review inline code comments quality for: #{inspect(task)}"
  defp build_review_prompt(role, task),         do: "#{role} review of: #{inspect(task)}"

  defp infer_severity(finding) when is_binary(finding) do
    cond do
      finding =~ ~r/\b(critical|blocker|must.fix)\b/i -> :critical
      finding =~ ~r/\b(error|bug|wrong|incorrect)\b/i -> :high
      finding =~ ~r/\b(warn|suggestion|consider)\b/i  -> :medium
      true -> :low
    end
  end
  defp infer_severity(_), do: :low

  defp conclude(state) do
    verdict = aggregate(state.opinions)
    Logger.info("[COUNCIL] deliberation complete — #{length(state.opinions)} opinion(s)")
    new_state = %{state | status: :concluded, verdict: verdict}

    if state.caller do
      GenServer.reply(state.caller, {:ok, verdict})
    end

    {:noreply, new_state}
  end

  # Aggregate opinions: group by severity, pick highest-confidence within each group.
  # Returns a map of %{summary, findings, highest_severity, confidence_score}.
  defp aggregate([]) do
    %{summary: "no opinions received", findings: [], highest_severity: :none, confidence_score: 0.0}
  end

  defp aggregate(opinions) do
    by_severity =
      opinions
      |> Enum.group_by(& &1.severity)
      |> Enum.map(fn {sev, ops} ->
        best = Enum.max_by(ops, & &1.confidence)
        {sev, best}
      end)
      |> Map.new()

    severity_order = [:critical, :high, :medium, :low]
    highest_severity = Enum.find(severity_order, :none, &Map.has_key?(by_severity, &1))

    findings =
      opinions
      |> Enum.sort_by(& &1.confidence, :desc)
      |> Enum.map(&%{role: &1.role, finding: &1.finding, severity: &1.severity, confidence: &1.confidence})

    total_weight = opinions |> Enum.map(& &1.weight) |> Enum.sum()
    received = length(opinions)
    confidence_score = if total_weight > 0, do: Float.round(total_weight / received, 3), else: 0.0

    %{
      summary: "#{received} councillor(s) reported; highest severity: #{highest_severity}",
      findings: findings,
      highest_severity: highest_severity,
      confidence_score: confidence_score,
      by_severity: by_severity
    }
  end
end
