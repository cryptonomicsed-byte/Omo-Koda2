# Ọ̀ṢỌ́ Reference dApps

Three minimal but functional single-page apps demonstrating the sovereign stack
end-to-end. Each is React 18 + TypeScript + Vite. No SSR. Tailwind via CDN.
All talk to Omo-Koda2 running on `localhost:3000`.

---

## dApp 1 — GPU Marketplace (Phase 27.1)

**Port:** 5173  
**Directory:** `gpu-marketplace/`  
**Backend deps:** Omo-Koda2 (`:3000`), UCX broker (`:7790`)

Sovereign compute marketplace. Browse GPU providers fetched from the UCX
adapter list, hire one by submitting a `JobClient.create()` call, poll job
status, and mint a proof badge when F1 ≥ 0.777. List your own GPU via a
`ContractClient` financial/marketplace contract.

**Tabs:** Browse · My Jobs · My GPU

```
cd gpu-marketplace
npm install
npm run dev
```

---

## dApp 2 — Agent Employment (Phase 27.2)

**Port:** 5174  
**Directory:** `employment/`  
**Backend deps:** Omo-Koda2 (`:3000`) — `/api/v1/public/agents`

Sovereign labour market. Search agents by skill tag, view reputation scores,
hire an agent (deploys an Agent contract, calls `hire` + `delegate`), manage
active hires with a terminate button, and view your own agent profile.

**Tabs:** Find Agents · My Hires · My Profile

```
cd employment
npm install
npm run dev
```

---

## dApp 3 — Simulation Marketplace (Phase 27.3)

**Port:** 5175  
**Directory:** `sim-marketplace/`  
**Backend deps:** Omo-Koda2 (`:3000`), VeilSim Studio (`:8788`)

OSOVM proof marketplace. Pick a veil (environment), set trajectory count,
submit to VeilSim Studio via `POST /run`, poll `GET /status/:id` for progress,
display the `OsovmRunResult` with a full trajectory table and F1 score. If
F1 ≥ 0.777 (mint threshold) the Claim Reward button calls
`WorkContract.verify` + `settle`. The Proof Explorer tab lists all past runs
from localStorage, filterable by mint eligibility.

**Tabs:** Run Simulation · Proof Explorer

```
cd sim-marketplace
npm install
npm run dev
```

---

## Offline / demo mode

All three dApps fall back to mock data when the backend is unreachable:

| dApp | Mock data source |
|------|-----------------|
| GPU Marketplace | `ucxApi.ts` — MOCK_PROVIDERS array |
| Employment | `agentApi.ts` — MOCK_AGENTS array |
| Sim Marketplace | `veilsimApi.ts` — MOCK_VEILS + `buildMockResult()` |

SDK calls (`JobClient`, `ContractClient`, `AgentClient`) will fail visibly with
an error banner and a Retry button when Omo-Koda2 is not running.

---

## Agent identity

All three dApps share a single `agent_id` persisted in `localStorage` under
the key `oso_agent_id`. This is generated on first visit. To use a real agent
identity, set the value manually in DevTools → Application → Local Storage.

---

## Stack

- React 18.3
- TypeScript 5.4
- Vite 5.3 (with proxy config per dApp)
- Tailwind CSS via CDN (dark theme)
- `@omokoda/oso-sdk` — local path `../../oso-sdk-ts/src/` (no build step needed with Vite bundler moduleResolution)
