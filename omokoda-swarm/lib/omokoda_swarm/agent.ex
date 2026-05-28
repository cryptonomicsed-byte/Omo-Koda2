defmodule OmokodaSwarm.Agent do
  @moduledoc """
  GenServer representing a sovereign agent in the swarm.

  When a task carries a recognized primitive type (:birth, :think, :act) with
  the appropriate fields, the agent dispatches it to the Rust Steward via
  StewardClient. Results are published to the TelemetryHub on the agent's
  own channel so observers can react without polling.

  Other task shapes (coordinator meta-tasks like "Plan X") are executed
  locally and return :ok so the existing coordination strategies keep working.
  """

  use GenServer
  require Logger

  defstruct [:id, :config, :state, :tasks, :steward_connected]

  # ---------------------------------------------------------------------------
  # Public API (unchanged surface so existing supervisor/tests still work)
  # ---------------------------------------------------------------------------

  def start_link(agent_id, config \\ %{}) do
    GenServer.start_link(__MODULE__, {agent_id, config}, name: process_name(agent_id))
  end

  def process_name(agent_id) do
    {:via, Registry, {OmokodaSwarm.Registry, agent_id}}
  end

  def get_id(pid), do: GenServer.call(pid, :get_id)

  def get_state(agent_id) do
    case GenServer.whereis(process_name(agent_id)) do
      nil -> {:error, :agent_not_found}
      pid -> GenServer.call(pid, :get_state)
    end
  end

  @doc """
  Enqueue a task for this agent. Returns :ok immediately; execution is async.
  Tasks with :birth/:think/:act types and the right fields are forwarded to
  the Rust Steward. All other tasks are simulated locally.
  """
  def delegate_task(agent_id, task) do
    case GenServer.whereis(process_name(agent_id)) do
      nil -> {:error, :agent_not_found}
      pid -> GenServer.call(pid, {:delegate_task, task})
    end
  end

  # ---------------------------------------------------------------------------
  # GenServer callbacks
  # ---------------------------------------------------------------------------

  @impl true
  def init({agent_id, config}) do
    state = %__MODULE__{
      id: agent_id,
      config: config,
      state: :idle,
      tasks: [],
      steward_connected: false
    }
    {:ok, state}
  end

  @impl true
  def handle_call(:get_id, _from, state), do: {:reply, state.id, state}

  @impl true
  def handle_call(:get_state, _from, state) do
    public = %{
      id: state.id,
      state: state.state,
      tasks: state.tasks,
      steward_connected: state.steward_connected
    }
    {:reply, {:ok, public}, state}
  end

  @impl true
  def handle_call({:delegate_task, task}, _from, state) do
    new_state = %{state | tasks: state.tasks ++ [task], state: :busy}
    Process.send_after(self(), {:process_task, task}, 10)
    {:reply, :ok, new_state}
  end

  @impl true
  def handle_info({:process_task, task}, state) do
    result = dispatch(task, state.id)

    # Publish result to TelemetryHub so observers can react.
    OmokodaSwarm.TelemetryHub.publish(state.id, %{
      task: task,
      result: result,
      timestamp: DateTime.utc_now()
    })

    new_tasks = List.delete(state.tasks, task)
    new_state = %{
      state
      | tasks: new_tasks,
        state: (if new_tasks == [], do: :idle, else: :busy),
        steward_connected: steward_result?(result)
    }

    {:noreply, new_state}
  end

  # ---------------------------------------------------------------------------
  # Task dispatch — Steward-aware
  # ---------------------------------------------------------------------------

  # Birth primitive: forward to Rust Steward.
  defp dispatch(%{type: :birth, name: name} = task, agent_id) do
    meta = Map.get(task, :meta, [])
    Logger.debug("[agent:#{agent_id}] birth #{name}")

    case OmokodaSwarm.StewardClient.birth(name, meta) do
      {:ok, result} -> {:ok, result}
      {:error, reason} -> steward_fallback(:birth, reason, agent_id)
    end
  end

  # Think primitive: forward to Rust Steward.
  defp dispatch(%{type: :think, prompt: prompt} = task, agent_id) do
    private = Map.get(task, :private, false)
    Logger.debug("[agent:#{agent_id}] think «#{String.slice(prompt, 0, 40)}»")

    case OmokodaSwarm.StewardClient.think(prompt, private) do
      {:ok, result} -> {:ok, result}
      {:error, reason} -> steward_fallback(:think, reason, agent_id)
    end
  end

  # Act primitive: forward to Rust Steward.
  defp dispatch(%{type: :act, tool: tool} = task, agent_id) do
    params = Map.get(task, :params, "{}")
    sandbox = Map.get(task, :sandbox, false)
    Logger.debug("[agent:#{agent_id}] act #{tool}")

    case OmokodaSwarm.StewardClient.act(tool, params, sandbox) do
      {:ok, result} -> {:ok, result}
      {:error, reason} -> steward_fallback(:act, reason, agent_id)
    end
  end

  # Coordinator meta-tasks (strings or unknown maps) — simulate locally.
  defp dispatch(task, agent_id) do
    Logger.debug("[agent:#{agent_id}] local task: #{inspect(task)}")
    Process.sleep(50)
    {:ok, %{simulated: true, task: inspect(task)}}
  end

  defp steward_fallback(primitive, reason, agent_id) do
    Logger.warning("[agent:#{agent_id}] steward unavailable for #{primitive}: #{inspect(reason)}")
    {:error, {:steward_unavailable, reason}}
  end

  defp steward_result?({:ok, _}), do: true
  defp steward_result?(_), do: false
end
