'use client'

const DAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const TODAY = new Date().getDay()

const MODIFIERS = [
  { label: 'Execution Boost', value: '+15%', active: true },
  { label: 'Memory Depth', value: '+2 layers', active: true },
  { label: 'Network Latency', value: 'nominal', active: false },
]

export default function StormPage() {
  return (
    <div className="p-8 max-w-3xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-storm">∿</span>
          <h1 className="text-2xl font-bold text-white">The Storm</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Time &amp; Flow — the temporal engine that governs all cycles.
        </p>
      </div>

      {/* 7-day cycle wheel */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-6 uppercase tracking-wider">
          7-Day Cycle
        </h2>
        <div className="flex justify-around items-end gap-2">
          {DAYS.map((day, i) => {
            const isToday = i === TODAY
            const height = 40 + Math.sin((i / 7) * Math.PI * 2) * 30
            return (
              <div key={day} className="flex flex-col items-center gap-2">
                <div
                  className={`w-8 rounded-t-md transition-all ${
                    isToday ? 'bg-storm' : 'bg-surface-4'
                  }`}
                  style={{ height: `${Math.max(height, 20)}px` }}
                />
                <span
                  className={`text-xs font-mono ${isToday ? 'text-storm font-bold' : 'text-zinc-600'}`}
                >
                  {day}
                </span>
                {isToday && (
                  <span className="text-xs text-storm">●</span>
                )}
              </div>
            )
          })}
        </div>
        <div className="mt-4 text-center text-sm text-zinc-500">
          Day {TODAY + 1} of 7 — <span className="text-storm">{DAYS[TODAY]}</span> cycle active
        </div>
      </div>

      {/* Temporal modifiers */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6 mb-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Active Modifiers
        </h2>
        <div className="space-y-3">
          {MODIFIERS.map(({ label, value, active }) => (
            <div key={label} className="flex items-center justify-between p-3 rounded-lg bg-surface-3">
              <div className="flex items-center gap-2">
                <span className={`w-2 h-2 rounded-full ${active ? 'bg-storm' : 'bg-surface-5'}`} />
                <span className="text-sm text-zinc-300">{label}</span>
              </div>
              <span className={`text-sm font-mono ${active ? 'text-storm' : 'text-zinc-600'}`}>
                {value}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Vortex resonance */}
      <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
        <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
          Vortex Resonance (3-6-9)
        </h2>
        <div className="flex items-center justify-center gap-8">
          {[3, 6, 9].map((n) => (
            <div key={n} className="flex flex-col items-center gap-2">
              <div
                className="w-16 h-16 rounded-full border-2 border-storm/40 flex items-center justify-center text-2xl font-mono font-bold text-storm"
                style={{
                  boxShadow: '0 0 20px rgba(16,185,129,0.15)',
                }}
              >
                {n}
              </div>
              <span className="text-xs text-zinc-600 font-mono">
                {n === 3 ? 'create' : n === 6 ? 'sustain' : 'dissolve'}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
