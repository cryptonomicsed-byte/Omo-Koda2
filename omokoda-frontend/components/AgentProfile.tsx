'use client'

import { useEffect, useState } from 'react'

interface Status {
  has_agent: boolean
  name?: string
  id?: string
  reputation?: number
  tier?: number
  synapse?: number
  onchain_nft_id?: string | null
}

const tierColor = (tier: number) =>
  ['text-gray-400', 'text-green-400', 'text-blue-400', 'text-purple-400', 'text-yellow-400'][tier] ?? 'text-red-400'
const tierName = (tier: number) =>
  ['Initiate', 'Apprentice', 'Adept', 'Master', 'Oracle'][tier] ?? `Tier ${tier}`

export function AgentProfile() {
  const [agent, setAgent] = useState<Status | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let alive = true
    fetch('/v1/status')
      .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
      .then((s: Status) => { if (alive) setAgent(s) })
      .catch((e) => { if (alive) setError(`could not load profile (${e})`) })
    return () => { alive = false }
  }, [])

  if (error) return <p className="text-red-300">{error}</p>
  if (!agent) return <p className="text-gray-400">Loading agent…</p>
  if (!agent.has_agent) return <p className="text-gray-300">No agent has been born yet.</p>

  const tier = agent.tier ?? 0
  const rep = agent.reputation ?? 0

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-white">Agent Profile</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-gray-800/50 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Identity</h3>
          <div className="space-y-3">
            <div><label className="text-sm text-gray-400">Name</label><p className="text-white font-medium">{agent.name ?? '—'}</p></div>
            <div><label className="text-sm text-gray-400">ID</label><p className="text-white font-mono text-sm break-all">{agent.id ?? '—'}</p></div>
            <div><label className="text-sm text-gray-400">On-chain NFT</label><p className="text-white font-mono text-sm break-all">{agent.onchain_nft_id ?? 'unminted'}</p></div>
          </div>
        </div>
        <div className="bg-gray-800/50 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Standing</h3>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between items-center mb-1">
                <label className="text-sm text-gray-400">Reputation</label>
                <span className="text-white font-medium">{rep.toFixed(2)}</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div className="bg-gradient-to-r from-green-400 to-blue-500 h-2 rounded-full"
                  style={{ width: `${Math.min((rep / 100) * 100, 100)}%` }} />
              </div>
            </div>
            <div>
              <div className="flex justify-between items-center mb-1">
                <label className="text-sm text-gray-400">Tier</label>
                <span className={`font-medium ${tierColor(tier)}`}>{tierName(tier)} ({tier})</span>
              </div>
              <div className="flex space-x-1">
                {[0, 1, 2, 3, 4].map((t) => (
                  <div key={t} className={`h-3 flex-1 rounded ${t <= tier ? 'bg-gradient-to-r from-yellow-400 to-orange-500' : 'bg-gray-600'}`} />
                ))}
              </div>
            </div>
            <div>
              <label className="text-sm text-gray-400">Synapse</label>
              <p className="text-white font-medium">{typeof agent.synapse === 'number' ? agent.synapse.toFixed(2) : '—'}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
