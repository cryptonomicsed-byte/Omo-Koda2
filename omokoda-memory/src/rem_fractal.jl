"""
REM fractal analysis — the weekly dream-state compression planner.

Mirrors the per-agent REM cycle in `omokoda-core/src/dream.rs` at hive scale:
Elixir (swarm coordinator) streams node summaries from many agents to this
module, which measures the box-counting fractal dimension of the activity
timeline (Mandelbrot's burst-noise insight: information clusters in
self-similar bursts separated by noise) and returns a compression *plan* —
which path-clusters of noise entries to fold into macro nodes and which
residual entries to prune. The caller applies the plan and writes back;
this module never mutates state.

Pure functions, no I/O. Served via POST /dream/rem in server.jl.
"""

"""
    rem_fractal_dimension(timestamps::Vector{<:Real}) -> Float64

Box-counting fractal dimension of an activity timeline, in [0, 1].
The span is divided into 2, 4, … 64 boxes; the least-squares slope of
ln N(ε) against ln(1/ε) is the dimension. Steady activity → ~1.0,
bursty activity → lower. Fewer than two distinct timestamps → 1.0.
"""
function rem_fractal_dimension(timestamps::Vector{<:Real})::Float64
    isempty(timestamps) && return 1.0
    lo, hi = extrema(timestamps)
    hi == lo && return 1.0
    span = Float64(hi - lo)

    xs = Float64[]
    ys = Float64[]
    for k in 1:6
        boxes = 2^k
        occupied = falses(boxes)
        for t in timestamps
            idx = min(Int(floor((Float64(t) - lo) / span * boxes)) + 1, boxes)
            occupied[idx] = true
        end
        push!(xs, log(boxes))
        push!(ys, log(count(occupied)))
    end

    m = length(xs)
    sx, sy = sum(xs), sum(ys)
    sxx = sum(x -> x^2, xs)
    sxy = sum(xs .* ys)
    denom = m * sxx - sx^2
    abs(denom) < eps() && return 1.0
    clamp((m * sxy - sx * sy) / denom, 0.0, 1.0)
end

"""
    rem_plan(nodes; noise_importance=0.35, min_fold_cluster=3) -> Dict

Compute a REM compression plan over node summaries. Each node is a Dict-like
with keys `id`, `path`, `importance`, `created_at`.

Returns:
  - `fractal_dimension` — of the created_at timeline
  - `folds`             — [{path, ids, preview_count}] clusters to collapse
  - `prune_ids`         — unclustered noise below noise_importance/2
  - `nodes_analyzed`
"""
function rem_plan(nodes::AbstractVector;
                  noise_importance::Float64=0.35,
                  min_fold_cluster::Int=3)::Dict{String,Any}
    timestamps = Float64[Float64(get(n, "created_at", 0)) for n in nodes]
    fd = rem_fractal_dimension(timestamps)

    # Group noise nodes by path.
    clusters = Dict{String,Vector{String}}()
    for n in nodes
        importance = Float64(get(n, "importance", 0.5))
        if importance <= noise_importance
            path = string(get(n, "path", ""))
            push!(get!(clusters, path, String[]), string(get(n, "id", "")))
        end
    end

    folds = Dict{String,Any}[]
    folded_ids = Set{String}()
    for path in sort(collect(keys(clusters)))
        ids = sort(clusters[path])
        length(ids) < min_fold_cluster && continue
        push!(folds, Dict{String,Any}(
            "path" => path,
            "ids" => ids,
            "preview_count" => min(length(ids), 3),
        ))
        union!(folded_ids, ids)
    end

    # Unclustered noise well below the line is structural fluff.
    prune_line = noise_importance / 2.0
    prune_ids = String[
        string(get(n, "id", "")) for n in nodes
        if Float64(get(n, "importance", 0.5)) < prune_line &&
           !(string(get(n, "id", "")) in folded_ids)
    ]

    Dict{String,Any}(
        "fractal_dimension" => fd,
        "folds" => folds,
        "prune_ids" => sort(prune_ids),
        "nodes_analyzed" => length(nodes),
    )
end
