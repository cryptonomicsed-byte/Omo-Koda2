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

export
    # Busy Beaver
    verify_bb_steps, run_known_bb, verify_custom_tm, simulate, BBRules,

    # NIST
    run_battery, validate_odu_entropy, NISTResult,
    test_frequency, test_block_frequency, test_runs, test_longest_run,
    test_approx_entropy, test_cumulative_sums, test_serial,

    # Augury
    MemoryDAG, MemoryNode, add_snapshot!, walk_path,
    predict, ses_predict, holt_predict, cosine_similarity, similar_sequences,
    summarise_dag,

    # DePIN Optimizer
    DePINNode, Task, Allocation,
    allocate_greedy, allocate_round_robin, allocate_least_connections,
    monte_carlo_reliability, parse_node, parse_task,

    # Garden Analytics
    Receipt, parse_receipt, analyse_receipts, augury_feed

end
