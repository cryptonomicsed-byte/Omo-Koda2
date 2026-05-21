defmodule OmokodaSwarm.Backend do
  @moduledoc """
  Behaviour for swarm execution backends.

  Ports Claw's `src/utils/swarm/backends/` abstraction (iTerm, Tmux, InProcess)
  to an Elixir OTP model: Local (in-process Task), Remote (:erpc), Container (Docker).
  """

  @type task :: term()
  @type opts :: keyword()
  @type result :: {:ok, term()} | {:error, term()}

  @doc "Unique atom identifier for this backend."
  @callback name() :: atom()

  @doc "Whether this backend is available in the current environment."
  @callback available?() :: boolean()

  @doc "Execute a task using this backend."
  @callback execute(task(), opts()) :: result()

  @doc "Release backend resources on shutdown."
  @callback terminate(reason :: term()) :: :ok
end
