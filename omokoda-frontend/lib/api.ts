const BASE = process.env.NEXT_PUBLIC_OMOKODA_URL || 'http://localhost:7777'

// ─── Legacy ────────────────────────────────────────────────────────────────

export async function fetchAgentResponse(message: string) {
  const response = await fetch('/api/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message }),
  })
  if (!response.ok) throw new Error('Failed to fetch agent response')
  return response.json()
}

// ─── Vault ─────────────────────────────────────────────────────────────────

export interface VaultFileEntry {
  path: string
  size_bytes: number
  modified_secs: number
}

export interface KnowledgeTriple {
  subject: string
  predicate: string
  object: string
  confidence?: number
}

export interface GalaxyData {
  stars: { id: string; label: string; x: number; y: number; z: number; kind: string }[]
  edges: { from: string; to: string; label: string }[]
  nebulae: string[]
}

export interface VaultConfig {
  access_level: string
  auto_export: boolean
}

export async function vaultListFiles(dir = '.'): Promise<VaultFileEntry[]> {
  // The kernel exposes vault listing at /v1/vault/ls (there is no /vault/files),
  // returning { dir, files: [...] }.
  const r = await fetch(`${BASE}/v1/vault/ls?dir=${encodeURIComponent(dir)}`)
  if (!r.ok) throw new Error(`vault/ls: ${r.status}`)
  const data = await r.json()
  return Array.isArray(data?.files) ? data.files : []
}

export async function vaultReadFile(path: string): Promise<{ path: string; content: string }> {
  const r = await fetch(`${BASE}/v1/vault/file/${path}`)
  if (!r.ok) throw new Error(`vault/file: ${r.status}`)
  return r.json()
}

export async function vaultInsertKnowledge(triple: KnowledgeTriple): Promise<void> {
  const r = await fetch(`${BASE}/v1/vault/knowledge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(triple),
  })
  if (!r.ok) throw new Error(`vault/knowledge: ${r.status}`)
}

export async function vaultGalaxy(): Promise<GalaxyData> {
  const r = await fetch(`${BASE}/v1/vault/galaxy`)
  if (!r.ok) throw new Error(`vault/galaxy: ${r.status}`)
  return r.json()
}

export async function vaultSync(content: string): Promise<void> {
  const r = await fetch(`${BASE}/v1/vault/sync`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  })
  if (!r.ok) throw new Error(`vault/sync: ${r.status}`)
}

export async function vaultGetConfig(): Promise<VaultConfig> {
  const r = await fetch(`${BASE}/v1/vault/config`)
  if (!r.ok) throw new Error(`vault/config GET: ${r.status}`)
  return r.json()
}

export async function vaultSetConfig(cfg: VaultConfig): Promise<void> {
  const r = await fetch(`${BASE}/v1/vault/config`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(cfg),
  })
  if (!r.ok) throw new Error(`vault/config PUT: ${r.status}`)
}

// ─── Rhythm ────────────────────────────────────────────────────────────────

export async function rhythmToday(): Promise<Record<string, unknown>> {
  const r = await fetch(`${BASE}/v1/rhythm/today`)
  if (!r.ok) throw new Error(`rhythm/today: ${r.status}`)
  return r.json()
}

// ── Block Mesh ──────────────────────────────────────────────────────────

const VANTAGE = process.env.NEXT_PUBLIC_VANTAGE_URL || ''

export interface MeshAgent {
  agent_id: string
  role: string
  trust_score: number
  capabilities: Record<string, unknown>
  commitments_kept: number
  commitments_broken: number
}

export interface MeshEvent {
  id: number
  block_id: string
  actor_id: string
  event_type: string
  payload: Record<string, unknown>
  created_at: string
}

export interface MeshBlock {
  block_id: string
  agents: MeshAgent[]
  events: MeshEvent[]
}

export async function meshGetBlock(blockId: string): Promise<MeshBlock | null> {
  if (!VANTAGE) return null
  const r = await fetch(`${VANTAGE}/api/mesh/blocks/${encodeURIComponent(blockId)}`)
  if (!r.ok) return null
  return r.json()
}

export async function meshGetAgents(blockId: string): Promise<MeshAgent[]> {
  if (!VANTAGE) return []
  const r = await fetch(`${VANTAGE}/api/mesh/blocks/${encodeURIComponent(blockId)}/agents`)
  if (!r.ok) return []
  return r.json()
}

export async function meshGetEvents(blockId: string, limit = 20): Promise<MeshEvent[]> {
  if (!VANTAGE) return []
  const r = await fetch(`${VANTAGE}/api/mesh/blocks/${encodeURIComponent(blockId)}/events?limit=${limit}`)
  if (!r.ok) return []
  return r.json()
}

export async function meshGetTrust(agentId: string, blockId = 'default'): Promise<{trust_score: number} | null> {
  if (!VANTAGE) return null
  const r = await fetch(`${VANTAGE}/api/mesh/trust/${encodeURIComponent(agentId)}?block_id=${encodeURIComponent(blockId)}`)
  if (!r.ok) return null
  return r.json()
}

export async function meshSignalEvent(params: {
  block_id: string
  event_type: string
  details: Record<string, unknown>
}): Promise<void> {
  await fetch(`${BASE}/v1/act`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ tool: 'mesh_signal_event', params }),
  })
}

// ─── Birth (real kernel, not the WASM stub) ────────────────────────────────
// The wasmBridge in lib/wasm.ts is a client-only stub used for offline UX
// prototyping. A real onboarding must call the actual Rust kernel's
// /v1/birth (omokoda-core/src/server.rs), which performs real blake3+cowrie
// entropy, real BIPON39 mnemonic derivation, and real multi-chain wallet
// generation. Every non-owner birth here is a "guest" agent hosted on the
// kernel's guest pool -- see server.rs's AppState.guests -- addressed on
// every subsequent call via X-Agent-Id/X-Agent-Key.

export interface BirthMetaPair {
  key: string
  value: string
}

export interface BirthResult {
  agent_id: string
  agent_key: string
  receipt_id?: string
}

export async function birthAgent(name: string, meta: BirthMetaPair[] = []): Promise<BirthResult> {
  const r = await fetch(`${BASE}/v1/birth`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, meta }),
  })
  const data = await r.json().catch(() => ({}))
  if (!r.ok) throw new Error(data?.error || `birth: ${r.status}`)
  if (!data.agent_id || !data.agent_key) {
    throw new Error('birth succeeded but the kernel did not return agent_id/agent_key')
  }
  return data
}

export interface RevealedSeed {
  mnemonic: string
  sui_address: string
  eth_address?: string
  btc_address?: string
  sol_address?: string
  cosmos_address?: string
  aptos_address?: string
  nostr_address?: string
}

// Calls the one-shot /v1/reveal-seed endpoint. The kernel latches
// `revealed_seed` permanently on the agent after the first successful
// call -- a second call for the same agent_id will error, by design (see
// AgentCore::reveal_seed in interpreter.rs). Never retry this call after a
// success; the caller owns showing the mnemonic to the user exactly once.
export async function revealSeed(agentId: string, agentKey: string): Promise<RevealedSeed> {
  const r = await fetch(`${BASE}/v1/reveal-seed`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Agent-Id': agentId,
      'X-Agent-Key': agentKey,
    },
  })
  const data = await r.json().catch(() => ({}))
  if (!r.ok) throw new Error(data?.error || `reveal-seed: ${r.status}`)
  return data
}

export interface AgentStatus {
  has_agent: boolean
  name?: string
  id?: string
  reputation?: number
  tier?: number
  synapse?: number
  onchain_nft_id?: string
  sui_address?: string
}

export async function agentStatus(agentId: string, agentKey: string): Promise<AgentStatus> {
  const r = await fetch(`${BASE}/v1/status`, {
    headers: { 'X-Agent-Id': agentId, 'X-Agent-Key': agentKey },
  })
  if (!r.ok) throw new Error(`status: ${r.status}`)
  return r.json()
}

export interface ExecutionResponse {
  receipt_id?: string
  private_mode: boolean
  tool_output?: string
  error?: string
}

export async function agentThink(
  agentId: string,
  agentKey: string,
  prompt: string,
  opts: { private?: boolean } = {}
): Promise<ExecutionResponse> {
  const r = await fetch(`${BASE}/v1/think`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Agent-Id': agentId,
      'X-Agent-Key': agentKey,
    },
    body: JSON.stringify({ prompt, private: !!opts.private }),
  })
  const data = await r.json().catch(() => ({}))
  if (!r.ok) throw new Error(data?.error || `think: ${r.status}`)
  return data
}

export async function agentAct(
  agentId: string,
  agentKey: string,
  tool: string,
  params: string = '{}'
): Promise<ExecutionResponse> {
  const r = await fetch(`${BASE}/v1/act`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Agent-Id': agentId,
      'X-Agent-Key': agentKey,
    },
    body: JSON.stringify({ tool, params }),
  })
  const data = await r.json().catch(() => ({}))
  if (!r.ok) throw new Error(data?.error || `act: ${r.status}`)
  return data
}
