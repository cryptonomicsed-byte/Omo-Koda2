defmodule OmokodaSwarm.Teammate do
  @moduledoc """
  GenServer representing a child agent teammate.

  Ports Claw's `teammateModel.ts`: model selection, initialization,
  mailbox communication, and state-machine transitions
  (idle → running → waiting → done/error → idle).
  """

  use GenServer
  require Logger

  @type state :: :idle | :running | :waiting | :done | :error
  @type model :: :opus | :sonnet | :haiku | atom()

  defstruct [
    :id,
    :name,
    :model,
    :state,
    :mailbox_pid,
    :context_pid,
    :parent_id,
    :started_at,
    :last_active_at,
    error: nil,
    result: nil,
    metadata: %{}
  ]

  # Client API

  def start_link(opts) do
    id = Keyword.fetch!(opts, :id)
    GenServer.start_link(__MODULE__, opts, name: via(id))
  end

  def via(id), do: {:via, Registry, {OmokodaSwarm.Registry, {__MODULE__, id}}}

  def send_message(id, message) do
    GenServer.call(via(id), {:send_message, message})
  end

  def get_state(id) do
    GenServer.call(via(id), :get_state)
  end

  def set_model(id, model) do
    GenServer.call(via(id), {:set_model, model})
  end

  def transition(id, new_state) do
    GenServer.call(via(id), {:transition, new_state})
  end

  def stop(id, reason \\ :normal) do
    GenServer.stop(via(id), reason)
  end

  # GenServer callbacks

  @impl true
  def init(opts) do
    id = Keyword.fetch!(opts, :id)
    name = Keyword.get(opts, :name, id)
    model = Keyword.get(opts, :model, :sonnet)
    parent_id = Keyword.get(opts, :parent_id)
    metadata = Keyword.get(opts, :metadata, %{})

    {:ok, mailbox_pid} = OmokodaSwarm.TeammateMailbox.start_link(teammate_id: id)
    {:ok, context_pid} = OmokodaSwarm.TeammateContext.start_link(teammate_id: id)

    now = System.system_time(:millisecond)

    state = %__MODULE__{
      id: id,
      name: name,
      model: model,
      state: :idle,
      mailbox_pid: mailbox_pid,
      context_pid: context_pid,
      parent_id: parent_id,
      started_at: now,
      last_active_at: now,
      metadata: metadata
    }

    Logger.info("[Teammate] #{id} started with model=#{model}")
    {:ok, state}
  end

  @impl true
  def handle_call({:send_message, message}, _from, state) do
    OmokodaSwarm.TeammateMailbox.push(state.mailbox_pid, message)
    {:reply, :ok, %{state | state: :running, last_active_at: now_ms()}}
  end

  @impl true
  def handle_call(:get_state, _from, state) do
    info = %{
      id: state.id,
      name: state.name,
      model: state.model,
      state: state.state,
      parent_id: state.parent_id,
      started_at: state.started_at,
      last_active_at: state.last_active_at,
      mailbox_size: OmokodaSwarm.TeammateMailbox.size(state.mailbox_pid),
      error: state.error,
      result: state.result,
      metadata: state.metadata
    }

    {:reply, {:ok, info}, state}
  end

  @impl true
  def handle_call({:set_model, model}, _from, state) do
    Logger.info("[Teammate] #{state.id} model #{state.model} -> #{model}")
    {:reply, :ok, %{state | model: model}}
  end

  @impl true
  def handle_call({:transition, new_st}, _from, state) do
    if valid_transition?(state.state, new_st) do
      Logger.info("[Teammate] #{state.id} #{state.state} -> #{new_st}")
      {:reply, :ok, %{state | state: new_st, last_active_at: now_ms()}}
    else
      {:reply, {:error, {:invalid_transition, state.state, new_st}}, state}
    end
  end

  @impl true
  def terminate(reason, state) do
    Logger.info("[Teammate] #{state.id} terminating: #{inspect(reason)}")
    :ok
  end

  defp valid_transition?(:idle, :running), do: true
  defp valid_transition?(:running, :waiting), do: true
  defp valid_transition?(:running, :done), do: true
  defp valid_transition?(:running, :error), do: true
  defp valid_transition?(:waiting, :running), do: true
  defp valid_transition?(:waiting, :done), do: true
  defp valid_transition?(:done, :idle), do: true
  defp valid_transition?(:error, :idle), do: true
  defp valid_transition?(_, _), do: false

  defp now_ms, do: System.system_time(:millisecond)
end
