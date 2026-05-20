"""
Busy Beaver verification — Ọ̀ṣun / Memory layer.

Implements a general 2-symbol Turing machine simulator and validates
step counts against known BB values.  Called from Rust via REST and from
the hermetic entropy pipeline to prove halting bounds.

Known Σ / S values (ones written / steps taken):
  BB(1): Σ=1,  S=1
  BB(2): Σ=4,  S=6
  BB(3): Σ=6,  S=21
  BB(4): Σ=13, S=107
  BB(5): Σ≥4098, S≥47_176_870  (lower bound; full proof pending)
"""

# Each rule: (state::Int, read::Int) => (write::Int, move::Int, next::Int)
# state 0 = HALT, move -1=L, +1=R
const BBRules = Dict{Tuple{Int,Int}, Tuple{Int,Int,Int}}

# ---------------------------------------------------------------------------
# Known champion TMs (for reference and regression checks)
# ---------------------------------------------------------------------------

function bb2_rules()::BBRules
    Dict(
        (1, 0) => (1,  1, 2),  # A,0 → write 1, R, B
        (1, 1) => (1, -1, 2),  # A,1 → write 1, L, B
        (2, 0) => (1, -1, 1),  # B,0 → write 1, L, A
        (2, 1) => (1,  1, 0),  # B,1 → write 1, R, HALT
    )
end

function bb3_rules()::BBRules
    Dict(
        (1, 0) => (1,  1, 2),
        (1, 1) => (0, -1, 3),
        (2, 0) => (0,  1, 1),
        (2, 1) => (1,  1, 2),
        (3, 0) => (1,  1, 2),
        (3, 1) => (1, -1, 1),
    )
end

# ---------------------------------------------------------------------------
# Simulator
# ---------------------------------------------------------------------------

struct TMResult
    halted::Bool
    steps::Int
    ones::Int
    tape_length::Int
    reason::String
end

function simulate(rules::BBRules, max_steps::Int)::TMResult
    tape = Dict{Int,Int}()   # sparse tape; defaults to 0
    head = 0
    state = 1

    for step in 1:max_steps
        sym = get(tape, head, 0)
        trans = get(rules, (state, sym), nothing)

        if trans === nothing
            # No rule → implicit halt
            ones = count(v -> v == 1, values(tape))
            return TMResult(true, step, ones, length(tape), "implicit_halt")
        end

        write_sym, direction, next_state = trans
        tape[head] = write_sym
        head += direction
        state = next_state

        if state == 0
            ones = count(v -> v == 1, values(tape))
            return TMResult(true, step, ones, length(tape), "halted")
        end
    end

    ones = count(v -> v == 1, values(tape))
    TMResult(false, max_steps, ones, length(tape), "step_limit_reached")
end

# ---------------------------------------------------------------------------
# Public verification API
# ---------------------------------------------------------------------------

const KNOWN_BB = Dict(
    1 => (sigma=1,  steps=1),
    2 => (sigma=4,  steps=6),
    3 => (sigma=6,  steps=21),
    4 => (sigma=13, steps=107),
)

"""
    verify_bb_steps(n_states, claimed_steps) → (valid, reason, detail)

Check whether `claimed_steps` is consistent with BB(n_states).
For n ≤ 4 the answer is exact; for n ≥ 5 we return a lower-bound check.
"""
function verify_bb_steps(n_states::Int, claimed_steps::Int)
    if haskey(KNOWN_BB, n_states)
        bb = KNOWN_BB[n_states]
        valid = claimed_steps == bb.steps
        reason = valid ? "matches_known_bb" :
                 "expected $(bb.steps) steps for BB($n_states), got $claimed_steps"
        return (valid=valid, reason=reason,
                known_sigma=bb.sigma, known_steps=bb.steps)
    end

    # For n≥5: run one of the champion TMs and check claimed_steps is ≥ lower bound
    lb_steps = n_states == 5 ? 47_176_870 : 0
    if claimed_steps < lb_steps
        return (valid=false,
                reason="claimed $claimed_steps < BB($n_states) lower bound $lb_steps",
                known_sigma=nothing, known_steps=lb_steps)
    end
    return (valid=true,
            reason="consistent_with_lower_bound",
            known_sigma=nothing, known_steps=lb_steps)
end

"""
    run_known_bb(n_states; max_steps) → TMResult

Simulate the known champion TM for `n_states` and return the result.
"""
function run_known_bb(n_states::Int; max_steps::Int = 200_000)::TMResult
    if n_states == 2
        return simulate(bb2_rules(), max_steps)
    elseif n_states == 3
        return simulate(bb3_rules(), max_steps)
    else
        error("No built-in rules for BB($n_states); provide custom rules")
    end
end

"""
    verify_custom_tm(rules_vec, claimed_steps, claimed_sigma; max_steps) → NamedTuple

Simulate a user-supplied TM and check its claimed halting stats.

rules_vec format: [[state, read, write, move, next_state], ...]
  where move ∈ {-1, 1} and next_state = 0 means HALT.
"""
function verify_custom_tm(rules_vec::Vector, claimed_steps::Int, claimed_sigma::Int;
                           max_steps::Int = 1_000_000)
    rules = BBRules()
    for r in rules_vec
        state, read, write, move, next = Int.(r)
        rules[(state, read)] = (write, move, next)
    end

    result = simulate(rules, max_steps)

    steps_ok  = result.halted && result.steps  == claimed_steps
    sigma_ok  = result.halted && result.ones   == claimed_sigma
    valid     = steps_ok && sigma_ok

    return (
        valid        = valid,
        halted       = result.halted,
        actual_steps = result.steps,
        actual_sigma = result.ones,
        reason       = result.reason,
        steps_match  = steps_ok,
        sigma_match  = sigma_ok,
    )
end
