# Archive

Superseded prototypes, kept for history/salvage rather than deleted. Nothing
here is deployed, referenced by any systemd unit, or called by the kernel.
See [[omokoda2-language-consolidation]] (agent memory) for the full audit
that led to this.

## `omokoda-go/` — superseded by `omokoda-ops/`

Both implemented Ọya (Go). `omokoda-ops` is the more complete implementation
(metrics, SSE, device management, full `/v1/*` proxy, richer per-tool
cooldown tracking) and is what's actually deployed as `ares-omokoda-oya`.
`omokoda-go`'s only unique capability (the SkillForge coordination routes)
was ported into `omokoda-ops/skillforge_handler.go` before archiving.

## `omokoda-elixir/` — superseded by `omokoda-swarm/`

Both implemented Yemọja (Elixir). `omokoda-swarm` is the mature, deployed
one (council, delegation, hive, teammate management, real Rust birth
wiring). `omokoda-elixir` had zero deployment until a SkillForge session
gave it a temporary job; that capability (the manifest-assembly route) was
ported into `omokoda-swarm/lib/omokoda_swarm/http_api.ex` before archiving.

## `omokoda-sui/` — superseded by `omokoda-on-chain/`

Both are Move contract sets for Ṣàngó. `omokoda-sui`'s `Move.toml` address
is `0x0` — it was **never published**. `omokoda-on-chain` is the real,
deployed package (`0x380e0599...`) that `omokoda-core/src/onchain.rs`
actually calls. Note: `omokoda-sui/sources/garden.move` and
`omokoda-on-chain/sources/garden.move` are **not the same contract** despite
the shared filename — the former is an unrelated IPFS plugin-marketplace
concept, the latter is agent registry + job escrow. Don't assume the name
means the same thing across these two directories.

## `omokoda-lisp/` — the Ọbàtálá (ethics) story is more nuanced

Three of its six files are genuinely redundant with existing, already-live
Rust code — not gaps worth porting:
- `ethics.lisp` (7-gate intent scoring) duplicates
  `omokoda-core/src/steward/gatekeeper.rs`'s `evaluate()`, which already
  runs the same 7 Hermetic-principle checks in-process (see its tests:
  `destructive_bash_without_complement_halted`,
  `cooldown_active_halted_at_rhythm`,
  `think_without_identity_halted_at_mentalism`).
- `consent_rules.lisp` (block external providers for private data) duplicates
  the hard-fail already in `interpreter.rs` ("Private thoughts require a
  local provider...").
- `policy_ast.lisp` (tier-based tool permissions as S-expressions) duplicates
  `permissions.rs`'s `default_steward_policy` / tier-gating.

The live, deployed Ọbàtálá is `omokoda-clojure/obatala.clj` (consent-mode
evaluation + SkillForge's `/skillforge/analyze`), and that's correct —
nothing from the redundant three files needs porting there.

**One file is genuinely novel, not a duplicate**, and was deliberately
**not** ported as part of this consolidation because it's really a new
feature, not a merge: `why_engine.lisp` — a constitutional-amendment voting
system (7/7 Òrìṣà unanimous consent + human veto) with human-readable
alignment-explanation generation. Nothing else in the codebase does this.
Worth a dedicated future session if wanted.
