'use client'

import { useState } from 'react'

const TOOLS = [
  { name: 'web_search', tier: 1, desc: 'Search the web for information', status: 'available' },
  { name: 'read_file', tier: 1, desc: 'Read files from the filesystem', status: 'available' },
  { name: 'write_file', tier: 2, desc: 'Write files to the filesystem', status: 'available' },
  { name: 'execute_bash', tier: 2, desc: 'Run bash commands in sandbox', status: 'available' },
  { name: 'fetch_url', tier: 1, desc: 'Fetch content from a URL', status: 'available' },
  { name: 'spawn_agent', tier: 3, desc: 'Spawn a child agent', status: 'locked' },
  { name: 'deploy_contract', tier: 5, desc: 'Deploy a smart contract', status: 'locked' },
]

const LOG = [
  { ts: '14:32:01', tool: 'web_search', status: 'ok', ms: 245, tokens: 12 },
  { ts: '14:31:45', tool: 'read_file', status: 'ok', ms: 18, tokens: 3 },
  { ts: '14:30:12', tool: 'execute_bash', status: 'ok', ms: 523, tokens: 28 },
  { ts: '14:28:55', tool: 'fetch_url', status: 'error', ms: 5001, tokens: 5 },
]

export default function ForgePage() {
  const [tab, setTab] = useState<'catalog' | 'log'>('catalog')

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-forge">⊕</span>
          <h1 className="text-2xl font-bold text-white">The Forge</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Tools &amp; Execution — every act is forged here.
        </p>
      </div>

      {/* Tab switcher */}
      <div className="flex gap-1 bg-surface-2 border border-border-DEFAULT rounded-lg p-1 w-fit mb-6">
        {(['catalog', 'log'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors capitalize ${
              tab === t ? 'bg-forge text-black' : 'text-zinc-500 hover:text-white'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {tab === 'catalog' && (
        <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
          <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
            Tool Catalog
          </h2>
          <div className="space-y-2">
            {TOOLS.map(({ name, tier, desc, status }) => (
              <div
                key={name}
                className={`flex items-center gap-4 p-3 rounded-lg ${
                  status === 'locked' ? 'opacity-50' : 'bg-surface-3'
                }`}
              >
                <span className={`text-sm font-mono font-bold ${status === 'locked' ? 'text-zinc-600' : 'text-forge'}`}>
                  ⚙
                </span>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-mono text-white">{name}</span>
                    <span className="text-xs text-zinc-600">Tier {tier}</span>
                    {status === 'locked' && (
                      <span className="text-xs text-zinc-600 bg-surface-4 px-1.5 py-0.5 rounded">🔒 locked</span>
                    )}
                  </div>
                  <p className="text-xs text-zinc-500 mt-0.5">{desc}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === 'log' && (
        <div className="bg-surface-2 border border-border-DEFAULT rounded-xl overflow-hidden">
          <div className="grid grid-cols-5 gap-4 px-4 py-2 border-b border-border-DEFAULT text-xs font-mono text-zinc-600 uppercase">
            <span>Time</span>
            <span>Tool</span>
            <span>Status</span>
            <span>Duration</span>
            <span>Tokens</span>
          </div>
          {LOG.map(({ ts, tool, status, ms, tokens }, i) => (
            <div
              key={i}
              className="grid grid-cols-5 gap-4 px-4 py-3 border-b border-border-subtle text-sm font-mono hover:bg-surface-3 transition-colors"
            >
              <span className="text-zinc-600">{ts}</span>
              <span className="text-zinc-300">{tool}</span>
              <span className={status === 'ok' ? 'text-green-400' : 'text-red-400'}>
                {status === 'ok' ? '✓ ok' : '✗ err'}
              </span>
              <span className="text-zinc-400">{ms}ms</span>
              <span className="text-zinc-400">{tokens}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
