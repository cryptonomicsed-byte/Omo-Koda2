defmodule OmokodaSwarm.TeammateMailbox do
  @moduledoc """
  FIFO message queue for a teammate's incoming messages.
  Each Teammate owns one mailbox PID started during its init.
  """

  use GenServer

  # Client API

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts)
  end

  def push(pid, message) do
    GenServer.cast(pid, {:push, message})
  end

  def pop(pid) do
    GenServer.call(pid, :pop)
  end

  def peek(pid) do
    GenServer.call(pid, :peek)
  end

  def drain(pid) do
    GenServer.call(pid, :drain)
  end

  def size(pid) do
    GenServer.call(pid, :size)
  end

  # GenServer callbacks

  @impl true
  def init(_opts) do
    {:ok, :queue.new()}
  end

  @impl true
  def handle_cast({:push, message}, queue) do
    {:noreply, :queue.in(message, queue)}
  end

  @impl true
  def handle_call(:pop, _from, queue) do
    case :queue.out(queue) do
      {{:value, item}, new_q} -> {:reply, {:ok, item}, new_q}
      {:empty, _} -> {:reply, {:error, :empty}, queue}
    end
  end

  @impl true
  def handle_call(:peek, _from, queue) do
    case :queue.peek(queue) do
      {:value, item} -> {:reply, {:ok, item}, queue}
      :empty -> {:reply, {:error, :empty}, queue}
    end
  end

  @impl true
  def handle_call(:drain, _from, queue) do
    {:reply, :queue.to_list(queue), :queue.new()}
  end

  @impl true
  def handle_call(:size, _from, queue) do
    {:reply, :queue.len(queue), queue}
  end
end
