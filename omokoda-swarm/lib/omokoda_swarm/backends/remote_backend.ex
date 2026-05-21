defmodule OmokodaSwarm.Backends.RemoteBackend do
  @moduledoc """
  Distributed backend — delegates tasks to a remote Erlang node via :erpc.
  Available only when at least one peer node is connected.
  """

  @behaviour OmokodaSwarm.Backend

  @impl true
  def name, do: :remote

  @impl true
  def available?, do: Node.list() != []

  @impl true
  def execute(task, opts) do
    case Node.list() do
      [] ->
        {:error, :no_remote_nodes}

      [default | _] ->
        node = Keyword.get(opts, :node, default)
        timeout = Keyword.get(opts, :timeout, 30_000)
        inner_opts = Keyword.delete(opts, :node)

        :erpc.call(node, OmokodaSwarm.Backends.LocalBackend, :execute, [task, inner_opts], timeout)
    end
  catch
    :error, reason -> {:error, reason}
    :exit, reason -> {:error, {:exit, reason}}
  end

  @impl true
  def terminate(_reason), do: :ok
end
