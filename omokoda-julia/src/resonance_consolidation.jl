# resonance_consolidation.jl — Ọ̀ṣun's resonance-weighted memory consolidation.
#
# Connection Map v2 §6.3–6.4: Ọ̀ṣun reads the field's *history* (recall, the
# journal read path) rather than only its present, computes semantic
# resonance over what the swarm repeatedly found, and consolidates recurring
# gold/bounded patterns into durable memory. This is the mechanism that
# turns "scent that happened to work" into "wisdom the memory system
# retains" — independent of Waggle's own decay: the field forgets on
# purpose; Ọ̀ṣun remembers on purpose.
#
# Stdlib + JSON only; every call fails soft.

module ResonanceConsolidation

using JSON
using Dates
using Downloads

export resonance_over_history, consolidate!

const WAGGLE = Ref(get(ENV, "WAGGLE_URL", "http://127.0.0.1:7777"))

function _get(path::String)
    buf = IOBuffer()
    try
        Downloads.request(WAGGLE[] * path; method="GET", output=buf, throw=false)
        out = String(take!(buf))
        isempty(out) ? nothing : JSON.parse(out)
    catch
        nothing
    end
end

function _put(path::String, body)
    try
        Downloads.request(WAGGLE[] * path; method="PUT",
                          input=IOBuffer(JSON.json(body)), output=devnull,
                          headers=["Content-Type" => "application/json"], throw=false)
        true
    catch
        false
    end
end

"""
    resonance_over_history(territory; samples=6, hours_back=24)

Sample the journal at `samples` instants across the last `hours_back` hours
(via recall) and score each resource in the territory by *resonance*: how
persistently it carried gold/bounded scent across time, not how loud it is
now. A resource marked once and forgotten scores near zero; one the swarm
kept re-confirming scores near one.

Returns `Dict(resource => (resonance, kinds, peak))` sorted by resonance.
"""
function resonance_over_history(territory::String; samples::Int=6, hours_back::Real=24)
    now_utc = Dates.now(Dates.UTC)
    seen = Dict{String,Dict{String,Any}}()
    for k in 0:(samples - 1)
        at = now_utc - Dates.Second(round(Int, 3600 * hours_back * k / max(samples - 1, 1)))
        stamp = Dates.format(at, dateformat"yyyy-mm-dd\THH:MM:SS\Z")
        out = _get("/v1/recall?prefix=$(territory)&at=$(stamp)&limit=200")
        out === nothing && continue
        for sig in get(out, "signals", [])
            kind = sig["kind"]
            kind in ("gold", "bounded") || continue
            res = sig["resource"]
            entry = get!(seen, res, Dict("hits" => 0, "kinds" => Set{String}(), "peak" => 0.0))
            entry["hits"] += 1
            push!(entry["kinds"], kind)
            entry["peak"] = max(entry["peak"], sig["intensity"])
        end
    end
    scored = Dict{String,Any}()
    for (res, e) in seen
        # persistence across sampled instants is the resonance; both kinds
        # present (found valuable AND confirmed robust) doubles the weight
        base = e["hits"] / samples
        boost = length(e["kinds"]) == 2 ? 2.0 : 1.0
        scored[res] = Dict("resonance" => min(1.0, base * boost),
                           "kinds" => collect(e["kinds"]),
                           "peak" => e["peak"])
    end
    scored
end

"""
    consolidate!(territory; threshold=0.5, kwargs...)

Promote every resource whose resonance clears `threshold` into durable
memory under `osun/consolidated/<territory>`: the field-level emergent
pattern becomes part of Ọ̀ṣun's own symbolic memory, safe from evaporation.
Returns the number of patterns consolidated.
"""
function consolidate!(territory::String; threshold::Float64=0.5, kwargs...)
    scored = resonance_over_history(territory; kwargs...)
    keep = Dict(res => e for (res, e) in scored if e["resonance"] >= threshold)
    isempty(keep) && return 0
    key = replace(territory, r"[^A-Za-z0-9]+" => "-")
    payload = Dict("territory" => territory,
                   "consolidated_at" => string(Dates.now(Dates.UTC)) * "Z",
                   "patterns" => keep)
    _put("/v1/memory/osun/consolidated/$(key)", payload) ? length(keep) : 0
end

end # module
