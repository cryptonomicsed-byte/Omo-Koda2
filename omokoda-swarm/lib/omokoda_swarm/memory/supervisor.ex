defmodule OmokodaSwarm.Memory.Supervisor do
  @moduledoc """
  DynamicSupervisor for per-agent daily memory processes.

  Each agent can have at most one DailyServer per calendar day active at once.
  `ensure_daily/2` is idempotent — safe to call on every session start.
  """

  use DynamicSupervisor

  def start_link(_opts) do
    DynamicSupervisor.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc "Ensure a DailyServer exists for `agent_id` on `date`. Returns {:ok, pid}."
  def ensure_daily(agent_id, date \\ Date.utc_today()) do
    spec = {OmokodaSwarm.Memory.DailyServer, {agent_id, date}}

    case DynamicSupervisor.start_child(__MODULE__, spec) do
      {:ok, pid} -> {:ok, pid}
      {:error, {:already_started, pid}} -> {:ok, pid}
      error -> error
    end
  end

  @impl true
  def init([]) do
    DynamicSupervisor.init(strategy: :one_for_one, max_children: 1000)
  end
end
