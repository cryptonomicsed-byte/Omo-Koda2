# C ABI exports for Rust FFI via libloading
# Build with: julia build.jl

Base.@ccallable function calculate_bbu_c(ptr::Ptr{UInt8}, len::Csize_t)::Cdouble
    code = unsafe_string(ptr, len)
    return calculate_bbu(code)
end

Base.@ccallable function validate_entropy_c(ptr::Ptr{UInt8}, len::Csize_t)::Cint
    seed = unsafe_wrap(Vector{UInt8}, ptr, len; own=false)
    return validate_entropy(seed) ? Cint(1) : Cint(0)
end

Base.@ccallable function check_bb_bound_c(tier::Cint, steps::Clonglong)::Cint
    return check_bb_bound(Int(tier), Int(steps)) ? Cint(1) : Cint(0)
end
