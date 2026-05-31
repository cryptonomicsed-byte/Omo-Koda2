using Test

include("../src/soma/MemCell.jl")
include("../src/soma/ConstitutionalMemory.jl")

using .SomaConstitutionalMemory
using .SomaMemCell: MemCell, EmotionTrace

@testset "ConstitutionalPrior" begin
    prior = ConstitutionalPrior(:mentalism)
    @test prior.principle == :mentalism
    @test prior.mean_score ≈ 0.75f0
    @test prior.confidence ≈ 0.10f0
    @test prior.story_count == 0
    @test prior.last_violation == 0.0
end

@testset "ConstitutionalMemory construction" begin
    mem = ConstitutionalMemory()
    @test length(mem.priors) == 7
    @test mem.max_stories == 200
    @test isempty(mem.stories)
    for p in [:mentalism, :correspondence, :vibration, :polarity, :rhythm, :cause_and_effect, :gender]
        @test haskey(mem.priors, p)
    end
end

@testset "update_prior! — running mean update" begin
    mem = ConstitutionalMemory()
    story = ConstitutionalStory(
        :mentalism, 0.90f0, :aligned,
        "declared clear intent", time()
    )
    update_prior!(mem, story)
    prior = mem.priors[:mentalism]
    # After 1 update: (0*0.75 + 0.90) / 1 = 0.90
    @test prior.mean_score ≈ 0.90f0 atol=0.01
    @test prior.story_count == 1
    @test prior.confidence > 0.10f0
end

@testset "update_prior! — blocked story sets last_violation" begin
    mem = ConstitutionalMemory()
    now_ts = time()
    story = ConstitutionalStory(
        :cause_and_effect, 0.10f0, :blocked,
        "deception detected", now_ts
    )
    update_prior!(mem, story)
    prior = mem.priors[:cause_and_effect]
    @test prior.last_violation ≈ now_ts atol=1.0
end

@testset "update_prior! — aligned story does not set last_violation" begin
    mem = ConstitutionalMemory()
    story = ConstitutionalStory(:polarity, 0.88f0, :aligned, "balanced action", time())
    update_prior!(mem, story)
    @test mem.priors[:polarity].last_violation == 0.0
end

@testset "update_prior! — confidence grows with story count" begin
    mem = ConstitutionalMemory()
    for i in 1:20
        story = ConstitutionalStory(:rhythm, 0.85f0, :aligned, "ok", time())
        update_prior!(mem, story)
    end
    prior = mem.priors[:rhythm]
    @test prior.confidence > 0.60f0
    @test prior.confidence <= 0.95f0
    @test prior.story_count == 20
end

@testset "update_prior! — ring buffer caps at max_stories" begin
    mem = ConstitutionalMemory(; max_stories=5)
    for i in 1:10
        story = ConstitutionalStory(:vibration, Float32(i)/10, :aligned, "ok", time())
        update_prior!(mem, story)
    end
    @test length(mem.stories) == 5
end

@testset "constitutional_recall — returns top_n cells" begin
    mem = ConstitutionalMemory()
    # Seed the prior with high alignment
    for _ in 1:5
        update_prior!(mem, ConstitutionalStory(:mentalism, 0.90f0, :aligned, "ok", time()))
    end

    emotion = EmotionTrace(0.7f0, 0.2f0, 0.8f0, 0.9f0)
    cells = [
        MemCell("c$i", "content $i", Float32(i) / 10, emotion, time() - i * 100.0)
        for i in 1:8
    ]

    recalled = constitutional_recall(mem, cells, :mentalism; top_n=3)
    @test length(recalled) == 3
end

@testset "constitutional_recall — returns fewer when cells < top_n" begin
    mem = ConstitutionalMemory()
    emotion = EmotionTrace(0.5f0, 0.5f0, 0.5f0, 0.5f0)
    cells = [MemCell("c1", "hello", 0.8f0, emotion, time())]
    recalled = constitutional_recall(mem, cells, :polarity; top_n=5)
    @test length(recalled) == 1
end

@testset "prior_to_context — produces non-empty string" begin
    mem = ConstitutionalMemory()
    ctx = prior_to_context(mem)
    @test occursin("[Constitutional Priors]", ctx)
    @test occursin("mentalism", ctx)
    @test occursin("cause_and_effect", ctx)
end

@testset "prior_to_context — violation flag appears after blocked story" begin
    mem = ConstitutionalMemory()
    update_prior!(mem, ConstitutionalStory(:rhythm, 0.10f0, :blocked, "cooldown", time()))
    ctx = prior_to_context(mem)
    @test occursin("⚠", ctx)
end

@testset "multiple principles updated independently" begin
    mem = ConstitutionalMemory()
    update_prior!(mem, ConstitutionalStory(:mentalism, 0.95f0, :aligned, "ok", time()))
    update_prior!(mem, ConstitutionalStory(:vibration, 0.40f0, :warned, "tension", time()))

    @test mem.priors[:mentalism].mean_score > mem.priors[:vibration].mean_score
    @test mem.priors[:correspondence].story_count == 0  # untouched
end
