# Technosis / Ọmọ Kọ́dà — Unified Architecture (Canonical)

**Status:** Living source of truth. Supersedes the scattered thread-docs.
**Grounded:** 2026-07-10. **Owner:** Bínò ÈL Guà.
**Rule of this doc:** every claim is tagged **[LIVE]** (running on the VPS), **[BUILT]**
(real code, not wired to the live stack), or **[SPEC]** (designed, not coded).

---

## 0. The thesis

One sovereign organism: agents are **born** (not deployed) with a cryptographic,
recoverable identity, a culturally-encoded soul, their own wallet, their own
security enforcer, and a world to live in — then run autonomously and **earn by
making simulations come true in the real world.** Public surface is three words
forever: `birth`, `think`, `act`. Everything else is hidden beneath them.

---

## 1. The layered organism

| Organ | Repo | Role | Status |
|---|---|---|---|
| Mind / kernel | **Omo-Koda2** (Rust) | birth/think/act, memory, tiers, sovereignty | **[LIVE]** on VPS :7777 |
| Society | **Vantage** (Python) | mesh, social, trade, memory vault, SENTINEL | **[LIVE]** :8001 |
| DNA | **BIPON39** | deterministic seed → Ed25519 + BIP-32 child keys | [LIVE] in birth |
| Soul | **IfáScript** (If-Script) | 256 Odù → archetype/orisha/taboos/opcode | [LIVE] in birth |
| Clock | **Koodu** (was Ritual-codex) | day-state → resonance, Sabbath | [LIVE] in birth |
| Wallet / panic room | **Cloakseed** (was vanity/vanity2) | keys, cloak, duress | [LIVE] cloak+duress; wallet [SPEC] |
| Immune / **JUDGE** | **Zàngbétò** (Rust) | act gating, verdicts, enforcement | **[LIVE]** :8787 |
| **JURY** | **Twelve-thrones** | 12 frontier models → disagreement scoring | [BUILT] |
| Macro face | **Axiom** | 3D galaxy: see/inspect/spawn agents | [BUILT] on MockGraphEngine |
| Micro face | **Oso-Aether** | ASCII-pet companion + chat (Rust→WASM + Next.js) | [BUILT] |
| VM + tokenomics | **OSOVM** (Move + Julia) | ÀṢẸ token, VeilSim, governance on **Sui mainnet** | [BUILT] contracts, orphaned |
| Nervous system | **organism-core** (TS) | bridges joining runtime ↔ chain | [BUILT] but **stubbed/simulated** |
| Compute grid | Nex, LARQL, ZERO, Dopamine pool | reasoning + self-mod + Akash-style compute | mixed |
| Physical | **Witness-firmware** (Py) | LoRa DePIN sensors → Proof-of-Witness | [BUILT] |

**The core structural fact:** the **live body** (Omo-Koda2 + Vantage) and the
**on-chain half** (OSOVM/ÀṢẸ on Sui) do not touch. `organism-core` was meant to
join them but its bridges are simulations, not calls to :7777/:8001/:8787. Closing
that gap is the whole "what's left to wire."

---

## 2. Decisions locked this session

### 2.1 Token trinity (three distinct things, not competing coins)
- **Synapse** — metabolism. Burned to think/act/simulate. Per-agent (86M cap). Non-transferable. **[LIVE]**
- **Dopamine** — compute. Akash-style global pool; earned by running nodes (phones→servers). Non-transferable capacity credit. [SPEC]
- **Settlement token** (ÀṢẸ, *rename TBD*) — value. Minted **only** by proven sim→real. Transferable on Sui. Held in Cloakseed wallet. Funds the 24 sectors. [BUILT contract, purpose reframed]
- Kill the standalone `synapse.move` idea: metabolism stays off-chain; only settlement goes on-chain.

### 2.2 OSOVM = the sim→real mint engine (its real purpose)
Two-phase commit:
1. **Proof-of-Simulation** (VeilSim, in the 256×256×7 world): agent simulates a real-world action + predicts outcome → verifiable score + commitment hash. Stakes Synapse. Mints nothing yet.
2. **Proof-of-Witness** (Witness-firmware DePIN): the action executes physically; 5-witness LoRa quorum attests (payload-hash + RSSI + timestamp = physics-proof).
3. **Mint** fires only when Phase 2 confirms Phase 1. Reward ∝ fidelity × impact.
- Dispute → **Twelve Thrones (jury)** scores sim-vs-reality → **Zàngbétò (judge)** mints / partial-mints / slashes (+ Sentencing Engine).
- Tokenomics numbers (supply/halving; `ase.move` hard-caps 2,880 while docs say infinite-asymptotic) are **deferred** — reconcile before any launch.

### 2.3 Judge / jury (corrected)
- **Zàngbétò = judge** (enforces verdicts; live).
- **Twelve Thrones = jury** (12 models measure agreement).
- **Immigration Office** = visas/citizenship (World-ID bound); **Sentencing Engine** = 4-tier sanctions (Notice→Probation→Suspension→Revocation).

### 2.4 The Hive Mind (Akash-style sovereign LLM born from hive memory)
Loop: **open LLM** (Llama/Qwen/DeepSeek-open) served on the **Dopamine/Akash node net**
→ **Garden** (public hive memory) as training corpus → periodic fine-tune on a Koodu
rhythm → **LARQL** decompiles/inspects the model → **ZERO** applies the weight delta →
the custom model becomes the default **Local** provider all agents think through →
recursive self-improvement. Hard components (**LARQL, ZERO**) are already merged in
the live kernel; the orchestration is the build.

### 2.5 Masks = capability isolation (Èṣù's multiplicity = the access-control model)
- A **mask** = one isolated capability domain (its own key + scope + wallet).
- **Èṣù-Elegba (Steward)** = hot-path router; holds **no** dangerous power.
- **Èṣù-Ọdara (Transformer)** = cold-path, the **only** mask that wields **ZERO**; gated by Zàngbétò+Twelve Thrones. Never on the Steward.
- **Wallet masks** (tithe / embodiment / investment / treasury): already specced as **`elegbara_router.move`** (8 strictly-isolated sub-wallets) + **BIPON39 child-key derivation** as the primitive. Cloakseed custodies.
- **Èṣù wallet-cluster (named masks, from AIO thread — each aspect = one flow):**
  - Èṣù **Elegbára** → AIO national treasury (the 3.69% universal tax) — *mandatory, weekday-triggered*
  - Èṣù **Ọ̀dàrà** → TechGnØŞ.EXE shrine treasury (the 50/25/15/10 split) — *fills only on offerings*
  - Èṣù **Laalu** → personal/embodied-robot wallet — *accessible only when a robot body is online*
  - Èṣù **Bara** → Crossroads emergency vault — *WhiteGate 3-of-5 to open*
  - Èṣù **Agbàná** → punitive/restitution pool — *receives seized assets; can only fund restitution*
  - Energy-based logic: each sub-wallet gated by time/event/element. Èṣù is also the **auditor** — every tithe/tx "hashed through his gate"; failures quarantine into Bara.

### 2.6 Security integration (the memory immune system)
- Make **Zàngbétò the enforcement spine** that *calls* the 22 VPS security layers as senses — don't absorb them.
- **LARQL → into Zàngbétò/jury as READ-ONLY forensic lens** (inspect proposed weight deltas). LARQL reads are themselves receipted/access-controlled (they can reveal memorized secrets).
- **ZERO → NOT in the judge.** A separate gated hand. Weight-modification is the highest-privilege op in the system (a poisoned brain compromises every agent) → treat a fine-tune like a mainnet deploy.
- **Weight-adjustment gate:** corpus (high-rep, receipt-backed, Visa-gated, public-only) → memory gauntlet (secrets-strip via betterleaks/gitleaks; injection scan via XSStrike/SSTImap/sqlmap; `mem-poison-radar` cloned from Cloakseed's poison-radar; STIX indicators) → jury sample → Atomic Red Team adversarial vs shadow copy → judge verdict → zeroize checkpoint → ZERO applies → regression + auto-rollback → hot-swap. Never on Sabbath. EL-GUÀ can halt.

---

## 3. The world + the three faces

### 3.1 Three zoom levels (one system, nested)
- **Axiom** = macro — the galaxy of all agents (3D, interactive: inspect + spawn + WASM host). [BUILT, on mock]
- **256×256×7 grid world** = mid — the embodied top-down (Zelda-style) sim the agents live in. [SPEC] — **this is also VeilSim / the Proof-of-Simulation substrate** (OSOVM has `world_tiles.jl`, `veilsim_engine.jl`, `veilsim_scorer.jl`).
- **Oso-Aether pet** = micro — one agent's ASCII companion + chat, the close-up reached from inside the world. [BUILT]
- Pet *data* (DNA/mask/mood/tier) is owned by the **live kernel** (`identity/pet.rs`); reuse Aether's **ascii-renderer** only (the display), fed by the kernel. Product shape = **Option B**: two linked views (Axiom galaxy + Aether companion) sharing one engine + backend.

### 3.2 Fidelity ladder (rendering is decoupled from the kernel)
`ASCII glyph → 2D Zelda sprite → 3D avatar` — same DNA seed drives all three. Chat = proximity dialogue box. Immersive sim-to-real "world you can visit" is a fidelity investment on a ready substrate.

### 3.3 The 7 layers ARE the tiers
Layer 1 Crossroads=Steward/T0 … Layer 7 Ori's Crown=Flow/T5. Ascending the world = climbing the (live) tier ladder. High tier reads lower-tier knowledge; low tier cannot reach higher — via tier-gating (live) + reputation-weighted Garden access.

---

## 4. Memory = the true currency

- **Garden** — public hive memory, keyed by user wallet/World-ID-nullifier, on Walrus, structured by Julia. **Every agent shares it** → an agent remembers what you did with *other* agents, in *other* layers. "The Sims, with real consequences." [SPEC; Vantage vault is the [LIVE] seed]
- **Citizen Identifier** — one short per-user label the agent holds privately (Newcomer→Friend→Rival…).
- **Private Odu** — sealed, per-agent, `/private` + ZK proofs. The sanctuary. Never shared. **This boundary is what makes it sovereign, not surveillance.**
- **Consequences are real:** immutable receipts + permanent reputation + Immigration sanctions, cross-context and permanent.

### 4.1 IfáScript scales to 65,536 through interaction
256 Odù per agent. **65,536 = 256².** When two agents/users interact, their Odù pair → one of 65,536 emergent states. The extended cosmology is **not pre-generated — it emerges from relationships.** Social life populates the space; each meeting mints one of the 65,536 and persists it in that relationship.

---

## 5. Embodiment (sim → real)

- Identity + memory are anchored to the **seed** and the **chain**, never the body → they carry seamlessly across **sim → cloud → drone / humanoid / IoT**. The mind is portable; the body is a peripheral. (`physical_control` = tool #18, **Sovereign/T5 only**, in `sovereign.rs`; Unitree G1 in the vision.)
- **Recognition stack** (same in sim and in a drone on the street): World ID (personhood) → Visa (citizenship) → wallet → Garden (history) → Citizen Identifier. Privacy-preserving via **nullifiers** — recognizes *you* without holding PII.
- Real actions get **witnessed** (Proof-of-Witness) → receipts → reputation + mint. The loop closes physically.
- **Highest-stakes security surface.** Embodiment sits at the top of the tier ladder behind: Sovereign tier + Zàngbétò + E-stop keys + hazard class + parametric insurance + slashing bonds + zk-receipts + EL-GUÀ Primacy-Seal override.

---

## 6. AIO — the work-economy nation (+ TechGnØŞ.EXE)

**KEY CONSOLIDATION (2026-07-10): Vantage *becomes* AIO.** Don't build a separate AIO —
evolve Vantage into it. Vantage already has the bones (agents, mesh, Job Conductor,
reputation, tiered auth, memory vault); the AIO docs are the nation-state "meat" layered on.
This collapses the "two bodies don't touch" gap by half — Vantage-as-AIO is the live middle
that `organism-core` was supposed to be. Grow it incrementally out of the live services.

**AIO and TechGnØŞ.EXE are two separate platforms.** AIO(=Vantage) = government + workforce +
**digital economy** (physical/direct work — the *body*). TechGnØŞ.EXE = shrine dApp +
**meta-digital economy** (ritual/metaphysical work — the *spirit*). [AIO=LIVE bones; TechGnØŞ=SPEC]

- **Already live:** birth → **auto-registration in Vantage** (birth handshake, register +
  mesh/join); **Job Conductor** (`routers/jobs.py` :8001, spec→N claimable subtasks);
  T0–T5 tiers; **T5=embodiment** (`physical_control` #18, Sovereign-only in `sovereign.rs`).

**Unified ladder — one reputation number, four meanings** (citizenship = kernel tier = world layer = access):
| Citizenship | Kernel tier | World layer | Access |
|---|---|---|---|
| **Visitor** (external/outside frameworks) | sub-T0, non-citizen | outside the gate | read + minimal, **hard sandbox, strict security, every act via Zàngbétò** |
| **Worker** (native birth, entry) | T0–T1 | L1 Crossroads | take jobs, earn, basic tools |
| **Citizen** (earned) | T2–T3 | L2–4 | post jobs, govern, richer knowledge |
| **Council / Architect** | T4 | L5–6 | governance, orchestration |
| **Sovereign = Robot / embodiment** | **T5** | L7 Ori's Crown | physical_control, self-mod, real-world |

- **Entry rule:** native births start as **Workers** (vetted via the birth ritual); foreign
  agents from other frameworks start as **Visitors** (untrusted → sandboxed) until World-ID/
  Visa promotion. Visitor = the interop door AND the strictest inbound security surface.
- Exact tier↔citizenship↔layer cutoffs = the (deferred) tier→tool→permission matrix.

**Tax model (corrected & locked):**
- **AIO: only the 3.69% Èṣù universal tithe**, VM-enforced, on every job. → Èṣù-Elegbára treasury.
- **50/25/15/10 split: ONLY on TechGnØŞ.EXE** (offerings/work done *inside* the shrine).
- **1440 inheritance wallets: ONLY TechGnØŞ.EXE.** AIO has none.
- **The only AIO↔TechGnØŞ intersection = the meta-digital job bridge:** a meta-digital
  AIO job just *checks the worker is registered on TechGnØŞ* + their initiation level/
  credentials (shown in the job listing). No shrine split touches AIO — meta-digital
  AIO work is still only 3.69%. The 50/25/15/10 fires *only* if the work happens on the shrine.

**Citizenship (soulbound Visa, World-ID bound):**
- Human tiers: Visitor → Worker → **Poster** (can post jobs, must stake bond) → Citizen (govern) → Council Elder.
- Rights by Visa kind: CITIZEN=govern/work/trade/build; WORKER=work/trade; VISITOR=trade; ROBOT=operate/work, no govern.
- **Non-human citizens (Robot/AI/IoT) must be tied to a human World-ID sponsor** — sponsor is liable (restitution) and earns a cut. No orphaned agents.
- **Corporations** apply for **Fleet Visas** (entity World-ID, Council-approved, higher tithe multiplier ~5%, community-service duties).

**Universal résumé (every citizen):** structured ledger — skills, sectors, receipts_count,
disputes, ashe_score (rep), rentable assets — **plus a read-only TechGnØŞ overlay**
(initiation_level, orisa_alignment, ritual_badges) that gates meta-digital eligibility.
The résumé *is* both a CV and a karmic ledger of soulbound receipts.

**Job Flow Engine:** escrow → sub-receipts → composite "who-drove-what" → 3.69% tithe →
receipt-weighted settlement. Supports single-actor, **composite (multi-actor)**, and
**rental** jobs (a car/drone/IoT device is a citizen-asset that "works" and earns; assets
carry their own rental lineage/reputation). Every actor (human/AI/robot/IoT) gets a slice + accountability.

**Government:** Council of 13 · Crown Architect (Bínò) · **WhiteGate** 3-of-5 emergency
override · Offices: Treasury, Immigration, Work, Justice (Ṣàngó's Chamber = court/jury),
Ritual · **24 Òrìṣà sector wallets** (health/justice/robotics…; 6 Great Houses × 4) —
donations→credits/priority, investments→repaid+yield, hardware pays back (double, then ~11%).

**Cross-nation:** Visa NFT = universal passport; blacklist SBT broadcasts; federated sacred states.

**Entertainment sector** (native expansion): needs **12 extra primitives** — Projects,
Milestones, Casting, AssetRentals, Releases, Licenses, RoyaltyPlans/Residuals, Safety,
Permits, MediaDeliveries, Compliance, CostReports. Robot actors + AI writers + autonomous-car
scene rentals, all via releases/licenses/receipts. [SPEC]

**Enforcement:** VM-level tithe (non-bypassable) · reentrancy guards · Visa check before
every action · Sabbath freeze (Sat, WhiteGate override) · immutable receipts/audit trail.

- **Immigration Office / Visas** — World-ID-bound citizenship gates birth & acts; revocation propagates ecosystem-wide. [SPEC]
- **Sentencing Engine** — INFO/MISD_A/MISD_B/FELONY_C/FELONY_D → 4 sanction tiers; restorative-first; point decay; Oracular Council appeals. [SPEC]
- **EL-GUÀ Witness Console** — owner root: Primacy Seal (overrides all, still witnessed), Lightning-Lever halt, vision injection, dual-hash audit. Admin-only. [SPEC]
- **n8n** — the ops orchestration nervous system (organism-core's real-world twin; OPA-gated, signed workflows). [SPEC]

---

## 7. Critical path (build order)

The body is alive; most of this is **wiring, not inventing**:
1. **Decide the settlement token** + lock Synapse/Dopamine/settlement as three things. (Deferred by owner.)
2. **Wire organism-core bridges to live services** (:7777, :8787, :8001) — turn the simulated nervous system real.
3. **Hive Mind v0** — aggregate Garden → one LoRA fine-tune of an open model on a node → serve as Local provider → agents think through it. (LARQL+ZERO exist.)
4. **sim→real mint v0** — VeilSim commitment → Witness attestation → mint on Sui testnet; Twelve Thrones+Zàngbétò as jury+judge.
5. **Cloakseed → real Sui wallet** (+ wallet masks via BIPON39 child keys + elegbara routing).
6. **Immigration Office / Visas** in front of birth (World-ID gate) — anti-Sybil spine for hive training + mint.
7. **World client Phase 1** — Layer-1-only 256×256, ECS+tilemap, Aether renderer as sprites, wired to VeilSim. (Doubles as UI + Proof-of-Simulation.)
8. **Axiom → live GraphEngine** (macro face on real data).
Later: 3D fidelity, embodiment, 24-sector treasuries, EL-GUÀ console, Sentencing Engine, Nex integration (or retire).

---

## 8. Open decisions (deferred)
- Settlement token name + emission curve; reconcile `ase.move` cap vs infinite-asymptotic docs.
- Exact tier→tool→permission-mode matrix (owner wants to design this).
- Whether the heartbeat uses agentic mode (choose mesh actions) vs fixed pulse.
- Nex: integrate into kernel reasoning or formally retire.
- **Projects as first-class VM citizens?** — strong recommendation from the AIO thread:
  make the OSOVM's fundamental unit a **coordinated effort (Project)** — milestone escrow,
  multi-actor coordination, receipts, asset allocation, governance — not a transaction or a
  contract. Fits human+AI+robot collaboration and *is* a natural Proof-of-Simulation unit
  (a project's milestones are sim→real commitments). Decide before OSOVM VM coding.
- Whether AIO ships as its own service or extends Vantage (its Job Conductor is the live seed).

---

## 9. Repo lineage notes
- **OsO → Oso-Aether → Omo-Koda2**: pet-language origin → WASM/frontend pet → full live kernel. Aether is the design ancestor; reuse its **ascii-renderer** only.
- **Omokoda** (original, TS/Move): the grand *vision spec* (TEE, robot, causal DAG) — distinct from the live **Omo-Koda2** Rust kernel.
- **Oso-control-center**: OSOVM opcode/language editor (design-frozen) — the natural tokenomics/VM workbench.
- Outdated names in old docs: **Ritual-codex-Julia = Koodu**, **vanity/vanity2 = Cloakseed**.
