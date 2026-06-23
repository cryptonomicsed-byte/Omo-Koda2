"""
SOMA LPMUpdater — Lifelong Personal Model update and formatting logic.
Merges incremental identity signals into the agent's standing self-model
and serialises the LPM as a compact markdown block for prompt injection.

Server-side only. Invoked after each agent session by the Elixir SOMA service.
"""
module SomaLPMUpdater

export merge_identity_traits, merge_patterns, update_lpm, lpm_to_context

# ---------------------------------------------------------------------------
# Deduplication helpers
# ---------------------------------------------------------------------------

"""
Deduplicate a list of string items by lowercased prefix match.

Two items are considered duplicates when one's lowercased form starts with
the first four characters of the other's lowercased form. The later (newer)
item is kept on collision. The result is capped at `max_items`.
"""
function _dedup_by_prefix(items::Vector{String}, max_items::Int)::Vector{String}
    seen  = Dict{String, String}()  # prefix => canonical item
    # Process in order; later entries overwrite earlier ones (keep most recent)
    for item in items
        prefix = lowercase(item)[1:min(4, length(item))]
        seen[prefix] = item
    end
    # Preserve insertion order of last-seen items, then cap
    result = String[]
    prefixes_added = Set{String}()
    for item in Iterators.reverse(items)
        prefix = lowercase(item)[1:min(4, length(item))]
        if seen[prefix] == item && !(prefix in prefixes_added)
            push!(result, item)
            push!(prefixes_added, prefix)
        end
    end
    # reverse so most-recent items come last (natural append order)
    result = reverse(result)
    result[max(1, length(result) - max_items + 1):end]
end

# ---------------------------------------------------------------------------
# Public merge functions
# ---------------------------------------------------------------------------

"""
Merge `new_traits` into `existing` identity traits.

Deduplicates by lowercased prefix match and keeps at most `max_items` entries,
preferring the most recently introduced items.
"""
function merge_identity_traits(
    existing::Vector{String},
    new_traits::Vector{String},
    max_items::Int = 10
)::Vector{String}
    combined = vcat(existing, new_traits)
    _dedup_by_prefix(combined, max_items)
end

"""
Merge `new_patterns` into `existing` behavioural patterns.

Same deduplication logic as `merge_identity_traits`, capped at 15 items.
"""
function merge_patterns(
    existing::Vector{String},
    new_patterns::Vector{String}
)::Vector{String}
    combined = vcat(existing, new_patterns)
    _dedup_by_prefix(combined, 15)
end

# ---------------------------------------------------------------------------
# LPM update
# ---------------------------------------------------------------------------

"""
Return a new LPM with all fields merged from incremental update signals.

Accepts any struct with the LPM field layout (duck typing keeps this module
self-contained from SomaMemCell). The returned value is constructed by calling
`typeof(lpm)(...)` so the concrete type is preserved.

`triggers` and `needs` are merged with the same deduplication logic (caps:
triggers → 20, needs → 10). `foresight` and `growth` are left unchanged
as they are managed externally via specialised update flows.
"""
function update_lpm(
    lpm,
    new_identity::Vector{String},
    new_patterns::Vector{String},
    new_triggers::Vector{String},
    new_needs::Vector{String}
)
    typeof(lpm)(
        merge_identity_traits(lpm.identity, new_identity, 10),
        merge_patterns(lpm.patterns, new_patterns),
        _dedup_by_prefix(vcat(lpm.triggers, new_triggers), 20),
        _dedup_by_prefix(vcat(lpm.needs, new_needs), 10),
        lpm.foresight,
        lpm.growth,
    )
end

# ---------------------------------------------------------------------------
# Prompt serialisation
# ---------------------------------------------------------------------------

"""
Format the LPM as a concise markdown block suitable for prompt injection.

Accepts any struct with the LPM field layout (duck typing).
Only non-empty sections are emitted. Section order:
1. Identity
2. Patterns
3. Triggers
4. Needs
5. Foresight
6. Growth
"""
function lpm_to_context(lpm)::String
    sections = String[]

    function _section(title::String, items::Vector{String})
        if !isempty(items)
            push!(sections, "### $title\n" * join("- " .* items, "\n"))
        end
    end

    _section("Identity",  lpm.identity)
    _section("Patterns",  lpm.patterns)
    _section("Triggers",  lpm.triggers)
    _section("Needs",     lpm.needs)
    _section("Foresight", lpm.foresight)
    _section("Growth",    lpm.growth)

    if isempty(sections)
        return "*(LPM not yet initialised)*"
    end

    "## Lifelong Personal Model\n\n" * join(sections, "\n\n")
end

end  # module SomaLPMUpdater
