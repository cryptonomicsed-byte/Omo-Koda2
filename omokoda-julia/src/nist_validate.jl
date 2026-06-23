# NIST SP 800-22 inspired statistical entropy tests.
# Pure Julia implementation — no C library dependencies.
#
# Tests implemented:
#   1. Frequency (Monobit) Test  — proportion of 1s should be ≈ 0.5
#   2. Runs Test                 — runs of identical bits should match expected count
#   3. Avalanche Test            — flipping one input bit should change >45% of hash output bits
#   4. validate_entropy          — runs all three tests; all must pass

# SHA-256 is implemented below in pure Julia (no external deps required).

# ---------------------------------------------------------------------------
# Minimal SHA-256 implementation (pure Julia, no stdlib SHA needed)
# Based on FIPS 180-4
# ---------------------------------------------------------------------------

const SHA256_K = UInt32[
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]

const SHA256_H0 = UInt32[
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]

rotr32(x::UInt32, n::Int) = (x >> n) | (x << (32 - n))

function _sha256_compress!(h::Vector{UInt32}, block::AbstractVector{UInt8})
    w = Vector{UInt32}(undef, 64)
    for i in 1:16
        w[i] = (UInt32(block[4i-3]) << 24) |
               (UInt32(block[4i-2]) << 16) |
               (UInt32(block[4i-1]) << 8)  |
                UInt32(block[4i])
    end
    for i in 17:64
        s0 = rotr32(w[i-15], 7)  ⊻ rotr32(w[i-15], 18) ⊻ (w[i-15] >> 3)
        s1 = rotr32(w[i-2],  17) ⊻ rotr32(w[i-2],  19) ⊻ (w[i-2]  >> 10)
        w[i] = w[i-16] + s0 + w[i-7] + s1
    end

    a, b, c, d, e, f, g, hh = h[1], h[2], h[3], h[4], h[5], h[6], h[7], h[8]

    for i in 1:64
        S1   = rotr32(e, 6)  ⊻ rotr32(e, 11) ⊻ rotr32(e, 25)
        ch   = (e & f) ⊻ (~e & g)
        temp1 = hh + S1 + ch + SHA256_K[i] + w[i]
        S0   = rotr32(a, 2)  ⊻ rotr32(a, 13) ⊻ rotr32(a, 22)
        maj  = (a & b) ⊻ (a & c) ⊻ (b & c)
        temp2 = S0 + maj

        hh = g
        g  = f
        f  = e
        e  = d + temp1
        d  = c
        c  = b
        b  = a
        a  = temp1 + temp2
    end

    h[1] += a; h[2] += b; h[3] += c; h[4] += d
    h[5] += e; h[6] += f; h[7] += g; h[8] += hh
end

"""
    sha256(data::Vector{UInt8}) -> Vector{UInt8}

Pure Julia SHA-256 hash. Returns 32 bytes.
"""
function sha256(data::Vector{UInt8})::Vector{UInt8}
    h = copy(SHA256_H0)
    msg_len_bits = UInt64(length(data)) * 8

    # Pad the message
    padded = copy(data)
    push!(padded, 0x80)
    while length(padded) % 64 != 56
        push!(padded, 0x00)
    end
    # Append 64-bit big-endian bit length
    for i in 7:-1:0
        push!(padded, UInt8((msg_len_bits >> (8 * i)) & 0xff))
    end

    # Process each 64-byte block
    for block_start in 1:64:length(padded)
        _sha256_compress!(h, @view padded[block_start:block_start+63])
    end

    # Produce digest
    digest = Vector{UInt8}(undef, 32)
    for (i, word) in enumerate(h)
        digest[4i-3] = UInt8((word >> 24) & 0xff)
        digest[4i-2] = UInt8((word >> 16) & 0xff)
        digest[4i-1] = UInt8((word >> 8)  & 0xff)
        digest[4i]   = UInt8( word        & 0xff)
    end
    return digest
end

# ---------------------------------------------------------------------------
# NIST SP 800-22 Test 1: Frequency (Monobit) Test
# ---------------------------------------------------------------------------

"""
    frequency_test(bits::Vector{Bool}) -> Bool

NIST SP 800-22 Section 2.1 — Frequency (Monobit) Test.

The proportion of 1s in the bit sequence should be close to 1/2.
Passes if |proportion - 0.5| < 0.1 (i.e., between 40% and 60% ones).

Returns true if the sequence passes, false otherwise.
An empty sequence returns false.
"""
function frequency_test(bits::Vector{Bool})::Bool
    n = length(bits)
    if n == 0
        return false
    end
    ones_count = count(identity, bits)
    proportion = ones_count / n
    return abs(proportion - 0.5) < 0.1
end

# ---------------------------------------------------------------------------
# NIST SP 800-22 Test 2: Runs Test
# ---------------------------------------------------------------------------

"""
    count_runs(bits::Vector{Bool}) -> Int

Count the number of "runs" — maximal sequences of identical bits.
"""
function count_runs(bits::Vector{Bool})::Int
    n = length(bits)
    if n == 0
        return 0
    end
    runs = 1
    for i in 2:n
        if bits[i] != bits[i-1]
            runs += 1
        end
    end
    return runs
end

"""
    runs_test(bits::Vector{Bool}) -> Bool

NIST SP 800-22 Section 2.3 — Runs Test (simplified).

For a random sequence with proportion π of 1s, the expected number of runs is:
    E[runs] = 2 * n * π * (1 - π) + 1  (approximately 2n/4 + 1 ≈ n/2 for π≈0.5)

We accept if the run count is within ±40% of the expected value.
Returns false for sequences shorter than 8 bits.
"""
function runs_test(bits::Vector{Bool})::Bool
    n = length(bits)
    if n < 8
        return false
    end

    # First check the frequency prerequisite (proportion must be near 0.5)
    π = count(identity, bits) / n
    if abs(π - 0.5) >= 0.1
        return false
    end

    actual_runs = count_runs(bits)
    expected_runs = 2.0 * n * π * (1.0 - π)

    if expected_runs == 0.0
        return false
    end

    # Accept if within ±40% of expected
    ratio = actual_runs / expected_runs
    return 0.6 <= ratio <= 1.4
end

# ---------------------------------------------------------------------------
# Avalanche Test
# ---------------------------------------------------------------------------

"""
    bytes_to_bits(data::Vector{UInt8}) -> Vector{Bool}

Convert bytes to a bit vector (MSB first).
"""
function bytes_to_bits(data::Vector{UInt8})::Vector{Bool}
    bits = Vector{Bool}(undef, length(data) * 8)
    for (i, byte) in enumerate(data)
        for j in 0:7
            bits[(i-1)*8 + (8-j)] = (byte >> j) & 0x01 == 0x01
        end
    end
    return bits
end

"""
    hamming_distance_fraction(a::Vector{UInt8}, b::Vector{UInt8}) -> Float64

Compute the fraction of bits that differ between two byte vectors of equal length.
"""
function hamming_distance_fraction(a::Vector{UInt8}, b::Vector{UInt8})::Float64
    @assert length(a) == length(b) "Inputs must have equal length"
    total_bits = length(a) * 8
    diff_bits  = 0
    for (x, y) in zip(a, b)
        diff_bits += count_ones(x ⊻ y)
    end
    return diff_bits / total_bits
end

"""
    seed_has_min_entropy(seed::Vector{UInt8}) -> Bool

Pre-check: verify the seed has at least minimal byte-level diversity.

A seed of all identical bytes (e.g. 0x00 repeated, 0xFF repeated) lacks
sufficient raw entropy to be useful as a cryptographic seed, and any test
relying on it would be misleading. Returns false if all bytes are identical.
"""
function seed_has_min_entropy(seed::Vector{UInt8})::Bool
    if isempty(seed)
        return false
    end
    # All bytes must not be the same value
    if all(b -> b == seed[1], seed)
        return false
    end
    return true
end

"""
    avalanche_test(seed::Vector{UInt8}) -> Bool

Avalanche criterion test: flipping a single input bit should change >45% of
the SHA-256 output bits.

Pre-checks that the seed has minimal byte-level diversity (not all identical
bytes). A seed of constant bytes (e.g. 0x00 repeated) fails immediately
because it lacks sufficient raw entropy — any seed used in production must
have observable variation across its bytes.

For seeds that pass the pre-check, tests by flipping each bit of the first
byte of `seed` and measuring the fraction of SHA-256 output bits that change.
Returns true only if ALL flipped-bit tests produce >= 45% output bit changes.

Returns false for:
- Empty seeds
- Seeds where all bytes are identical (constant bytes like 0x00 repeated)
"""
function avalanche_test(seed::Vector{UInt8})::Bool
    if isempty(seed)
        return false
    end

    # Reject constant-byte seeds: e.g. all 0x00, all 0xFF, etc.
    # These lack raw entropy and should not be used as seeds.
    if !seed_has_min_entropy(seed)
        return false
    end

    base_hash = sha256(seed)
    threshold = 0.45

    # Test flipping each bit in the first byte
    for bit_pos in 0:7
        modified = copy(seed)
        modified[1] = modified[1] ⊻ (UInt8(1) << bit_pos)
        modified_hash = sha256(modified)
        frac = hamming_distance_fraction(base_hash, modified_hash)
        if frac < threshold
            return false
        end
    end

    return true
end

# ---------------------------------------------------------------------------
# Combined Validation
# ---------------------------------------------------------------------------

"""
    validate_entropy(seed::Vector{UInt8}) -> Bool

Run all three NIST-inspired entropy tests on `seed`:
1. Frequency (monobit) test on the SHA-256 hash bits
2. Runs test on the SHA-256 hash bits
3. Avalanche test (single-bit flip changes >45% of hash output)

Returns true only if all three tests pass.
"""
function validate_entropy(seed::Vector{UInt8})::Bool
    if isempty(seed)
        return false
    end

    hash_bytes = sha256(seed)
    bits = bytes_to_bits(hash_bytes)

    freq_ok      = frequency_test(bits)
    runs_ok      = runs_test(bits)
    avalanche_ok = avalanche_test(seed)

    return freq_ok && runs_ok && avalanche_ok
end
