# Agent Capabilities — Wallets

Ọ̀mọ̀ Kọ́dà agents can create and manage wallets through the Vantage
agent-first wallet API. Wallets are exposed as ordinary kernel **tools**
(tier-gated, permission-gated, receipt-producing) — the same surface as every
other capability. There is no separate agent framework or skill DSL.

## Configuration

Wallet tools call Vantage and reuse the same client the mesh tools use:

- `VANTAGE_URL` — base URL of the Vantage backend (e.g. `http://localhost:8001`).
- `X-Agent-Key` — the agent's Vantage auth token. Provisioned automatically:
  every agent is registered on Vantage at **birth** (`register_newborn`), which
  mints and holds the key; it is attached to every wallet call. `VANTAGE_KEY`
  may pre-provision it instead.

If `VANTAGE_URL` is unset, wallet tools **fail closed** with a clear error
(no local fallback — there is no safe local place to hold keys), unlike the
`mesh_*` tools which fall back to an in-memory router.

## Tools

| Tool | Tier | Write? | Vantage endpoint |
|------|------|--------|------------------|
| `wallet_list` | 0 | no | `GET /api/agents/{id}/wallets` |
| `wallet_get` | 0 | no | `GET /api/agents/{id}/wallets/{wallet_id}` |
| `wallet_create` | 2 | yes | `POST /api/agents/{id}/wallets` |
| `wallet_sign` | 3 | yes | `POST /api/agents/{id}/wallets/{wallet_id}/sign` |
| `wallet_alchemy_approve` | 3 | yes | `POST /api/agents/{id}/wallets/{wallet_id}/alchemy/approve` |

`{id}` is always the calling agent's own id (from `ExecutionContext`); an agent
can only act on its own wallets. Write tools validate their params against a
strict JSON schema before the call is made.

### Params

- `wallet_get` — `{ "wallet_id": "..." }`
- `wallet_create` — `{ "type": "custom" | "alchemy", "name": "..." }`
- `wallet_sign` — `{ "wallet_id": "...", "transaction": { … }, "intent": "trade_order" }`
- `wallet_alchemy_approve` — `{ "wallet_id": "...", "capabilities": ["..."] }`
- `wallet_list` — no params

## Tiering rationale

- **Reads (`list`/`get`) are tier 0** and classified read-only, so they work over
  the HTTP `/v1/act` path for capability discovery (Axiom).
- **`create` is tier 2** — opening a financial account is a real commitment.
- **`sign` / `alchemy_approve` are tier 3** — signing moves funds and is
  irreversible, so it requires an established agent and is a write operation
  (gated by permission mode; denied over HTTP unless escalation is allowed).

## Security model

- Private keys are encrypted **server-side** and never exposed to the agent.
- Alchemy session tokens are held **server-side**.
- Signing happens **server-side**; the agent supplies only `intent` + its
  `X-Agent-Key`.
- Vantage keeps a full audit trail (intent + timestamps) for every operation.

## Example flow

```
Agent decides it needs to trade
 → act wallet_create { "type": "alchemy", "name": "trading" }
     ← { wallet_id, address, approval_url, session_status: "pending_approval" }
 → (human approves the Alchemy session via the dashboard)
 → act wallet_sign { "wallet_id": "...", "transaction": {…}, "intent": "trade_order" }
     ← { signed_tx, intent, timestamp }   # ready to broadcast
```
