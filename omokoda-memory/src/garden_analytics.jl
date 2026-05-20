"""
Garden Analytics — Walrus receipt log processing.

Ọ̀ṣun / Memory layer.  Processes act receipt logs from the Walrus storage layer,
feeds insights back to the Augury engine, and provides hive intelligence about
agent behaviour at scale.

Each receipt:
  {
    "receipt_id": "...",
    "agent_id":   "...",
    "tool":       "bash",
    "tier":       2,
    "timestamp":  "2025-01-01T00:00:00Z",
    "success":    true,
    "duration_ms": 123,
    "synapse_cost": 1000.0,
    "dopamine_cost": 500.0
  }
"""

using Statistics: mean, median, std, quantile

# ---------------------------------------------------------------------------
# Data type
# ---------------------------------------------------------------------------

struct Receipt
    receipt_id::String
    agent_id::String
    tool::String
    tier::Int
    timestamp::Float64       # Unix epoch seconds
    success::Bool
    duration_ms::Float64
    synapse_cost::Float64
    dopamine_cost::Float64
end

function parse_receipt(d::AbstractDict)::Receipt
    ts = begin
        raw = get(d, "timestamp", 0)
        raw isa Number ? Float64(raw) : _parse_iso8601(string(raw))
    end
    Receipt(
        string(get(d, "receipt_id", "")),
        string(get(d, "agent_id", "")),
        string(get(d, "tool", "unknown")),
        Int(get(d, "tier", 0)),
        ts,
        Bool(get(d, "success", true)),
        Float64(get(d, "duration_ms", 0)),
        Float64(get(d, "synapse_cost", 0)),
        Float64(get(d, "dopamine_cost", 0)),
    )
end

# Minimal ISO 8601 → epoch (handles "YYYY-MM-DDTHH:MM:SSZ")
function _parse_iso8601(s::String)::Float64
    try
        parts = split(s, r"[-T:Z]", keepempty=false)
        length(parts) < 6 && return 0.0
        y, mo, d, h, m, sec = parse.(Int, parts[1:6])
        # Days from 1970-01-01
        days_to_year(yr) = 365(yr-1970) + (yr-1969)÷4 - (yr-1901)÷100 + (yr-1601)÷400
        MONTH_DAYS = [31,28,31,30,31,30,31,31,30,31,30,31]
        is_leap(yr) = (yr%4==0 && yr%100!=0) || yr%400==0
        MONTH_DAYS[2] = is_leap(y) ? 29 : 28
        day_of_year = d + sum(MONTH_DAYS[1:mo-1])
        Float64((days_to_year(y) + day_of_year - 1) * 86400 + h*3600 + m*60 + sec)
    catch
        0.0
    end
end

# ---------------------------------------------------------------------------
# Analytics
# ---------------------------------------------------------------------------

"""
    analyse_receipts(receipts) → NamedTuple

Compute a full analytics report over a batch of receipts.
"""
function analyse_receipts(receipts::Vector{Receipt})
    n = length(receipts)
    n == 0 && return _empty_report()

    successes    = filter(r -> r.success,  receipts)
    failures     = filter(r -> !r.success, receipts)
    durations    = [r.duration_ms   for r in receipts]
    synapse_vals = [r.synapse_cost   for r in receipts]
    dopamine_vals= [r.dopamine_cost  for r in receipts]

    # Tool frequency
    tool_freq = Dict{String,Int}()
    for r in receipts
        tool_freq[r.tool] = get(tool_freq, r.tool, 0) + 1
    end

    # Tier distribution
    tier_dist = Dict{Int,Int}()
    for r in receipts
        tier_dist[r.tier] = get(tier_dist, r.tier, 0) + 1
    end

    # Per-agent stats
    agents = unique(r.agent_id for r in receipts)
    agent_stats = Dict(
        a => _per_agent(filter(r -> r.agent_id == a, receipts))
        for a in agents
    )

    # Latency percentiles
    sorted_dur = sort(durations)
    p50 = _percentile(sorted_dur, 0.50)
    p95 = _percentile(sorted_dur, 0.95)
    p99 = _percentile(sorted_dur, 0.99)

    # Time-series throughput (receipts per minute buckets)
    throughput = _throughput_buckets(receipts, bucket_secs=60)

    (
        total            = n,
        success_count    = length(successes),
        failure_count    = length(failures),
        success_rate     = length(successes) / n,
        tool_frequency   = tool_freq,
        tier_distribution= tier_dist,
        latency = (
            mean_ms = mean(durations),
            median_ms = median(durations),
            p95_ms  = p95,
            p99_ms  = p99,
            max_ms  = maximum(durations),
        ),
        economy = (
            total_synapse   = sum(synapse_vals),
            total_dopamine  = sum(dopamine_vals),
            mean_synapse    = mean(synapse_vals),
            mean_dopamine   = mean(dopamine_vals),
        ),
        agent_count  = length(agents),
        agent_stats  = agent_stats,
        throughput   = throughput,
    )
end

function _per_agent(receipts::Vector{Receipt})
    n = length(receipts)
    n == 0 && return (total=0, success_rate=0.0, mean_duration_ms=0.0)
    success = count(r -> r.success, receipts)
    durs    = [r.duration_ms for r in receipts]
    (
        total            = n,
        success_rate     = success / n,
        mean_duration_ms = mean(durs),
        total_synapse    = sum(r.synapse_cost for r in receipts),
    )
end

function _percentile(sorted::Vector{Float64}, p::Float64)::Float64
    isempty(sorted) && return 0.0
    idx = clamp(ceil(Int, p * length(sorted)), 1, length(sorted))
    sorted[idx]
end

function _throughput_buckets(receipts::Vector{Receipt}, bucket_secs::Int=60)
    isempty(receipts) && return []
    timestamps = sort([r.timestamp for r in receipts])
    t_start = timestamps[1]
    t_end   = timestamps[end]
    n_buckets = max(1, ceil(Int, (t_end - t_start) / bucket_secs) + 1)

    buckets = zeros(Int, n_buckets)
    for t in timestamps
        idx = clamp(floor(Int, (t - t_start) / bucket_secs) + 1, 1, n_buckets)
        buckets[idx] += 1
    end

    [Dict("bucket" => i-1, "count" => buckets[i]) for i in 1:n_buckets]
end

function _empty_report()
    (total=0, success_count=0, failure_count=0, success_rate=0.0,
     tool_frequency=Dict(), tier_distribution=Dict(),
     latency=(mean_ms=0.0, median_ms=0.0, p95_ms=0.0, p99_ms=0.0, max_ms=0.0),
     economy=(total_synapse=0.0, total_dopamine=0.0, mean_synapse=0.0, mean_dopamine=0.0),
     agent_count=0, agent_stats=Dict(), throughput=[])
end

# ---------------------------------------------------------------------------
# Augury feed — distil insights for the prediction engine
# ---------------------------------------------------------------------------

"""
    augury_feed(receipts) → Vector{Float64}

Produce a compact numeric feature vector from recent receipts suitable
for input to the Augury prediction model.

Features (12-dimensional):
  [success_rate, mean_dur, p95_dur, mean_synapse, mean_dopamine,
   tool_entropy, tier_mean, tier_std, receipts_per_min,
   agent_diversity, failure_spike, trend_slope]
"""
function augury_feed(receipts::Vector{Receipt})::Vector{Float64}
    isempty(receipts) && return zeros(12)

    report    = analyse_receipts(receipts)
    n         = length(receipts)
    durs      = [r.duration_ms for r in receipts]
    tiers     = Float64[r.tier for r in receipts]

    # Tool entropy (Shannon H)
    tool_freq = values(report.tool_frequency)
    tool_probs= [c/n for c in tool_freq]
    h_tool    = -sum(p * log(p + 1e-12) for p in tool_probs)

    # Time span
    ts        = sort([r.timestamp for r in receipts])
    span_min  = max((ts[end] - ts[1]) / 60.0, 1e-6)
    recs_pm   = n / span_min

    # Failure spike: fraction of failures in last 20% of time window
    cutoff    = ts[1] + 0.8 * (ts[end] - ts[1])
    recent    = filter(r -> r.timestamp >= cutoff, receipts)
    fail_spike= isempty(recent) ? 0.0 : count(!r.success for r in recent) / length(recent)

    # Linear trend in success rate over time buckets
    buckets   = _throughput_buckets(receipts, 120)
    trend     = length(buckets) >= 2 ?
                    (buckets[end]["count"] - buckets[1]["count"]) / max(buckets[1]["count"], 1) :
                    0.0

    [
        report.success_rate,
        mean(durs),
        _percentile(sort(durs), 0.95),
        report.economy.mean_synapse,
        report.economy.mean_dopamine,
        h_tool,
        isempty(tiers) ? 0.0 : mean(tiers),
        length(tiers) >= 2 ? std(tiers; corrected=true) : 0.0,
        recs_pm,
        Float64(report.agent_count),
        fail_spike,
        trend,
    ]
end
