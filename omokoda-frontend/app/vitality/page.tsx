'use client'

import { useState, useEffect } from 'react'
import { rhythmToday } from '@/lib/api'

const PLUGINS = [
  { name: 'bipon39', version: '0.1.1', status: 'active', desc: 'Mnemonic identity engine' },
  { name: 'ifascript', version: '0.2.0', status: 'active', desc: 'Odu divination VM' },
  { name: 'nist-entropy', version: '0.1.0', status: 'active', desc: 'NIST-compliant entropy' },
  { name: 'nautilus', version: '0.1.0', status: 'inactive', desc: 'Navigation integration' },
  { name: 'wallet-kit', version: '0.2.0', status: 'active', desc: 'Sui wallet integration' },
]

const NODES = [
  { id: 'node-01', region: 'us-east', uptime: '99.8%', latency: '12ms' },
  { id: 'node-02', region: 'eu-west', uptime: '99.2%', latency: '34ms' },
  { id: 'node-03', region: 'ap-south', uptime: '97.4%', latency: '89ms' },
]

interface Facet { id: number; name: string; value: string }

interface Resonance {
  day?: string
  yoruba_name?: string
  archetype?: string
  principle?: string
  frequency?: string
  color?: string
  facets?: Facet[]
  ritual_practice?: { mantra?: string; dress?: string; food?: string[] }
}

export default function VitalityPage() {
  const [resonance, setResonance] = useState<Resonance | null>(null)
  const [resonanceError, setResonanceError] = useState<string | null>(null)

  useEffect(() => {
    rhythmToday()
      .then((r) => setResonance(r as Resonance))
      .catch((e) => setResonanceError(e.message))
  }, [])

  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-brand-400">◉</span>
          <h1 className="text-2xl font-bold text-white">The Vitality</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Plugins · Nodes · Daily Kóòdù Resonance
        </p>
      </div>

      {/* Daily Resonance (Kóòdù) */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Today&apos;s Resonance
        </h2>

        {resonanceError && (
          <p className="text-xs text-red-400">{resonanceError}</p>
        )}

        {!resonance && !resonanceError && (
          <p className="text-xs text-zinc-600">Loading…</p>
        )}

        {resonance && (
          <div className="space-y-4">
            <div className="flex flex-wrap gap-3">
              {[
                { label: 'Day', value: resonance.yoruba_name ?? resonance.day },
                { label: 'Archetype', value: resonance.archetype },
                { label: 'Principle', value: resonance.principle },
                { label: 'Frequency', value: resonance.frequency },
                { label: 'Color', value: resonance.color },
              ].filter(x => x.value).map(({ label, value }) => (
                <div key={label} className="bg-surface-3 rounded-lg px-3 py-2 text-center min-w-[100px]">
                  <div className="text-xs text-zinc-500 mb-0.5">{label}</div>
                  <div className="text-sm text-zinc-200 font-medium">{value}</div>
                </div>
              ))}
            </div>

            {resonance.ritual_practice?.mantra && (
              <div className="bg-surface-3 rounded-lg p-4 border-l-2 border-brand-600">
                <div className="text-xs text-zinc-500 mb-1">Mantra</div>
                <div className="text-sm text-zinc-200 italic">
                  &ldquo;{resonance.ritual_practice.mantra}&rdquo;
                </div>
              </div>
            )}

            {resonance.facets && resonance.facets.length > 0 && (
              <details className="group">
                <summary className="text-xs text-zinc-500 cursor-pointer hover:text-zinc-300 select-none">
                  Show all {resonance.facets.length} facets ▾
                </summary>
                <div className="mt-3 grid grid-cols-1 gap-1.5">
                  {resonance.facets.map((f) => (
                    <div key={f.id} className="flex gap-2 text-xs">
                      <span className="text-zinc-600 w-4 text-right flex-shrink-0">{f.id}</span>
                      <span className="text-zinc-500 w-36 flex-shrink-0">{f.name}</span>
                      <span className="text-zinc-300">{f.value}</span>
                    </div>
                  ))}
                </div>
              </details>
            )}
          </div>
        )}
      </div>

      {/* Plugin browser */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Installed Plugins
        </h2>
        <div className="space-y-2">
          {PLUGINS.map(({ name, version, status, desc }) => (
            <div key={name} className="flex items-center gap-4 p-3 rounded-lg bg-surface-3">
              <span className={`w-2 h-2 rounded-full flex-shrink-0 ${status === 'active' ? 'bg-green-400' : 'bg-zinc-600'}`} />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-mono text-white">{name}</span>
                  <span className="text-xs text-zinc-600">v{version}</span>
                </div>
                <p className="text-xs text-zinc-500 mt-0.5">{desc}</p>
              </div>
              <span className={`text-xs ${status === 'active' ? 'text-green-400' : 'text-zinc-600'}`}>{status}</span>
            </div>
          ))}
        </div>
      </div>

      {/* DePIN nodes */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          DePIN Nodes
        </h2>
        <div className="space-y-3">
          {NODES.map(({ id, region, uptime, latency }) => (
            <div key={id} className="flex items-center gap-4 p-3 rounded-lg bg-surface-3">
              <span className="w-2 h-2 rounded-full bg-green-400" />
              <span className="text-sm font-mono text-zinc-300 flex-1">{id}</span>
              <span className="text-xs text-zinc-500">{region}</span>
              <span className="text-xs font-mono text-green-400">{uptime}</span>
              <span className="text-xs font-mono text-zinc-400">{latency}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
