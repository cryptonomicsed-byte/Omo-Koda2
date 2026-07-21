'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'

interface Status {
  has_agent: boolean
  name?: string
  id?: string
  tier?: number
  reputation?: number
  synapse?: number
}
interface VaultConfig {
  access?: string
  auto_export?: boolean
}

export default function SettingsPage() {
  const [status, setStatus] = useState<Status | null>(null)
  const [vault, setVault] = useState<VaultConfig | null>(null)
  const [msg, setMsg] = useState<string | null>(null)

  useEffect(() => {
    fetch('/v1/status').then((r) => (r.ok ? r.json() : null)).then(setStatus).catch(() => {})
    fetch('/v1/vault/config').then((r) => (r.ok ? r.json() : null)).then(setVault).catch(() => {})
  }, [])

  async function setAccess(access: string) {
    setMsg(null)
    const r = await fetch('/v1/vault/config', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ access, auto_export: vault?.auto_export ?? false }),
    })
    if (r.ok) { setVault(await r.json()); setMsg('Saved.') } else { setMsg(`Error: ${r.status}`) }
  }

  return (
    <div className="p-8 max-w-2xl mx-auto text-zinc-200">
      <div className="flex items-center gap-3 mb-6">
        <Link href="/" className="text-brand-400 hover:underline text-sm">← Back</Link>
        <h1 className="text-2xl font-bold">⊞ Settings</h1>
      </div>

      <section className="mb-8 rounded-lg bg-white/5 border border-white/10 p-5">
        <h2 className="text-sm uppercase tracking-widest text-zinc-400 mb-3">Agent</h2>
        {status?.has_agent ? (
          <dl className="grid grid-cols-2 gap-y-2 text-sm">
            <dt className="text-zinc-400">Name</dt><dd>{status.name}</dd>
            <dt className="text-zinc-400">ID</dt><dd className="font-mono text-xs break-all">{status.id}</dd>
            <dt className="text-zinc-400">Tier</dt><dd>{status.tier}</dd>
            <dt className="text-zinc-400">Reputation</dt><dd>{status.reputation?.toFixed(2)}</dd>
            <dt className="text-zinc-400">Synapse</dt><dd>{status.synapse?.toFixed(2)}</dd>
          </dl>
        ) : (
          <p className="text-zinc-400 text-sm">{status ? 'No agent born yet.' : 'Loading…'}</p>
        )}
      </section>

      <section className="rounded-lg bg-white/5 border border-white/10 p-5">
        <h2 className="text-sm uppercase tracking-widest text-zinc-400 mb-3">Memory Vault Access</h2>
        <div className="flex flex-wrap gap-2 mb-2">
          {['private', 'followers', 'federated', 'public'].map((a) => (
            <button
              key={a}
              onClick={() => setAccess(a)}
              className={`px-3 py-1.5 rounded-md text-sm border transition-colors ${
                vault?.access === a
                  ? 'bg-purple-900/40 border-purple-600 text-purple-200'
                  : 'bg-surface-3 border-white/10 text-zinc-400 hover:text-zinc-200'
              }`}
            >
              {a}
            </button>
          ))}
        </div>
        <p className="text-xs text-zinc-500">Current: {vault?.access ?? '…'}{msg ? ` — ${msg}` : ''}</p>
      </section>
    </div>
  )
}
