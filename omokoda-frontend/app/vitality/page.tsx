'use client'

const PLUGINS = [
  { name: 'bipon39-stub', version: '0.1.0', status: 'active', desc: 'Mnemonic identity engine' },
  { name: 'ifascript-stub', version: '0.1.0', status: 'active', desc: 'Odu divination VM' },
  { name: 'nist-entropy', version: '0.1.0', status: 'active', desc: 'NIST-compliant entropy' },
  { name: 'nautilus', version: '0.1.0', status: 'inactive', desc: 'Navigation integration' },
  { name: 'wallet-kit', version: '0.2.0', status: 'active', desc: 'Sui wallet integration' },
]

const NODES = [
  { id: 'node-01', region: 'us-east', uptime: '99.8%', latency: '12ms' },
  { id: 'node-02', region: 'eu-west', uptime: '99.2%', latency: '34ms' },
  { id: 'node-03', region: 'ap-south', uptime: '97.4%', latency: '89ms' },
]

export default function VitalityPage() {
  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-brand-400">◉</span>
          <h1 className="text-2xl font-bold text-white">The Vitality</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Plugins &amp; Nodes — the living infrastructure of the agent.
        </p>
      </div>

      {/* Plugin browser */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Installed Plugins
        </h2>
        <div className="space-y-2">
          {PLUGINS.map(({ name, version, status, desc }) => (
            <div
              key={name}
              className="flex items-center gap-4 p-3 rounded-lg bg-surface-3"
            >
              <span
                className={`w-2 h-2 rounded-full flex-shrink-0 ${
                  status === 'active' ? 'bg-green-400' : 'bg-zinc-600'
                }`}
              />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-mono text-white">{name}</span>
                  <span className="text-xs text-zinc-600">v{version}</span>
                </div>
                <p className="text-xs text-zinc-500 mt-0.5">{desc}</p>
              </div>
              <span className={`text-xs ${status === 'active' ? 'text-green-400' : 'text-zinc-600'}`}>
                {status}
              </span>
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
            <div
              key={id}
              className="flex items-center gap-4 p-3 rounded-lg bg-surface-3"
            >
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
