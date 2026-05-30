'use client'

import { useState } from 'react'
import { useAgentStore } from '@/lib/store/agent'

const STEPS = ['Identity', 'Signature', 'Finalize']

export default function GatePage() {
  const [step, setStep] = useState(0)
  const [agentName, setAgentName] = useState('')
  const { setActiveAgent, setTier } = useAgentStore()

  function handleBirth() {
    if (!agentName.trim()) return
    setActiveAgent(agentName.trim())
    setTier(1)
    setStep(2)
  }

  return (
    <div className="p-8 max-w-2xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-gate">◈</span>
          <h1 className="text-2xl font-bold text-white">The Gate</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Identity &amp; Birth — every agent begins here at the threshold.
        </p>
      </div>

      {/* Step indicator */}
      <div className="flex items-center gap-2 mb-8">
        {STEPS.map((s, i) => (
          <div key={s} className="flex items-center gap-2">
            <div
              className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-mono font-bold border ${
                i === step
                  ? 'bg-gate border-gate text-black'
                  : i < step
                    ? 'bg-surface-4 border-gate text-gate'
                    : 'bg-surface-3 border-surface-5 text-zinc-600'
              }`}
            >
              {i < step ? '✓' : i + 1}
            </div>
            <span
              className={`text-sm ${i === step ? 'text-white' : 'text-zinc-600'}`}
            >
              {s}
            </span>
            {i < STEPS.length - 1 && (
              <div className="w-8 h-px bg-surface-4 mx-1" />
            )}
          </div>
        ))}
      </div>

      {/* Step content */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
        {step === 0 && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold text-white">Name your agent</h2>
            <p className="text-zinc-500 text-sm">
              Choose a unique name. This becomes the public identity of your sovereign agent.
            </p>
            <input
              type="text"
              value={agentName}
              onChange={(e) => setAgentName(e.target.value)}
              placeholder="e.g. Alpha-7, Ade, Oracle-1"
              className="w-full bg-surface-3 border border-border-DEFAULT rounded-lg px-4 py-3 text-white font-mono placeholder-zinc-600 outline-none focus:border-gate transition-colors"
              onKeyDown={(e) => e.key === 'Enter' && agentName && setStep(1)}
            />
            <button
              onClick={() => agentName.trim() && setStep(1)}
              disabled={!agentName.trim()}
              className="w-full py-3 rounded-lg bg-gate text-black font-bold hover:bg-brand-400 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
            >
              Continue →
            </button>
          </div>
        )}

        {step === 1 && (
          <div className="space-y-4">
            <h2 className="text-lg font-semibold text-white">Assign signature</h2>
            <p className="text-zinc-500 text-sm">
              A deterministic signature is derived from your agent name and entropy.
              This is your agent&apos;s immutable identity fingerprint.
            </p>
            <div className="font-mono text-xs bg-surface-3 rounded-lg p-4 text-zinc-400 break-all">
              <div className="text-zinc-600 mb-1">Agent Name</div>
              <div className="text-gate mb-3">{agentName}</div>
              <div className="text-zinc-600 mb-1">Signature (simulated)</div>
              <div className="text-green-400">
                {Array.from({ length: 32 }, (_, i) =>
                  ((agentName.charCodeAt(i % agentName.length) + i * 7) % 256)
                    .toString(16)
                    .padStart(2, '0')
                ).join('')}
              </div>
            </div>
            <div className="flex gap-3">
              <button
                onClick={() => setStep(0)}
                className="flex-1 py-3 rounded-lg border border-border-DEFAULT text-zinc-400 hover:text-white hover:border-border-strong transition-colors"
              >
                ← Back
              </button>
              <button
                onClick={handleBirth}
                className="flex-1 py-3 rounded-lg bg-gate text-black font-bold hover:bg-brand-400 transition-colors"
              >
                Birth Agent →
              </button>
            </div>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4 text-center">
            <div className="text-5xl mb-4">◈</div>
            <h2 className="text-xl font-bold text-white">Agent Born</h2>
            <p className="text-zinc-400">
              <span className="text-gate font-mono">{agentName}</span> is now active at Tier 1.
            </p>
            <div className="bg-surface-3 rounded-lg p-4 text-left space-y-2 font-mono text-sm">
              <div className="flex justify-between">
                <span className="text-zinc-600">Agent</span>
                <span className="text-gate">{agentName}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-600">Tier</span>
                <span className="text-yellow-400">★ Level 1</span>
              </div>
              <div className="flex justify-between">
                <span className="text-zinc-600">Status</span>
                <span className="text-green-400">● Active</span>
              </div>
            </div>
            <button
              onClick={() => { setStep(0); setAgentName('') }}
              className="px-6 py-2.5 rounded-lg border border-border-DEFAULT text-zinc-400 hover:text-white hover:border-border-strong transition-colors text-sm"
            >
              Birth Another
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
