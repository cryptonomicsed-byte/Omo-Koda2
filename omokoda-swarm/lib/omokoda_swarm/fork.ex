defmodule OmokodaSwarm.Fork do
  @moduledoc """
  Fork-join subagent spawning with DNA inheritance, restricted tool subsets,
  and memory snapshots.

  Ports `forkSubagent.ts` / `spawnMultiAgent.ts` / `spawnInProcess.ts` patterns.
  """

  require Logger

  @type fork_opts :: [
    tools: [String.t()],
    max_turns: pos_integer(),
    memory_snapshot: map() | nil,
    inherit_reputation: boolean(),
    label: String.t() | nil
  ]

  @type fork_result :: %{
    agent_id: String.t(),
    parent_id: String.t(),
    output: term(),
    status: :ok | :error | :timeout,
    duration_ms: non_neg_integer()
  }

  @default_max_turns 10
  @default_timeout_ms 30_000

  @doc """
  Fork a single child agent from a parent, run `task`, collect result.
  The child inherits the parent's DNA fingerprint prefix and gets an isolated session.

  Options:
    - `tools:` - restricted list of allowed tool names
    - `max_turns:` - max agentic loop turns (default: #{@default_max_turns})
    - `memory_snapshot:` - initial memory map to seed child (default: nil)
    - `inherit_reputation:` - carry parent reputation into child (default: false)
    - `label:` - human-readable label for logging
  """
  @spec fork(String.t(), term(), fork_opts()) :: {:ok, fork_result()} | {:error, term()}
  def fork(parent_id, task, opts \\ []) do
    tools = Keyword.get(opts, :tools, :all)
    max_turns = Keyword.get(opts, :max_turns, @default_max_turns)
    memory_snapshot = Keyword.get(opts, :memory_snapshot, nil)
    inherit_reputation = Keyword.get(opts, :inherit_reputation, false)
    label = Keyword.get(opts, :label, "fork")
    timeout_ms = Keyword.get(opts, :timeout_ms, @default_timeout_ms)

    child_id = child_agent_id(parent_id, label)

    Logger.info("[Fork] #{parent_id} -> #{child_id} | task: #{inspect(task)}")

    config = %{
      role: :subagent,
      parent_id: parent_id,
      allowed_tools: tools,
      max_turns: max_turns,
      memory_snapshot: memory_snapshot,
      inherit_reputation: inherit_reputation
    }

    t0 = System.monotonic_time(:millisecond)

    result =
      case OmokodaSwarm.SwarmSupervisor.start_agent(child_id, config) do
        {:ok, _pid} ->
          agent_result =
            try do
              do_run_task(child_id, task, timeout_ms)
            after
              OmokodaSwarm.SwarmSupervisor.stop_agent(child_id)
            end

          agent_result

        {:error, {:already_started, _}} ->
          do_run_task(child_id, task, timeout_ms)

        {:error, reason} ->
          {:error, reason}
      end

    duration = System.monotonic_time(:millisecond) - t0

    case result do
      {:ok, output} ->
        {:ok,
         %{
           agent_id: child_id,
           parent_id: parent_id,
           output: output,
           status: :ok,
           duration_ms: duration
         }}

      {:error, reason} ->
        {:ok,
         %{
           agent_id: child_id,
           parent_id: parent_id,
           output: {:error, reason},
           status: :error,
           duration_ms: duration
         }}
    end
  end

  @doc """
  Fork multiple child agents in parallel (fan-out), then join all results (fan-in).
  Each `{task, opts}` pair spawns one child. Returns a list of `fork_result` maps in order.

  Example:

      Fork.fork_join("parent_1", [
        {"summarize document A", tools: ["read_file"]},
        {"summarize document B", tools: ["read_file"]},
      ])
  """
  @spec fork_join(String.t(), [{term(), fork_opts()}]) :: [fork_result()]
  def fork_join(parent_id, tasks_with_opts) when is_list(tasks_with_opts) do
    tasks_with_opts
    |> Enum.with_index()
    |> Enum.map(fn {{task, opts}, idx} ->
      labeled_opts = Keyword.put_new(opts, :label, "fork_#{idx}")
      Task.async(fn -> fork(parent_id, task, labeled_opts) end)
    end)
    |> Task.await_many(@default_timeout_ms)
    |> Enum.map(fn
      {:ok, result} -> result
      {:error, reason} -> %{agent_id: nil, parent_id: parent_id, output: {:error, reason}, status: :error, duration_ms: 0}
    end)
  end

  @doc """
  Fork agents for each item in `items`, apply `task_fn` to each, join results.
  This is a parallel map over a swarm.

  Example:

      Fork.map("parent_1", ["doc_a", "doc_b", "doc_c"], fn doc ->
        "summarize \#{doc}"
      end)
  """
  @spec map(String.t(), [term()], (term() -> term()), fork_opts()) :: [fork_result()]
  def map(parent_id, items, task_fn, opts \\ []) do
    items
    |> Enum.with_index()
    |> Enum.map(fn {item, idx} ->
      labeled_opts = Keyword.put_new(opts, :label, "map_#{idx}")
      Task.async(fn -> fork(parent_id, task_fn.(item), labeled_opts) end)
    end)
    |> Task.await_many(@default_timeout_ms)
    |> Enum.map(fn
      {:ok, result} -> result
      {:error, reason} -> %{agent_id: nil, parent_id: parent_id, output: {:error, reason}, status: :error, duration_ms: 0}
    end)
  end

  # --- Private ---

  defp child_agent_id(parent_id, label) do
    suffix = :crypto.strong_rand_bytes(4) |> Base.encode16(case: :lower)
    "#{parent_id}_#{label}_#{suffix}"
  end

  defp do_run_task(agent_id, task, timeout_ms) do
    task_ref = make_ref()
    caller = self()

    spawn(fn ->
      result = OmokodaSwarm.Agent.delegate_task(agent_id, task)
      send(caller, {task_ref, result})
    end)

    receive do
      {^task_ref, :ok} -> {:ok, :delegated}
      {^task_ref, {:ok, output}} -> {:ok, output}
      {^task_ref, {:error, reason}} -> {:error, reason}
    after
      timeout_ms -> {:error, :timeout}
    end
  end
end
