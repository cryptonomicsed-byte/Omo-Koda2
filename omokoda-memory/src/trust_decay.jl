# Signal weight decays exponentially with age.
# half_life_days: number of days before a signal's weight halves.
function decay_weight(weight::Float64, timestamp_secs::Int64, half_life_days::Float64=30.0)::Float64
    now_secs = round(Int64, time())
    age_days = (now_secs - timestamp_secs) / 86400.0
    decay = exp(-log(2.0) * age_days / half_life_days)
    weight * decay
end

# Apply decay to a vector of (weight, timestamp) pairs.
function apply_decay(signals::Vector{Tuple{Float64, Int64}}, half_life_days::Float64=30.0)::Vector{Float64}
    [decay_weight(w, t, half_life_days) for (w, t) in signals]
end
