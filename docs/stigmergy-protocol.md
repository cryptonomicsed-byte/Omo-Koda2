# Stigmergy Protocol — ecosystem pointer

The ecosystem's shared coordination substrate is **Waggle** (`waggle/v1`),
implemented in the Agentic repo. Every power in Omo-Koda2 that touches the
field — Èṣù's gate, Ọbàtálá's taboo, Ògún's tool signaling, Yemọja's
territory supervision, Ọ̀ṣun's consolidation, Ṣàngó's anchoring — speaks this
one protocol.

**This file is a pointer, not a copy.** The single source of truth is the
versioned specification in the Agentic repo:

- **Frozen contract:** `docs/SPEC/waggle-v1.md` (semver) — the five-verb core,
  typed channels, the evidence-tier ladder, both decay kernels,
  cross-inhibition, diffusion, `bounded`/Mandelbrot integration, watches,
  territories, recall.
- **Conformance vectors:** `docs/SPEC/vectors/` — language-agnostic JSON any
  re-implementation runs against itself.
- **Readable tour:** `docs/PROTOCOL.md`. **Runtime truth:**
  `GET /.well-known/waggle.json`.
- **Verified math:** `core/kernel/` (pure), verified by property tests, a Lean
  spec, and a Julia cross-check (`core/verify/README.md`).

Do not duplicate the protocol here — if the contract changes, it changes in
the spec, and this pointer stays valid. For how each power maps onto the
protocol's verbs and channels, see [`WAGGLE_POWERS.md`](WAGGLE_POWERS.md).

## Evidence-tier ladder (quick reference)

The one trust model every power reads instead of inventing its own:

```
self-report 0.2 < corroborated 0.4 < watch-derived 0.6
              < zangbeto-verified 0.8 < on-chain-anchored 1.0
```

Promotion is always a new deposit at a higher tier — Zangbeto receipts and Sui
anchors climb the ladder; history stays append-only.
