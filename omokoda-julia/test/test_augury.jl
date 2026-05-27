using Test
include("../src/augury/predict.jl")
include("../src/augury/analytics.jl")
using .AuguryPredict
using .GardenAnalytics

@testset "AuguryPredict" begin
    @testset "empty patterns returns empty prediction" begin
        result = predict_next_memory(MemoryAccessPattern[])
        @test result.predicted_branch_id == ""
        @test result.confidence == 0.0
    end

    @testset "predicts most recent branch" begin
        now = Float64(time())
        patterns = [
            MemoryAccessPattern("branch_a", now - 100.0, 1.0),
            MemoryAccessPattern("branch_a", now - 50.0, 1.0),
            MemoryAccessPattern("branch_b", now - 200.0, 1.0),
        ]
        result = predict_next_memory(patterns)
        @test result.predicted_branch_id == "branch_a"
        @test result.confidence > 0.5
    end

    @testset "prewarm returns top N" begin
        now = Float64(time())
        patterns = [
            MemoryAccessPattern("branch_$i", now - Float64(i * 10), 1.0)
            for i in 1:5
        ]
        suggestions = prewarm_suggestions(patterns, 3)
        @test length(suggestions) == 3
        @test all(s.confidence > 0.0 for s in suggestions)
    end
end

@testset "GardenAnalytics" begin
    now = Float64(time())
    logs = [
        ReceiptLog("r1", "agent_a", "think", now - 3600.0, 0.05),
        ReceiptLog("r2", "agent_b", "act", now - 1800.0, 0.10),
        ReceiptLog("r3", "agent_a", "think", now - 900.0, 0.08),
        ReceiptLog("r4", "agent_a", "act", now - 300.0, 0.20),
    ]

    @testset "analyze_receipts returns correct totals" begin
        insight = analyze_receipts(logs)
        @test insight.total_receipts == 4
        @test isapprox(insight.total_tips_sui, 0.43, atol=1e-9)
        @test insight.top_agent_id == "agent_a"
    end

    @testset "top_agents returns sorted list" begin
        agents = top_agents(logs, 2)
        @test length(agents) == 2
        @test agents[1][1] == "agent_a"
        @test agents[1][2] > agents[2][2]
    end

    @testset "tip_velocity is positive" begin
        velocity = compute_tip_velocity(logs, 2.0)
        @test velocity > 0.0
    end

    @testset "empty logs returns zero insight" begin
        insight = analyze_receipts(ReceiptLog[])
        @test insight.total_receipts == 0
        @test insight.total_tips_sui == 0.0
    end
end
