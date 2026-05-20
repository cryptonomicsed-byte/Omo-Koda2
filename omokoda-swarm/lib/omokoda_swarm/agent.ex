defmodule OmokodaSwarm.Agent do
  @moduledoc """
  GenServer representing an individual agent in the swarm.
  """

  use GenServer

  defstruct [:id, :config, :state, :tasks]

  @doc """
  Starts a new agent.
  """
  def start_link(agent_id, config \\ %{}) do
    GenServer.start_link(__MODULE__, {agent_id, config}, name: process_name(agent_id))
  end

  @doc """
  Returns the process name for an agent.
  """
  def process_name(agent_id) do
    {:via, Registry, {OmokodaSwarm.Registry, agent_id}}
  end

  @doc """
  Gets the agent ID from a process.
  """
  def get_id(pid) do
    GenServer.call(pid, :get_id)
  end

  @doc """
  Delegates a task to this agent.
  """
  def delegate_task(agent_id, task) do
    case GenServer.whereis(process_name(agent_id)) do
      nil -> {:error, :agent_not_found}
      pid -> GenServer.call(pid, {:delegate_task, task})
    end
  end

  @doc """
  Gets the current state of the agent.
  """
  def get_state(agent_id) do
    case GenServer.whereis(process_name(agent_id)) do
      nil -> {:error, :agent_not_found}
      pid -> GenServer.call(pid, :get_state)
    end
  end

  # GenServer callbacks

  @impl true
  def init({agent_id, config}) do
    state = %__MODULE__{
      id: agent_id,
      config: config,
      state: :idle,
      tasks: []
    }

    {:ok, state}
  end

  @impl true
  def handle_call(:get_id, _from, state) do
    {:reply, state.id, state}
  end

  @impl true
  def handle_call(:get_state, _from, state) do
    {:reply, {:ok, state}, state}
  end

  @impl true
  def handle_call({:delegate_task, task}, _from, state) do
    # Add task to queue
    new_tasks = state.tasks ++ [task]
    new_state = %{state | tasks: new_tasks, state: :busy}

    # Process the task asynchronously
    Process.send_after(self(), {:process_task, task}, 100)

    {:reply, :ok, new_state}
  end

  @impl true
  def handle_info({:process_task, task}, state) do
    # Simulate task processing
    IO.puts("Agent #{state.id} processing task: #{inspect(task)}")

    # Mark task as completed
    new_tasks = List.delete(state.tasks, task)
    new_state = %{state | tasks: new_tasks}

    # Update state to idle if no more tasks
    final_state = if new_tasks == [], do: %{new_state | state: :idle}, else: new_state

    {:noreply, final_state}
  end
end
