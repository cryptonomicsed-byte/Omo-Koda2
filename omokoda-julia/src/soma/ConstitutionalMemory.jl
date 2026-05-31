module SomaConstitutionalMemory

using ..SomaMemCell: MemCell

export ConstitutionalStory, ConstitutionalPrior, ConstitutionalMemory
export update_prior!, constitutional_recall, prior_to_context

"""A lived constitutional alignment event that shapes future priors."""
struct ConstitutionalStory
    principle::Symbol          # :mentalism | :correspondence | :vibration | :polarity | :rhythm | :cause_and_effect | :gender
    alignment_score::Float32   # 0.0–1.0
    outcome::Symbol            # :aligned | :warned | :blocked
    summary::String
    timestamp::Float64         # Unix seconds
end

"""
Per-principle Bayesian prior built from lived constitutional stories.
The agent learns from its own alignment history — each story updates the mean.
"""
mutable struct ConstitutionalPrior
    principle::Symbol
    mean_score::Float32        # Running Bayesian mean (starts at 0.75 — cautiously optimistic)
    confidence::Float32        # 0–1; grows with story count, caps at 0.95
    story_count::Int
    last_violation::Float64    # Unix timestamp of most recent :blocked event; 0 = never
end

ConstitutionalPrior(principle::Symbol) =
    ConstitutionalPrior(principle, 0.75f0, 0.10f0, 0, 0.0)

const HERMETIC_PRINCIPLES = [
    :mentalism, :correspondence, :vibration,
    :polarity, :rhythm, :cause_and_effect, :gender,
]

"""All 7 principle priors for one sovereign agent."""
mutable struct ConstitutionalMemory
    priors::Dict{Symbol, ConstitutionalPrior}
    stories::Vector{ConstitutionalStory}
    max_stories::Int
end

function ConstitutionalMemory(; max_stories::Int = 200)
    priors = Dict(p => ConstitutionalPrior(p) for p in HERMETIC_PRINCIPLES)
    ConstitutionalMemory(priors, ConstitutionalStory[], max_stories)
end

"""
Update the prior for a principle with a new lived story.
Uses a Bayesian running mean: new_mean = (n * mean + score) / (n + 1).
Confidence grows with story count, capped at 0.95.
"""
function update_prior!(mem::ConstitutionalMemory, story::ConstitutionalStory)
    # Ring-buffer: drop oldest story when at capacity
    push!(mem.stories, story)
    if length(mem.stories) > mem.max_stories
        deleteat!(mem.stories, 1)
    end

    prior = get!(mem.priors, story.principle, ConstitutionalPrior(story.principle))
    n = prior.story_count
    prior.mean_score = Float32((n * prior.mean_score + story.alignment_score) / (n + 1))
    prior.story_count = n + 1
    # Confidence converges toward 0.95 asymptotically
    prior.confidence = min(0.95f0, Float32(prior.story_count) / Float32(prior.story_count + 10))

    if story.outcome == :blocked
        prior.last_violation = story.timestamp
    end

    return prior
end

"""
Recall memories weighted by constitutional alignment.

Memories from high-alignment contexts score higher; the prior's confidence
modulates how much constitutional history influences retrieval.
"""
function constitutional_recall(
    mem::ConstitutionalMemory,
    cells::Vector{MemCell},
    principle::Symbol;
    top_n::Int = 5,
)::Vector{MemCell}
    prior = get(mem.priors, principle, ConstitutionalPrior(principle))

    scored = map(cells) do cell
        alignment_boost = prior.mean_score * prior.confidence * 0.30f0
        age_hours = Float32((time() - cell.created_at) / 3600.0)
        recency = 0.999f0 ^ age_hours
        score = cell.importance * 0.50f0 + recency * 0.20f0 + alignment_boost
        (score, cell)
    end

    sorted = sort(scored; by = x -> x[1], rev = true)
    [x[2] for x in sorted[1:min(top_n, length(sorted))]]
end

"""Render the constitutional memory as a context block for injection into `think`."""
function prior_to_context(mem::ConstitutionalMemory)::String
    lines = ["[Constitutional Priors]"]
    for p in HERMETIC_PRINCIPLES
        prior = mem.priors[p]
        flag = prior.last_violation > 0.0 ? " ⚠" : ""
        push!(lines,
            "  $(p): mean=$(round(prior.mean_score; digits=2))" *
            " conf=$(round(prior.confidence; digits=2))" *
            " n=$(prior.story_count)$(flag)")
    end
    join(lines, "\n")
end

end  # module SomaConstitutionalMemory
