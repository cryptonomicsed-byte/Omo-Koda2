'use client'

import { useEffect, useState, useCallback } from 'react'
import { meshGetAgents, meshGetEvents, MeshAgent, MeshEvent } from '@/lib/api'

const REFRESH_MS = 30_000

function trustColor(score: number): string {
  if (score >= 70) return 'text-green-400'
  if (score >= 40) return 'text-yellow-400'
  return 'text-red-400'
}

function AgentCard({ agent }: { agent: MeshAgent }) {
  const total = agent.commitments_kept + agent.commitments_broken
  const rate = total > 0 ? Math.round((agent.commitments_kept / total) * 100) : null
  return (
    <div className="rounded-xl border border-border-DEFAULT bg-surface-2 p-4 flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <span className="font-mono text-sm text-brand-400 truncate">{agent.agent_id}</span>
        <span className={`text-lg font-bold ${trustColor(agent.trust_score)}`}>
          {agent.trust_score.toFixed(1)}
        </span>
      </div>
      <div className="flex items-center gap-3 text-xs text-zinc-500">
        <span className="capitalize">{agent.role}</span>
        {rate !== null && <span>&#9889; {rate}% fulfillment</span>}
        <span>&#10003; {agent.commitments_kept}</span>
        <span>&#10007; {agent.commitments_broken}</span>
      </div>
    </div>
  )
}

function EventRow({ event }: { event: MeshEvent }) {
  const time = new Date(event.created_at).toLocaleTimeString()
  return (
    <div className="flex items-start gap-3 py-2 border-b border-border-DEFAULT last:border-0">
      <span className="text-xs text-zinc-600 shrink-0 font-mono">{time}</span>
      <span className="text-xs text-brand-400 shrink-0 truncate max-w-[140px]">{event.actor_id}</span>
      <span className="text-xs text-zinc-300">{event.event_type}</span>
    </div>
  )
}

export default function MeshPage() {
  const [blockId, setBlockId] = useState('default')
  const [agents, setAgents] = useState<MeshAgent[]>([])
  const [events, setEvents] = useState<MeshEvent[]>([])
  const [loading, setLoading] = useState(false)
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null)
  const [vantageConfigured, setVantageConfigured] = useState(true)

  const refresh = useCallback(async () => {
    setLoading(true)
    try {
      const [a, e] = await Promise.all([
        meshGetAgents(blockId),
        meshGetEvents(blockId),
      ])
      if (a.length === 0 && e.length === 0) setVantageConfigured(false)
      else setVantageConfigured(true)
      setAgents(a)
      setEvents(e)
      setLastUpdated(new Date())
    } finally {
      setLoading(false)
    }
  }, [blockId])

  useEffect(() => {
    refresh()
    const id = setInterval(refresh, REFRESH_MS)
    return () => clearInterval(id)
  }, [refresh])

  return (
    <main className="min-h-screen bg-surface-1 p-6">
      <div className="max-w-5xl mx-auto flex flex-col gap-6">

        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white">Block Mesh</h1>
            <p className="text-sm text-zinc-500 mt-1">
              Sovereign agent coordination fabric
            </p>
          </div>
          <div className="flex items-center gap-3">
            <input
              value={blockId}
              onChange={e => setBlockId(e.target.value)}
              placeholder="block id"
              className="rounded border border-border-DEFAULT bg-surface-2 px-3 py-1.5 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-brand-400"
            />
            <button
              onClick={refresh}
              disabled={loading}
              className="rounded border border-border-DEFAULT bg-surface-2 px-3 py-1.5 text-sm text-zinc-400 hover:text-white disabled:opacity-50"
            >
              {loading ? '…' : '↺'}
            </button>
          </div>
        </div>

        {/* Vantage not configured warning */}
        {!vantageConfigured && (
          <div className="rounded-xl border border-yellow-500/30 bg-yellow-500/10 p-4 text-sm text-yellow-300">
            Set <code className="font-mono">NEXT_PUBLIC_VANTAGE_URL</code> to connect to the Vantage coordination backend.
          </div>
        )}

        {/* Stats bar */}
        <div className="grid grid-cols-3 gap-4">
          {[
            { label: 'Agents', value: agents.length },
            { label: 'Avg Trust', value: agents.length > 0 ? (agents.reduce((s, a) => s + a.trust_score, 0) / agents.length).toFixed(1) : '—' },
            { label: 'Events', value: events.length },
          ].map(({ label, value }) => (
            <div key={label} className="rounded-xl border border-border-DEFAULT bg-surface-2 p-4 text-center">
              <div className="text-2xl font-bold text-brand-400">{value}</div>
              <div className="text-xs text-zinc-500 mt-1">{label}</div>
            </div>
          ))}
        </div>

        {/* Agents grid */}
        <section>
          <h2 className="text-sm font-semibold text-zinc-400 uppercase tracking-wider mb-3">
            Active Agents
          </h2>
          {agents.length === 0 ? (
            <div className="text-sm text-zinc-600">No agents on this block yet.</div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {agents.map(a => <AgentCard key={a.agent_id} agent={a} />)}
            </div>
          )}
        </section>

        {/* Events feed */}
        <section>
          <h2 className="text-sm font-semibold text-zinc-400 uppercase tracking-wider mb-3">
            Recent Events
          </h2>
          <div className="rounded-xl border border-border-DEFAULT bg-surface-2 p-4">
            {events.length === 0 ? (
              <p className="text-sm text-zinc-600">No events yet.</p>
            ) : (
              events.map(e => <EventRow key={e.id} event={e} />)
            )}
          </div>
        </section>

        {/* Footer */}
        {lastUpdated && (
          <p className="text-xs text-zinc-600 text-right">
            Updated {lastUpdated.toLocaleTimeString()} &middot; auto-refresh every 30s
          </p>
        )}
      </div>
    </main>
  )
}
