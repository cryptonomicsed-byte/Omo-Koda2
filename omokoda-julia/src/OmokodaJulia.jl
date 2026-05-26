module OmokodaJulia

include("bb_known.jl")
include("bb_approx.jl")
include("complexity.jl")
include("ffi_exports.jl")

export calculate_bbu, validate_entropy, check_bb_bound

end # module
