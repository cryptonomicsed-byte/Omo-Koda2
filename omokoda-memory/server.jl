"""
Ọọ̀ Kọ́dà Memory HTTP server — Ọ̀ṣun / Memory layer.
Listens on :7778 (override with --port N or MEMORY_PORT env var).

Endpoints:
  GET  /health                     → { ok: true, version: "0.1.0" }
  GET  /capabilities               → list of available operations

  POST /bb_verify                  → Busy Beaver step verification
  POST /bb_simulate                → Run a custom TM and return result

  POST /nist/test                  → Single NIST SP 800-22 test
  POST /nist/validate              → Full 15-test battery (ÌfáScript pre-mainnet gate)

  POST /predict                    → Augury time-series prediction
  POST /augury/dag/snapshot        → Add snapshot to in-memory DAG
  GET  /augury/dag/summary         → DAG structure summary

  POST /optimize                   → DePIN resource allocation
  POST /optimize/reliability       → Monte Carlo reliability simulation

  POST /garden/analyse             → Receipt log analytics
  POST /garden/feed                → Augury feature vector from receipts

  POST /mesh/score                 → Julia trust score computation
  POST /mesh/resonance             → IfáScript ResonancePacket → resonance score
  POST /mesh/correlations          → Cross-agent trust correlations
  POST /mesh/forecast              → Resource demand forecast
  POST /mesh/reliability           → Agent commitment reliability report

  POST /dream/rem                  → REM fractal compression plan (weekly dream state)
"""

# -- bootstrap ---------------------------------------------------------------
import Pkg
Pkg.activate(joinpath(@__DIR__))   # activate omokoda-memory project

using HTTP
using JSON3
push!(LOAD_PATH, joinpath(@__DIR__, "src"))
using OmokodaMemory
using Statistics

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

function get_port()::Int
    idx = findfirst(==("--port"), ARGS)
    idx !== nothing && return parse(Int, ARGS[idx+1])
    parse(Int, get(ENV, "MEMORY_PORT", "7778"))
end

const VERSION = "0.1.0"

const GLOBAL_DAG = Ref(MemoryDAG())

include(joinpath(@__DIR__, "src", "mesh_analytics.jl"))
include(joinpath(@__DIR__, "src", "rem_fractal.jl"))
include(joinpath(@__DIR__, "src", "vantage_bridge.jl"))
include(joinpath(@__DIR__, "src", "soma_bridge.jl"))
include(joinpath(@__DIR__, "src", "resonance.jl"))

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

function json_ok(data)
    body = JSON3.write(data)
    HTTP.Response(200, ["Content-Type" => "application/json"], body)
end

function json_err(msg::String, code::Int=400)
    body = JSON3.write(Dict("error" => msg))
    HTTP.Response(code, ["Content-Type" => "application/json"], body)
end

function parse_body(req::HTTP.Request)
    isempty(req.body) && return Dict{String,Any}()
    JSON3.read(req.body, Dict{String,Any})
end

# ---------------------------------------------------------------------------
# Handlers
# ---------------------------------------------------------------------------

function handle_health(req::HTTP.Request)
    json_ok(Dict("ok" => true, "version" => VERSION,
                 "dag_nodes" => length(GLOBAL_DAG[].nodes)))
end

function handle_capabilities(req::HTTP.Request)
    json_ok(Dict(
        "bb"       => ["/bb_verify", "/bb_simulate"],
        "nist"     => ["/nist/test", "/nist/validate"],
        "augury"   => ["/predict", "/augury/dag/snapshot", "/augury/dag/summary"],
        "optimize" => ["/optimize", "/optimize/reliability"],
        "garden"   => ["/garden/analyse", "/garden/feed"],
        "mesh"     => ["/mesh/score", "/mesh/resonance", "/mesh/correlations", "/mesh/forecast", "/mesh/reliability"],
        "dream"    => ["/dream/rem"],
    ))
end

function handle_bb_verify(req::HTTP.Request)
    body = parse_body(req)
    n_states     = Int(get(body, "states", 2))
    claimed_steps= Int(get(body, "steps",  6))
    result = verify_bb_steps(n_states, claimed_steps)
    json_ok(result)
end

function handle_bb_simulate(req::HTTP.Request)
    body      = parse_body(req)
    n_states  = Int(get(body, "states",    2))
    max_steps = Int(get(body, "max_steps", 200_000))

    if haskey(body, "rules")
        rules_vec     = [collect(r) for r in body["rules"]]
        claimed_steps = Int(get(body, "claimed_steps", 0))
        claimed_sigma = Int(get(body, "claimed_sigma", 0))
        result = verify_custom_tm(rules_vec, claimed_steps, claimed_sigma;
                                   max_steps=max_steps)
        json_ok(result)
    else
        result = run_known_bb(n_states; max_steps=max_steps)
        json_ok(Dict(
            "halted"    => result.halted,
            "steps"     => result.steps,
            "ones"      => result.ones,
            "tape_size" => result.tape_length,
            "reason"    => result.reason,
        ))
    end
end

const NIST_TEST_MAP = Dict(
    "frequency"       => test_frequency,
    "block_frequency" => bits -> test_block_frequency(bits),
    "runs"            => test_runs,
    "longest_run"     => test_longest_run,
    "approx_entropy"  => bits -> test_approx_entropy(bits),
    "cumulative_sums" => test_cumulative_sums,
    "serial"          => bits -> test_serial(bits),
    "binary_matrix_rank"     => test_binary_matrix_rank,
    "dft"                    => test_dft,
    "non_overlapping"        => test_non_overlapping,
    "overlapping"            => test_overlapping,
    "universal"              => test_universal,
    "linear_complexity"      => test_linear_complexity,
    "random_excursions"      => test_random_excursions,
    "random_excursions_var"  => test_random_excursions_var,
)

function handle_nist_test(req::HTTP.Request)
    body = parse_body(req)
    test_name = string(get(body, "test", "frequency"))
    fn = get(NIST_TEST_MAP, test_name, nothing)
    fn === nothing && return json_err("unknown test '$test_name'")
    data = get(body, "data", Int[])
    bits = parse_bits(data)
    r    = fn(bits)
    json_ok(Dict(
        "name"      => r.name,
        "passed"    => r.passed,
        "p_value"   => r.p_value,
        "statistic" => r.statistic,
        "note"      => r.note,
    ))
end

function handle_nist_validate(req::HTTP.Request)
    body = parse_body(req)
    data = get(body, "data", Int[])
    result = validate_odu_entropy(data)
    json_ok(Dict(
        "all_passed"  => result.all_passed,
        "passed"      => result.passed,
        "total"       => result.total,
        "results"     => [
            Dict("name" => r.name, "passed" => r.passed,
                 "p_value" => r.p_value, "note" => r.note)
            for r in result.results
        ],
    ))
end

function handle_predict(req::HTTP.Request)
    body    = parse_body(req)
    series  = Float64.(get(body, "series",  Float64[]))
    horizon = Int(get(body, "horizon", 5))
    method  = Symbol(get(body, "method", "holt"))
    α       = Float64(get(body, "alpha", 0.3))
    β       = Float64(get(body, "beta",  0.1))
    isempty(series) && return json_err("series must be a non-empty array of numbers")
    result = predict(series, horizon; method=method, α=α, β=β)
    json_ok(Dict(
        "predictions" => result.predictions,
        "lower"       => result.lower,
        "upper"       => result.upper,
        "method"      => result.method,
        "horizon"     => horizon,
    ))
end

function handle_dag_snapshot(req::HTTP.Request)
    body      = parse_body(req)
    id        = string(get(body, "id", string(rand(UInt32), base=16)))
    values    = Float64.(get(body, "values", Float64[]))
    timestamp = Float64(get(body, "timestamp", time()))
    parent    = get(body, "parent_id", nothing)
    parent_id = parent === nothing ? nothing : string(parent)
    add_snapshot!(GLOBAL_DAG[], id, values, timestamp, parent_id)
    json_ok(Dict("added" => id, "dag" => summarise_dag(GLOBAL_DAG[])))
end

function handle_dag_summary(req::HTTP.Request)
    json_ok(summarise_dag(GLOBAL_DAG[]))
end

function handle_optimize(req::HTTP.Request)
    body     = parse_body(req)
    nodes    = [parse_node(d) for d in get(body, "nodes", [])]
    tasks    = [parse_task(d) for d in get(body, "tasks", [])]
    strategy = string(get(body, "strategy", "greedy"))
    max_util = Float64(get(body, "max_utilisation", 0.9))
    isempty(nodes) && return json_err("nodes must be a non-empty array")
    result = if strategy == "round_robin"
        allocate_round_robin(nodes, tasks)
    elseif strategy == "least_connections"
        allocate_least_connections(nodes, tasks)
    else
        allocate_greedy(nodes, tasks; max_utilisation=max_util)
    end
    json_ok(Dict(
        "strategy"   => result.strategy,
        "score"      => result.score,
        "unassigned" => result.unassigned,
        "allocations"=> [
            Dict("node_id" => a.node_id,
                 "tasks"   => a.task_ids,
                 "utilisation" => a.utilisation)
            for a in result.allocations
        ],
    ))
end

function handle_reliability(req::HTTP.Request)
    body     = parse_body(req)
    nodes    = [parse_node(d) for d in get(body, "nodes", [])]
    tasks    = [parse_task(d) for d in get(body, "tasks", [])]
    n_trials = Int(get(body, "n_trials", 10_000))
    isempty(nodes) && return json_err("nodes must be a non-empty array")
    result = monte_carlo_reliability(nodes, tasks, n_trials)
    json_ok(result)
end

function handle_garden_analyse(req::HTTP.Request)
    body     = parse_body(req)
    raw_recs = get(body, "receipts", [])
    receipts = [parse_receipt(d) for d in raw_recs]
    result   = analyse_receipts(receipts)
    json_ok(result)
end

function handle_garden_feed(req::HTTP.Request)
    body     = parse_body(req)
    raw_recs = get(body, "receipts", [])
    receipts = [parse_receipt(d) for d in raw_recs]
    features = augury_feed(receipts)
    json_ok(Dict(
        "features"    => features,
        "description" => [
            "success_rate", "mean_duration_ms", "p95_duration_ms",
            "mean_synapse", "mean_dopamine", "tool_entropy",
            "tier_mean", "tier_std", "receipts_per_min",
            "agent_diversity", "failure_spike", "trend_slope",
        ],
    ))
end

# ---------------------------------------------------------------------------
# Mesh score handler (Julia trust computation for Rust OsunClient)
# ---------------------------------------------------------------------------

function handle_mesh_score(req::HTTP.Request)
    body = parse_body(req)
    agent_id    = string(get(body, "agent_id", "unknown"))
    neighbor_id = string(get(body, "neighbor_id", "unknown"))
    signals     = get(body, "signals", [])
    prior       = Float64(get(body, "prior", 0.5))

    weights = Float64[Float64(get(s, "weight", 0.0)) for s in signals]
    score = if isempty(weights)
        prior
    else
        clamp(prior + sum(weights) / max(length(weights), 1), 0.0, 1.0)
    end

    json_ok(Dict(
        "agent_id"    => agent_id,
        "neighbor_id" => neighbor_id,
        "trust_score" => score,
        "signal_count" => length(weights),
    ))
end

# Serves IfáScript's ritual_codex::julia_bridge: a ResonancePacket
# {odu_id, tier, day, timestamp, intent} in, {"resonance": x} out. Scoring is in
# src/resonance.jl (pure, unit-tested in CI).
function handle_mesh_resonance(req::HTTP.Request)
    body   = parse_body(req)
    odu_id = Int(get(body, "odu_id", 0))
    tier   = Int(get(body, "tier", 1))
    intent = string(get(body, "intent", ""))

    json_ok(Dict(
        "odu_id"    => odu_id,
        "tier"      => tier,
        "resonance" => compute_resonance(odu_id, tier, intent),
    ))
end

# ---------------------------------------------------------------------------
# Mesh analytics handlers
# ---------------------------------------------------------------------------

function handle_mesh_correlations(req::HTTP.Request)
    body = parse_body(req)
    block_id = string(get(body, "block_id", "local"))
    window = Int(get(body, "window", 7))
    findings = mesh_correlations(block_id, window, GLOBAL_DAG[])
    json_ok(Dict(
        "block_id" => block_id,
        "window_days" => window,
        "findings" => [Dict(
            "agent_a" => f.agent_a,
            "agent_b" => f.agent_b,
            "correlation" => f.correlation,
        ) for f in findings],
    ))
end

function handle_mesh_forecast(req::HTTP.Request)
    body = parse_body(req)
    resource_id = string(get(body, "resource_id", "unknown"))
    horizon = Int(get(body, "horizon", 7))
    history = Float64.(get(body, "history", Float64[]))
    result = mesh_demand_forecast(resource_id, horizon, history)
    json_ok(Dict(
        "resource_id" => result.resource_id,
        "horizon_days" => result.horizon_days,
        "predictions" => result.predictions,
        "confidence" => result.confidence,
    ))
end

function handle_mesh_reliability(req::HTTP.Request)
    body = parse_body(req)
    agent_id = string(get(body, "agent_id", "unknown"))
    receipt_log = get(body, "receipt_log", [])
    parsed = [Dict{String,Any}(string(k) => v for (k, v) in r) for r in receipt_log]
    report = mesh_agent_reliability(agent_id, parsed)
    json_ok(Dict(
        "agent_id" => report.agent_id,
        "commitments_analyzed" => report.commitments_analyzed,
        "fulfillment_rate" => report.fulfillment_rate,
        "mean_latency_secs" => report.mean_latency_secs,
        "reliability_score" => report.reliability_score,
    ))
end

# ---------------------------------------------------------------------------
# Dream / REM handlers
# ---------------------------------------------------------------------------

# Hive-scale REM planning: Elixir streams node summaries from many agents;
# the plan (folds + prune_ids) goes back for the caller to apply and write
# back. Pure planning — this endpoint never mutates state. Logic is in
# src/rem_fractal.jl; the per-agent equivalent lives in omokoda-core dream.rs.
function handle_dream_rem(req::HTTP.Request)
    body = parse_body(req)
    nodes = [Dict{String,Any}(string(k) => v for (k, v) in n) for n in get(body, "nodes", [])]
    noise_importance = Float64(get(body, "noise_importance", 0.35))
    min_fold_cluster = Int(get(body, "min_fold_cluster", 3))
    plan = rem_plan(nodes;
                    noise_importance=noise_importance,
                    min_fold_cluster=min_fold_cluster)
    json_ok(plan)
end

# ---------------------------------------------------------------------------
# Router
# ---------------------------------------------------------------------------

const ROUTER = HTTP.Router()

HTTP.register!(ROUTER, "GET",  "/health",              handle_health)
HTTP.register!(ROUTER, "GET",  "/capabilities",        handle_capabilities)

HTTP.register!(ROUTER, "POST", "/bb_verify",           handle_bb_verify)
HTTP.register!(ROUTER, "POST", "/bb_simulate",         handle_bb_simulate)

HTTP.register!(ROUTER, "POST", "/nist/test",           handle_nist_test)
HTTP.register!(ROUTER, "POST", "/nist/validate",       handle_nist_validate)

HTTP.register!(ROUTER, "POST", "/predict",             handle_predict)
HTTP.register!(ROUTER, "POST", "/augury/dag/snapshot", handle_dag_snapshot)
HTTP.register!(ROUTER, "GET",  "/augury/dag/summary",  handle_dag_summary)

HTTP.register!(ROUTER, "POST", "/optimize",            handle_optimize)
HTTP.register!(ROUTER, "POST", "/optimize/reliability",handle_reliability)

HTTP.register!(ROUTER, "POST", "/garden/analyse",      handle_garden_analyse)
HTTP.register!(ROUTER, "POST", "/garden/feed",         handle_garden_feed)

HTTP.register!(ROUTER, "POST", "/mesh/score",          handle_mesh_score)
HTTP.register!(ROUTER, "POST", "/mesh/resonance",      handle_mesh_resonance)
HTTP.register!(ROUTER, "POST", "/mesh/correlations",   handle_mesh_correlations)
HTTP.register!(ROUTER, "POST", "/mesh/forecast",       handle_mesh_forecast)
HTTP.register!(ROUTER, "POST", "/mesh/reliability",    handle_mesh_reliability)
HTTP.register!(ROUTER, "POST", "/mesh/score",          handle_mesh_score)

HTTP.register!(ROUTER, "POST", "/dream/rem",           handle_dream_rem)

HTTP.register!(ROUTER, "POST", "/vantage/ingest",      handle_vantage_ingest)
HTTP.register!(ROUTER, "POST", "/vantage/similar",     handle_vantage_similar)
HTTP.register!(ROUTER, "POST", "/vantage/predict",     handle_vantage_predict)
HTTP.register!(ROUTER, "POST", "/vantage/patterns",    handle_vantage_patterns)

HTTP.register!(ROUTER, "POST", "/soma/store",          handle_soma_store)
HTTP.register!(ROUTER, "POST", "/soma/reconstruct",    handle_soma_reconstruct)

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

port = get_port()
@info "Ọọ̀ Kọ́dà Memory server starting" port=port version=VERSION
HTTP.serve(ROUTER, "0.0.0.0", port)
