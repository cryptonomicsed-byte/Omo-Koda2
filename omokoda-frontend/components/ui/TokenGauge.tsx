'use client'

interface TokenGaugeProps {
  used: number
  max: number
  showLabel?: boolean
  className?: string
}

export function TokenGauge({
  used,
  max,
  showLabel = true,
  className = '',
}: TokenGaugeProps) {
  const pct = max > 0 ? Math.min((used / max) * 100, 100) : 0
  const color =
    pct < 60
      ? 'bg-green-500'
      : pct < 85
        ? 'bg-yellow-500'
        : 'bg-red-500'

  return (
    <div className={`flex flex-col gap-1 ${className}`}>
      {showLabel && (
        <div className="flex justify-between text-xs text-zinc-400 font-mono">
          <span>Context</span>
          <span>
            {used.toLocaleString()} / {max.toLocaleString()} tokens ({Math.round(pct)}%)
          </span>
        </div>
      )}
      <div className="h-1.5 w-full rounded-full bg-surface-4 overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-300 ${color}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  )
}
