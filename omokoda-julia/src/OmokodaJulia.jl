module OmokodaJulia

include("bb_known.jl")
include("bb_approx.jl")
include("complexity.jl")
include("bb_verifier.jl")
include("ffi_exports.jl")
include("augury/predict.jl")
include("augury/analytics.jl")
include("soma/MemCell.jl")
include("soma/Resonance.jl")
include("soma/LPMUpdater.jl")

export calculate_bbu, validate_entropy, check_bb_bound
export MemCell, MemScene, LPM, EmotionTrace
export score_retrieval, score_all, reconstruct
export cosine_similarity, keyword_score, resonance_boost
export update_lpm, lpm_to_context

end # module
