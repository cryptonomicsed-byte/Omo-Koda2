# Resonance scoring for IfáScript's ritual_codex::julia_bridge.
#
# The bridge POSTs a ResonancePacket {odu_id, tier, day, timestamp, intent} to
# POST /mesh/resonance and reads back {"resonance": x}. `compute_resonance`
# is the pure, stdlib-only core (so it can be unit-tested without HTTP/JSON3):
# a deterministic score in [0, 1] that blends three factors —
#
#   * odu_phase   — the Odù's harmonic position within the 256 base Calabash,
#   * tier_factor — the agent's tier capacity (1..7), monotonic,
#   * intent_factor — a bounded, deterministic contribution from the intent text.
#
# The bridge is fail-open, so a stable shape (in range, monotonic in tier)
# matters more than the exact curve.

function compute_resonance(odu_id::Integer, tier::Integer, intent::AbstractString)::Float64
    odu_phase     = mod(odu_id, 256) / 255.0
    tier_factor   = clamp(tier, 1, 7) / 7.0
    intent_factor = isempty(intent) ? 0.0 : mod(sum(codeunits(String(intent))), 97) / 97.0
    return clamp(0.4 * odu_phase + 0.4 * tier_factor + 0.2 * intent_factor, 0.0, 1.0)
end
