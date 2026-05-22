defmodule OmokodaSwarm.Backends.LocalBackend do
  @moduledoc """
  In-process backend — runs tasks as supervised Elixir Tasks.
  The default backend when no distributed nodes are available.
  """

  @behaviour OmokodaSwarm.Backend

  @impl true
  def name, do: :local

  @impl true
  def available?, do: true

  @impl true
  def execute(task, opts) do
    timeout = Keyword.get(opts, :timeout, 30_000)

    task_ref =
      Task.async(fn ->
        case task do
          %{fun: fun} when is_function(fun, 0) -> {:ok, fun.()}
          %{fun: fun, args: args} when is_function(fun) -> {:ok, apply(fun, args)}
          _ -> {:ok, task}
        end
      end)

    case Task.yield(task_ref, timeout) || Task.shutdown(task_ref) do
      {:ok, result} -> result
      {:exit, reason} -> {:error, {:exit, reason}}
      nil -> {:error, :timeout}
    end
  end

  @impl true
  def terminate(_reason), do: :ok
end
