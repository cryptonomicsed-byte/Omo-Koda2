# glyph_memory.jl — GlyphIndex sovereign memory woven into SOMA
#
# Ọ̀ṣun layer of the GlyphIndex contract (spec: OSOVM/GLYPHINDEX_SPEC.md;
# canonical reference: Vantage/backend/glyph_index.py). Omo-Koda2 owns the
# agent lifecycle, so this module covers the kernel-side responsibilities:
#
#   • birth      — forge the agent's glyph vault root and the registration
#                  event Vantage's mesh expects
#   • think/act  — journal salient chunks as glyph nodes and mirror them as
#                  MemoryDAG snapshots so Augury predicts over lived memory
#   • REM        — project glyph nodes into the rem_plan() shape and apply
#                  the returned folds as macro-glyphs (scale-invariant memory)
#
# Sealing happens in the identity layer (BIPỌ̀N39/Cloakseed); this module
# handles fold identity, journaling, and dream-cycle compression only.

using SHA

# ── GIX-FOLD-v1 (identical constants in every ecosystem repo) ───────────────

const GIX_FOLD_RANGES = ((0x0020, 0xD7FF - 0x0020 + 1),
                         (0xE000, 0xFDCF - 0xE000 + 1),
                         (0xFDF0, 0xFFFD - 0xFDF0 + 1))
const GIX_FOLD_TOTAL = sum(r[2] for r in GIX_FOLD_RANGES)

glyph_content_hash(text::AbstractString) = sha256(Vector{UInt8}(codeunits(text)))

function glyph_fold(digest::Vector{UInt8})
    length(digest) == 32 || error("glyph_fold requires a 32-byte digest")
    rem::UInt64 = 0
    for b in digest
        rem = (rem << 8 | UInt64(b)) % UInt64(GIX_FOLD_TOTAL)
    end
    idx = UInt32(rem)
    for (start, count) in GIX_FOLD_RANGES
        idx < count && return Char(start + idx)
        idx -= count
    end
    error("unreachable")
end

glyph_odu_link(digest::Vector{UInt8}) =
    (Int(digest[1]), Int(digest[1]) << 8 | Int(digest[2]))

# ── agent glyph vault (kernel view) ─────────────────────────────────────────

mutable struct GlyphMemoryNode
    canonical_id::String
    glyph::Char
    odu_base::Int
    odu_composed::Int
    ts::Float64
    importance::Float64      # decays in REM; raised on retrieval
    macro_of::Vector{String} # canonical ids folded into this macro-glyph
end

mutable struct AgentGlyphVault
    agent_id::String
    wallet::String
    nodes::Dict{String, GlyphMemoryNode}
    born_at::Float64
end

"""Birth: forge the vault and return the registration event for the Vantage
mesh (`vantage_bridge` posts it verbatim). The vault root id commits to the
agent + wallet identity, so a mesh entry can't be replayed for another soul."""
function glyph_birth(agent_id::AbstractString, wallet::AbstractString;
                     ts::Float64=time())
    vault = AgentGlyphVault(String(agent_id), String(wallet),
                            Dict{String, GlyphMemoryNode}(), ts)
    root_digest = glyph_content_hash("gix-root:$(agent_id):$(wallet)")
    event = Dict(
        "type" => "glyph_vault_registered",
        "agent_id" => String(agent_id),
        "wallet" => String(wallet),
        "vault_root" => bytes2hex(root_digest),
        "vault_glyph" => string(glyph_fold(root_digest)),
        "born_at" => ts,
    )
    vault, event
end

"""Journal one salient chunk out of think/act. Returns the node; optionally
mirrors a snapshot into an Augury `MemoryDAG` so prediction runs over memory
formation (values = [odu_base, importance])."""
function glyph_remember!(vault::AgentGlyphVault, chunk::AbstractString;
                         ts::Float64=time(), importance::Float64=0.5,
                         dag::Union{MemoryDAG,Nothing}=nothing,
                         dag_parent::Union{String,Nothing}=nothing)
    digest = glyph_content_hash(chunk)
    cid = bytes2hex(digest)
    base, composed = glyph_odu_link(digest)
    node = GlyphMemoryNode(cid, glyph_fold(digest), base, composed, ts,
                           importance, String[])
    vault.nodes[cid] = node
    if dag !== nothing
        add_snapshot!(dag, cid, Float64[Float64(base), importance], ts, dag_parent)
    end
    node
end

"""Retrieval touch: remembering something makes it matter more."""
function glyph_touch!(vault::AgentGlyphVault, canonical_id::AbstractString;
                      boost::Float64=0.1)
    node = vault.nodes[String(canonical_id)]
    node.importance = min(node.importance + boost, 1.0)
    node
end

# ── REM / dream-cycle integration ───────────────────────────────────────────

"""Project the vault into the shape `rem_plan` consumes: one dict per node,
with `path` = the node's base-Odù lineage, so the fractal fold clusters
memories that share Calabash ancestry."""
function glyph_rem_nodes(vault::AgentGlyphVault)
    [Dict{String,Any}(
        "id" => n.canonical_id,
        "path" => "odu/$(n.odu_base)",
        "importance" => n.importance,
        "created_at" => n.ts,
     ) for n in values(vault.nodes)]
end

"""Apply a `rem_plan` result: each fold collapses its cluster into one
macro-glyph node (fold of the sorted member ids — deterministic, so every
replica dreams the same dream); pruned ids are dropped outright.
Returns (macro_nodes_created, pruned_count)."""
function glyph_apply_rem!(vault::AgentGlyphVault, plan::Dict{String,Any})
    created = 0
    for fold in get(plan, "folds", [])
        ids = String.(fold["ids"])
        members = [vault.nodes[id] for id in ids if haskey(vault.nodes, id)]
        isempty(members) && continue
        macro_digest = glyph_content_hash("gix-macro:" * join(sort(ids), ","))
        cid = bytes2hex(macro_digest)
        base, composed = glyph_odu_link(macro_digest)
        macro_node = GlyphMemoryNode(cid, glyph_fold(macro_digest), base, composed,
                                     maximum(m.ts for m in members),
                                     maximum(m.importance for m in members),
                                     sort(ids))
        for id in ids
            delete!(vault.nodes, id)
        end
        vault.nodes[cid] = macro_node
        created += 1
    end
    pruned = 0
    for id in get(plan, "prune_ids", [])
        if haskey(vault.nodes, String(id))
            delete!(vault.nodes, String(id))
            pruned += 1
        end
    end
    created, pruned
end

"""One full dream: plan over the current vault, then apply it."""
function glyph_dream!(vault::AgentGlyphVault;
                      noise_importance::Float64=0.35, min_fold_cluster::Int=3)
    plan = rem_plan(glyph_rem_nodes(vault);
                    noise_importance=noise_importance,
                    min_fold_cluster=min_fold_cluster)
    created, pruned = glyph_apply_rem!(vault, plan)
    Dict{String,Any}(
        "fractal_dimension" => plan["fractal_dimension"],
        "macro_glyphs_created" => created,
        "pruned" => pruned,
        "nodes_remaining" => length(vault.nodes),
    )
end
