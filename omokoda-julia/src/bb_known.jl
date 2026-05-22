# Busy Beaver known values
# BB(n) = Σ(n): maximum number of 1s a halting n-state, 2-symbol Turing machine can write.
# These are proven exact values from computability theory.

"""
    BB_KNOWN

Dictionary of known exact Busy Beaver Σ values.
- BB(1) = 1   (proven)
- BB(2) = 6   (proven, Rado 1962)
- BB(3) = 21  (proven, Lin & Rado 1965)
- BB(4) = 107 (proven, Brady 1983)
- BB(5) = 47_176_870 (proven, Marxen & Buntrock 1990, Heiner Marxen 2004)
- BB(6) and beyond: unknown/uncomputed as of 2024
"""
const BB_KNOWN = Dict{Int, Int}(
    1 => 1,
    2 => 6,
    3 => 21,
    4 => 107,
    5 => 47_176_870,
)

"""
    bb_value(n::Int) -> Union{Int, Nothing}

Return the exact known Busy Beaver Σ(n) value for states 1-5, or `nothing` if unknown.
"""
function bb_value(n::Int)::Union{Int, Nothing}
    return get(BB_KNOWN, n, nothing)
end

"""
    bb_known(n::Int) -> Bool

Return true if BB(n) is exactly known.
"""
function bb_known(n::Int)::Bool
    return haskey(BB_KNOWN, n)
end
