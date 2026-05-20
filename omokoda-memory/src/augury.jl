"""
Augury — predictive memory modeling over agent memory DAGs.

Ọ̀ṣun / Memory layer.  Predicts likely next memory branches so the Elixir
Augury service can pre-warm caches before agent requests arrive.

Architecture (BikoDB three-tier memory pattern):
  • Graph tier  — DAG of memory snapshots (node = agent state, edge = transition)
  • Vector tier — embedding of recent access sequences for similarity lookup
  • Document tier — full state snapshots (stored externally, referenced by ID)

This module provides:
  1. Memory DAG operations  (add_snapshot!, walk_path)
  2. Time-series prediction (exponential smoothing, Holt's linear)
  3. Pattern similarity     (cosine similarity over access vectors)

When Flux.jl is available, replace _holt_predict with a trained LSTM.
"""

using Statistics: mean, std

# ---------------------------------------------------------------------------
# Memory DAG
# ---------------------------------------------------------------------------

mutable struct MemoryNode
    id::String
    timestamp::Float64          # Unix epoch
    values::Vector{Float64}     # agent metric snapshot (reputation, synapse, …)
    children::Vector{String}    # child node IDs
end

mutable struct MemoryDAG
    nodes::Dict{String, MemoryNode}
    roots::Vector{String}
end

MemoryDAG() = MemoryDAG(Dict{String,MemoryNode}(), String[])

function add_snapshot!(dag::MemoryDAG, id::String, values::Vector{Float64},
                        timestamp::Float64, parent_id::Union{String,Nothing}=nothing)
    node = MemoryNode(id, timestamp, values, String[])
    dag.nodes[id] = node
    if parent_id === nothing
        push!(dag.roots, id)
    elseif haskey(dag.nodes, parent_id)
        push!(dag.nodes[parent_id].children, id)
    end
    node
end

"""Return the linear path of values from `root_id` through the longest chain."""
function walk_path(dag::MemoryDAG, root_id::String)::Vector{Vector{Float64}}
    path = Vector{Float64}[]
    node = get(dag.nodes, root_id, nothing)
    while node !== nothing
        push!(path, node.values)
        child = isempty(node.children) ? nothing : get(dag.nodes, first(node.children), nothing)
        node = child
    end
    path
end

# ---------------------------------------------------------------------------
# Time-series prediction
# ---------------------------------------------------------------------------

"""Simple Exponential Smoothing (SES) — no trend or seasonality."""
function ses_predict(series::Vector{Float64}, horizon::Int; α::Float64=0.3)::Vector{Float64}
    isempty(series) && return fill(0.0, horizon)
    s = series[1]
    for x in series[2:end]
        s = α * x + (1 - α) * s
    end
    fill(s, horizon)
end

"""Holt's Linear (Double Exponential) Smoothing — captures linear trend."""
function holt_predict(series::Vector{Float64}, horizon::Int;
                       α::Float64=0.3, β::Float64=0.1)::Vector{Float64}
    n = length(series)
    n == 0 && return fill(0.0, horizon)
    n == 1 && return fill(series[1], horizon)

    # Initialise
    L = series[1]
    T = series[2] - series[1]

    for i in 2:n
        L_prev, T_prev = L, T
        L = α * series[i] + (1 - α) * (L_prev + T_prev)
        T = β * (L - L_prev) + (1 - β) * T_prev
    end

    [L + h * T for h in 1:horizon]
end

"""95% prediction interval half-width using residual std."""
function _confidence_interval(series::Vector{Float64}, preds::Vector{Float64})::Vector{Float64}
    n = length(series)
    n < 3 && return fill(abs(mean(series)) * 0.2 + 1e-6, length(preds))
    residuals = [series[i] - (i > 1 ? series[i-1] : series[1]) for i in 2:n]
    sigma = std(residuals; corrected=true)
    [1.96 * sigma * sqrt(h) for h in 1:length(preds)]
end

# ---------------------------------------------------------------------------
# Pattern similarity (vector tier)
# ---------------------------------------------------------------------------

function cosine_similarity(a::Vector{Float64}, b::Vector{Float64})::Float64
    length(a) != length(b) && error("vectors must be same length")
    dot    = sum(a .* b)
    norm_a = sqrt(sum(a .^ 2))
    norm_b = sqrt(sum(b .^ 2))
    (norm_a == 0 || norm_b == 0) ? 0.0 : dot / (norm_a * norm_b)
end

"""
Find the k most similar past sequences to `query_window`.
Returns [(node_id, similarity), …] sorted by descending similarity.
"""
function similar_sequences(dag::MemoryDAG, query_window::Vector{Float64},
                             k::Int=5)::Vector{Tuple{String,Float64}}
    scores = Tuple{String,Float64}[]
    for (id, node) in dag.nodes
        length(node.values) == length(query_window) || continue
        sim = cosine_similarity(query_window, node.values)
        push!(scores, (id, sim))
    end
    sort!(scores, by=x->x[2], rev=true)
    first(scores, k)
end

# ---------------------------------------------------------------------------
# Public prediction API
# ---------------------------------------------------------------------------

"""
    predict(series, horizon; method, α, β) → (predictions, lower, upper)

Predict `horizon` steps ahead from the given time series.
`method` ∈ :ses (simple), :holt (linear trend).
"""
function predict(series::Vector{Float64}, horizon::Int;
                  method::Symbol=:holt, α::Float64=0.3, β::Float64=0.1)
    preds = if method == :holt
        holt_predict(series, horizon; α=α, β=β)
    else
        ses_predict(series, horizon; α=α)
    end

    half_width = _confidence_interval(series, preds)
    lower = preds .- half_width
    upper = preds .+ half_width

    (predictions=preds, lower=lower, upper=upper, method=string(method))
end

"""
    summarise_dag(dag) → NamedTuple

Return a compact summary of the memory DAG for status endpoints.
"""
function summarise_dag(dag::MemoryDAG)
    (
        node_count = length(dag.nodes),
        root_count = length(dag.roots),
        depth      = isempty(dag.roots) ? 0 :
                         maximum(length(walk_path(dag, r)) for r in dag.roots),
    )
end
