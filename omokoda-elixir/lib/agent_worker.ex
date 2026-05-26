defmodule Yemoja.AgentWorker do
  @moduledoc """
  GenServer per agent: handles think/act/memory_checkpoint messages.
  Public memory contributions flow to HiveAggregator.
  Private memory never leaves this process.
  """
  use GenServer
  require Logger

  defstruct [:id, :model, :public_memory, :session_id, state: :idle]

  def start_link(opts) do
    id = Keyword.fetch!(opts, :id)
    GenServer.start_link(__MODULE__, opts, name: via(id))
  end

  def via(id), do: {:via, Registry, {Yemoja.Registry, {__MODULE__, id}}}

  def think(id, prompt, opts \\ []) do
    GenServer.call(via(id), {:think, prompt, opts}, 60_000)
  end

  def act(id, tool, params, opts \\ []) do
    GenServer.call(via(id), {:act, tool, params, opts}, 60_000)
  end

  def get_state(id) do
    GenServer.call(via(id), :get_state)
  end

  def stop(id), do: GenServer.stop(via(id))

  @impl true
  def init(opts) do
    id = Keyword.fetch!(opts, :id)
    model = Keyword.get(opts, :model, :sonnet)
    state = %__MODULE__{id: id, model: model, public_memory: [], session_id: generate_session_id()}
    Logger.info("[AgentWorker] #{id} started model=#{model}")
    {:ok, state}
  end

  @impl true
  def handle_call({:think, prompt, opts}, _from, state) do
    Logger.info("[AgentWorker] #{state.id} think: #{String.slice(prompt, 0, 80)}")
    result = %{thought: "stub-response", prompt: prompt, model: state.model}
    {:reply, {:ok, result}, %{state | state: :idle}}
  end

  @impl true
  def handle_call({:act, tool, params, opts}, _from, state) do
    Logger.info("[AgentWorker] #{state.id} act: #{tool}")
    result = %{tool: tool, params: params, status: :ok}
    {:reply, {:ok, result}, %{state | state: :idle}}
  end

  @impl true
  def handle_call(:get_state, _from, state) do
    {:reply, {:ok, state}, state}
  end

  defp generate_session_id do
    :crypto.strong_rand_bytes(8) |> Base.encode16(case: :lower)
  end
end
