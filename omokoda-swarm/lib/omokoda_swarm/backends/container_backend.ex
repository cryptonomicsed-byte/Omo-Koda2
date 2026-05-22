defmodule OmokodaSwarm.Backends.ContainerBackend do
  @moduledoc """
  Container backend — executes tasks via Docker container runs.
  Available only when the `docker` executable is on PATH.
  """

  @behaviour OmokodaSwarm.Backend

  @impl true
  def name, do: :container

  @impl true
  def available? do
    System.find_executable("docker") != nil
  end

  @impl true
  def execute(task, opts) do
    image = Keyword.get(opts, :image, "elixir:latest")
    cmd = Keyword.get(opts, :cmd, inspect(task))
    env = Keyword.get(opts, :env, [])

    env_flags = Enum.flat_map(env, fn {k, v} -> ["-e", "#{k}=#{v}"] end)
    args = ["run", "--rm"] ++ env_flags ++ [image, "sh", "-c", cmd]

    case System.cmd("docker", args, stderr_to_stdout: true) do
      {output, 0} -> {:ok, String.trim(output)}
      {output, code} -> {:error, {:exit_code, code, output}}
    end
  rescue
    e -> {:error, e}
  end

  @impl true
  def terminate(_reason), do: :ok
end
