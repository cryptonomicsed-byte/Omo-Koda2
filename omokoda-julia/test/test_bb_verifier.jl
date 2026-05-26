using Test
include("../src/bb_verifier.jl")
using .BbVerifier

@testset "BbVerifier" begin
    @testset "BB_KNOWN values" begin
        @test BB_KNOWN[1] == 1
        @test BB_KNOWN[2] == 6
        @test BB_KNOWN[3] == 21
        @test BB_KNOWN[4] == 107
        @test BB_KNOWN[5] == 47_176_870
    end

    @testset "pocw_floor_for_tier" begin
        @test pocw_floor_for_tier(0) == 0
        @test pocw_floor_for_tier(1) == 21
        @test pocw_floor_for_tier(2) == 107
        @test pocw_floor_for_tier(3) == 47_176_870
        @test pocw_floor_for_tier(4) == 47_176_870
        @test pocw_floor_for_tier(5) == 47_176_870
    end

    @testset "verify_pocw_claim" begin
        @test verify_pocw_claim(21, 1)      # exact BB(3)
        @test verify_pocw_claim(1000, 1)    # more than enough
        @test !verify_pocw_claim(20, 1)     # one short of BB(3)
        @test verify_pocw_claim(107, 2)     # exact BB(4)
        @test !verify_pocw_claim(106, 2)    # one short of BB(4)
        @test verify_pocw_claim(0, 0)       # tier 0 always passes
    end
end
