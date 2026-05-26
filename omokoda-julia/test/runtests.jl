using Test

include("../src/bb_known.jl")
include("../src/bb_approx.jl")
include("../src/complexity.jl")

@testset "BB Known Limits" begin
    @test bb_known_limit(1) == 1
    @test bb_known_limit(2) == 6
    @test bb_known_limit(3) == 21
    @test bb_known_limit(4) == 107
    @test bb_known_limit(5) == 47_176_870
    @test bb_known_limit(6) === nothing
end

@testset "BB Bound Checking" begin
    @test within_bb_bound(1, 1) == true
    @test within_bb_bound(1, 2) == false
    @test within_bb_bound(5, 47_176_870) == true
    @test within_bb_bound(5, 47_176_871) == false
end

@testset "BBU Calculation" begin
    score = calculate_bbu("fn main() { for i in 0..10 { if i > 5 { println!(i); } } }")
    @test 1.0 <= score <= 47.1
    simple = calculate_bbu("x = 1")
    @test simple < score
end

@testset "Entropy Validation" begin
    good_seed = collect(UInt8, 0:31)
    @test validate_entropy(good_seed) == true
    short_seed = UInt8[1, 2, 3]
    @test validate_entropy(short_seed) == false
end

@testset "check_bb_bound" begin
    @test check_bb_bound(0, 1) == true
    @test check_bb_bound(0, 2) == false
    @test check_bb_bound(5, 47_176_870) == true
end
