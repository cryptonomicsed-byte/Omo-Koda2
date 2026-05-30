'use client'

const MEMORY_ENTRIES = [
  { id: 1, type: 'working', content: 'Last tool call: web_search("sui price")', time: '2m ago' },
  { id: 2, type: 'short', content: 'Agent born as Alpha-7, Tier 1', time: '1h ago' },
  { id: 3, type: 'long', content: 'Mission: optimize delivery routes for Block 7A', time: '2d ago' },
]

const TYPE_COLOR: Record<string, string> = {
  working: 'text-gate',
  short: 'text-mirror',
  long: 'text-ocean',
}

export default function MirrorPage() {
  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-mirror">◎</span>
          <h1 className="text-2xl font-bold text-white">The Mirror</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Memory &amp; Context — what the agent knows and has seen.
        </p>
      </div>

      {/* Memory pyramid */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Memory Pyramid
        </h2>
        <div className="flex flex-col items-center gap-2">
          {[
            { label: 'Working', width: '33%', color: 'bg-gate/30 border-gate/50', count: 1 },
            { label: 'Short-term', width: '55%', color: 'bg-mirror/20 border-mirror/40', count: 3 },
            { label: 'Long-term', width: '80%', color: 'bg-ocean/20 border-ocean/40', count: 12 },
          ].map(({ label, width, color, count }) => (
            <div
              key={label}
              className={`border rounded-lg flex items-center justify-between px-4 py-2 ${color} transition-all`}
              style={{ width }}
            >
              <span className="text-sm text-zinc-300">{label}</span>
              <span className="text-xs text-zinc-500 font-mono">{count} entries</span>
            </div>
          ))}
        </div>
      </div>

      {/* Augury prediction */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-3 uppercase tracking-wider">
          Augury Prediction
        </h2>
        <div className="flex items-center justify-between">
          <div>
            <div className="text-white font-medium">Next likely branch: <span className="text-mirror">optimize_route</span></div>
            <div className="text-zinc-500 text-sm mt-1">Based on recent memory patterns</div>
          </div>
          <div className="text-right">
            <div className="text-2xl font-mono font-bold text-mirror">82%</div>
            <div className="text-xs text-zinc-600">confidence</div>
          </div>
        </div>
      </div>

      {/* Memory entries */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Recent Memory
        </h2>
        <div className="space-y-3">
          {MEMORY_ENTRIES.map((entry) => (
            <div
              key={entry.id}
              className="flex items-start gap-3 p-3 rounded-lg bg-surface-3"
            >
              <span className={`text-xs font-mono mt-0.5 ${TYPE_COLOR[entry.type] ?? 'text-zinc-400'}`}>
                [{entry.type}]
              </span>
              <div className="flex-1 min-w-0">
                <p className="text-sm text-zinc-300 truncate">{entry.content}</p>
              </div>
              <span className="text-xs text-zinc-600 flex-shrink-0">{entry.time}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
