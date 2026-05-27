# Known BB(n) values for 2-symbol, n-state Turing machines
# Source: Radó 1962, Lin & Radó 1965, Brady 1983, Marxen & Buntrock 1989, Busy Beaver Competition

const BB_KNOWN = Dict{Int, Int}(
    1 => 1,
    2 => 6,
    3 => 21,
    4 => 107,
    5 => 47_176_870,
)

function bb_known_limit(n::Int)::Union{Int, Nothing}
    get(BB_KNOWN, n, nothing)
end
