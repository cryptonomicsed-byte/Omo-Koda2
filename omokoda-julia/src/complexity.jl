using SHA

"""
    calculate_bbu(code::String) -> Float64

Heuristic Busy Beaver Unit score (1.0–47.1) for a code string.
Based on: lexical entropy + estimated loop depth + branch count.
"""
function calculate_bbu(code::String)::Float64
    # Lexical entropy estimate
    bytes = Vector{UInt8}(code)
    freq = zeros(Int, 256)
    for b in bytes
        freq[b + 1] += 1
    end
    n = length(bytes)
    entropy = 0.0
    for f in freq
        if f > 0
            p = f / n
            entropy -= p * log2(p)
        end
    end
    # Normalize entropy (max is 8 bits for uniform)
    norm_entropy = entropy / 8.0

    # Estimate loop depth by counting keywords
    loop_keywords = ["for", "while", "loop", "recursion", "fold", "map"]
    loop_count = sum(count(kw, code) for kw in loop_keywords)
    loop_score = min(loop_count * 0.5, 3.0)

    # Branch count
    branch_keywords = ["if", "else", "match", "case", "when", "cond"]
    branch_count = sum(count(kw, code) for kw in branch_keywords)
    branch_score = min(branch_count * 0.2, 2.0)

    bbu = 1.0 + norm_entropy * 40.0 + loop_score + branch_score
    return clamp(bbu, 1.0, 47.1)
end

"""
    validate_entropy(seed::Vector{UInt8}) -> Bool

Basic entropy validation: checks byte diversity and avalanche property.
"""
function validate_entropy(seed::Vector{UInt8})::Bool
    length(seed) < 32 && return false

    # Check byte diversity
    unique_bytes = length(Set(seed))
    unique_bytes < 16 && return false

    # Avalanche: flip one bit, check output changes significantly
    h1 = sha256(seed)
    flipped = copy(seed)
    flipped[1] = xor(flipped[1], UInt8(0x01))
    h2 = sha256(flipped)
    diff_bits = sum(count_ones(xor(a, b)) for (a, b) in zip(h1, h2))
    return diff_bits >= 96  # at least 37.5% of 256 bits must differ
end

"""
    check_bb_bound(tier::Int, steps::Int) -> Bool

Check if `steps` is within the known/estimated BB bound for the given tier.
Tier → state count mapping: T0→1, T1→2, T2→3, T3→4, T4→4, T5→5
"""
function check_bb_bound(tier::Int, steps::Int)::Bool
    state_map = Dict(0 => 1, 1 => 2, 2 => 3, 3 => 4, 4 => 4, 5 => 5)
    n = get(state_map, tier, 5)
    return within_bb_bound(n, steps)
end
