# Conservative upper bounds for Busy Beaver values beyond n=5.
# Since BB is non-computable, these are engineering estimates based on known
# growth patterns and computability theory, used as safety thresholds only.

# Known growth relationship:
#   BB(5) = 47_176_870
#   BB(6) >= 10^18267 (Green's number lower bound, beyond practical representation)
#   BB grows faster than any computable function (Ackermann, etc.)
#
# For practical engineering use, we return Float64 upper bounds that saturate
# at Inf for n >= 6 since the values are unrepresentably large.

"""
    bb_upper_bound(n::Int) -> Float64

Return a conservative upper bound for BB(n).

For n <= 5, returns the exact known value as Float64.
For n == 6, returns Inf (the true lower bound exceeds Float64 max).
For n > 6, returns Inf (non-computable; growth exceeds any computable function).

This function is intended for threshold comparisons only. When the result is Inf,
any finite estimated_steps value is considered within bounds (the machine would
never actually reach a halting decision in practice).
"""
function bb_upper_bound(n::Int)::Float64
    if n <= 0
        return 0.0
    elseif n == 1
        return 1.0
    elseif n == 2
        return 6.0
    elseif n == 3
        return 21.0
    elseif n == 4
        return 107.0
    elseif n == 5
        return 47_176_870.0
    else
        # BB(6) has a lower bound of ~10^18267, which vastly exceeds Float64 max (~1.8e308).
        # Return Inf as a conservative safe upper bound — any finite computation
        # is trivially within this bound.
        return Inf
    end
end

"""
    within_bb_bound(n::Int, estimated_steps::UInt64) -> Bool

Return true if `estimated_steps` is within the upper bound for BB(n).
For n >= 6, always returns true since BB(n) exceeds Float64.
"""
function within_bb_bound(n::Int, estimated_steps::UInt64)::Bool
    bound = bb_upper_bound(n)
    if isinf(bound)
        return true
    end
    return Float64(estimated_steps) <= bound
end
