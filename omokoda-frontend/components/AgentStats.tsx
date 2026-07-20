'use client'

import { useEffect, useState } from 'react'

interface Status {
  has_agent: boolean
  name?: string
  reputation?: number
  tier?: number
  synapse?: number
  onchain_nft_id?: string | null
}
interface VaultStatus {
  enabled?: boolean
  config?: { access?: string }
}

export function AgentStats() {
  const [status, setStatus] = useState<Status | null>(null)
  const [vault, setVault] = useState<VaultStatus | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let alive = true
    async function load() {
      try {
        const [s, v] = await Promise.all([
          fetch('/v1/status').then((r) => (r.ok ? r.json() : Promise.reject(r.status))),
          fetch('/v1/vault').then((r) => (r.ok ? r.json() : null)).catch(() => null),
        ])
        if (!alive) return
        setStatus(s)
        setVault(v)
      } catch (e) {
        if (alive) setError(`could not load status (${e})`)
      }
    }
    load()
    const t = setInterval(load, 10_000)
    return () => { alive = false; clearInterval(t) }
  }, [])

  const fmt = (n?: number, d = 2) => (typeof n === 'number' ? n.toFixed(d) : '—')
  const stats = [
    { label: 'Agent', value: status?.name ?? (status ? 'unborn' : '…') },
    { label: 'Reputation', value: fmt(status?.reputation) },
    { label: 'Tier', value: status?.tier != null ? String(status.tier) : '—' },
    { label: 'Synapse', value: fmt(status?.synapse) },
    { label: 'Vault', value: vault?.enabled ? (vault.config?.access ?? 'on') : 'off' },
    { label: 'On-chain NFT', value: status?.onchain_nft_id ? 'minted' : 'none' },
  ]

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold text-white">Agent Statistics</h2>
      {error && <p className="text-sm text-red-300">{error}</p>}
      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
        {stats.map((stat) => (
          <div key={stat.label} className="rounded-3xl bg-white/10 p-5 border border-white/10 shadow-sm">
            <p className="text-xs uppercase tracking-[0.2em] text-gray-400">{stat.label}</p>
            <p className="mt-3 text-2xl font-semibold text-white break-words">{stat.value}</p>
          </div>
        ))}
      </div>
    </div>
  )
}
