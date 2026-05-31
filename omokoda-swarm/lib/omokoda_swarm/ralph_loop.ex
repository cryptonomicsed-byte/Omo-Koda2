defmodule OmokodaSwarm.RalphLoop do
  @moduledoc """
  Autonomous iteration loop (RALPH pattern): runs a single agent on the same task
  repeatedly until it signals completion or a stop condition is met.

  Inspired by the "work until done" pattern where the Stop hook intercepts early exits
  and re-queues the task rather than terminating. Useful for long-horizon tasks that
  need multiple passes — linting, refactoring, iterative generation.

  ## Usage

      {:ok, pid} = RalphLoop.start_link(task: "refactor src/", max_iterations: 10)
      RalphLoop.await(pid)  # blocks until complete or max_iterations hit
  """

  use GenServer
  require Logger

  @default_max_iterations 20
  @default_iteration_timeout_ms 60_000

  defstruct [
    :task,
    :agent_id,
    :max_iterations,
    :iteration_timeout_ms,
    :current_iteration,
    :status,
    :result,
    :caller,
    :started_at
  ]

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc """
  Starts a RALPH loop for `task`.

  Options:
  - `:task` — the task string / map (required)
  - `:agent_id` — which agent to use; defaults to spawning a temporary worker
  - `:max_iterations` — hard stop after N passes (default: #{@default_max_iterations})
  - `:iteration_timeout_ms` — per-iteration timeout (default: #{@default_iteration_timeout_ms} ms)
  """
  def start_link(opts) do
    GenServer.start_link(__MODULE__, opts)
  end

  @doc """
  Blocks until the loop finishes or times out.
  Returns `{:ok, result}` or `{:error, reason}`.
  """
  def await(pid, timeout_ms \\ :infinity) do
    GenServer.call(pid, :await, timeout_ms)
  end

  @doc """
  Returns the current loop state snapshot without blocking.
  """
  def status(pid) do
    GenServer.call(pid, :status)
  end

  @doc """
  Forces the loop to stop after the current iteration completes.
  """
  def stop_loop(pid) do
    GenServer.cast(pid, :stop)
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init(opts) do
    task = Keyword.fetch!(opts, :task)
    agent_id = Keyword.get(opts, :agent_id, "ralph_worker_#{System.unique_integer([:positive])}")
    max_iters = Keyword.get(opts, :max_iterations, @default_max_iterations)
    timeout_ms = Keyword.get(opts, :iteration_timeout_ms, @default_iteration_timeout_ms)

    state = %__MODULE__{
      task: task,
      agent_id: agent_id,
      max_iterations: max_iters,
      iteration_timeout_ms: timeout_ms,
      current_iteration: 0,
      status: :running,
      result: nil,
      caller: nil,
      started_at: System.system_time(:millisecond)
    }

    # Kick off the first iteration immediately.
    send(self(), :iterate)

    {:ok, state}
  end

  @impl true
  def handle_call(:await, from, %{status: :running} = state) do
    # Suspend caller; reply is sent when the loop terminates.
    {:noreply, %{state | caller: from}}
  end

  def handle_call(:await, _from, state) do
    {:reply, loop_result(state), state}
  end

  @impl true
  def handle_call(:status, _from, state) do
    {:reply,
     %{
       task: state.task,
       iteration: state.current_iteration,
       max_iterations: state.max_iterations,
       status: state.status,
       elapsed_ms: System.system_time(:millisecond) - state.started_at
     }, state}
  end

  @impl true
  def handle_cast(:stop, state) do
    Logger.info("[RALPH] stop requested after iteration #{state.current_iteration}")
    finish(state, :stopped, "loop stopped by caller")
  end

  @impl true
  def handle_info(:iterate, %{status: :running} = state) do
    iter = state.current_iteration + 1
    Logger.info("[RALPH] iteration #{iter}/#{state.max_iterations} for task: #{inspect(state.task)}")

    outcome = run_iteration(state.agent_id, state.task, state.iteration_timeout_ms)

    new_state = %{state | current_iteration: iter}

    cond do
      # Agent declared itself done
      done?(outcome) ->
        finish(new_state, :complete, extract_result(outcome))

      # Hard cap reached
      iter >= state.max_iterations ->
        Logger.warn("[RALPH] max_iterations (#{state.max_iterations}) reached without completion")
        finish(new_state, :max_iterations_reached, extract_result(outcome))

      # Continue looping
      true ->
        send(self(), :iterate)
        {:noreply, new_state}
    end
  end

  def handle_info(:iterate, state) do
    # Loop already finished — ignore stray iterate messages
    {:noreply, state}
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp run_iteration(agent_id, task, timeout_ms) do
    # Ensure a worker agent exists
    ensure_agent(agent_id)

    task_prompt = build_prompt(task)

    ref = make_ref()
    parent = self()

    spawn(fn ->
      result =
        case OmokodaSwarm.Agent.delegate_task(agent_id, task_prompt) do
          :ok -> {:ok, "iteration_complete"}
          {:ok, r} -> {:ok, r}
          {:error, r} -> {:error, r}
        end

      send(parent, {:iteration_result, ref, result})
    end)

    receive do
      {:iteration_result, ^ref, result} -> result
    after
      timeout_ms ->
        {:error, :timeout}
    end
  end

  defp ensure_agent(agent_id) do
    case OmokodaSwarm.Agent.get_state(agent_id) do
      {:ok, _} -> :ok
      {:error, :agent_not_found} -> OmokodaSwarm.SwarmSupervisor.start_agent(agent_id, %{role: :ralph_worker})
    end
  end

  defp build_prompt(task) when is_binary(task), do: task
  defp build_prompt(task) when is_map(task) do
    Map.get(task, :prompt, Map.get(task, "prompt", inspect(task)))
  end
  defp build_prompt(task), do: inspect(task)

  # An iteration is considered "done" when the agent explicitly signals completion.
  # Convention: result string contains "DONE", "complete", or `:complete` atom.
  defp done?({:ok, result}) when is_binary(result) do
    result =~ ~r/\bDONE\b|\bcomplete\b|\bfinished\b/i
  end
  defp done?({:ok, :complete}), do: true
  defp done?(_), do: false

  defp extract_result({:ok, r}), do: r
  defp extract_result({:error, r}), do: r
  defp extract_result(r), do: r

  defp loop_result(%{status: :complete, result: r}), do: {:ok, r}
  defp loop_result(%{status: s, result: r}), do: {:error, {s, r}}

  defp finish(state, status, result) do
    new_state = %{state | status: status, result: result}

    if state.caller do
      GenServer.reply(state.caller, loop_result(new_state))
    end

    Logger.info("[RALPH] finished with status=#{status} after #{state.current_iteration} iteration(s)")
    {:noreply, new_state}
  end
end
