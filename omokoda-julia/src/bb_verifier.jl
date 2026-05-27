"""
Busy Beaver step-count verifier for Omo-koda PoCW (Proof of Cognitive Work).

The Busy Beaver function BB(n) grows faster than any computable function.
Cannot shortcut. Cannot predict without running. The work IS the proof.

Known values: BB(1)=1, BB(2)=6, BB(3)=21, BB(4)=107, BB(5)=47,176,870
"""
module BbVerifier

using SHA

export verify_bb_steps, pocw_floor_for_tier, BB_KNOWN

# Frozen BB values — mathematical constants, not configurable
const BB_KNOWN = Dict{Int, Int}(
    1 => 1,
    2 => 6,
    3 => 21,
    4 => 107,
    5 => 47_176_870,
)

"""
Minimum PoCW steps required for act tier elevation.
Mirrors Rust PoCWProof::min_for_tier().
"""
function pocw_floor_for_tier(tier::Int)::Int
    if tier <= 0
        return 0
    elseif tier == 1
        return BB_KNOWN[3]  # BB(3) = 21
    elseif tier == 2
        return BB_KNOWN[4]  # BB(4) = 107
    else
        return BB_KNOWN[5]  # BB(5) = 47_176_870
    end
end

"""
Turing Machine state — used for BB step simulation.
"""
struct TmState
    tape::Vector{Int}
    head::Int
    state::Int
    steps::Int
end

"""
Simulate a 2-symbol Turing Machine from a transition table.

transitions[state, symbol] = (write_symbol, direction, next_state)
direction: 1 = right, -1 = left, 0 = halt

Returns (halted::Bool, steps::Int, tape::Vector{Int})
"""
function simulate_tm(
    transitions::Matrix{Tuple{Int,Int,Int}},
    max_steps::Int = 50_000_000;
    initial_tape::Union{Vector{Int}, Nothing} = nothing
)::Tuple{Bool, Int, Vector{Int}}
    if initial_tape !== nothing
        tape = copy(initial_tape)
        # Pad to 1000 if shorter
        if length(tape) < 1000
            append!(tape, zeros(Int, 1000 - length(tape)))
        end
        head = div(length(tape), 2)  # start in middle
    else
        tape = zeros(Int, 1000)
        head = 500  # start in middle
    end
    state = 1
    steps = 0

    while state > 0 && steps < max_steps
        # Expand tape if needed
        if head < 1
            prepend!(tape, zeros(Int, 100))
            head += 100
        elseif head > length(tape)
            append!(tape, zeros(Int, 100))
        end

        symbol = tape[head]
        if state > size(transitions, 1) || symbol + 1 > size(transitions, 2)
            break
        end

        (write_sym, direction, next_state) = transitions[state, symbol + 1]
        tape[head] = write_sym
        head += direction
        state = next_state
        steps += 1
    end

    halted = state == 0 || (state > 0 && steps < max_steps)
    return (halted, steps, tape)
end

"""
Verify that a claimed PoCW step count meets the BB floor for a given tier.

Parameters:
- claimed_steps: number of steps the agent claims to have computed
- tier: act tier (0-5) being claimed

Returns true if claimed_steps >= BB floor for tier.
"""
function verify_pocw_claim(claimed_steps::Int, tier::Int)::Bool
    floor = pocw_floor_for_tier(tier)
    claimed_steps >= floor
end

"""
Verify that a tape hash matches an expected value.
Used by Justice to verify PoCW without re-running the full TM.
"""
function verify_tape_hash(tape::Vector{Int}, expected_hash::String)::Bool
    tape_bytes = reinterpret(UInt8, tape)
    computed = bytes2hex(sha256(tape_bytes))
    computed == expected_hash
end

"""
    verify_bb_steps(tape, transitions, expected_steps) -> Bool

Verify a Busy Beaver computation by running the Turing Machine and confirming
it halts at exactly `expected_steps` steps.

The work IS the proof — there is no shortcut around running the TM.

Parameters:
- tape: initial tape configuration (Vector{Int} of 0s and 1s)
- transitions: n×2 Matrix{Tuple{Int,Int,Int}} where each entry is
  (write_symbol, direction, next_state); direction 1=right, -1=left, 0=halt
- expected_steps: claimed step count to verify against

Returns true if the TM halts at exactly `expected_steps` steps.
"""
function verify_bb_steps(
    tape::Vector{Int},
    transitions::Matrix{Tuple{Int,Int,Int}},
    expected_steps::Int
)::Bool
    (halted, actual_steps, _) = simulate_tm(
        transitions, expected_steps + 1;
        initial_tape = tape
    )
    halted && actual_steps == expected_steps
end

end  # module BbVerifier
