"""
SOMA MemCell — Self-Organizing Memory Architecture core types and retrieval scoring.
Defines the memory cell primitives, scene groupings, and Lifelong Personal Model (LPM)
used throughout the Omo-Koda agent runtime.

Server-side only. Called via REST API from Elixir or FFI from Rust.
"""
module SomaMemCell

using Dates
using JSON3

export EmotionTrace, MemCell, MemScene, LPM
export score_retrieval, score_all, reconstruct

# ---------------------------------------------------------------------------
# Structs
# ---------------------------------------------------------------------------

"""
An emotional fingerprint captured at the moment a memory was formed.

Fields:
- `score`      — overall emotional intensity, [0, 1]
- `valence`    — polarity: :positive, :negative, or :neutral
- `tension`    — internal conflict or arousal level, [0, 1]
- `connection` — sense of interpersonal connectedness, [0, 1]
"""
struct EmotionTrace
    score::Float32
    valence::Symbol          # :positive | :negative | :neutral
    tension::Float32
    connection::Float32
end

"""
A single memory cell — the atomic unit of agent experience.

Fields:
- `id`          — unique identifier (UUID string)
- `timestamp`   — wall-clock time when the memory was recorded
- `text`        — raw text content of the memory
- `emotion`     — emotional trace at time of encoding
- `importance`  — salience weight assigned at encoding, [0, 1]
- `foresight`   — forward-looking observations derived from this memory
- `tags`        — free-form categorical labels
- `theme`       — optional scene/topic grouping label
- `activations` — number of times this cell has been retrieved
- `vector`      — optional dense embedding for similarity search
"""
struct MemCell
    id::String
    timestamp::DateTime
    text::String
    emotion::EmotionTrace
    importance::Float32
    foresight::Vector{String}
    tags::Vector{String}
    theme::Union{String, Nothing}
    activations::UInt32
    vector::Union{Vector{Float32}, Nothing}
end

"""
A named scene grouping several related MemCells.

Fields:
- `theme`       — human-readable scene label
- `summary`     — short description of what the scene covers
- `cell_ids`    — IDs of member MemCells
- `strength`    — accumulated activation count across member cells
- `last_active` — most recent time any member cell was retrieved
"""
struct MemScene
    theme::String
    summary::String
    cell_ids::Vector{String}
    strength::UInt32
    last_active::DateTime
end

"""
Lifelong Personal Model — a living self-portrait of the agent's identity.

Fields:
- `identity`  — core identity statements (who the agent is)
- `patterns`  — recurring behavioural / cognitive patterns
- `triggers`  — known emotional or situational triggers
- `needs`     — fundamental needs and motivations
- `foresight` — standing forward-looking commitments or intentions
- `growth`    — tracked growth edges and developmental arcs
"""
struct LPM
    identity::Vector{String}
    patterns::Vector{String}
    triggers::Vector{String}
    needs::Vector{String}
    foresight::Vector{String}
    growth::Vector{String}
end

# ---------------------------------------------------------------------------
# Retrieval scoring
# ---------------------------------------------------------------------------

"""
Score a single MemCell for retrieval relevance given the current moment and
the agent's present emotional state.

Formula:
    score = (recency × 0.25) + (importance × 0.35) + (emotion_score × 0.25)
            + emotion_boost + activation_boost

Where:
- recency          = 0.995^age_hours  (exponential decay)
- emotion_boost    = +0.15 if both tensions > 0.5, +0.1 if both connections > 0.6
- activation_boost = min(0.2, activations × 0.02)

Returns a Float32. Not clamped — callers may clamp to [0, 1] if needed.
"""
function score_retrieval(cell::MemCell, now::DateTime, emotion::EmotionTrace)::Float32
    age_hours = Float32(max(0.0, Dates.value(now - cell.timestamp) / 3_600_000.0))
    recency   = Float32(0.995)^age_hours

    emotion_boost = Float32(0.0)
    if cell.emotion.tension > Float32(0.5) && emotion.tension > Float32(0.5)
        emotion_boost += Float32(0.15)
    end
    if cell.emotion.connection > Float32(0.6) && emotion.connection > Float32(0.6)
        emotion_boost += Float32(0.1)
    end

    activation_boost = min(Float32(0.2), Float32(cell.activations) * Float32(0.02))

    (recency * Float32(0.25)) +
    (cell.importance * Float32(0.35)) +
    (cell.emotion.score * Float32(0.25)) +
    emotion_boost +
    activation_boost
end

"""
Score a collection of MemCells in batch.

Returns a `Vector{Float32}` of the same length as `cells`, preserving order.
"""
function score_all(
    cells::Vector{MemCell},
    now::DateTime,
    emotion::EmotionTrace
)::Vector{Float32}
    [score_retrieval(c, now, emotion) for c in cells]
end

# ---------------------------------------------------------------------------
# Context reconstruction
# ---------------------------------------------------------------------------

"""
Assemble a markdown context string from memory for prompt injection.

Assembly order:
1. Foresight lines from the LPM (standing forward-looking commitments)
2. LPM patterns that share keywords with `query`
3. LPM triggers, only when current emotional tension > 0.4
4. Active MemScenes ordered by descending strength
5. Core identity statements from the LPM

Returns a non-empty markdown string. Always includes the identity section even
when all other sources are empty.
"""
function reconstruct(
    cells::Vector{MemCell},
    scenes::Vector{MemScene},
    lpm::LPM,
    query::String,
    emotion::EmotionTrace
)::String
    parts = String[]

    # 1. Foresight
    if !isempty(lpm.foresight)
        push!(parts, "## Foresight\n" * join("- " .* lpm.foresight, "\n"))
    end

    # 2. Relevant patterns — keyword match against query
    query_words = Set(w for w in split(lowercase(query)) if length(w) > 3)
    relevant_patterns = filter(lpm.patterns) do p
        pattern_words = Set(split(lowercase(p)))
        !isempty(intersect(query_words, pattern_words))
    end
    if !isempty(relevant_patterns)
        push!(parts, "## Relevant Patterns\n" * join("- " .* relevant_patterns, "\n"))
    end

    # 3. Triggers — only surface when tension is elevated
    if emotion.tension > Float32(0.4) && !isempty(lpm.triggers)
        push!(parts, "## Active Triggers\n" * join("- " .* lpm.triggers, "\n"))
    end

    # 4. Active scenes ordered by strength descending
    active_scenes = sort(scenes, by=s -> s.strength, rev=true)
    if !isempty(active_scenes)
        scene_lines = ["### $(s.theme)\n$(s.summary)" for s in active_scenes]
        push!(parts, "## Active Scenes\n" * join(scene_lines, "\n\n"))
    end

    # 5. Core identity — always included
    identity_section = if isempty(lpm.identity)
        "## Identity\n*(not yet established)*"
    else
        "## Identity\n" * join("- " .* lpm.identity, "\n")
    end
    push!(parts, identity_section)

    join(parts, "\n\n")
end

end  # module SomaMemCell
