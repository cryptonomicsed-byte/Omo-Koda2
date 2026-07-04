"""
Soma bridge — HTTP handlers for the /soma/* route group.

Exposes memory reconstruction and storage endpoints consumed by the Rust
OsunClient (HttpOsunClient in omokoda-core/src/bus/clients.rs).
"""

using JSON3

"""
POST /soma/store
Body: { agent_id, text, importance, timestamp }
Stores a memory snapshot from a completed `think` turn into the DAG.
"""
function handle_soma_store(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    agent_id  = string(get(body, :agent_id, "unknown"))
    text      = string(get(body, :text, ""))
    importance = Float64(get(body, :importance, 0.5))
    ts        = Float64(get(body, :timestamp, time()))

    values = Float64[
        importance,
        Float64(length(text)) / 1000.0,
        Float64(hash(text) % 10000) / 10000.0,
    ]

    id = "soma_$(agent_id)_$(round(Int, ts))"
    add_snapshot!(GLOBAL_DAG[], id, values, ts, nothing)

    return HTTP.Response(200, JSON3.write(Dict("stored" => true, "id" => id)))
end

"""
POST /soma/reconstruct
Body: { agent_id, prompt }
Returns a SomaContext for the given agent: predicted needs and behavioral patterns
drawn from the agent's memory DAG.
"""
function handle_soma_reconstruct(req::HTTP.Request)
    body = JSON3.read(String(req.body))
    agent_id = string(get(body, :agent_id, "unknown"))

    predicted_needs  = String[]
    patterns         = String[]
    active_themes    = String[]
    identity_anchors = String[]

    nodes = GLOBAL_DAG[].nodes
    agent_nodes = filter(((id, _),) -> startswith(id, "soma_$(agent_id)_"), collect(nodes))

    # Time-series prediction from the agent's past importance values
    if length(agent_nodes) >= 3
        series = Float64[n.values[1] for (_, n) in agent_nodes]
        try
            pred = predict(series, 3)
            if !isempty(pred.predictions)
                level = round(mean(pred.predictions), digits=2)
                push!(predicted_needs, "forecast activity weight: $(level)")
            end
        catch
        end
    end

    # Semantic similarity to find recurring patterns
    if !isempty(agent_nodes)
        try
            query_vec = Float64[0.5, 0.1, 0.5]
            results = similar_sequences(GLOBAL_DAG[], query_vec, 3)
            for (node_id, sim) in results
                if sim > 0.7
                    push!(patterns, "recurring pattern near $(node_id) (similarity $(round(sim, digits=2)))")
                end
            end
        catch
        end
    end

    return HTTP.Response(200, JSON3.write(Dict(
        "predicted_needs"  => predicted_needs,
        "patterns"         => patterns,
        "triggers"         => String[],
        "active_themes"    => active_themes,
        "identity_anchors" => identity_anchors,
    )))
end
