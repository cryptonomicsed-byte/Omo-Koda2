'use client'

const RECEIPTS = [
  { id: '0x7a3f...', tool: 'web_search', tokens: 12, time: '14:32', status: 'verified' },
  { id: '0x1b2c...', tool: 'read_file', tokens: 3, time: '14:31', status: 'verified' },
  { id: '0x9d4e...', tool: 'execute_bash', tokens: 28, time: '14:30', status: 'pending' },
]

const LEADERBOARD = [
  { rank: 1, agent: 'Alpha-7', rep: 847, tier: 3 },
  { rank: 2, agent: 'Oracle-5', rep: 623, tier: 2 },
  { rank: 3, agent: 'Beta-2', rep: 412, tier: 2 },
  { rank: 4, agent: 'Gamma-1', rep: 389, tier: 1 },
  { rank: 5, agent: 'Delta-3', rep: 201, tier: 1 },
]

export default function ThunderPage() {
  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-thunder">⚡</span>
          <h1 className="text-2xl font-bold text-white">The Thunder</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Economy &amp; Justice — receipts, reputation, and token flow.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        {/* Token stats */}
        <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
          <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
            Token Flow
          </h2>
          <div className="space-y-3">
            {[
              { label: 'Input Tokens', value: '8,240', color: 'text-mirror' },
              { label: 'Output Tokens', value: '3,120', color: 'text-storm' },
              { label: 'Cache Tokens', value: '1,640', color: 'text-ocean' },
              { label: 'Total Cost', value: '$0.04', color: 'text-thunder' },
            ].map(({ label, value, color }) => (
              <div key={label} className="flex items-center justify-between">
                <span className="text-sm text-zinc-500">{label}</span>
                <span className={`text-sm font-mono font-bold ${color}`}>{value}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Reputation */}
        <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
          <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
            My Reputation
          </h2>
          <div className="text-5xl font-mono font-bold text-thunder text-center py-4">847</div>
          <div className="text-center text-xs text-zinc-600">
            Rank #1 · Tier 3 · ★★★☆☆☆☆
          </div>
          <div className="mt-4 h-1.5 bg-surface-4 rounded-full overflow-hidden">
            <div className="h-full bg-thunder rounded-full" style={{ width: '42%' }} />
          </div>
          <div className="flex justify-between text-xs text-zinc-600 mt-1">
            <span>Tier 3</span>
            <span>847 / 2000 → Tier 4</span>
          </div>
        </div>
      </div>

      {/* Leaderboard */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Leaderboard
        </h2>
        <div className="space-y-2">
          {LEADERBOARD.map(({ rank, agent, rep, tier }) => (
            <div
              key={agent}
              className="flex items-center gap-4 p-3 rounded-lg bg-surface-3"
            >
              <span className="text-sm font-mono text-zinc-600 w-4 text-right">{rank}</span>
              <span className="text-sm text-zinc-300 flex-1 font-mono">{agent}</span>
              <span className="text-xs text-yellow-400">{'★'.repeat(tier)}{'☆'.repeat(7 - tier)}</span>
              <span className="text-sm font-mono text-thunder font-bold">{rep}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Receipt chain */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl overflow-hidden">
        <div className="grid grid-cols-4 gap-4 px-4 py-2 border-b border-border-DEFAULT text-xs font-mono text-zinc-600 uppercase">
          <span>Receipt</span>
          <span>Tool</span>
          <span>Tokens</span>
          <span>Status</span>
        </div>
        {RECEIPTS.map(({ id, tool, tokens, time, status }) => (
          <div
            key={id}
            className="grid grid-cols-4 gap-4 px-4 py-3 border-b border-border-subtle text-sm font-mono hover:bg-surface-3 transition-colors"
          >
            <span className="text-zinc-400 truncate">{id}</span>
            <span className="text-zinc-300">{tool}</span>
            <span className="text-zinc-400">{tokens}</span>
            <span className={status === 'verified' ? 'text-green-400' : 'text-yellow-400'}>
              {status === 'verified' ? '✓ verified' : '⏳ pending'}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}
