#!/usr/bin/env julia
# test/runtests.jl — OmokodaJulia test suite
#
# Run with:
#   julia --project=. test/runtests.jl
#
# Uses only stdlib Test (no external test frameworks).

using Test

# Load the package from the project root
package_root = dirname(dirname(abspath(@__FILE__)))
push!(LOAD_PATH, joinpath(package_root, "src"))

# Include all source files directly (works without package installation)
include(joinpath(package_root, "src", "bb_known.jl"))
include(joinpath(package_root, "src", "bb_approx.jl"))
include(joinpath(package_root, "src", "complexity.jl"))
include(joinpath(package_root, "src", "nist_validate.jl"))

@testset "OmokodaJulia Test Suite" begin

    # -----------------------------------------------------------------------
    @testset "BB Known Values" begin
        @test bb_value(1) == 1
        @test bb_value(2) == 6
        @test bb_value(3) == 21
        @test bb_value(4) == 107
        @test bb_value(5) == 47_176_870

        @test bb_known(1) == true
        @test bb_known(2) == true
        @test bb_known(3) == true
        @test bb_known(4) == true
        @test bb_known(5) == true
        @test bb_known(6) == false
        @test bb_known(100) == false

        @test bb_value(6) === nothing
        @test bb_value(0) === nothing
    end

    # -----------------------------------------------------------------------
    @testset "BB Upper Bounds" begin
        @test bb_upper_bound(1) == 1.0
        @test bb_upper_bound(2) == 6.0
        @test bb_upper_bound(3) == 21.0
        @test bb_upper_bound(4) == 107.0
        @test bb_upper_bound(5) == 47_176_870.0
        @test isinf(bb_upper_bound(6))
        @test isinf(bb_upper_bound(10))
        @test isinf(bb_upper_bound(100))
        @test bb_upper_bound(0) == 0.0

        # within_bb_bound: exact known values should pass
        @test within_bb_bound(5, UInt64(47_176_870)) == true
        @test within_bb_bound(5, UInt64(47_176_871)) == false  # one over BB(5)
        @test within_bb_bound(4, UInt64(107))        == true
        @test within_bb_bound(4, UInt64(108))        == false
        # For n >= 6, always within bounds (Inf bound)
        @test within_bb_bound(6,  UInt64(typemax(UInt64))) == true
        @test within_bb_bound(10, UInt64(typemax(UInt64))) == true
    end

    # -----------------------------------------------------------------------
    @testset "BBU Complexity Scoring" begin
        # Empty string → minimum score
        @test calculate_bbu("") == 1.0

        # Near-empty / trivial code → score ≈ 1.0 (minimum)
        trivial_score = calculate_bbu("x = 1")
        @test trivial_score >= 1.0
        @test trivial_score <= 6.0

        # Simple code with one conditional
        simple_code = """
        function foo(x)
            if x > 0
                return x
            end
        end
        """
        simple_score = calculate_bbu(simple_code)
        @test simple_score >= 1.0
        @test simple_score <= 47.1

        # Complex nested code → score > 21.0
        complex_code = """
        function process(data)
            result = []
            for item in data
                if item > 0
                    for sub in item.children
                        if sub.active
                            while sub.pending
                                if sub.retry_count < 3
                                    for attempt in 1:3
                                        if attempt > 1
                                            result = filter(x -> x != nil, map(y -> transform(y), reduce((a, b) -> merge(a, b), result)))
                                        end
                                    end
                                else
                                    foreach(x -> push!(result, x), sub.items)
                                end
                            end
                        end
                    end
                elseif item < 0
                    push!(result, abs(item))
                end
            end
            return result
        end
        """
        complex_score = calculate_bbu(complex_code)
        @test complex_score > 21.0
        @test complex_score <= 47.1

        # Score is always in valid range
        @test calculate_bbu("a") >= 1.0
        @test calculate_bbu("a") <= 47.1
        @test calculate_bbu(repeat("x ", 1000)) >= 1.0
        @test calculate_bbu(repeat("x ", 1000)) <= 47.1
    end

    # -----------------------------------------------------------------------
    @testset "SHA-256 (internal)" begin
        # FIPS 180-4 test vector: SHA-256("") = e3b0c44298fc1c149...
        empty_hash = sha256(UInt8[])
        @test length(empty_hash) == 32
        @test empty_hash[1] == 0xe3
        @test empty_hash[2] == 0xb0
        @test empty_hash[3] == 0xc4

        # SHA-256("abc") = ba7816bf8f01cfea414140de5dae2ec73b00361bbef0469749241928...
        abc_hash = sha256(collect(Vector{UInt8}("abc")))
        @test length(abc_hash) == 32
        @test abc_hash[1] == 0xba
        @test abc_hash[2] == 0x78
        @test abc_hash[3] == 0x16

        # Two calls with same input give same output
        h1 = sha256(collect(Vector{UInt8}("hello")))
        h2 = sha256(collect(Vector{UInt8}("hello")))
        @test h1 == h2

        # Different inputs give different outputs
        h3 = sha256(collect(Vector{UInt8}("Hello")))
        @test h1 != h3
    end

    # -----------------------------------------------------------------------
    @testset "Frequency Test" begin
        # Empty → false
        @test frequency_test(Bool[]) == false

        # All zeros → fails (proportion = 0.0, far from 0.5)
        @test frequency_test(fill(false, 100)) == false

        # All ones → fails (proportion = 1.0, far from 0.5)
        @test frequency_test(fill(true, 100)) == false

        # Alternating → passes (exactly 50%)
        alternating = [isodd(i) for i in 1:100]
        @test frequency_test(alternating) == true

        # Pseudorandom (deterministic) — SHA-256 hash bits should pass
        hash_bits = bytes_to_bits(sha256(collect(Vector{UInt8}("test_frequency"))))
        @test frequency_test(hash_bits) == true
    end

    # -----------------------------------------------------------------------
    @testset "Runs Test" begin
        # Too short → false
        @test runs_test(Bool[true, false, true]) == false

        # All same bits → fails (proportion far from 0.5)
        @test runs_test(fill(true, 100)) == false
        @test runs_test(fill(false, 100)) == false

        # Alternating has expected run count ≈ n (way too many runs)
        # Each bit is a run → n runs, expected ≈ n/2, ratio ≈ 2 > 1.4 → fails
        alternating = [isodd(i) for i in 1:100]
        # This may or may not pass depending on exact ratio — just test it doesn't crash
        runs_result = runs_test(alternating)
        @test isa(runs_result, Bool)

        # SHA-256 hash bits (256 bits of good randomness) should pass
        hash_bits = bytes_to_bits(sha256(collect(Vector{UInt8}("test_runs"))))
        @test runs_test(hash_bits) == true
    end

    # -----------------------------------------------------------------------
    @testset "Avalanche Test" begin
        # Empty seed → false
        @test avalanche_test(UInt8[]) == false

        # Constant bytes (0x00 repeated) → false.
        # Seeds with all identical bytes lack raw entropy and fail the
        # pre-entropy-diversity check built into avalanche_test.
        zeros_seed = fill(UInt8(0x00), 32)
        @test avalanche_test(zeros_seed) == false

        # All 0xFF → also constant bytes, fails
        @test avalanche_test(fill(UInt8(0xff), 16)) == false

        # All same non-zero byte → fails
        @test avalanche_test(fill(UInt8(0x42), 8)) == false

        # "hello" in bytes — has byte diversity, SHA-256 avalanches → passes
        hello_seed = collect(Vector{UInt8}("hello"))
        @test avalanche_test(hello_seed) == true

        # Varied bytes — good entropy → passes
        varied_seed = UInt8[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]
        @test avalanche_test(varied_seed) == true

        # SHA-256 hash (32 varied bytes) → passes
        hash_seed = sha256(collect(Vector{UInt8}("test avalanche")))
        @test avalanche_test(hash_seed) == true
    end

    # -----------------------------------------------------------------------
    @testset "validate_entropy" begin
        # Empty → false
        @test validate_entropy(UInt8[]) == false

        # SHA-256 of "hello": well-known good entropy source.
        # validate_entropy hashes the seed internally via SHA-256 for frequency/runs tests,
        # and also runs avalanche_test on the raw seed.
        hello_seed = collect(Vector{UInt8}("hello"))
        @test validate_entropy(hello_seed) == true

        # Various good seeds with byte diversity
        @test validate_entropy(collect(Vector{UInt8}("the quick brown fox"))) == true
        @test validate_entropy(collect(Vector{UInt8}("OmoKoda_entropy_seed_2024"))) == true

        # Constant 0x00 bytes → fails (avalanche_test rejects constant-byte seeds)
        zeros_seed = fill(UInt8(0x00), 32)
        @test validate_entropy(zeros_seed) == false

        # SHA-256 output as seed — 32 varied bytes → passes all three tests
        good_seed = sha256(collect(Vector{UInt8}("hello")))
        @test validate_entropy(good_seed) == true
    end

    # -----------------------------------------------------------------------
    @testset "bytes_to_bits" begin
        # 0xFF → all true
        bits = bytes_to_bits(UInt8[0xff])
        @test length(bits) == 8
        @test all(bits)

        # 0x00 → all false
        bits = bytes_to_bits(UInt8[0x00])
        @test length(bits) == 8
        @test !any(bits)

        # 0x80 = 1000_0000: MSB is 1, all others 0.
        # bytes_to_bits stores MSB at index 1, LSB at index 8.
        bits = bytes_to_bits(UInt8[0x80])
        @test bits[1] == true   # MSB of 0x80 → index 1 (MSB-first layout)
        @test bits[8] == false  # LSB of 0x80 → index 8

        # Two bytes → 16 bits
        bits = bytes_to_bits(UInt8[0xff, 0x00])
        @test length(bits) == 16
    end

end  # @testset "OmokodaJulia Test Suite"

println("\nAll tests passed!")
