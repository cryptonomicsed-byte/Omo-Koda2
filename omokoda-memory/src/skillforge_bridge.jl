"""
SkillForge × Julia bridge — the Memory stage's dedup/similarity check.

Analysis (Python legwork + Clojure classification) hands over a proposed
skill name+description; this compares it against the agent's existing skill
manifest (passed in the request body — stateless, no shared state with the
Augury DAG other stages use) and reports the closest match plus a
canonical-name suggestion. Word-overlap (Jaccard) similarity: cheap,
dependency-free, and good enough for short manifest descriptions — this is
deliberately not the DAG-vector similarity `/vantage/similar` does over
memory snapshots, which is a different domain (agent memories, not skill
manifests).
"""

using JSON3

function _tokenize(s::AbstractString)
    Set(split(lowercase(s), r"[^a-z0-9]+"; keepempty=false))
end

function _jaccard(a::Set{<:AbstractString}, b::Set{<:AbstractString})
    isempty(a) && isempty(b) && return 0.0
    inter = length(intersect(a, b))
    uni = length(union(a, b))
    uni == 0 ? 0.0 : inter / uni
end

"""
POST /skillforge/similar
Body: { name, description, existing: [{name, description}, ...] }
Returns the closest existing skill (if any) by Jaccard similarity over
name+description tokens, and a canonical name suggestion if a near-duplicate
is found (score >= 0.5 is treated as "probably the same project").
"""
function handle_skillforge_similar(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    name = String(get(body, :name, ""))
    description = String(get(body, :description, ""))
    existing = get(body, :existing, [])

    query_tokens = union(_tokenize(name), _tokenize(description))

    best_score = 0.0
    best_name = nothing
    for entry in existing
        e_name = String(get(entry, :name, ""))
        e_desc = String(get(entry, :description, ""))
        e_tokens = union(_tokenize(e_name), _tokenize(e_desc))
        score = _jaccard(query_tokens, e_tokens)
        if score > best_score
            best_score = score
            best_name = e_name
        end
    end

    is_duplicate = best_score >= 0.5
    return HTTP.Response(200, JSON3.write(Dict(
        "closest_match" => best_name,
        "similarity" => best_score,
        "likely_duplicate" => is_duplicate,
        "suggested_name" => is_duplicate ? best_name : name,
    )))
end
