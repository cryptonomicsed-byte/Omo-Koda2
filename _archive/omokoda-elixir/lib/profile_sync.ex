defmodule Yemoja.ProfileSync do
  @moduledoc """
  Handles cross-agent profile handoffs within the Yemọja swarm.

  `sync_profile/3` transfers a **public** profile map from one agent to
  another.  The function enforces the Ọmọ Kọ́dà sovereignty model: if the
  caller attempts to include `:private_memory` (or its string equivalent) in
  the profile, the transfer is hard-refused with an `{:error, :private_key_detected}`
  tuple.  This is not a soft warning — it is a protocol violation.

  ## What counts as "private"

  The following keys are forbidden in a profile passed to this module:

    * `:private_memory`
    * `"private_memory"`

  Any future expansion of the private namespace should be added to
  `@private_keys`.
  """

  require Logger

  @private_keys [:private_memory, "private_memory"]

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @type agent_id :: binary()
  @type profile :: map()

  @doc """
  Transfers `profile` from `from_agent_id` to `to_agent_id`.

  Steps:

  1. Validates that `profile` contains no private keys (hard fail).
  2. Looks up both agents in the Registry.
  3. Sends the profile to the target agent for merging into its public state.

  Returns `:ok` on success, `{:error, reason}` on any failure.

  ## Error reasons

    * `:private_key_detected` — the profile map contains a forbidden private key.
    * `:from_agent_not_found` — source agent is not registered.
    * `:to_agent_not_found` — target agent is not registered.
  """
  @spec sync_profile(agent_id(), agent_id(), profile()) ::
          :ok | {:error, :private_key_detected | :from_agent_not_found | :to_agent_not_found}
  def sync_profile(from_agent_id, to_agent_id, profile)
      when is_binary(from_agent_id) and is_binary(to_agent_id) and is_map(profile) do
    with :ok <- validate_no_private_keys(profile),
         :ok <- assert_agent_registered(from_agent_id, :from_agent_not_found),
         :ok <- assert_agent_registered(to_agent_id, :to_agent_not_found) do
      deliver_profile(from_agent_id, to_agent_id, profile)
    end
  end

  @doc """
  Validates that a profile map contains no private keys.

  Returns `:ok` or `{:error, :private_key_detected}`.
  This is intentionally a public function so callers can pre-validate before
  calling `sync_profile/3`.
  """
  @spec validate_no_private_keys(profile()) :: :ok | {:error, :private_key_detected}
  def validate_no_private_keys(profile) when is_map(profile) do
    detected =
      Enum.find(@private_keys, fn key -> Map.has_key?(profile, key) end)

    if detected do
      Logger.error(
        "[ProfileSync] SOVEREIGNTY VIOLATION — profile contains forbidden key #{inspect(detected)}"
      )

      {:error, :private_key_detected}
    else
      :ok
    end
  end

  # ---------------------------------------------------------------------------
  # Private helpers
  # ---------------------------------------------------------------------------

  defp assert_agent_registered(agent_id, error_tag) do
    case Registry.lookup(Yemoja.Registry, agent_id) do
      [{_pid, _}] -> :ok
      [] -> {:error, error_tag}
    end
  end

  defp deliver_profile(from_agent_id, to_agent_id, profile) do
    via = Yemoja.AgentWorker.via(to_agent_id)

    # We use a GenServer call so the caller knows when delivery is complete.
    case GenServer.call(via, {:receive_profile, from_agent_id, profile}) do
      :ok ->
        Logger.info(
          "[ProfileSync] profile synced from=#{from_agent_id} to=#{to_agent_id} keys=#{map_size(profile)}"
        )

        :ok

      {:error, reason} ->
        Logger.warning("[ProfileSync] delivery failed reason=#{inspect(reason)}")
        {:error, reason}
    end
  end
end
