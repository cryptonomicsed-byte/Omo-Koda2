"""
GlyphIndex memory — SOMA/REM integration + agent birth registration.

Omo-Koda2's leg of the ecosystem GlyphIndex contract
(spec: OSOVM/GLYPHINDEX_SPEC.md; canonical reference implementation:
Vantage/backend/glyph_index.py). This module is the keyless projection
that binds the Ọ̀ṣun memory layer to the sovereign glyph address space:

  • GIX-FOLD-v1 fold + Odù linkage (content → glyph, Digital Calabash).
  • A content-addressed GlyphGraph with the LQL-style verbs
    (DESCRIBE / SELECT / WALK / INFER) and A2A merge, wire-compatible with
    every other leg (Rust larql-glyph, TypeScript mnemopi, …).
  • SOMA store integration: a `think`-turn snapshot both lands in the
    Augury `MemoryDAG` (numeric metrics) *and* registers its glyph node,
    so REM consolidation walks the same identities the mesh anchors.
  • Birth registration: an agent's genesis fact is glyphed and pinned as
    the REM-cluster root, giving the agent a sovereign memory address the
    moment it is born.

Sealing/keys live in the identity layer (BIPỌ̀N39 / Cloakseed); this
module holds only metadata — plaintext never enters the glyph graph.
"""

using SHA: sha256

# ---------------------------------------------------------------------------
# GIX-FOLD-v1  (content → glyph)
# ---------------------------------------------------------------------------

const _FOLD_RANGES = ((0x0020, 0xD7FF - 0x0020 + 1),
                      (0xE000, 0xFDCF - 0xE000 + 1),
                      (0xFDF0, 0xFFFD - 0xFDF0 + 1))
const _FOLD_TOTAL = sum(last(r) for r in _FOLD_RANGES)   # 63,422
const GIX1_EMPTY_ROOT = "58cc47f0d238cea8bb764f7a927a54b398c8baf5de0a2332c03008038c3fd9a8"

glyph_content_hash(text::AbstractString) = sha256(Vector{UInt8}(codeunits(text)))

glyph_canonical_id(digest::Vector{UInt8}) = bytes2hex(digest)

function glyph_fold(digest::Vector{UInt8})
    rem = UInt64(0)
    for byte in digest
        rem = (rem << 8 | UInt64(byte)) % UInt64(_FOLD_TOTAL)
    end
    idx = Int(rem)
    for (start, count) in _FOLD_RANGES
        idx < count && return string(Char(Int(start) + idx))
        idx -= count
    end
    error("unreachable: fold index exceeds range total")
end

"""Digital Calabash linkage: (base Odù 0..255, composed Odù 0..65535)."""
odu_link(digest::Vector{UInt8}) = (Int(digest[1]), Int(digest[1]) << 8 | Int(digest[2]))

# ---------------------------------------------------------------------------
# Glyph graph  (content-addressed metadata projection)
# ---------------------------------------------------------------------------

mutable struct GlyphMemoryNode
    canonical_id::String
    glyph::String
    odu_base::Int
    odu_composed::Int
    ts::Float64
    tags::Vector{String}
    walrus_blob_id::Union{String,Nothing}
end

function glyph_node(chunk::AbstractString, ts::Real)
    digest = glyph_content_hash(chunk)
    base, composed = odu_link(digest)
    GlyphMemoryNode(glyph_canonical_id(digest), glyph_fold(digest),
                    base, composed, Float64(ts), String[], nothing)
end

mutable struct GlyphMemory
    nodes::Dict{String,GlyphMemoryNode}
    edges::Set{NTuple{3,String}}
end

GlyphMemory() = GlyphMemory(Dict{String,GlyphMemoryNode}(), Set{NTuple{3,String}}())

function glyph_insert!(mem::GlyphMemory, node::GlyphMemoryNode)
    occursin(r"^[0-9a-f]{64}$", node.canonical_id) ||
        error("canonical id must be 64 lowercase hex chars")
    node.tags = sort(unique(node.tags))
    mem.nodes[node.canonical_id] = node
    node
end

function glyph_link!(mem::GlyphMemory, from::String, to::String, relation::String)
    for id in (from, to)
        haskey(mem.nodes, id) || error("unknown node $id")
    end
    push!(mem.edges, (from, to, relation))
    mem
end

_sorted_edges(mem::GlyphMemory) = sort(collect(mem.edges))

"""`DESCRIBE <id>` — node metadata plus incident edges."""
function glyph_describe(mem::GlyphMemory, id::String)
    haskey(mem.nodes, id) || error("unknown node $id")
    edges = _sorted_edges(mem)
    (node = mem.nodes[id],
     outgoing = [e for e in edges if e[1] == id],
     incoming = [e for e in edges if e[2] == id])
end

"""`SELECT ... WHERE tag`."""
glyph_select_by_tag(mem::GlyphMemory, tag::String) =
    [mem.nodes[id] for id in sort(collect(keys(mem.nodes))) if tag in mem.nodes[id].tags]

"""`WALK <id> DEPTH <n>` — undirected BFS, start excluded, discovery order."""
function glyph_walk(mem::GlyphMemory, start::String, depth::Int)
    haskey(mem.nodes, start) || error("unknown node $start")
    edges = _sorted_edges(mem)
    seen = Set([start])
    queue = Tuple{String,Int}[(start, 0)]
    out = GlyphMemoryNode[]
    while !isempty(queue)
        (id, d) = popfirst!(queue)
        d == depth && continue
        for (from, to, _) in edges
            next = from == id ? to : (to == id ? from : nothing)
            (next === nothing || next in seen) && continue
            push!(seen, next)
            push!(out, mem.nodes[next])
            push!(queue, (next, d + 1))
        end
    end
    out
end

"""`INFER` — materialise shared-odu edges over base-Odù lineage."""
function glyph_infer_shared_odu!(mem::GlyphMemory)
    ids = sort(collect(keys(mem.nodes)))
    added = 0
    for i in eachindex(ids), j in (i+1):lastindex(ids)
        mem.nodes[ids[i]].odu_base == mem.nodes[ids[j]].odu_base || continue
        edge = (ids[i], ids[j], "shared-odu")
        if !(edge in mem.edges)
            push!(mem.edges, edge)
            added += 1
        end
    end
    added
end

"""A2A merge: tag union, earliest ts wins, existing locators kept."""
function glyph_merge!(mem::GlyphMemory, other::GlyphMemory)
    nodes_added = 0
    for id in sort(collect(keys(other.nodes)))
        theirs = other.nodes[id]
        if haskey(mem.nodes, id)
            existing = mem.nodes[id]
            existing.tags = sort(unique(vcat(existing.tags, theirs.tags)))
            existing.ts = min(existing.ts, theirs.ts)
            existing.walrus_blob_id === nothing && (existing.walrus_blob_id = theirs.walrus_blob_id)
        else
            glyph_insert!(mem, deepcopy(theirs))
            nodes_added += 1
        end
    end
    edges_added = length(setdiff(other.edges, mem.edges))
    union!(mem.edges, other.edges)
    (nodes_added, edges_added)
end

# ---------------------------------------------------------------------------
# SOMA / REM integration + birth registration
# ---------------------------------------------------------------------------

"""
Register an agent's genesis fact as the sovereign root of its glyph
memory. The birth glyph is tagged `birth` and `agent:<id>` and becomes the
anchor every later REM cluster links back to. Returns the birth node.
"""
function register_birth!(mem::GlyphMemory, agent_id::AbstractString,
                         genesis::AbstractString, ts::Real)
    node = glyph_node(genesis, ts)
    node.tags = ["agent:$agent_id", "birth"]
    node.walrus_blob_id = "osun://$agent_id/birth"
    glyph_insert!(mem, node)
end

"""
Store a completed `think`-turn snapshot in both memory layers: the Augury
`MemoryDAG` (numeric metrics for prediction) and the glyph graph (sovereign
address for anchoring). The glyph node is chained to `parent_id`'s glyph with
a `follows` edge so REM consolidation walks the same lineage the mesh anchors.
Returns (MemoryNode, GlyphMemoryNode).
"""
function soma_store!(dag::MemoryDAG, mem::GlyphMemory, agent_id::AbstractString,
                     text::AbstractString, values::Vector{Float64}, ts::Real;
                     parent_text::Union{AbstractString,Nothing}=nothing)
    glyph = glyph_node(text, ts)
    glyph.tags = ["agent:$agent_id", "soma"]
    glyph.walrus_blob_id = "osun://$agent_id/$(glyph.canonical_id)"
    parent_glyph_id = parent_text === nothing ? nothing :
        glyph_canonical_id(glyph_content_hash(parent_text))
    glyph_insert!(mem, glyph)
    if parent_glyph_id !== nothing && haskey(mem.nodes, parent_glyph_id) &&
       parent_glyph_id != glyph.canonical_id
        glyph_link!(mem, parent_glyph_id, glyph.canonical_id, "follows")
    end
    dag_parent = parent_glyph_id === nothing ? nothing :
        (haskey(dag.nodes, parent_glyph_id) ? parent_glyph_id : nothing)
    dag_node = add_snapshot!(dag, glyph.canonical_id, values, Float64(ts), dag_parent)
    (dag_node, glyph)
end

"""
REM consolidation cluster: link every glyph tagged `soma` for an agent back
to its birth root with a `rem-cluster` edge, then infer shared-Odù lineage
across the cluster. Returns the number of new edges added. This is the
sleep-cycle step that turns a session's loose snapshots into one navigable
constellation anchored on the agent's birth glyph.
"""
function rem_consolidate!(mem::GlyphMemory, agent_id::AbstractString)
    birth = nothing
    for node in values(mem.nodes)
        if "birth" in node.tags && "agent:$agent_id" in node.tags
            birth = node
            break
        end
    end
    birth === nothing && error("agent $agent_id has no birth registration")
    added = 0
    for id in sort(collect(keys(mem.nodes)))
        node = mem.nodes[id]
        ("soma" in node.tags && "agent:$agent_id" in node.tags && id != birth.canonical_id) || continue
        edge = (birth.canonical_id, id, "rem-cluster")
        if !(edge in mem.edges)
            push!(mem.edges, edge)
            added += 1
        end
    end
    added + glyph_infer_shared_odu!(mem)
end

# ---------------------------------------------------------------------------
# Anchoring (keyless Merkle root over receipt pairs)
# ---------------------------------------------------------------------------

"""GIX1 keyless structural audit."""
gix1_audit(blob::Vector{UInt8}) =
    length(blob) >= 34 && blob[1:4] == b"GIX1" && blob[5] == 0x01 && blob[6] <= 0x01

"""
Sui-anchorable Merkle root over receipt pairs `(canonical_id, blob_sha256)` —
leaf = SHA-256(id bytes || blob hash), leaves sorted by id, odd leaf promoted.
"""
function glyph_merkle_root(entries::Vector{Tuple{String,Vector{UInt8}}})
    isempty(entries) && return GIX1_EMPTY_ROOT
    sorted = sort(entries; by = first)
    level = [sha256(vcat(hex2bytes(id), blob_hash)) for (id, blob_hash) in sorted]
    while length(level) > 1
        next = Vector{Vector{UInt8}}()
        for i in 1:2:length(level)-1
            push!(next, sha256(vcat(level[i], level[i+1])))
        end
        isodd(length(level)) && push!(next, level[end])
        level = next
    end
    bytes2hex(level[1])
end
