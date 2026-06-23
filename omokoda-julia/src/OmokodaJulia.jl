"""
    OmokodaJulia

Julia package implementing the Ọmọ Kọ́dà BB Oracle and NIST entropy validation system.

## Submodules / included files:
- `bb_known.jl`    — Exact known Busy Beaver Σ values (BB(1)–BB(5))
- `bb_approx.jl`   — Conservative upper bounds for BB(n), n > 5
- `complexity.jl`  — BBU (Busy Beaver Unit) code complexity scoring
- `nist_validate.jl` — NIST SP 800-22 inspired statistical entropy tests (pure Julia)
- `ffi_exports.jl` — C-callable `@ccallable` exports for Rust/libloading FFI

## C FFI surface (compiled into libomokoda_julia.so):
```c
double calculate_bbu_c(const uint8_t *code_ptr, size_t code_len);
int    validate_entropy_c(const uint8_t *seed_ptr, size_t seed_len);
int    check_bb_bound_c(unsigned int tier, unsigned long long estimated_steps);
```

## Julia API:
```julia
using OmokodaJulia

# Busy Beaver values
bb_value(5)              # => 47176870
bb_known(6)              # => false
bb_upper_bound(6)        # => Inf

# Complexity scoring
calculate_bbu("")                 # => 1.0
calculate_bbu("if x > 0; end")   # => some value in [1.0, 47.1]

# Entropy validation
seed = collect(Vector{UInt8}("hello"))
validate_entropy(seed)            # => true (SHA-256 of "hello" passes all tests)
frequency_test(rand(Bool, 1000))  # => likely true
```
"""
module OmokodaJulia

# ---------------------------------------------------------------------------
# Include submodules in dependency order
# ---------------------------------------------------------------------------

include("bb_known.jl")
include("bb_approx.jl")
include("complexity.jl")
include("nist_validate.jl")
include("ffi_exports.jl")

# ---------------------------------------------------------------------------
# Public exports
# ---------------------------------------------------------------------------

# Busy Beaver known values
export BB_KNOWN
export bb_value
export bb_known

# Busy Beaver approximation / bounds
export bb_upper_bound
export within_bb_bound

# BBU complexity scoring
export calculate_bbu

# NIST entropy validation
export frequency_test
export runs_test
export avalanche_test
export validate_entropy
export sha256
export bytes_to_bits
export hamming_distance_fraction
export seed_has_min_entropy

# C FFI exports (also callable from Julia directly)
export calculate_bbu_c
export validate_entropy_c
export check_bb_bound_c

end # module OmokodaJulia
