include("trust_decay.jl")
include("belief_prop.jl")

struct TrustSignal
    kind::String        # "commitment_fulfilled", "dispute", "interaction", "follow", "tro_fulfilled"
    weight::Float64     # -1.0 to 1.0 (positive = trust, negative = distrust)
    timestamp::Int64    # unix seconds
end

# Convert kind string to base weight modifier
function kind_multiplier(kind::String)::Float64
    multipliers = Dict(
        "commitment_fulfilled" => 1.5,
        "tro_fulfilled"        => 1.3,
        "interaction"          => 1.0,
        "follow"               => 0.7,
        "dispute"              => 1.4,   # disputes have high magnitude (negative weight amplified)
    )
    get(multipliers, kind, 1.0)
end

# Compute a trust score from a list of signals.
# Returns a value in [0.0, 1.0].
# 0.5 is the neutral starting point (unknown agent).
function compute_trust_score(
    signals::Vector{TrustSignal};
    prior::Float64=0.5,
    half_life_days::Float64=30.0,
    neighbor_scores::Vector{Tuple{Float64, Float64}}=Tuple{Float64, Float64}[]
)::Float64
    if isempty(signals)
        return run_belief_prop(prior, neighbor_scores)
    end

    # Apply decay and kind multipliers
    decayed_weights = Float64[]
    for sig in signals
        base = decay_weight(sig.weight, sig.timestamp, half_life_days)
        push!(decayed_weights, base * kind_multiplier(sig.kind))
    end

    # Aggregate: mean of decayed signals, clipped to [-1, 1]
    mean_signal = clamp(sum(decayed_weights) / length(decayed_weights), -1.0, 1.0)

    # Convert to [0, 1] range: 0.5 + signal/2
    raw_score = clamp(0.5 + mean_signal / 2.0, 0.0, 1.0)

    # Propagate through neighbor graph
    run_belief_prop(raw_score, neighbor_scores)
end

# HTTP handler for POST /mesh/score
function handle_mesh_score(req::HTTP.Request)::HTTP.Response
    try
        body = JSON3.read(req.body)
        agent_id = get(body, :agent_id, "unknown")
        neighbor_id = get(body, :neighbor_id, "unknown")

        raw_signals = get(body, :signals, Any[])
        signals = TrustSignal[]
        for s in raw_signals
            push!(signals, TrustSignal(
                String(get(s, :kind, "interaction")),
                Float64(get(s, :weight, 0.0)),
                Int64(get(s, :timestamp, round(Int64, time())))
            ))
        end

        prior = Float64(get(body, :prior, 0.5))

        # neighbor_scores: list of {score, edge_weight} objects from body
        raw_neighbors = get(body, :neighbor_scores, Any[])
        neighbor_scores = Tuple{Float64, Float64}[
            (Float64(get(n, :score, 0.5)), Float64(get(n, :edge_weight, 1.0)))
            for n in raw_neighbors
        ]

        score = compute_trust_score(signals; prior=prior, neighbor_scores=neighbor_scores)

        json_ok(Dict(
            "agent_id"    => agent_id,
            "neighbor_id" => neighbor_id,
            "trust_score" => score,
            "signal_count" => length(signals),
        ))
    catch e
        json_err("trust scoring failed: $e")
    end
end
