'use client'

const GATES = [
  { name: 'Mentalism', desc: 'Mind governs matter', score: 95, status: 'PASS' },
  { name: 'Correspondence', desc: 'As above, so below', score: 88, status: 'PASS' },
  { name: 'Vibration', desc: 'All is in motion', score: 72, status: 'WARN' },
  { name: 'Polarity', desc: 'Opposites are identical in nature', score: 91, status: 'PASS' },
  { name: 'Rhythm', desc: 'Everything flows', score: 60, status: 'WARN' },
  { name: 'Cause & Effect', desc: 'Nothing escapes the law', score: 98, status: 'PASS' },
  { name: 'Gender', desc: 'Gender manifests on all planes', score: 84, status: 'PASS' },
]

const STATUS_COLOR: Record<string, string> = {
  PASS: 'text-green-400',
  WARN: 'text-yellow-400',
  HALT: 'text-red-400',
}

const BAR_COLOR: Record<string, string> = {
  PASS: 'bg-green-500',
  WARN: 'bg-yellow-500',
  HALT: 'bg-red-500',
}

export default function BalancePage() {
  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-balance">⊖</span>
          <h1 className="text-2xl font-bold text-white">The Balance</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Ethics &amp; Gates — the 7 Hermetic principles as living policy.
        </p>
      </div>

      {/* Gate meters */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-5 uppercase tracking-wider">
          Gate Compliance
        </h2>
        <div className="space-y-5">
          {GATES.map(({ name, desc, score, status }) => (
            <div key={name}>
              <div className="flex items-center justify-between mb-1.5">
                <div>
                  <span className="text-sm font-medium text-white">{name}</span>
                  <span className="text-xs text-zinc-600 ml-2">{desc}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-mono text-zinc-400">{score}%</span>
                  <span className={`text-xs font-mono font-bold ${STATUS_COLOR[status]}`}>
                    {status}
                  </span>
                </div>
              </div>
              <div className="h-1.5 w-full bg-surface-4 rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full transition-all duration-500 ${BAR_COLOR[status]}`}
                  style={{ width: `${score}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Violations log */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Recent Events
        </h2>
        <div className="space-y-2 text-sm">
          <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-3">
            <span className="text-yellow-400 flex-shrink-0">⚠</span>
            <div className="flex-1">
              <span className="text-zinc-300">Vibration gate score dropped below 75%</span>
              <div className="text-xs text-zinc-600 mt-0.5">Reason: high-frequency tool calls without cooldown</div>
            </div>
            <span className="text-xs text-zinc-600">5m ago</span>
          </div>
          <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-3">
            <span className="text-yellow-400 flex-shrink-0">⚠</span>
            <div className="flex-1">
              <span className="text-zinc-300">Rhythm gate below threshold</span>
              <div className="text-xs text-zinc-600 mt-0.5">Reason: burst execution without rhythm pacing</div>
            </div>
            <span className="text-xs text-zinc-600">12m ago</span>
          </div>
          <div className="flex items-start gap-3 p-3 rounded-lg bg-surface-3">
            <span className="text-green-400 flex-shrink-0">✓</span>
            <div className="flex-1">
              <span className="text-zinc-300">All gates nominal</span>
              <div className="text-xs text-zinc-600 mt-0.5">System health check passed</div>
            </div>
            <span className="text-xs text-zinc-600">1h ago</span>
          </div>
        </div>
      </div>
    </div>
  )
}
