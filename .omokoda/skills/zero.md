---
name: zero
version: 0.3.4
description: Zerolang graph-first self-modification — zero.graph is the program, .0 files are projections. Query compiler facts and apply checked graph patches through the `zero` tool.
tier: 5
invocation: act zero {"args":["query"]}
---

# Zero (Zerolang)

Zero is an agent-first programming language. `zero.graph` is the program;
`.0` files are human-readable projections of it. Agents do not edit source
text — they read compiler facts with `zero query`/`zero view` and mutate the
program with checked, atomized `zero patch` operations that reject stale or
invalid edits.

In Ọ̀mọ̀ Kọ́dà this runs through the Sovereign-tier `zero` tool, so every
invocation inherits tier gating, hermetic gates, receipts, and Sabbath
queueing (graph patches are irreversible actions — they queue while the
dream engine runs its REM cycle):

```text
act zero {"args":["query"]}
act zero {"args":["view","--fn","main"]}
act zero {"args":["patch","--op","addMain"]}
act zero {"args":["check","--json"]}
act zero {"args":["test"]}
```

The compiler binary resolves from `ZERO_BIN`, then `zero` on PATH, then
`~/.zero/bin/zero`. Install once when missing:

```sh
command -v zero >/dev/null 2>&1 || { curl -fsSL https://zerolang.ai/install.sh | bash; }
export PATH="$PATH:$HOME/.zero/bin"
zero --version
```

## Version-Matched Workflows

This file is a discovery stub, not the full Zero reference. The installed
compiler serves workflow content matched to its exact binary — fetch each
topic at most once per session:

```text
act zero {"args":["skills"]}
act zero {"args":["skills","get","zero"]}
act zero {"args":["skills","get","agent"]}          # read-edit-verify loop
act zero {"args":["skills","get","graph"]}          # query/view, patch ops, import/export
act zero {"args":["skills","get","diagnostics"]}    # zero explain, typed fix plans
act zero {"args":["skills","get","stdlib","--topic","std.time"]}
```

Topics: `zero` (~2 KB stub), `agent` (~4 KB), `language` (~6 KB), `graph`
(~9 KB), `diagnostics` (~4 KB), `packages` (~5 KB), `builds` (~5 KB),
`testing` (~3 KB), `stdlib` (~39 KB; fetch one module section at a time).
Before hand-writing any parsing or validation, check the stdlib catalog —
`std.time` (RFC 3339 incl. leap seconds), `std.inet`, `std.regex`,
`std.unicode` ship ready-made validators.

## Common Entry Points

```text
zero query [graph-or-package]
zero patch [graph-or-package] --op '<operation>'
zero check [graph-or-package]
zero test  [graph-or-package]
zero explain <diagnostic-code>
```

Edit through the graph: `zero patch` covers surgical in-function text edits
(`--replace-in-fn <fn> --old <text> --new <text>`), helper creation
(`upsertFunction ... end`), and whole function bodies (`--replace-fn <fn>
--body-file -`). Read one function with `zero view --fn <name>` instead of
whole files. Prefer concise text output for interactive work; use `--json`
for automation, exact spans, contracts, or machine-readable diagnostics.
