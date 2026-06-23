"""
SOMA Resonance — emotional resonance scoring and vector similarity utilities.
Used by the retrieval pipeline to weight memories by emotional alignment with
the agent's current state.

Server-side only. Called from SomaMemCell scoring and the Elixir SOMA service.
"""
module SomaResonance

using LinearAlgebra
using Statistics

export cosine_similarity, keyword_score, resonance_boost, emotional_distance

# ---------------------------------------------------------------------------
# Vector similarity
# ---------------------------------------------------------------------------

"""
Compute cosine similarity between two Float32 embedding vectors.

Returns 0.0 when either vector has zero norm, avoiding NaN propagation.
"""
function cosine_similarity(a::Vector{Float32}, b::Vector{Float32})::Float32
    norm_a = norm(a)
    norm_b = norm(b)
    if norm_a == 0.0f0 || norm_b == 0.0f0
        return 0.0f0
    end
    Float32(dot(a, b) / (norm_a * norm_b))
end

# ---------------------------------------------------------------------------
# Keyword overlap
# ---------------------------------------------------------------------------

"""
Score textual overlap between `text` and `query` by counting shared words
longer than 3 characters.

Normalised to [0, 1] via `tanh(matches / 3.0)` so that 3 shared words
saturates near 1.0 without a hard cap.
"""
function keyword_score(text::String, query::String)::Float32
    text_words  = Set(w for w in split(lowercase(text))  if length(w) > 3)
    query_words = Set(w for w in split(lowercase(query)) if length(w) > 3)
    matches     = length(intersect(text_words, query_words))
    Float32(tanh(matches / 3.0))
end

# ---------------------------------------------------------------------------
# Emotional resonance
# ---------------------------------------------------------------------------

"""
Compute an additive resonance boost between a stored cell's emotion and the
agent's current emotional state.

Accepts any struct with `tension::Float32` and `connection::Float32` fields
(i.e. an `EmotionTrace`). Using duck typing keeps this module self-contained.

Rules (additive, capped at 0.25 total):
- +0.15 if both tension values exceed 0.5
- +0.10 if both connection values exceed 0.6

The combined maximum is 0.25.
"""
function resonance_boost(cell_emotion, current_emotion)::Float32
    boost = Float32(0.0)
    if cell_emotion.tension > Float32(0.5) && current_emotion.tension > Float32(0.5)
        boost += Float32(0.15)
    end
    if cell_emotion.connection > Float32(0.6) && current_emotion.connection > Float32(0.6)
        boost += Float32(0.10)
    end
    min(boost, Float32(0.25))
end

# ---------------------------------------------------------------------------
# Emotional distance
# ---------------------------------------------------------------------------

"""
Euclidean distance between two EmotionTraces across four dimensions:
    score, tension, connection, energy (derived as 1 - tension)

Accepts any struct with `score`, `tension`, and `connection` Float32 fields.
A distance of 0.0 means the two emotional states are identical.
"""
function emotional_distance(a, b)::Float32
    energy_a = Float32(1.0) - a.tension
    energy_b = Float32(1.0) - b.tension

    sqrt(
        (a.score      - b.score)^2      +
        (a.tension    - b.tension)^2    +
        (a.connection - b.connection)^2 +
        (energy_a     - energy_b)^2
    )
end

end  # module SomaResonance
