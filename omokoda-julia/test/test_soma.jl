using Test
using Dates

include("../src/soma/MemCell.jl")
include("../src/soma/Resonance.jl")
include("../src/soma/LPMUpdater.jl")

using .SomaMemCell
using .SomaResonance
using .SomaLPMUpdater

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

function make_emotion(; score=0.5f0, valence=:neutral, tension=0.3f0, connection=0.3f0)
    EmotionTrace(score, valence, tension, connection)
end

function make_cell(;
    id        = "cell-1",
    timestamp = DateTime(2026, 5, 31, 12, 0, 0),
    text      = "test memory",
    emotion   = make_emotion(),
    importance = 0.5f0,
    foresight  = String[],
    tags       = String[],
    theme      = nothing,
    activations = UInt32(0),
    vector     = nothing
)
    MemCell(id, timestamp, text, emotion, importance, foresight, tags, theme, activations, vector)
end

# ---------------------------------------------------------------------------
# score_retrieval
# ---------------------------------------------------------------------------

@testset "score_retrieval" begin
    @testset "returns Float32" begin
        now  = DateTime(2026, 5, 31, 13, 0, 0)
        cell = make_cell()
        e    = make_emotion()
        s    = score_retrieval(cell, now, e)
        @test s isa Float32
    end

    @testset "result is in [0, 1] for typical inputs" begin
        now  = DateTime(2026, 5, 31, 13, 0, 0)
        cell = make_cell(importance=0.8f0, activations=UInt32(5))
        e    = make_emotion(score=0.9f0)
        s    = score_retrieval(cell, now, e)
        @test 0.0f0 <= s <= 1.0f0
    end

    @testset "recent cell scores higher than old cell" begin
        now    = DateTime(2026, 5, 31, 12, 0, 0)
        recent = make_cell(timestamp=DateTime(2026, 5, 31, 11, 30, 0))   # 30 min ago
        old    = make_cell(timestamp=DateTime(2026, 5, 29, 12, 0, 0))    # 48 hrs ago
        e      = make_emotion()
        @test score_retrieval(recent, now, e) > score_retrieval(old, now, e)
    end

    @testset "high tension + high connection boosts score" begin
        now     = DateTime(2026, 5, 31, 12, 0, 0)
        ts      = DateTime(2026, 5, 31, 11, 0, 0)
        # cell and current emotion both have tension > 0.5 and connection > 0.6
        tense_emotion  = make_emotion(tension=0.8f0, connection=0.7f0)
        neutral_emotion = make_emotion(tension=0.1f0, connection=0.1f0)
        boosted = make_cell(timestamp=ts, emotion=tense_emotion)
        plain   = make_cell(timestamp=ts, emotion=neutral_emotion)
        @test score_retrieval(boosted, now, tense_emotion) >
              score_retrieval(plain,   now, neutral_emotion)
    end

    @testset "activation_boost caps at 0.2" begin
        now       = DateTime(2026, 5, 31, 12, 0, 0)
        ts        = DateTime(2026, 5, 31, 11, 50, 0)
        few_acts  = make_cell(timestamp=ts, activations=UInt32(2))
        many_acts = make_cell(timestamp=ts, activations=UInt32(100))
        e         = make_emotion()
        @test score_retrieval(many_acts, now, e) > score_retrieval(few_acts, now, e)
        # The gap between 10 and 100 activations should be zero (both cap at 0.2)
        acts10  = make_cell(timestamp=ts, activations=UInt32(10))
        acts100 = make_cell(timestamp=ts, activations=UInt32(100))
        @test score_retrieval(acts10, now, e) == score_retrieval(acts100, now, e)
    end
end

# ---------------------------------------------------------------------------
# score_all
# ---------------------------------------------------------------------------

@testset "score_all" begin
    @testset "returns Vector{Float32} with correct length" begin
        now   = DateTime(2026, 5, 31, 12, 0, 0)
        cells = [make_cell(id="c$i") for i in 1:5]
        e     = make_emotion()
        scores = score_all(cells, now, e)
        @test scores isa Vector{Float32}
        @test length(scores) == 5
    end

    @testset "empty input returns empty vector" begin
        now    = DateTime(2026, 5, 31, 12, 0, 0)
        scores = score_all(MemCell[], now, make_emotion())
        @test isempty(scores)
    end
end

# ---------------------------------------------------------------------------
# reconstruct
# ---------------------------------------------------------------------------

@testset "reconstruct" begin
    @testset "returns non-empty string when LPM has identity" begin
        lpm = LPM(
            ["I am a curious agent"],
            String[],
            String[],
            String[],
            String[],
            String[]
        )
        result = reconstruct(MemCell[], MemScene[], lpm, "test query", make_emotion())
        @test !isempty(result)
        @test occursin("I am a curious agent", result)
    end

    @testset "includes foresight section when present" begin
        lpm = LPM(
            ["identity statement"],
            String[],
            String[],
            String[],
            ["remember to be kind"],
            String[]
        )
        result = reconstruct(MemCell[], MemScene[], lpm, "query", make_emotion())
        @test occursin("Foresight", result)
        @test occursin("remember to be kind", result)
    end

    @testset "triggers shown only when tension > 0.4" begin
        lpm = LPM(
            ["id"],
            String[],
            ["conflict triggers me"],
            String[],
            String[],
            String[]
        )
        low_tension  = make_emotion(tension=0.2f0)
        high_tension = make_emotion(tension=0.7f0)
        low_result   = reconstruct(MemCell[], MemScene[], lpm, "x", low_tension)
        high_result  = reconstruct(MemCell[], MemScene[], lpm, "x", high_tension)
        @test !occursin("conflict triggers me", low_result)
        @test occursin("conflict triggers me", high_result)
    end

    @testset "pattern keyword matching works" begin
        lpm = LPM(
            ["id"],
            ["agent learns from feedback loops", "agent eats breakfast"],
            String[],
            String[],
            String[],
            String[]
        )
        result = reconstruct(MemCell[], MemScene[], lpm, "feedback and learning", make_emotion())
        @test occursin("feedback", result)
    end

    @testset "scenes ordered by strength descending" begin
        now    = DateTime(2026, 5, 31)
        scenes = [
            MemScene("weak", "summary A", String[], UInt32(1), now),
            MemScene("strong", "summary B", String[], UInt32(99), now),
        ]
        lpm    = LPM(["id"], String[], String[], String[], String[], String[])
        result = reconstruct(MemCell[], scenes, lpm, "q", make_emotion())
        idx_strong = findfirst("strong", result)
        idx_weak   = findfirst("weak",   result)
        @test !isnothing(idx_strong)
        @test !isnothing(idx_weak)
        @test first(idx_strong) < first(idx_weak)
    end

    @testset "identity placeholder when LPM identity is empty" begin
        lpm    = LPM(String[], String[], String[], String[], String[], String[])
        result = reconstruct(MemCell[], MemScene[], lpm, "q", make_emotion())
        @test occursin("not yet established", result)
    end
end

# ---------------------------------------------------------------------------
# cosine_similarity
# ---------------------------------------------------------------------------

@testset "cosine_similarity" begin
    @testset "identical vectors return 1.0" begin
        v = Float32[1.0, 2.0, 3.0]
        @test isapprox(cosine_similarity(v, v), 1.0f0, atol=1e-6)
    end

    @testset "orthogonal vectors return 0.0" begin
        a = Float32[1.0, 0.0, 0.0]
        b = Float32[0.0, 1.0, 0.0]
        @test isapprox(cosine_similarity(a, b), 0.0f0, atol=1e-6)
    end

    @testset "zero vector returns 0.0 without error" begin
        a = Float32[0.0, 0.0, 0.0]
        b = Float32[1.0, 2.0, 3.0]
        @test cosine_similarity(a, b) == 0.0f0
        @test cosine_similarity(b, a) == 0.0f0
    end

    @testset "opposite vectors return -1.0" begin
        a = Float32[1.0, 0.0]
        b = Float32[-1.0, 0.0]
        @test isapprox(cosine_similarity(a, b), -1.0f0, atol=1e-6)
    end

    @testset "returns Float32" begin
        a = Float32[1.0, 0.5]
        b = Float32[0.5, 1.0]
        @test cosine_similarity(a, b) isa Float32
    end
end

# ---------------------------------------------------------------------------
# keyword_score
# ---------------------------------------------------------------------------

@testset "keyword_score" begin
    @testset "returns 0.0 for no shared words" begin
        s = keyword_score("apple orange banana", "elephant giraffe zebra")
        @test s == 0.0f0
    end

    @testset "returns > 0 for shared words longer than 3 chars" begin
        s = keyword_score("agent learns from feedback", "feedback is important")
        @test s > 0.0f0
    end

    @testset "short words (<= 3 chars) are ignored" begin
        # 'the', 'a', 'is', 'to' are all <= 3 chars
        s = keyword_score("the a is to", "the a is to")
        @test s == 0.0f0
    end

    @testset "result is in [0, 1]" begin
        s = keyword_score("learning agent patterns memory retrieval context", "learning patterns memory")
        @test 0.0f0 <= s <= 1.0f0
    end

    @testset "returns Float32" begin
        @test keyword_score("hello world", "hello world") isa Float32
    end
end

# ---------------------------------------------------------------------------
# resonance_boost
# ---------------------------------------------------------------------------

@testset "resonance_boost" begin
    @testset "no boost when both tensions low" begin
        a = make_emotion(tension=0.3f0, connection=0.3f0)
        b = make_emotion(tension=0.3f0, connection=0.3f0)
        @test resonance_boost(a, b) == 0.0f0
    end

    @testset "tension boost of 0.15 when both > 0.5" begin
        a = make_emotion(tension=0.8f0, connection=0.3f0)
        b = make_emotion(tension=0.7f0, connection=0.3f0)
        @test isapprox(resonance_boost(a, b), 0.15f0, atol=1e-6)
    end

    @testset "connection boost of 0.10 when both > 0.6" begin
        a = make_emotion(tension=0.2f0, connection=0.9f0)
        b = make_emotion(tension=0.2f0, connection=0.8f0)
        @test isapprox(resonance_boost(a, b), 0.10f0, atol=1e-6)
    end

    @testset "combined boost capped at 0.25" begin
        a = make_emotion(tension=0.9f0, connection=0.9f0)
        b = make_emotion(tension=0.9f0, connection=0.9f0)
        @test isapprox(resonance_boost(a, b), 0.25f0, atol=1e-6)
    end
end

# ---------------------------------------------------------------------------
# emotional_distance
# ---------------------------------------------------------------------------

@testset "emotional_distance" begin
    @testset "distance of emotion with itself is 0.0" begin
        e = make_emotion(score=0.6f0, tension=0.4f0, connection=0.5f0)
        @test isapprox(emotional_distance(e, e), 0.0f0, atol=1e-6)
    end

    @testset "distance is positive for different emotions" begin
        a = make_emotion(score=0.1f0, tension=0.1f0, connection=0.1f0)
        b = make_emotion(score=0.9f0, tension=0.9f0, connection=0.9f0)
        @test emotional_distance(a, b) > 0.0f0
    end
end

# ---------------------------------------------------------------------------
# merge_identity_traits
# ---------------------------------------------------------------------------

@testset "merge_identity_traits" begin
    @testset "deduplicates by prefix" begin
        existing   = ["curious learner", "empathetic listener"]
        new_traits = ["curious thinker"]   # shares prefix with "curious learner"
        result     = merge_identity_traits(existing, new_traits, 10)
        curious_count = count(t -> startswith(lowercase(t), "curi"), result)
        @test curious_count == 1
    end

    @testset "keeps most recent on duplicate" begin
        existing   = ["pattern old version"]
        new_traits = ["pattern new version"]
        result     = merge_identity_traits(existing, new_traits, 10)
        @test any(t -> t == "pattern new version", result)
        @test !any(t -> t == "pattern old version", result)
    end

    @testset "caps at max_items" begin
        existing   = ["trait $i" for i in 1:8]
        new_traits = ["extra $i" for i in 1:8]
        result     = merge_identity_traits(existing, new_traits, 5)
        @test length(result) <= 5
    end

    @testset "returns Vector{String}" begin
        result = merge_identity_traits(String[], String[], 10)
        @test result isa Vector{String}
    end
end

# ---------------------------------------------------------------------------
# merge_patterns
# ---------------------------------------------------------------------------

@testset "merge_patterns" begin
    @testset "caps at 15" begin
        existing = ["pattern $i" for i in 1:10]
        new_p    = ["extra $i"   for i in 1:10]
        result   = merge_patterns(existing, new_p)
        @test length(result) <= 15
    end

    @testset "deduplicates" begin
        existing = ["recurring avoidance"]
        new_p    = ["recurring approach"]
        result   = merge_patterns(existing, new_p)
        recu_count = count(p -> startswith(lowercase(p), "recu"), result)
        @test recu_count == 1
    end
end

# ---------------------------------------------------------------------------
# update_lpm
# ---------------------------------------------------------------------------

@testset "update_lpm" begin
    @testset "returns LPM with merged fields" begin
        lpm = LPM(
            ["original identity"],
            ["original pattern"],
            ["original trigger"],
            ["original need"],
            ["original foresight"],
            ["original growth"],
        )
        updated = update_lpm(
            lpm,
            ["new identity"],
            ["new pattern"],
            ["new trigger"],
            ["new need"],
        )
        @test updated isa LPM
        @test any(t -> occursin("identity", lowercase(t)), updated.identity)
        @test updated.foresight == lpm.foresight   # unchanged
        @test updated.growth    == lpm.growth       # unchanged
    end
end

# ---------------------------------------------------------------------------
# lpm_to_context
# ---------------------------------------------------------------------------

@testset "lpm_to_context" begin
    @testset "returns non-empty string for populated LPM" begin
        lpm    = LPM(["I am an agent"], ["I reflect"], ["stress"], ["connection"], ["grow"], ["courage"])
        result = lpm_to_context(lpm)
        @test !isempty(result)
        @test occursin("Identity", result)
    end

    @testset "returns placeholder for empty LPM" begin
        lpm    = LPM(String[], String[], String[], String[], String[], String[])
        result = lpm_to_context(lpm)
        @test occursin("not yet initialised", result)
    end

    @testset "omits empty sections" begin
        lpm    = LPM(["core identity"], String[], String[], String[], String[], String[])
        result = lpm_to_context(lpm)
        @test !occursin("Patterns", result)
        @test occursin("Identity", result)
    end
end
