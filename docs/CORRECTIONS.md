# docs/ Corrections & Errata

**Status:** living errata. Last updated 2026-07-21.

Several older `docs/` files predate decisions and code changes that superseded
them. Rather than rewrite frozen specs / ADRs in place, this file records the
current-state truth. **When a doc disagrees with this file, this file wins**
(and `UNIFIED_ARCHITECTURE.md` remains the crown synthesis for everything else).

---

## 1. Human-facing settlement: SUI → USDC

Many docs (`mission.md`, `BIRTH_PIPELINE.md`, `Repo Connection.md`,
`adr/` records, the birth specs) state **"human-facing payment is SUI only."**

**Current truth:** the human-facing settlement token evolved to **USDC**.
Synapse (metabolism, 86M/agent) and Dopamine (compute pool, 86B) remain the
internal/metabolic resources — that part is unchanged. The **"no Àṣẹ token"**
rule (ADR-2) still holds. Only the external settlement asset changed
(SUI → USDC); on-chain identity/receipt anchoring may still use Sui rails.

## 2. WASM tool sandbox: wasmtime path is gated OFF by default

`BIRTH_PIPELINE.md` (Security Properties) and older audit docs describe
**"WASM sandboxing: Wasmtime with capability-limited WASI for all tool
execution."**

**Current truth:** the in-process `wasm` tool is **unregistered by default** —
`tools/mod.rs` gates it behind `OMOKODA_ENABLE_WASM=1`. The pinned
`wasmtime 13.0.1` has two CRITICAL sandbox-escape CVEs (RUSTSEC-2026-0095/0096),
so the vulnerable path is intentionally unreachable in the live deployment.
The real fix is retiring the in-process WASM sandbox in favour of the
per-agent microVM/gVisor tier (design-only today). Tool execution today runs
through the `ExecutionContext` path with tier-gating, permission modes,
path-boundary checks, timeouts and output caps — not via wasmtime.

## 3. Day → Òrìṣà mapping is NOT canonical yet (do not hard-code)

The 7-day resonance tables conflict across docs. Concrete examples:

- `Ritual Codex.md` maps **Saturday** to **both Ọbàtálá and Ọya** (in different
  paragraphs of the same file).
- `SIM 369.md` maps **Saturday → Èṣù**, **Monday → Ṣàngó**.
- `256---65536.md` and `Ọ̀rúnmìlà.md` give yet other day/tier orderings.

**Current truth:** there is **no single ratified day→Òrìṣà table**. Treat every
day-mapping table in `docs/` as illustrative, not authoritative. The code's
rhythm/facet system (`omokoda-hermetic` fractal + `koodu/*.json`) is the
mechanical source of truth; the mythic day-labels must be reconciled into ONE
table before anything depends on them. Until then, do not encode a day→Òrìṣà
mapping as a hard constant.

---

## Also worth knowing (state, not contradiction)

- **Dashboard:** Axiom (`:8876`, real Three.js graph on the live kernel `/v1/*`)
  is THE dashboard. The legacy Next.js frontend (`:3010`) is superseded; docs
  written before the switch predate this.
- **Audit scores are stale:** `audit/overview.md` (5.5/10, "plaintext private
  memory is a fiction") and `audit/EXTERNAL_SECURITY_AUDIT.md` (May 2026)
  predate the sealed ChaCha20Poly1305 private memory, Ed25519 receipt chain,
  UTF-8 truncation fix, and the parser `MAX_INPUT` guard (which closes audit
  finding **H1**). Read them as history.
- **ifascript-stub is gone:** `omokoda-core` now depends on `ifascript` as a
  pinned git dependency; the old vendored stub was removed. Do not recreate it.
