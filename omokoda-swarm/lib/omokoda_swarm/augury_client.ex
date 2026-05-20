defmodule OmokodaSwarm.AuguryClient do
  @moduledoc """
  HTTP client for the Julia Memory service (/v1/* on :7778).

  Provides Elixir access to the Ọ̀ṣun / Memory layer:
    - Augury time-series prediction (pre-warm agent caches)
    - NIST SP 800-22 entropy validation
    - DePIN resource optimisation
    - Garden receipt analytics

  Uses OTP's built-in :httpc + Jason — no extra deps beyond what omokoda-swarm
  already requires.
  """

  @default_url "http://localhost:7778"

  defp memory_url, do: System.get_env("MEMORY_URL", @default_url)

  # ---------------------------------------------------------------------------
  # Public API
  # ---------------------------------------------------------------------------

  @doc "Health-check the Julia Memory service."
  def health, do: get("/health")

  @doc """
  Predict the next `horizon` values from a time series.

  Options:
    - method: "holt" (default) | "ses"
    - alpha:  smoothing factor α (default 0.3)
    - beta:   trend factor β for Holt's method (default 0.1)
  """
  def predict(series, horizon \\ 5, opts \\ []) when is_list(series) do
    body =
      %{
        series: series,
        horizon: horizon,
        method: Keyword.get(opts, :method, "holt"),
        alpha: Keyword.get(opts, :alpha, 0.3),
        beta: Keyword.get(opts, :beta, 0.1)
      }
      |> Jason.encode!()

    post("/predict", body)
  end

  @doc """
  Add a memory snapshot to the Julia DAG.

  `values` is a list of floats representing the agent metric vector
  (e.g. [reputation, synapse, tier, active_tasks]).
  """
  def add_dag_snapshot(id, values, opts \\ []) when is_binary(id) and is_list(values) do
    body =
      %{
        id: id,
        values: values,
        timestamp: DateTime.utc_now() |> DateTime.to_unix(),
        parent_id: Keyword.get(opts, :parent_id)
      }
      |> Jason.encode!()

    post("/augury/dag/snapshot", body)
  end

  @doc "Return a summary of the in-memory DAG structure."
  def dag_summary, do: get("/augury/dag/summary")

  @doc """
  Run a single NIST SP 800-22 test on a bitstream.

  `test` must be one of: frequency, block_frequency, runs, longest_run,
  approx_entropy, cumulative_sums, serial (implemented), or any of the
  remaining 8 (returns not_implemented).
  """
  def nist_test(test, bits) when is_binary(test) and is_list(bits) do
    body = Jason.encode!(%{test: test, data: bits})
    post("/nist/test", body)
  end

  @doc """
  Run the full 15-test NIST battery — IfáScript pre-mainnet entropy gate.
  Returns {:ok, %{all_passed: bool, passed: int, total: int, results: [...]}}
  """
  def nist_validate(bits) when is_list(bits) do
    body = Jason.encode!(%{data: bits})
    post("/nist/validate", body)
  end

  @doc """
  Verify Busy Beaver step count.

  Returns {:ok, %{valid: bool, reason: str, known_sigma: int|nil, known_steps: int}}
  """
  def bb_verify(states \\ 2, claimed_steps \\ 6) do
    body = Jason.encode!(%{states: states, steps: claimed_steps})
    post("/bb_verify", body)
  end

  @doc """
  Allocate DePIN tasks to nodes.

  `nodes` — list of maps with id, capacity, weight, reliability, region
  `tasks` — list of maps with id, load, region
  `strategy` — "greedy" | "round_robin" | "least_connections"
  """
  def optimize(nodes, tasks, strategy \\ "greedy") do
    body = Jason.encode!(%{nodes: nodes, tasks: tasks, strategy: strategy})
    post("/optimize", body)
  end

  @doc """
  Monte Carlo reliability simulation for a DePIN network.
  Returns probability estimates for system uptime under random node failures.
  """
  def reliability_simulation(nodes, tasks, n_trials \\ 10_000) do
    body = Jason.encode!(%{nodes: nodes, tasks: tasks, n_trials: n_trials})
    post("/optimize/reliability", body)
  end

  @doc """
  Analyse a batch of Walrus act receipts.
  Returns throughput, latency percentiles, tool frequency, and economy stats.
  """
  def garden_analyse(receipts) when is_list(receipts) do
    body = Jason.encode!(%{receipts: receipts})
    post("/garden/analyse", body)
  end

  @doc """
  Extract a 12-dimensional feature vector from recent receipts for Augury input.
  """
  def garden_feed(receipts) when is_list(receipts) do
    body = Jason.encode!(%{receipts: receipts})
    post("/garden/feed", body)
  end

  # ---------------------------------------------------------------------------
  # Internal helpers
  # ---------------------------------------------------------------------------

  defp post(path, body) do
    url = memory_url() <> path
    request = {to_charlist(url), [], ~c"application/json", String.to_charlist(body)}

    case :httpc.request(:post, request, [timeout: 30_000], []) do
      {:ok, {{_, status, _}, _headers, resp_body}} when status >= 200 and status < 300 ->
        {:ok, Jason.decode!(to_string(resp_body))}

      {:ok, {{_, status, _}, _headers, resp_body}} ->
        {:error, {status, safe_decode(resp_body)}}

      {:error, reason} ->
        {:error, {:http_error, reason}}
    end
  end

  defp get(path) do
    url = memory_url() <> path
    request = {to_charlist(url), []}

    case :httpc.request(:get, request, [timeout: 10_000], []) do
      {:ok, {{_, status, _}, _headers, resp_body}} when status >= 200 and status < 300 ->
        {:ok, Jason.decode!(to_string(resp_body))}

      {:ok, {{_, status, _}, _headers, resp_body}} ->
        {:error, {status, safe_decode(resp_body)}}

      {:error, reason} ->
        {:error, {:http_error, reason}}
    end
  end

  defp safe_decode(body) do
    case Jason.decode(to_string(body)) do
      {:ok, decoded} -> decoded
      _ -> to_string(body)
    end
  end
end
