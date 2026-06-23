# One round of belief propagation over a flat neighbor vector.
# neighbors: vector of (neighbor_trust_score, edge_weight) pairs
# prior: the agent's own raw trust estimate before propagation
# Returns: updated trust estimate
function propagate_beliefs(prior::Float64, neighbors::Vector{Tuple{Float64, Float64}}, damping::Float64=0.85)::Float64
    if isempty(neighbors)
        return prior
    end
    total_weight = sum(w for (_, w) in neighbors)
    if total_weight <= 0.0
        return prior
    end
    neighbor_signal = sum(score * w for (score, w) in neighbors) / total_weight
    # Weighted combination: damping * prior + (1-damping) * neighbor influence
    clamped = damping * prior + (1.0 - damping) * neighbor_signal
    clamp(clamped, 0.0, 1.0)
end

# Run belief propagation for `iters` rounds.
function run_belief_prop(prior::Float64, neighbors::Vector{Tuple{Float64, Float64}}, iters::Int=3)::Float64
    score = prior
    for _ in 1:iters
        score = propagate_beliefs(score, neighbors)
    end
    score
end
