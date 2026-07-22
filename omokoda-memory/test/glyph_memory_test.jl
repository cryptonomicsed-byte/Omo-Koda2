# Conformance test for the GlyphIndex memory leg — SOMA/REM + birth
# registration. Run standalone (no test deps):  julia glyph_memory_test.jl
#
# Pins the frozen cross-language vectors (OSOVM/GLYPHINDEX_SPEC.md §6) and
# exercises the SOMA store + REM consolidation flow that binds the Ọ̀ṣun
# memory layer to the sovereign glyph address space.

include(joinpath(@__DIR__, "..", "src", "augury.jl"))       # MemoryDAG for soma_store!
include(joinpath(@__DIR__, "..", "src", "glyph_memory.jl"))

check(cond, label) = cond || error("conformance failure: $label")

# --- GIX-FOLD-v1 frozen vectors ---------------------------------------------
frozen = [
    ("Àṣẹ", 21841, 227, 58152),
    ("hello", 23636, 44, 11506),
    ("GlyphIndex", 13726, 68, 17595),
    ("😊🚀 Unicode test", 64591, 189, 48626),
    ("Ọ̀rúnmìlà", 17963, 204, 52390),
]
for (text, cp, base, composed) in frozen
    digest = glyph_content_hash(text)
    check(Int(only(glyph_fold(digest))) == cp, "fold $text")
    b, c = odu_link(digest)
    check(b == base && c == composed, "odu $text")
end

# --- graph verbs + merge -----------------------------------------------------
mem = GlyphMemory()
a = glyph_node("shared chunk", 0.0)
glyph_insert!(mem, a)
glyph_insert!(mem, glyph_node("only mine", 1.0))

peer = GlyphMemory()
p = glyph_node("shared chunk", 99.0)
p.tags = ["from:peer"]; p.walrus_blob_id = "walrus://peer/blob"
glyph_insert!(peer, p)
glyph_insert!(peer, glyph_node("only theirs", 3.0))
nodes_added, edges_added = glyph_merge!(mem, peer)
check(nodes_added == 1 && edges_added == 0, "merge stats")
check(mem.nodes[a.canonical_id].ts == 0.0, "earliest ts wins")
check(mem.nodes[a.canonical_id].walrus_blob_id == "walrus://peer/blob", "locator adopted")
n2, e2 = glyph_merge!(mem, peer)
check(n2 == 0 && e2 == 0, "merge idempotent")

# --- SOMA store + birth registration + REM consolidation --------------------
dag = MemoryDAG()
vault = GlyphMemory()
birth = register_birth!(vault, "agent-esu", "I am Èṣù, born at the crossroads.", 0.0)
check("birth" in birth.tags, "birth tagged")

_, g1 = soma_store!(dag, vault, "agent-esu", "first think turn", [0.5, 0.1], 1.0)
_, g2 = soma_store!(dag, vault, "agent-esu", "second think turn", [0.6, 0.2], 2.0;
                    parent_text = "first think turn")
check(haskey(dag.nodes, g1.canonical_id), "soma snapshot landed in DAG")
# g1 -> g2 follows edge exists
desc = glyph_describe(vault, g1.canonical_id)
check(any(e -> e[2] == g2.canonical_id && e[3] == "follows", desc.outgoing), "soma follows edge")

added = rem_consolidate!(vault, "agent-esu")
check(added >= 2, "rem consolidation links soma nodes to birth")
# birth now reaches both soma snapshots via rem-cluster edges
reachable = glyph_walk(vault, birth.canonical_id, 1)
reachable_ids = Set(n.canonical_id for n in reachable)
check(g1.canonical_id in reachable_ids && g2.canonical_id in reachable_ids,
      "birth anchors the rem cluster")

# --- anchoring ---------------------------------------------------------------
check(glyph_merkle_root(Tuple{String,Vector{UInt8}}[]) == GIX1_EMPTY_ROOT, "empty root")
entries = [(glyph_canonical_id(glyph_content_hash(text)),
            glyph_content_hash(glyph_canonical_id(glyph_content_hash(text))))
           for (text, _, _, _) in frozen]
check(glyph_merkle_root(entries) == "b6c97879f0b04824c626cef414c8be9f459abd853743e013b25ccb34256015ed",
      "cross-language merkle vector root")

blob = vcat(Vector{UInt8}(b"GIX1"), UInt8[1, 0], zeros(UInt8, 28))
check(gix1_audit(blob), "gix1 audit accepts")
bad = copy(blob); bad[5] = 0x09
check(!gix1_audit(bad), "bad version rejected")

println("glyphindex omokoda-memory conformance ok")
