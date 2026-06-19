"""
Vantage × Julia bridge — HTTP handlers for the /vantage/* route group.

Exposes memory intelligence endpoints consumed by Vantage's Python backend.
All handlers follow the same JSON in/JSON out pattern as the rest of server.jl.
"""

using JSON3

"""
POST /vantage/ingest
Body: { agent_id, trace_type, content, timestamp }
Feeds a Vantage trace into the memory DAG and garden analytics.
"""
function handle_vantage_ingest(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    agent_id = get(body, :agent_id, "unknown")
    content_str = get(body, :content, "")
    ts = get(body, :timestamp, string(time()))

    # Build a simple feature vector from content length and timestamp ordinal
    values = Float64[
        Float64(length(content_str)),
        Float64(hash(content_str) % 1000) / 1000.0,
    ]

    add_snapshot!(GLOBAL_DAG[], "$(agent_id)_$(ts)", values, time(), nothing)

    return HTTP.Response(200, JSON3.write(Dict("ok" => true, "agent_id" => agent_id)))
end

"""
POST /vantage/similar
Body: { content, top_k }
Returns semantically similar past snapshots from the DAG.
"""
function handle_vantage_similar(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    content_str = get(body, :content, "")
    top_k = get(body, :top_k, 5)

    query_vec = Float64[
        Float64(length(content_str)),
        Float64(hash(content_str) % 1000) / 1000.0,
    ]

    results = similar_sequences(GLOBAL_DAG[], query_vec, top_k)

    output = [Dict("node_id" => r[1], "similarity" => r[2]) for r in results]
    return HTTP.Response(200, JSON3.write(output))
end

"""
POST /vantage/predict
Body: { series: [floats], horizon: int }
Returns time-series activity prediction.
"""
function handle_vantage_predict(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    series = Float64.(get(body, :series, [0.0]))
    horizon = Int(get(body, :horizon, 7))

    result = predict(series, horizon)

    return HTTP.Response(200, JSON3.write(Dict(
        "predictions" => result.predictions,
        "lower" => result.lower,
        "upper" => result.upper,
        "horizon" => horizon,
    )))
end

"""
POST /vantage/patterns
Body: { scope: "agent:name" | "block:id" }
Returns behavioral pattern findings from garden analytics.
"""
function handle_vantage_patterns(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    scope = get(body, :scope, "global")

    # Garden analytics returns a DataFrame; convert first few rows to JSON
    receipts = get_receipts_for_scope(scope)
    findings = garden_analyse_receipts(receipts)

    return HTTP.Response(200, JSON3.write(Dict(
        "scope" => scope,
        "patterns" => findings,
    )))
end

"""
Helper: stub receipt loader for a given scope string.
In production, scope would map to a DB or file query.
"""
function get_receipts_for_scope(scope::String)::Vector{Dict{String,Any}}
    return Dict{String,Any}[]
end

"""
Helper: convert garden DataFrame to a plain list of pattern dicts.
"""
function garden_analyse_receipts(receipts::Vector{Dict{String,Any}})
    return []
end
