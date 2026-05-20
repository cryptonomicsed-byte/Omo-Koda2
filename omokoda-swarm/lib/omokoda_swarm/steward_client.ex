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

  @doc "Birth a new sovereign agent by name."
  def birth(name, meta \\ []) when is_binary(name) do
    body = OmokodaSwarm.JSON.encode!(%{name: name, meta: meta})
    post("/v1/birth", body)
  end

  @doc "Send a think primitive to the active agent."
  def think(prompt, private \\ false) when is_binary(prompt) do
    body = OmokodaSwarm.JSON.encode!(%{prompt: prompt, private: private})
    post("/v1/think", body)
  end

  @doc "Execute an act primitive via the active agent."
  def act(tool, params \\ "{}", sandbox \\ false) when is_binary(tool) do
    body = OmokodaSwarm.JSON.encode!(%{tool: tool, params: params, sandbox: sandbox})
    post("/v1/act", body)
  end

  @doc "Fetch agent status summary from the Steward."
  def status, do: get("/v1/status")

  @doc "Health-check the Steward."
  def health, do: get("/v1/health")

  # ---------------------------------------------------------------------------
  # Internal helpers
  # ---------------------------------------------------------------------------

  defp post(path, body) do
    url = steward_url() <> path
    request = {to_charlist(url), [], ~c"application/json", String.to_charlist(body)}

    case :httpc.request(:post, request, [timeout: 10_000], []) do
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

    case :httpc.request(:get, request, [timeout: 10_000], []) do
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
