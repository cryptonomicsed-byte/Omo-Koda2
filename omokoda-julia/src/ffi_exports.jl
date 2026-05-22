# C-callable FFI exports using Julia's @ccallable macro.
#
# These functions are compiled into libomokoda_julia.so by PackageCompiler
# and loaded by Rust via libloading.
#
# C signatures:
#   double calculate_bbu_c(const uint8_t *code_ptr, size_t code_len);
#   int    validate_entropy_c(const uint8_t *seed_ptr, size_t seed_len);
#   int    check_bb_bound_c(unsigned int tier, unsigned long long estimated_steps);

"""
    calculate_bbu_c(code_ptr::Ptr{UInt8}, code_len::Csize_t) -> Cdouble

C-callable FFI export.

Reads `code_len` bytes from `code_ptr`, interprets them as UTF-8 text,
and returns the BBU complexity score in [1.0, 47.1].

Returns 1.0 on null pointer or zero length.
"""
Base.@ccallable function calculate_bbu_c(
    code_ptr::Ptr{UInt8},
    code_len::Csize_t
)::Cdouble
    if code_ptr == C_NULL || code_len == 0
        return 1.0
    end
    try
        bytes = unsafe_wrap(Array, code_ptr, Int(code_len); own=false)
        text  = String(copy(bytes))
        return calculate_bbu(text)
    catch
        return 1.0
    end
end

"""
    validate_entropy_c(seed_ptr::Ptr{UInt8}, seed_len::Csize_t) -> Cint

C-callable FFI export.

Reads `seed_len` bytes from `seed_ptr` and runs all NIST-inspired entropy tests.

Returns:
  1  — all tests pass (seed has good entropy)
  0  — one or more tests failed, or invalid input
"""
Base.@ccallable function validate_entropy_c(
    seed_ptr::Ptr{UInt8},
    seed_len::Csize_t
)::Cint
    if seed_ptr == C_NULL || seed_len == 0
        return Cint(0)
    end
    try
        bytes = unsafe_wrap(Array, seed_ptr, Int(seed_len); own=false)
        seed  = Vector{UInt8}(copy(bytes))
        result = validate_entropy(seed)
        return result ? Cint(1) : Cint(0)
    catch
        return Cint(0)
    end
end

"""
    check_bb_bound_c(tier::Cuint, estimated_steps::Culonglong) -> Cint

C-callable FFI export.

Checks whether `estimated_steps` falls within the Busy Beaver upper bound
for the given `tier` (number of Turing machine states).

Returns:
  1  — estimated_steps is within BB(tier) bound (or tier >= 6, always within)
  0  — estimated_steps exceeds the BB(tier) upper bound
"""
Base.@ccallable function check_bb_bound_c(
    tier::Cuint,
    estimated_steps::Culonglong
)::Cint
    try
        n = Int(tier)
        steps = UInt64(estimated_steps)
        result = within_bb_bound(n, steps)
        return result ? Cint(1) : Cint(0)
    catch
        return Cint(0)
    end
end
