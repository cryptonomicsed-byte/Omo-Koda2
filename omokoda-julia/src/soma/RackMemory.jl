"""
SOMA RackMemory — Resonance-Addressable Content Kernel (RACK).

A three-layer memory store (identity / long_term / short_term) built on
top of the existing MemCell and Resonance modules.  Content is addressed by
resonance score — a product of importance, temporal recency, and retrieval
frequency — rather than by key or timestamp alone.

omo-mem pattern mapping:
    SOUL.md         → identity_layer
    MEMORY.md       → long_term_layer
    memory/YYYY-MM-DD.md → short_term_layer
"""
module SomaRackMemory

using Dates, LinearAlgebra

using ..SomaMemCell: MemCell, EmotionTrace
using ..SomaResonance: cosine_similarity, keyword_score, resonance_boost

export Rack, RackLayer, rack_write!, rack_recall, rack_prune!, rack_context, rack_stats
export identity_layer, long_term_layer, short_term_layer

@enum RackLayer identity_layer long_term_layer short_term_layer

"""
The RACK store: three parallel arrays for cells, layers, and resonance scores.
Kept as plain arrays (not a Dict) so retrieval is a single vectorised sort.
"""
mutable struct Rack
    cells::Vector{MemCell}
    layers::Vector{RackLayer}
    resonance_scores::Vector{Float32}
end

Rack() = Rack(MemCell[], RackLayer[], Float32[])

# ---------------------------------------------------------------------------
# Resonance computation
# ---------------------------------------------------------------------------

"""
Standing resonance score for a MemCell.

Formula: importance × recency × log1p(activations + 1)
- recency  = exp(−age_days / 30)   — 30-day half-life
- This cached score is recomputed only on write, not at query time.
  Query-time scoring layered on top via rack_recall.
"""
function _resonance(cell::MemCell)::Float32
    age_days = Float32(max(0.0, Dates.value(now() - cell.timestamp) / 86_400_000.0))
    recency  = exp(-age_days / 30.0f0)
    Float32(cell.importance * recency * log1p(Float32(cell.activations) + 1.0f0))
end

# ---------------------------------------------------------------------------
# CRUD
# ---------------------------------------------------------------------------

"""
Write a MemCell into the RACK.

Identity-layer cells are exempt from pruning — they model the agent's core self.
Long-term cells require resonance > 0 to be kept past the next prune cycle.
"""
function rack_write!(rack::Rack, cell::MemCell, layer::RackLayer = long_term_layer)
    push!(rack.cells, cell)
    push!(rack.layers, layer)
    push!(rack.resonance_scores, _resonance(cell))
    nothing
end

"""
Retrieve the top-k cells most relevant to `query` under the agent's `emotion`.

Scoring:
    combined = standing_resonance × (0.5·cosine_sim + 0.3·keyword + 0.2·emotion_boost)

Falls back to keyword overlap when cell embedding is absent or dimension-mismatched.
`layer_filter` restricts recall to one layer (nothing = all layers).
"""
function rack_recall(
    rack::Rack,
    query::String,
    emotion::EmotionTrace;
    top_k::Int = 5,
    layer_filter::Union{RackLayer, Nothing} = nothing,
)::Vector{MemCell}
    isempty(rack.cells) && return MemCell[]

    query_vec = _simple_embed(query)
    scored = Tuple{Float32, Int}[]

    for (i, cell) in enumerate(rack.cells)
        layer_filter !== nothing && rack.layers[i] !== layer_filter && continue

        sim = if cell.vector !== nothing &&
                 !isempty(cell.vector) &&
                 length(cell.vector) == length(query_vec)
            cosine_similarity(query_vec, cell.vector)
        else
            keyword_score(cell.text, query)
        end

        boost    = resonance_boost(cell.emotion, emotion)
        kw       = keyword_score(cell.text, query)
        combined = rack.resonance_scores[i] * (0.5f0 * sim + 0.3f0 * kw + 0.2f0 * boost)
        push!(scored, (combined, i))
    end

    sort!(scored; by = x -> x[1], rev = true)
    [rack.cells[i] for (_, i) in scored[1:min(top_k, length(scored))]]
end

"""
Remove lowest-resonance non-identity cells until at most `max_entries` remain.

Returns the number of cells pruned.  Identity-layer cells are never pruned.
"""
function rack_prune!(rack::Rack, max_entries::Int = 1000)::Int
    excess = length(rack.cells) - max_entries
    excess <= 0 && return 0

    non_id = [(rack.resonance_scores[i], i)
              for i in eachindex(rack.cells)
              if rack.layers[i] !== identity_layer]
    isempty(non_id) && return 0

    sort!(non_id; by = x -> x[1])
    drop = Set{Int}(i for (_, i) in non_id[1:min(excess, length(non_id))])
    keep = [i for i in eachindex(rack.cells) if !(i in drop)]

    rack.cells            = rack.cells[keep]
    rack.layers           = rack.layers[keep]
    rack.resonance_scores = rack.resonance_scores[keep]

    length(drop)
end

"""
Render a markdown context block for prompt injection.

Mirrors omo-mem's three-layer injection order:
    Identity (SOUL.md) → Long-term (MEMORY.md) → Short-term (daily note)
"""
function rack_context(
    rack::Rack,
    query::String,
    emotion::EmotionTrace;
    top_n::Int = 7,
)::String
    parts = String[]

    id_cells = rack_recall(rack, query, emotion; top_k = 3,     layer_filter = identity_layer)
    lt_cells = rack_recall(rack, query, emotion; top_k = top_n, layer_filter = long_term_layer)
    st_cells = rack_recall(rack, query, emotion; top_k = 3,     layer_filter = short_term_layer)

    !isempty(id_cells) && push!(parts, "## Identity\n"  * join("- " .* [c.text for c in id_cells], "\n"))
    !isempty(lt_cells) && push!(parts, "## Memory\n"   * join("- " .* [c.text for c in lt_cells], "\n"))
    !isempty(st_cells) && push!(parts, "## Recent\n"   * join("- " .* [c.text for c in st_cells], "\n"))

    isempty(parts) ? "" : join(parts, "\n\n")
end

"""Return (total, identity, long_term, short_term) cell counts."""
function rack_stats(rack::Rack)
    id_n = count(l -> l === identity_layer,   rack.layers)
    lt_n = count(l -> l === long_term_layer,  rack.layers)
    st_n = count(l -> l === short_term_layer, rack.layers)
    (total = length(rack.cells), identity = id_n, long_term = lt_n, short_term = st_n)
end

# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

function _simple_embed(text::String)::Vector{Float32}
    words = split(lowercase(text))
    vec   = zeros(Float32, 64)
    isempty(words) && return vec

    for (j, w) in enumerate(words[1:min(64, length(words))])
        for ch in w
            vec[j] += Float32(UInt8(ch)) / 255.0f0
        end
    end

    n = norm(vec)
    n > 0.0f0 ? vec ./ n : vec
end

end  # module SomaRackMemory
