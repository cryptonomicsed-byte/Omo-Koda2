# Conservative upper bounds for n > 5 (not exact — BB is uncomputable for n≥6)
# These are generous upper bounds used for tier gating only

function bb_conservative_bound(n::Int)::Float64
    if n <= 5
        return Float64(BB_KNOWN[n])
    end
    # Ackermann-like super-exponential growth estimate
    return 2.0^(2.0^n)
end

function within_bb_bound(n::Int, steps::Int)::Bool
    bound = bb_known_limit(n)
    if bound !== nothing
        return steps <= bound
    end
    return steps <= bb_conservative_bound(n)
end
