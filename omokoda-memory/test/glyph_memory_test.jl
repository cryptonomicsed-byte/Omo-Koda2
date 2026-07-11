# glyph_memory_test.jl — GlyphIndex ↔ SOMA integration tests
#
# Run:  julia --project=omokoda-memory omokoda-memory/test/glyph_memory_test.jl
#
# Fold vectors are the frozen cross-language vectors from the canonical
# Python reference implementation (Vantage) — shared by every ecosystem repo.

using Test

include(joinpath(@__DIR__, "..", "src", "OmokodaMemory.jl"))
using .OmokodaMemory

@testset "GIX-FOLD-v1 canonical vectors" begin
    for (text, cid, codepoint, base, composed) in [
        ("Àṣẹ", "e32866670f27c0ccaeda5facc74fcfc3f8c17b18bcae2fb9dc150d91c601db1b", 21841, 227, 58152),
        ("hello", "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824", 23636, 44, 11506),
        ("GlyphIndex", "44bb6336e45b2f5daf764930ac1d1f2798ad92c34048f0395686ac4509a0a7ec", 13726, 68, 17595),
        ("😊🚀 Unicode test", "bdf299182a61f04e31c6445f96a6a68d927d7e6cd9c56f883c8f1cc7cfac8683", 64591, 189, 48626),
        ("Ọ̀rúnmìlà", "cca6a38cbd2874b7f2b4809ba11ee5177660c4ad5fb4851991414722729fd523", 17963, 204, 52390),
    ]
        digest = glyph_content_hash(text)
        @test bytes2hex(digest) == cid
        @test Int(glyph_fold(digest)) == codepoint
        @test glyph_odu_link(digest) == (base, composed)
    end
end

@testset "birth forges vault + registration event" begin
    vault, event = glyph_birth("agent-7", "0xabc123"; ts=1000.0)
    @test vault.agent_id == "agent-7"
    @test isempty(vault.nodes)
    @test event["type"] == "glyph_vault_registered"
    @test length(event["vault_root"]) == 64
    # Deterministic: same soul, same root; different wallet, different root.
    _, event2 = glyph_birth("agent-7", "0xabc123"; ts=2000.0)
    @test event2["vault_root"] == event["vault_root"]
    _, event3 = glyph_birth("agent-7", "0xother")
    @test event3["vault_root"] != event["vault_root"]
end

@testset "remember mirrors into Augury MemoryDAG" begin
    vault, _ = glyph_birth("agent-7", "0xabc123")
    dag = MemoryDAG()
    n1 = glyph_remember!(vault, "first thought"; ts=1.0, dag=dag)
    n2 = glyph_remember!(vault, "second thought"; ts=2.0, dag=dag, dag_parent=n1.canonical_id)
    @test length(vault.nodes) == 2
    @test haskey(dag.nodes, n1.canonical_id)
    @test n2.canonical_id in dag.nodes[n1.canonical_id].children
    path = walk_path(dag, n1.canonical_id)
    @test length(path) == 2
    @test path[1][1] == Float64(n1.odu_base)
    # touching raises importance, capped at 1.0
    glyph_touch!(vault, n1.canonical_id; boost=0.9)
    @test vault.nodes[n1.canonical_id].importance == 1.0
end

@testset "dream folds noise into macro-glyphs" begin
    vault, _ = glyph_birth("agent-7", "0xabc123")
    # Find ≥3 low-importance chunks sharing one base Odù so rem_plan clusters them.
    target_base = glyph_odu_link(glyph_content_hash("noise-1"))[1]
    stored = 0
    i = 1
    while stored < 3 && i < 20000
        chunk = "noise-$i"
        if glyph_odu_link(glyph_content_hash(chunk))[1] == target_base
            glyph_remember!(vault, chunk; ts=Float64(i), importance=0.1)
            stored += 1
        end
        i += 1
    end
    @test stored == 3
    keeper = glyph_remember!(vault, "a precious memory"; ts=99.0, importance=0.9)

    report = glyph_dream!(vault)
    @test report["macro_glyphs_created"] == 1
    @test haskey(vault.nodes, keeper.canonical_id)
    macros = [n for n in values(vault.nodes) if !isempty(n.macro_of)]
    @test length(macros) == 1
    @test length(macros[1].macro_of) == 3
    @test report["nodes_remaining"] == 2
end
