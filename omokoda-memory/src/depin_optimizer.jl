"""
DePIN Resource Optimizer — Ọ̀ṣun / Memory layer.

Load balancing and resource allocation for decentralized physical infrastructure
networks: hotspots, compute nodes, energy meters, sensors.

Strategies:
  • Greedy bin-packing  — minimize node count, respect capacity constraints
  • Weighted round-robin — distribute proportional to node weight/capacity
  • Least-connections    — always assign to node with fewest active tasks
  • Monte Carlo reliability — simulate node failures, estimate system uptime

All results include a `score` ∈ [0,1] measuring allocation quality.
"""

using Statistics: mean, std
using LinearAlgebra: norm

# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------

struct DePINNode
    id::String
    capacity::Float64       # maximum load it can handle
    weight::Float64         # relative routing weight (default 1.0)
    reliability::Float64    # P(node is up) ∈ [0,1]
    location::String        # geographic region (for locality routing)
end

struct Task
    id::String
    load::Float64           # resource demand
    region::String          # preferred region (empty = any)
end

struct Allocation
    node_id::String
    task_ids::Vector{String}
    utilisation::Float64    # assigned_load / capacity
end

# ---------------------------------------------------------------------------
# Greedy bin-packing
# ---------------------------------------------------------------------------

"""
    allocate_greedy(nodes, tasks; max_utilisation) → (allocations, unassigned, score)

Assign tasks to nodes using first-fit decreasing bin-packing.
Tasks are sorted by load descending; each goes to the first node
with enough remaining capacity.
"""
function allocate_greedy(nodes::Vector{DePINNode}, tasks::Vector{Task};
                          max_utilisation::Float64=0.9)
    sorted_tasks = sort(tasks, by=t->t.load, rev=true)
    remaining    = Dict(n.id => n.capacity * max_utilisation for n in nodes)
    assigned     = Dict(n.id => String[] for n in nodes)
    unassigned   = String[]

    for task in sorted_tasks
        # Prefer nodes in the same region
        candidates = sort(nodes,
            by = n -> (n.region == task.region ? 0 : 1, remaining[n.id]),
            rev = false
        )
        placed = false
        for node in candidates
            if remaining[node.id] >= task.load
                remaining[node.id] -= task.load
                push!(assigned[node.id], task.id)
                placed = true
                break
            end
        end
        placed || push!(unassigned, task.id)
    end

    allocs = [
        Allocation(n.id, assigned[n.id],
                   (n.capacity * max_utilisation - remaining[n.id]) / n.capacity)
        for n in nodes
        if !isempty(assigned[n.id])
    ]

    total_load  = sum(t.load for t in tasks)
    placed_load = sum(t.load for t in tasks if t.id ∉ unassigned)
    score       = total_load > 0 ? placed_load / total_load : 1.0

    (allocations=allocs, unassigned=unassigned, score=score,
     strategy="greedy_bin_packing")
end

# ---------------------------------------------------------------------------
# Weighted round-robin
# ---------------------------------------------------------------------------

"""
    allocate_round_robin(nodes, tasks) → (allocations, unassigned, score)

Distribute tasks proportionally to node weight.  Ignores hard capacity limits
(capacity soft-checked via score penalty only).
"""
function allocate_round_robin(nodes::Vector{DePINNode}, tasks::Vector{Task})
    isempty(nodes) && return (allocations=Allocation[], unassigned=[t.id for t in tasks],
                               score=0.0, strategy="weighted_round_robin")

    total_weight = sum(n.weight for n in nodes)
    assigned     = Dict(n.id => String[] for n in nodes)
    loads        = Dict(n.id => 0.0     for n in nodes)

    # Cumulative weight thresholds for proportional selection
    cumulative = Float64[]
    cum = 0.0
    for n in nodes
        cum += n.weight / total_weight
        push!(cumulative, cum)
    end

    for (i, task) in enumerate(tasks)
        # Pick node via deterministic hash (repeatable for same inputs)
        r    = (hash(task.id) % 10_000) / 10_000.0
        idx  = findfirst(c -> r <= c, cumulative)
        idx  === nothing && (idx = length(nodes))
        node = nodes[idx]
        push!(assigned[node.id], task.id)
        loads[node.id] += task.load
    end

    allocs = [
        Allocation(n.id, assigned[n.id], loads[n.id] / n.capacity)
        for n in nodes
        if !isempty(assigned[n.id])
    ]

    # Score: penalise over-utilised nodes
    over = count(a -> a.utilisation > 1.0, allocs)
    score = max(0.0, 1.0 - over / length(nodes))

    (allocations=allocs, unassigned=String[], score=score,
     strategy="weighted_round_robin")
end

# ---------------------------------------------------------------------------
# Least-connections
# ---------------------------------------------------------------------------

"""
    allocate_least_connections(nodes, tasks) → (allocations, unassigned, score)

Always route to the node currently handling the fewest tasks.
"""
function allocate_least_connections(nodes::Vector{DePINNode}, tasks::Vector{Task})
    assigned  = Dict(n.id => String[] for n in nodes)
    loads     = Dict(n.id => 0.0     for n in nodes)
    unassigned = String[]

    for task in tasks
        # Prefer least-loaded node that still has capacity
        node = argmin(n -> loads[n.id] / n.capacity, nodes)
        if loads[node.id] + task.load <= node.capacity
            push!(assigned[node.id], task.id)
            loads[node.id] += task.load
        else
            push!(unassigned, task.id)
        end
    end

    allocs = [
        Allocation(n.id, assigned[n.id], loads[n.id] / n.capacity)
        for n in nodes if !isempty(assigned[n.id])
    ]

    placed = length(tasks) - length(unassigned)
    score  = placed / max(length(tasks), 1)
    (allocations=allocs, unassigned=unassigned, score=score,
     strategy="least_connections")
end

# ---------------------------------------------------------------------------
# Monte Carlo reliability simulation
# ---------------------------------------------------------------------------

"""
    monte_carlo_reliability(nodes, tasks, n_trials; seed) → NamedTuple

Simulate `n_trials` random failure scenarios.  In each trial every node
is either up (P = node.reliability) or down.  Compute:
  - mean fraction of tasks that can be served
  - P(all tasks served)
  - P(system capacity ≥ total load)
"""
function monte_carlo_reliability(nodes::Vector{DePINNode}, tasks::Vector{Task},
                                  n_trials::Int=10_000; seed::Int=42)
    total_load = sum(t.load for t in tasks)
    n_tasks    = length(tasks)

    rng_state = seed
    function _rand()
        rng_state = (rng_state * 6_364_136_223_846_793_005 + 1_442_695_040_888_963_407) & 0xffffffffffffffff
        (rng_state >> 33) / (2^31)
    end

    served_fracs  = Float64[]
    all_served    = 0
    cap_sufficient = 0

    for _ in 1:n_trials
        active_cap = sum(n.capacity for n in nodes if _rand() < n.reliability)

        # Greedy assignment with only active capacity
        remaining  = active_cap
        served     = 0
        for task in sort(tasks, by=t->t.load, rev=true)
            if remaining >= task.load
                remaining -= task.load
                served += 1
            end
        end

        frac = served / max(n_tasks, 1)
        push!(served_fracs, frac)
        served == n_tasks   && (all_served    += 1)
        active_cap >= total_load && (cap_sufficient += 1)
    end

    (
        mean_served_fraction  = mean(served_fracs),
        p_all_tasks_served    = all_served    / n_trials,
        p_capacity_sufficient = cap_sufficient / n_trials,
        std_served_fraction   = std(served_fracs; corrected=true),
        n_trials              = n_trials,
    )
end

# ---------------------------------------------------------------------------
# Helpers to parse JSON input
# ---------------------------------------------------------------------------

function parse_node(d::AbstractDict)::DePINNode
    DePINNode(
        string(d["id"]),
        Float64(get(d, "capacity", 100.0)),
        Float64(get(d, "weight", 1.0)),
        Float64(get(d, "reliability", 0.99)),
        string(get(d, "region", "")),
    )
end

function parse_task(d::AbstractDict)::Task
    Task(
        string(d["id"]),
        Float64(get(d, "load", 1.0)),
        string(get(d, "region", "")),
    )
end
