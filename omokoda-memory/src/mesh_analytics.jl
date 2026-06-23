"""
Mesh block analytics — correlations, demand forecasting, and agent reliability.
Calls into augury.jl for time-series prediction primitives.
"""

using Statistics

struct CorrelationFinding
    block_id::String
    agent_a::String
    agent_b::String
    correlation::Float64
    window_days::Int
end

struct ForecastResult
    resource_id::String
    horizon_days::Int
    predictions::Vector{Float64}
    confidence::Vector{Float64}
end

struct ReliabilityReport
    agent_id::String
    commitments_analyzed::Int
    fulfillment_rate::Float64
    mean_latency_secs::Float64
    reliability_score::Float64
end

"""
Mine the event DAG for correlations between agents on a block over `window` days.
Returns a list of CorrelationFinding structs.
"""
function mesh_correlations(block_id::String, window::Int, dag::MemoryDAG)::Vector{CorrelationFinding}
    findings = CorrelationFinding[]

    # Walk all paths in the DAG and look for co-occurring agent activity
    paths = [walk_path(dag, id) for id in keys(dag.nodes)]

    for (i, path_a) in enumerate(paths)
        for path_b in paths[i+1:end]
            if isempty(path_a) || isempty(path_b)
                continue
            end

            # Flatten paths to scalar activity scores
            scores_a = [mean(isempty(v) ? [0.0] : v) for v in path_a]
            scores_b = [mean(isempty(v) ? [0.0] : v) for v in path_b]

            len = min(length(scores_a), length(scores_b), window)
            if len < 2
                continue
            end

            r = cor(scores_a[1:len], scores_b[1:len])

            if abs(r) > 0.6
                push!(findings, CorrelationFinding(block_id, "agent_a", "agent_b", r, window))
            end
        end
    end

    return findings
end

"""
Forecast resource demand for `resource_id` over `horizon` days using ses_predict.
Expects a recent reservation history series (floats representing hourly demand).
"""
function mesh_demand_forecast(
    resource_id::String,
    horizon::Int,
    history::Vector{Float64},
)::ForecastResult
    if isempty(history)
        history = zeros(Float64, max(horizon, 7))
    end

    preds = ses_predict(history, horizon)
    ci = _confidence_interval(history, preds)

    return ForecastResult(resource_id, horizon, preds, ci)
end

"""
Analyze the commitment fulfillment record for `agent_id` from the receipt log.
Returns a ReliabilityReport.
"""
function mesh_agent_reliability(
    agent_id::String,
    receipt_log::Vector{Dict{String,Any}},
)::ReliabilityReport
    relevant = filter(r -> get(r, "agent_id", "") == agent_id, receipt_log)

    if isempty(relevant)
        return ReliabilityReport(agent_id, 0, 0.0, 0.0, 0.0)
    end

    total = length(relevant)
    fulfilled = count(r -> get(r, "fulfilled", false), relevant)
    rate = fulfilled / total

    latencies = [get(r, "latency_secs", 0.0) for r in relevant]
    mean_lat = isempty(latencies) ? 0.0 : mean(latencies)

    # Composite reliability: 70% fulfillment rate + 30% latency score (lower is better)
    latency_score = 1.0 / (1.0 + mean_lat / 3600.0)
    score = 0.7 * rate + 0.3 * latency_score

    return ReliabilityReport(agent_id, total, rate, mean_lat, score)
end
