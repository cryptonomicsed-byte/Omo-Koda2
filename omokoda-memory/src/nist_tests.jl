"""
NIST SP 800-22 Statistical Randomness Tests — Ọ̀ṣun / Memory layer.

Pre-mainnet requirement: all IfáScript entropy sources must pass this battery
before being admitted as seed material for the Odu oracle or birth DNA generation.

Implemented (7 of 15):
  1.  Frequency (Monobit)
  2.  Frequency within a Block
  3.  Runs
  4.  Longest Run of Ones in a Block
  5.  Approximate Entropy
  6.  Cumulative Sums (forward)
  7.  Serial

Stubbed (8 of 15) — correct interface, returns :not_implemented p-value of -1:
  8.  Binary Matrix Rank
  9.  Discrete Fourier Transform (Spectral)
  10. Non-overlapping Template Matching
  11. Overlapping Template Matching
  12. Maurer's Universal Statistical
  13. Linear Complexity
  14. Random Excursions
  15. Random Excursions Variant

All functions return a NamedTuple: (passed::Bool, p_value::Float64, statistic::Float64, name::String)
p_value threshold: 0.01 (NIST recommended α = 0.01)
"""

using Statistics: mean

const ALPHA = 0.01   # significance level per NIST SP 800-22

struct NISTResult
    name::String
    passed::Bool
    p_value::Float64
    statistic::Float64
    note::String
end

# ---------------------------------------------------------------------------
# Math helpers
# ---------------------------------------------------------------------------

# erfc moved to SpecialFunctions.jl in Julia 1.0; local A&S 7.1.26 approx (max error ~1.5e-7)
function _erfc(x::Float64)::Float64
    x < 0.0 && return 2.0 - _erfc(-x)
    t = 1.0 / (1.0 + 0.3275911 * x)
    p = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))))
    p * exp(-x * x)
end

"""Wilson-Hilferty normal approximation for χ²(k) upper tail probability."""
function chi2_pvalue(chi2::Float64, k::Int)::Float64
    k <= 0 && return 0.0
    chi2 <= 0 && return 1.0
    # Wilson-Hilferty: (χ²/k)^(1/3) ~ Normal(1-2/9k, 2/9k)
    h = (chi2 / k)^(1/3)
    mu = 1.0 - 2.0 / (9k)
    sigma = sqrt(2.0 / (9k))
    z = (h - mu) / sigma
    return 0.5 * _erfc(z / sqrt(2.0))
end

"""Parse input: accepts Vector{Int}, Vector{Bool}, or hex/binary String."""
function parse_bits(input)::Vector{Int}
    if input isa Vector
        return Int.(input)
    elseif input isa String
        s = strip(input)
        # Hex string: "0xABCD" or "ABCD"
        if startswith(s, "0x") || startswith(s, "0X")
            hex = s[3:end]
            bits = Int[]
            for c in hex
                nibble = parse(Int, string(c), base=16)
                for shift in 3:-1:0
                    push!(bits, (nibble >> shift) & 1)
                end
            end
            return bits
        end
        # Binary string: "010110..."
        return [parse(Int, c) for c in s if c in ('0', '1')]
    end
    error("unsupported bit input type: $(typeof(input))")
end

# ---------------------------------------------------------------------------
# 1. Frequency (Monobit) Test
# ---------------------------------------------------------------------------
"""
Checks whether the number of 1s equals the number of 0s.
Reference: NIST SP 800-22 Section 2.1
"""
function test_frequency(bits::Vector{Int})::NISTResult
    n = length(bits)
    n < 100 && return NISTResult("frequency", false, 0.0, 0.0, "n < 100 minimum")
    S_n  = sum(2 .* bits .- 1)   # ±1 representation
    s_obs = abs(S_n) / sqrt(n)
    p = _erfc(s_obs / sqrt(2.0))
    NISTResult("frequency", p >= ALPHA, p, s_obs, "")
end

# ---------------------------------------------------------------------------
# 2. Frequency within a Block
# ---------------------------------------------------------------------------
"""
Divides the sequence into M blocks of length m and tests each.
Reference: NIST SP 800-22 Section 2.2
"""
function test_block_frequency(bits::Vector{Int}; m::Int=128)::NISTResult
    n = length(bits)
    n < 100 && return NISTResult("block_frequency", false, 0.0, 0.0, "n < 100 minimum")
    M = n ÷ m
    M < 1 && return NISTResult("block_frequency", false, 0.0, 0.0, "block size m too large")

    chi2 = sum(1:M) do i
        block = bits[(i-1)*m+1 : i*m]
        pi_i  = sum(block) / m
        4m * (pi_i - 0.5)^2
    end

    p = chi2_pvalue(chi2, M)
    NISTResult("block_frequency", p >= ALPHA, p, chi2, "m=$m M=$M")
end

# ---------------------------------------------------------------------------
# 3. Runs Test
# ---------------------------------------------------------------------------
"""
Checks the total number of runs (uninterrupted sequences of identical bits).
Reference: NIST SP 800-22 Section 2.3
"""
function test_runs(bits::Vector{Int})::NISTResult
    n = length(bits)
    n < 100 && return NISTResult("runs", false, 0.0, 0.0, "n < 100 minimum")

    pi = sum(bits) / n
    if abs(pi - 0.5) >= 2.0 / sqrt(n)
        return NISTResult("runs", false, 0.0, Float64(n), "proportion test failed (π=$pi)")
    end

    V = 1 + sum(bits[i] != bits[i+1] for i in 1:n-1)
    expected = 2n * pi * (1 - pi)
    denom    = 2 * sqrt(2n) * pi * (1 - pi)
    s_obs    = abs(V - expected) / denom
    p        = _erfc(s_obs / sqrt(2.0))
    NISTResult("runs", p >= ALPHA, p, Float64(V), "V=$V expected=$(round(expected, digits=2))")
end

# ---------------------------------------------------------------------------
# 4. Longest Run of Ones in a Block
# ---------------------------------------------------------------------------
"""
Examines runs of consecutive 1s within 8-bit blocks.
Reference: NIST SP 800-22 Section 2.4
"""
function test_longest_run(bits::Vector{Int})::NISTResult
    n = length(bits)
    n < 128 && return NISTResult("longest_run", false, 0.0, 0.0, "n < 128 minimum")

    m = 8  # block length
    M = n ÷ m

    # Count blocks by their longest-run length (capped at 4)
    freq = zeros(Int, 5)   # bins: run ≤ 1, 2, 3, 4+

    for i in 1:M
        block   = bits[(i-1)*m+1 : i*m]
        max_run = _max_run_ones(block)
        idx     = clamp(max_run, 1, 4) + 1   # shift: 0→1, 1→2 ... 4→5
        freq[idx] += 1
    end

    # NIST SP 800-22 Table 2.4.4 for n ∈ [128,6272)
    pi_table = [0.2148, 0.3672, 0.2305, 0.1875]
    k = 3  # 4 categories
    chi2 = sum(1:4) do j
        expected = M * pi_table[j]
        (freq[j+1] - expected)^2 / expected
    end

    p = chi2_pvalue(chi2, k)
    NISTResult("longest_run", p >= ALPHA, p, chi2, "M=$M blocks")
end

function _max_run_ones(block::Vector{Int})::Int
    max_r = 0; cur = 0
    for b in block
        cur = b == 1 ? cur + 1 : 0
        max_r = max(max_r, cur)
    end
    max_r
end

# ---------------------------------------------------------------------------
# 5. Approximate Entropy Test
# ---------------------------------------------------------------------------
"""
Compares the frequency of overlapping m-bit blocks vs (m+1)-bit blocks.
Reference: NIST SP 800-22 Section 2.12
"""
function test_approx_entropy(bits::Vector{Int}; m::Int=10)::NISTResult
    n = length(bits)
    n < 100 && return NISTResult("approx_entropy", false, 0.0, 0.0, "n < 100 minimum")

    phi_m   = _approx_phi(bits, m)
    phi_m1  = _approx_phi(bits, m + 1)
    apen    = phi_m - phi_m1
    chi2    = 2n * (log(2) - apen)
    df      = 2^m
    p       = chi2_pvalue(chi2, df)
    NISTResult("approx_entropy", p >= ALPHA, p, chi2, "m=$m ApEn=$(round(apen, digits=6))")
end

function _approx_phi(bits::Vector{Int}, m::Int)::Float64
    n = length(bits)
    # Augment sequence to wrap: append first m-1 bits
    aug = vcat(bits, bits[1:m-1])
    counts = Dict{Vector{Int}, Int}()
    for i in 1:n
        blk = aug[i:i+m-1]
        counts[blk] = get(counts, blk, 0) + 1
    end
    phi = 0.0
    for c in values(counts)
        p = c / n
        phi += p * log(p)
    end
    phi
end

# ---------------------------------------------------------------------------
# 6. Cumulative Sums Test (forward)
# ---------------------------------------------------------------------------
"""
Tests whether the cumulative sum of ±1 values stays close to 0.
Reference: NIST SP 800-22 Section 2.13
"""
function test_cumulative_sums(bits::Vector{Int})::NISTResult
    n = length(bits)
    n < 100 && return NISTResult("cumulative_sums", false, 0.0, 0.0, "n < 100 minimum")

    x  = 2 .* bits .- 1    # ±1
    S  = cumsum(x)
    z  = maximum(abs.(S))

    # p-value from NIST SP 800-22 eq. 2.13.5 (partial sum approximation)
    p = _cusum_pvalue(Float64(z), n)
    NISTResult("cumulative_sums", p >= ALPHA, p, Float64(z), "mode=forward")
end

function _cusum_pvalue(z::Float64, n::Int)::Float64
    # NIST SP 800-22 Section 2.13 closed-form approximation
    start  = Int(floor((-n/z + 1) / 4))
    stop   = Int(floor(( n/z - 1) / 4))
    sum1 = sum(start:stop) do k
        _Φ((4k + 1) * z / sqrt(n)) - _Φ((4k - 1) * z / sqrt(n))
    end

    start2 = Int(floor((-n/z - 3) / 4))
    stop2  = Int(floor(( n/z - 1) / 4))
    sum2 = sum(start2:stop2) do k
        _Φ((4k + 3) * z / sqrt(n)) - _Φ((4k + 1) * z / sqrt(n))
    end

    1.0 - sum1 + sum2
end

# Standard normal CDF
_Φ(x::Float64) = 0.5 * _erfc(-x / sqrt(2.0))

# ---------------------------------------------------------------------------
# 7. Serial Test
# ---------------------------------------------------------------------------
"""
Measures the uniformity of all 2^m m-bit overlapping patterns.
Reference: NIST SP 800-22 Section 2.11
"""
function test_serial(bits::Vector{Int}; m::Int=16)::NISTResult
    n = length(bits)
    n < 100 && return NISTResult("serial", false, 0.0, 0.0, "n < 100 minimum")

    ψm  = _serial_psi(bits, m)
    ψm1 = _serial_psi(bits, m - 1)
    ψm2 = _serial_psi(bits, m - 2)

    Δψ2  = ψm  - ψm1
    Δ2ψ2 = ψm  - 2ψm1 + ψm2

    p1 = chi2_pvalue(Δψ2,  2^(m-1))
    p2 = chi2_pvalue(Δ2ψ2, 2^(m-2))

    p = min(p1, p2)
    NISTResult("serial", p >= ALPHA, p, Δψ2,
               "m=$m Δψ²=$(round(Δψ2,digits=4)) Δ²ψ²=$(round(Δ2ψ2,digits=4))")
end

function _serial_psi(bits::Vector{Int}, m::Int)::Float64
    m <= 0 && return 0.0
    n   = length(bits)
    aug = vcat(bits, bits[1:m-1])
    counts = Dict{Vector{Int}, Int}()
    for i in 1:n
        blk = aug[i:i+m-1]
        counts[blk] = get(counts, blk, 0) + 1
    end
    (2^m / n) * sum(c^2 for c in values(counts)) - n
end

# ---------------------------------------------------------------------------
# Stubbed tests (correct interface, not yet implemented)
# ---------------------------------------------------------------------------

function _stub(name::String)::NISTResult
    NISTResult(name, false, -1.0, -1.0, "not_yet_implemented")
end

test_binary_matrix_rank(bits)    = _stub("binary_matrix_rank")
test_dft(bits)                   = _stub("dft_spectral")
test_non_overlapping(bits; t="") = _stub("non_overlapping_template")
test_overlapping(bits)           = _stub("overlapping_template")
test_universal(bits)             = _stub("universal_statistical")
test_linear_complexity(bits)     = _stub("linear_complexity")
test_random_excursions(bits)     = _stub("random_excursions")
test_random_excursions_var(bits) = _stub("random_excursions_variant")

# ---------------------------------------------------------------------------
# Full battery
# ---------------------------------------------------------------------------

"""
    run_battery(bits_input; m_block, m_apen, m_serial) → Vector{NISTResult}

Run all 15 NIST SP 800-22 tests.  Returns a result per test.
"""
function run_battery(bits_input; m_block=128, m_apen=10, m_serial=16)::Vector{NISTResult}
    bits = parse_bits(bits_input)
    [
        test_frequency(bits),
        test_block_frequency(bits; m=m_block),
        test_runs(bits),
        test_longest_run(bits),
        test_binary_matrix_rank(bits),
        test_dft(bits),
        test_non_overlapping(bits),
        test_overlapping(bits),
        test_universal(bits),
        test_linear_complexity(bits),
        test_serial(bits; m=m_serial),
        test_approx_entropy(bits; m=m_apen),
        test_cumulative_sums(bits),
        test_random_excursions(bits),
        test_random_excursions_var(bits),
    ]
end

"""
Not-yet-implemented NIST SP 800-22 tests. Each needs its own real statistical
algorithm (binary matrix rank, DFT/spectral, template matching ×2, universal
statistical, linear complexity, random excursions ×2) — these are stubs, not
approximations, and are excluded from pass/fail via the p_value<0 sentinel
`validate_odu_entropy` already filters on. Implement for real before treating
`run_battery`'s output as the full 15-test battery the module docstring
advertises.
"""
function test_binary_matrix_rank(bits::Vector{Int})::NISTResult
    NISTResult("binary_matrix_rank", false, -1.0, 0.0, "not implemented")
end
function test_dft(bits::Vector{Int})::NISTResult
    NISTResult("dft", false, -1.0, 0.0, "not implemented")
end
function test_non_overlapping(bits::Vector{Int})::NISTResult
    NISTResult("non_overlapping", false, -1.0, 0.0, "not implemented")
end
function test_overlapping(bits::Vector{Int})::NISTResult
    NISTResult("overlapping", false, -1.0, 0.0, "not implemented")
end
function test_universal(bits::Vector{Int})::NISTResult
    NISTResult("universal", false, -1.0, 0.0, "not implemented")
end
function test_linear_complexity(bits::Vector{Int})::NISTResult
    NISTResult("linear_complexity", false, -1.0, 0.0, "not implemented")
end
function test_random_excursions(bits::Vector{Int})::NISTResult
    NISTResult("random_excursions", false, -1.0, 0.0, "not implemented")
end
function test_random_excursions_var(bits::Vector{Int})::NISTResult
    NISTResult("random_excursions_var", false, -1.0, 0.0, "not implemented")
end

"""
    validate_odu_entropy(bitstream) → (all_passed, passed_count, results)

High-level entry point for IfáScript Odu entropy validation.
A bitstream is admitted only if all implemented tests pass (p ≥ 0.01).
Stubbed tests are excluded from the pass/fail count.
"""
function validate_odu_entropy(bitstream)
    results  = run_battery(bitstream)
    active   = filter(r -> r.p_value >= 0.0, results)  # exclude stubs
    passed   = filter(r -> r.passed, active)
    all_pass = length(passed) == length(active)
    (all_passed=all_pass, passed=length(passed), total=length(active), results=results)
end
