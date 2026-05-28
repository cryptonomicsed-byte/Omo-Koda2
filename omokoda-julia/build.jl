#!/usr/bin/env julia
# build.jl — PackageCompiler build script for OmokodaJulia
#
# Creates lib/libomokoda_julia.so (Linux) or lib/libomokoda_julia.dylib (macOS).
#
# Usage:
#   julia --project=. build.jl
#
# Prerequisites:
#   - Julia 1.10+
#   - PackageCompiler installed in the project environment:
#       julia --project=. -e 'using Pkg; Pkg.instantiate()'
#
# Output:
#   lib/libomokoda_julia.so   (or .dylib on macOS)
#
# The resulting shared library exposes three C symbols:
#   double calculate_bbu_c(const uint8_t *code_ptr, size_t code_len);
#   int    validate_entropy_c(const uint8_t *seed_ptr, size_t seed_len);
#   int    check_bb_bound_c(unsigned int tier, unsigned long long estimated_steps);
#
# Rust usage (libloading):
#   let lib = Library::new("./lib/libomokoda_julia.so")?;
#   let calculate_bbu: Symbol<unsafe extern fn(*const u8, usize) -> f64> =
#       lib.get(b"calculate_bbu_c")?;

using Pkg

# Ensure we're operating on this project
project_dir = dirname(abspath(@__FILE__))
Pkg.activate(project_dir)
Pkg.instantiate()

using PackageCompiler

# Output directory
lib_dir = joinpath(project_dir, "lib")
mkpath(lib_dir)

@info "Building libomokoda_julia shared library..." project=project_dir output=lib_dir

# Determine platform-specific library name
lib_name = if Sys.iswindows()
    "omokoda_julia.dll"
elseif Sys.isapple()
    "libomokoda_julia.dylib"
else
    "libomokoda_julia.so"
end

lib_path = joinpath(lib_dir, lib_name)

# Build the shared library.
# The @ccallable functions in ffi_exports.jl are automatically exported.
create_library(
    project_dir,
    lib_dir;
    lib_name         = "omokoda_julia",
    precompile_execution_file = joinpath(project_dir, "test", "runtests.jl"),
    incremental      = false,
    filter_stdlibs   = true,
    force            = true,
)

@info "Build complete." library=lib_path

# Verify the expected symbols are present
@info "Verifying exported symbols..."
if Sys.islinux() || Sys.isapple()
    nm_tool  = Sys.isapple() ? `nm -gU $lib_path` : `nm -D $lib_path`
    nm_output = read(nm_tool, String)
    expected_symbols = [
        "calculate_bbu_c",
        "validate_entropy_c",
        "check_bb_bound_c",
    ]
    all_found = true
    for sym in expected_symbols
        if occursin(sym, nm_output)
            @info "  ✓ $sym"
        else
            @warn "  ✗ $sym NOT FOUND"
            all_found = false
        end
    end
    if all_found
        @info "All expected C symbols verified successfully."
    else
        @error "Some symbols are missing — check @ccallable annotations in ffi_exports.jl"
        exit(1)
    end
end
