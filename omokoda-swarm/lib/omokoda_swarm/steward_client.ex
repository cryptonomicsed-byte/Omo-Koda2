defmodule OmokodaSwarm.StewardClient do
  @moduledoc """
  HTTP client for the Rust Steward API (/v1/*).

  Uses Erlang's built-in :httpc (part of :inets) — no extra HTTP library
  required. JSON is handled by OmokodaSwarm.JSON.

  The Steward URL defaults to http://localhost:7777 and can be overridden
  with the STEWARD_URL environment variable.
  """

  @default_url "http://localhost:7777"

  defp steward_url, do: System.get_env("STEWARD_URL", @default_url)

  # ---------------------------------------------------------------------------
  # Public API — mirrors the three sovereign primitives
  # ---------------------------------------------------------------------------

  @doc """
  Birth a new agent by name. `meta` with `sovereign: true` births/resumes
  the process-wide owner (unchanged existing behavior); any other `meta`
  births a brand-new **guest** agent on the same kernel process instead --
  the response includes `agent_id`/`agent_key`, which callers must then
  pass to `think/4` and `act/5` to address that specific guest rather than
  falling through to the owner (see `dispatch_for_request` in server.rs).
  This is the real mechanism subagent spawning uses: one kernel process,
  many independent guest agents, addressed per-request.
  """
  def birth(name, meta \\ []) when is_binary(name) do
    body = OmokodaSwarm.JSON.encode!(%{name: name, meta: meta})
    post("/v1/birth", body)
  end

  @doc """
  Send a think primitive. `agent_id`/`agent_key` (from a prior `birth/2`
  guest response) target that guest specifically; `nil` (the default)
  targets the process-wide owner, matching every pre-existing caller.
  """
  def think(prompt, private \\ false, agent_id \\ nil, agent_key \\ nil) when is_binary(prompt) do
    body = OmokodaSwarm.JSON.encode!(%{prompt: prompt, private: private})
    post("/v1/think", body, agent_headers(agent_id, agent_key))
  end

  @doc """
  Execute an act primitive. `agent_id`/`agent_key` target a specific guest
  agent, same as `think/4` -- see its doc for why this matters.
  """
  def act(tool, params \\ "{}", sandbox \\ false, agent_id \\ nil, agent_key \\ nil) when is_binary(tool) do
    body = OmokodaSwarm.JSON.encode!(%{tool: tool, params: params, sandbox: sandbox})
    post("/v1/act", body, agent_headers(agent_id, agent_key))
  end

  defp agent_headers(nil, _), do: []
  defp agent_headers(agent_id, agent_key) do
    [
      {~c"X-Agent-Id", to_charlist(agent_id)},
      {~c"X-Agent-Key", to_charlist(agent_key || agent_id)}
    ]
  end

  @doc "Fetch agent status summary from the Steward."
  def status, do: get("/v1/status")

  @doc "Health-check the Steward."
  def health, do: get("/v1/health")

  # ---------------------------------------------------------------------------
  # Internal helpers
  # ---------------------------------------------------------------------------

  defp post(path, body, extra_headers \\ []) do
    url = steward_url() <> path
    request = {to_charlist(url), extra_headers, ~c"application/json", String.to_charlist(body)}

    case :httpc.request(:post, request, [timeout: 40_000], []) do
      {:ok, {{_, status, _}, _headers, resp_body}} when status >= 200 and status < 300 ->
        {:ok, OmokodaSwarm.JSON.decode!(to_string(resp_body))}

      {:ok, {{_, status, _}, _headers, resp_body}} ->
        {:error, {status, safe_decode(resp_body)}}

      {:error, reason} ->
        {:error, {:http_error, reason}}
    end
  end

  defp get(path) do
    url = steward_url() <> path
    request = {to_charlist(url), []}

    case :httpc.request(:get, request, [timeout: 40_000], []) do
      {:ok, {{_, status, _}, _headers, resp_body}} when status >= 200 and status < 300 ->
        {:ok, OmokodaSwarm.JSON.decode!(to_string(resp_body))}

      {:ok, {{_, status, _}, _headers, resp_body}} ->
        {:error, {status, safe_decode(resp_body)}}

      {:error, reason} ->
        {:error, {:http_error, reason}}
    end
  end

  defp safe_decode(body) do
    case OmokodaSwarm.JSON.decode(to_string(body)) do
      {:ok, decoded} -> decoded
      _ -> to_string(body)
    end
  end
end
