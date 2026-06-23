'use client'

const NODES = [
  { id: 'alpha-7', label: 'Alpha-7', status: 'active', x: 50, y: 40 },
  { id: 'beta-2', label: 'Beta-2', status: 'idle', x: 20, y: 65 },
  { id: 'gamma-1', label: 'Gamma-1', status: 'active', x: 75, y: 65 },
  { id: 'delta-3', label: 'Delta-3', status: 'error', x: 40, y: 80 },
  { id: 'epsilon-4', label: 'Epsilon-4', status: 'idle', x: 60, y: 20 },
]

const STATUS_COLOR: Record<string, string> = {
  active: '#10B981',
  idle: '#6B7280',
  error: '#EF4444',
}

export default function OceanPage() {
  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-ocean">≋</span>
          <h1 className="text-2xl font-bold text-white">The Ocean</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Swarm &amp; Network — the collective hive in motion.
        </p>
      </div>

      {/* Stats bar */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        {[
          { label: 'Active Agents', value: '2', color: 'text-green-400' },
          { label: 'Idle Agents', value: '2', color: 'text-zinc-400' },
          { label: 'Errors', value: '1', color: 'text-red-400' },
        ].map(({ label, value, color }) => (
          <div
            key={label}
            className="bg-surface-2 border border-border-DEFAULT rounded-xl p-4 text-center"
          >
            <div className={`text-2xl font-mono font-bold ${color}`}>{value}</div>
            <div className="text-xs text-zinc-600 mt-1">{label}</div>
          </div>
        ))}
      </div>

      {/* Network graph (SVG placeholder) */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Agent Network
        </h2>
        <div className="relative w-full" style={{ paddingBottom: '56%' }}>
          <svg
            className="absolute inset-0 w-full h-full"
            viewBox="0 0 100 100"
            preserveAspectRatio="xMidYMid meet"
          >
            {/* Edges */}
            {[
              ['alpha-7', 'beta-2'],
              ['alpha-7', 'gamma-1'],
              ['alpha-7', 'delta-3'],
              ['gamma-1', 'epsilon-4'],
            ].map(([a, b]) => {
              const na = NODES.find((n) => n.id === a)!
              const nb = NODES.find((n) => n.id === b)!
              return (
                <line
                  key={`${a}-${b}`}
                  x1={na.x}
                  y1={na.y}
                  x2={nb.x}
                  y2={nb.y}
                  stroke="rgba(6,182,212,0.2)"
                  strokeWidth="0.5"
                />
              )
            })}
            {/* Nodes */}
            {NODES.map(({ id, label, status, x, y }) => (
              <g key={id}>
                <circle
                  cx={x}
                  cy={y}
                  r="4"
                  fill={STATUS_COLOR[status] ?? '#6B7280'}
                  opacity="0.8"
                />
                <text
                  x={x}
                  y={y + 7}
                  textAnchor="middle"
                  fontSize="3"
                  fill="#9CA3AF"
                >
                  {label}
                </text>
              </g>
            ))}
          </svg>
        </div>
      </div>

      {/* Agent list */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Agent Roster
        </h2>
        <div className="space-y-2">
          {NODES.map(({ id, label, status }) => (
            <div
              key={id}
              className="flex items-center gap-3 p-3 rounded-lg bg-surface-3"
            >
              <span
                className="w-2 h-2 rounded-full flex-shrink-0"
                style={{ backgroundColor: STATUS_COLOR[status] }}
              />
              <span className="text-sm text-zinc-300 font-mono flex-1">{label}</span>
              <span className="text-xs text-zinc-500 capitalize">{status}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
