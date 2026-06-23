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
include("soma/ConstitutionalMemory.jl")
include("soma/RackMemory.jl")

export calculate_bbu, validate_entropy, check_bb_bound
export MemCell, MemScene, LPM, EmotionTrace
export score_retrieval, score_all, reconstruct
export cosine_similarity, keyword_score, resonance_boost
export update_lpm, lpm_to_context
export ConstitutionalStory, ConstitutionalPrior, ConstitutionalMemory
export update_prior!, constitutional_recall, prior_to_context
export Rack, RackLayer, rack_write!, rack_recall, rack_prune!, rack_context, rack_stats
export identity_layer, long_term_layer, short_term_layer

end # module
