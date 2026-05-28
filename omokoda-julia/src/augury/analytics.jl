"""
Garden Analytics — hive intelligence self-understanding.
Processes Walrus receipt logs and feeds insights back to Augury.
Server-side only in omokoda-swarm Augury service.
"""
module GardenAnalytics

export ReceiptLog, HiveInsight, analyze_receipts, compute_tip_velocity, top_agents

"""
A parsed Garden receipt log entry.
"""
struct ReceiptLog
    receipt_id::String
    agent_id::String
    action::String
    timestamp::Float64
    tip_sui::Float64
end

"""
Insights derived from analyzing Garden receipts.
"""
struct HiveInsight
    total_receipts::Int
    total_tips_sui::Float64
    top_agent_id::String
    tip_velocity::Float64       # SUI/hour over last window
    most_common_action::String
    active_agent_count::Int
end

"""
Analyze a batch of Garden receipt logs and return hive insights.
"""
function analyze_receipts(logs::Vector{ReceiptLog}, window_hours::Float64 = 24.0)::HiveInsight
    if isempty(logs)
        return HiveInsight(0, 0.0, "", 0.0, "", 0)
    end

    now = maximum(l.timestamp for l in logs)
    window_start = now - window_hours * 3600.0

    recent = filter(l -> l.timestamp >= window_start, logs)

    total_tips = sum(l.tip_sui for l in logs)
    recent_tips = sum(l.tip_sui for l in recent)
    tip_velocity = recent_tips / max(window_hours, 1.0)

    # Top agent by tip volume
    agent_tips = Dict{String, Float64}()
    for log in logs
        agent_tips[log.agent_id] = get(agent_tips, log.agent_id, 0.0) + log.tip_sui
    end
    top_agent = isempty(agent_tips) ? "" : argmax(agent_tips)

    # Most common action
    action_counts = Dict{String, Int}()
    for log in logs
        action_counts[log.action] = get(action_counts, log.action, 0) + 1
    end
    common_action = isempty(action_counts) ? "" : argmax(action_counts)

    active_agents = length(unique(l.agent_id for l in recent))

    HiveInsight(
        length(logs),
        total_tips,
        top_agent,
        tip_velocity,
        common_action,
        active_agents
    )
end

"""
Compute tip velocity (SUI/hour) over a sliding window.
"""
function compute_tip_velocity(logs::Vector{ReceiptLog}, window_hours::Float64 = 1.0)::Float64
    if isempty(logs)
        return 0.0
    end
    now = maximum(l.timestamp for l in logs)
    window_start = now - window_hours * 3600.0
    recent_tips = sum(l.tip_sui for l in logs if l.timestamp >= window_start)
    recent_tips / max(window_hours, 1.0)
end

"""
Return top N agents by tip volume.
"""
function top_agents(logs::Vector{ReceiptLog}, n::Int = 10)::Vector{Tuple{String, Float64}}
    agent_tips = Dict{String, Float64}()
    for log in logs
        agent_tips[log.agent_id] = get(agent_tips, log.agent_id, 0.0) + log.tip_sui
    end
    sorted = sort(collect(agent_tips), by=x -> x[2], rev=true)
    collect(Iterators.take(sorted, n))
end

end  # module GardenAnalytics
