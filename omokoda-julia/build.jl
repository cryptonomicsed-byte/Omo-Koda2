# Build shared library using PackageCompiler
# Usage: julia build.jl
using PackageCompiler

create_library(
    ".",
    "lib";
    lib_name = "omokoda_julia",
    precompile_execution_file = "src/OmokodaJulia.jl",
    force = true,
)
println("Built lib/libomokoda_julia.so")
