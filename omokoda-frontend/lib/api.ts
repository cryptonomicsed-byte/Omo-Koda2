const BASE = process.env.NEXT_PUBLIC_OMOKODA_URL || 'http://localhost:7400'

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

export async function vaultListFiles(): Promise<VaultFileEntry[]> {
  const r = await fetch(`${BASE}/v1/vault/files`)
  if (!r.ok) throw new Error(`vault/files: ${r.status}`)
  return r.json()
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
