defmodule OmokodaSwarm.TerritorySupervisor do
  @moduledoc """
  YEMỌJA territory-aligned supervision (Connection Map v2 §6.5-6.6).

  Ported from the archived `Yemoja.TerritorySupervisor` (omokoda-elixir,
  pre-consolidation) into the live `omokoda-swarm` app -- the logic was
  real and complete, just left behind when the Elixir apps were merged.
  Reuses `OmokodaSwarm.Registry` (already started in application.ex)
  rather than introducing a second registry.

  The supervision tree mirrors the scent-territory topology Waggle already
  computes: one `DynamicSupervisor` per territory (a depth-1 URI prefix of
  the field), registered by territory name. A supervisor crash or restart
  then only affects agents working the same scent-territory -- the blast
  radius follows the same boundaries the field recognizes as coherent,
  instead of `SwarmSupervisor`'s single global `AgentSupervisor`.

  Spawn decisions are quorum-gated gradient reads: `spawn_plan/2` asks the
  substrate for the trust-weighted gradient rolled up to territory level and
  distributes a worker budget proportionally -- territory with more (and
  better-evidenced) scent gets more workers, cold territory gets none.

  Mandelbrot throttling (§8.3): before spawning into a territory, the
  aggregate `bounded` reading gates the count -- a swarm trending toward a
  fragile boundary gets *fewer* new agents, not more.

  All reads fail soft: no substrate, uniform fallback, supervision unchanged.
  """

  require Logger

  @waggle System.get_env("WAGGLE_URL", "http://127.0.0.1:7777")

  # -- territory-aligned supervision tree ----------------------------------

  @doc "Registry-scoped supervisor name for a territory prefix."
  def via(territory), do: {:via, Registry, {OmokodaSwarm.Registry, {:territory, territory}}}

  @doc """
  Ensure a DynamicSupervisor exists for the territory (idempotent). Workers
  for that territory are started under it, so `one_for_one` failures stay
  inside the scent boundary.
  """
  def ensure_territory(territory) do
    case Registry.lookup(OmokodaSwarm.Registry, {:territory, territory}) do
      [{pid, _}] ->
        {:ok, pid}

      [] ->
        DynamicSupervisor.start_link(strategy: :one_for_one, name: via(territory))
    end
  end

  @doc "Start a worker under its territory's supervisor."
  def start_worker(territory, child_spec) do
    with {:ok, _} <- ensure_territory(territory) do
      DynamicSupervisor.start_child(via(territory), child_spec)
    end
  end

  @doc "List agent ids running under a given territory's supervisor."
  def list_territory(territory) do
    case Registry.lookup(OmokodaSwarm.Registry, {:territory, territory}) do
      [{pid, _}] -> DynamicSupervisor.which_children(pid)
      [] -> []
    end
  end

  # -- gradient-proportional spawn planning --------------------------------

  @doc """
  Distribute `budget` workers across the field's live territories,
  proportional to the trust-weighted gradient at depth 1 and damped by each
  territory's aggregate bounded stability.

  Returns `[{territory, count}]`. Without a reachable substrate returns `[]`
  (spawn nothing on no information -- the conservative failure mode).
  """
  def spawn_plan(budget, prefix \\ "") do
    with {:ok, hotspots} <- gradient(prefix) do
      totals =
        for h <- hotspots, into: %{} do
          territory = h["resource"]
          damp = stability_damp(territory)
          {territory, h["total"] * damp}
        end

      grand = totals |> Map.values() |> Enum.sum()

      if grand <= 0 do
        []
      else
        totals
        |> Enum.map(fn {t, v} -> {t, round(budget * v / grand)} end)
        |> Enum.reject(fn {_, n} -> n <= 0 end)
      end
    else
      _ -> []
    end
  end

  @doc """
  The Mandelbrot spawn throttle: the territory's aggregate bounded reading
  (0-10 intensity = 0-1 stability) scales its share of the spawn budget.
  No verdict -> no damping; a fragile verdict (< 0.33) cuts the share to a
  quarter -- mirror of the substrate's own dead-cat floor.
  """
  def stability_damp(territory) do
    case sniff(territory, "bounded") do
      {:ok, [%{"intensity" => i} | _]} ->
        s = i / 10.0

        cond do
          s >= 0.66 -> 1.0
          s >= 0.33 -> 0.6
          true -> 0.25
        end

      _ ->
        1.0
    end
  end

  # -- substrate reads (:httpc, stdlib only) -------------------------------

  defp gradient(prefix) do
    query = URI.encode_query(%{"depth" => "1", "weighted" => "1", "prefix" => prefix})

    case http_get("#{@waggle}/v1/gradient?#{query}") do
      {:ok, %{"hotspots" => hs}} -> {:ok, hs}
      other -> other
    end
  end

  defp sniff(resource, kind) do
    query = URI.encode_query(%{"resource" => resource, "kind" => kind})

    case http_get("#{@waggle}/v1/sniff?#{query}") do
      {:ok, %{"signals" => sigs}} -> {:ok, sigs}
      other -> other
    end
  end

  defp http_get(url) do
    :inets.start()

    case :httpc.request(:get, {String.to_charlist(url), []}, [{:timeout, 5000}], []) do
      {:ok, {{_, 200, _}, _, body}} ->
        # :json (Erlang/OTP 27+ stdlib) instead of Jason -- this app has zero
        # deps and stdlib-only JSON avoids adding one just for this read.
        {:ok, :json.decode(to_string(body))}

      other ->
        Logger.debug("waggle unreachable: #{inspect(other)}")
        {:error, :unreachable}
    end
  rescue
    e ->
      Logger.debug("waggle read failed: #{inspect(e)}")
      {:error, :unreachable}
  end
end
