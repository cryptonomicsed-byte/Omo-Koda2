"""
OmokodaMemory — Ọ̀ṣun / Memory layer of the Omo-Koda 7-language DePIN OS.

Provides:
  • Busy Beaver step-count verification (BB(1)–BB(4) exact, BB(5)+ lower-bound)
  • NIST SP 800-22 entropy validation battery (7 implemented, 8 stubbed)
  • Augury predictive memory modeling over agent memory DAGs
  • DePIN resource optimizer (greedy, round-robin, least-connections, Monte Carlo)
  • Garden analytics over Walrus receipt logs (throughput, latency, economy)

HTTP service on :7778 — see server.jl.
Elixir Augury service calls /predict and /garden/feed via HTTP.
Rust hermetic pipeline calls /nist/validate directly.
"""
module OmokodaMemory

include("busy_beaver.jl")
include("nist_tests.jl")
include("augury.jl")
include("depin_optimizer.jl")
include("garden_analytics.jl")
include("glyph_memory.jl")

export
    # Busy Beaver
    verify_bb_steps, run_known_bb, verify_custom_tm, simulate, BBRules,

    # NIST
    run_battery, validate_odu_entropy, NISTResult,
    test_frequency, test_block_frequency, test_runs, test_longest_run,
    test_approx_entropy, test_cumulative_sums, test_serial,
    test_binary_matrix_rank, test_dft, test_non_overlapping, test_overlapping,
    test_universal, test_linear_complexity, test_random_excursions,
    test_random_excursions_var,

    # Augury
    MemoryDAG, MemoryNode, add_snapshot!, walk_path,
    predict, ses_predict, holt_predict, cosine_similarity, similar_sequences,
    summarise_dag,

    # DePIN Optimizer
    DePINNode, Task, Allocation,
    allocate_greedy, allocate_round_robin, allocate_least_connections,
    monte_carlo_reliability, parse_node, parse_task,

    # Garden Analytics
    Receipt, parse_receipt, analyse_receipts, augury_feed,

    # GlyphIndex memory (SOMA/REM + birth registration)
    glyph_content_hash, glyph_canonical_id, glyph_fold, odu_link, glyph_node,
    GlyphMemory, GlyphMemoryNode, glyph_insert!, glyph_link!, glyph_describe,
    glyph_select_by_tag, glyph_walk, glyph_infer_shared_odu!, glyph_merge!,
    register_birth!, soma_store!, rem_consolidate!,
    gix1_audit, glyph_merkle_root, GIX1_EMPTY_ROOT

end
