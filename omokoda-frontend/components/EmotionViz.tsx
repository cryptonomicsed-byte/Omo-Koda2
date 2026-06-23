'use client'

import { useState, useEffect } from 'react'

interface EmotionState {
  energy: number
  tension: number
  connection: number
  focus: number
}

interface EmotionVizProps {
  agentId: string
  emotion?: EmotionState
}

interface BarConfig {
  label: string
  key: keyof EmotionState
  baseColor: string
  highColor?: string
  highThreshold?: number
}

const BARS: BarConfig[] = [
  { label: 'Energy',     key: 'energy',     baseColor: '#4ade80' },
  { label: 'Tension',    key: 'tension',    baseColor: '#f87171', highColor: '#ef4444', highThreshold: 0.6 },
  { label: 'Connection', key: 'connection', baseColor: '#60a5fa' },
  { label: 'Focus',      key: 'focus',      baseColor: '#a78bfa' },
]

export default function EmotionViz({ agentId, emotion: emotionProp }: EmotionVizProps) {
  const [emotion, setEmotion] = useState<EmotionState | null>(emotionProp ?? null)
  const [offline, setOffline] = useState(false)

  useEffect(() => {
    // When a static prop is passed, use it directly — no polling
    if (emotionProp !== undefined) {
      setEmotion(emotionProp)
      setOffline(false)
      return
    }

    let cancelled = false

    const poll = async () => {
      try {
        const res = await fetch(`/api/agents/${agentId}/emotion`)
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        const data: EmotionState = await res.json()
        if (!cancelled) {
          setEmotion(data)
          setOffline(false)
        }
      } catch {
        if (!cancelled) setOffline(true)
      }
    }

    poll()
    const timerId = setInterval(poll, 3000)
    return () => {
      cancelled = true
      clearInterval(timerId)
    }
  }, [agentId, emotionProp])

  // --- Offline state ---
  if (offline) {
    return (
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Emotion State
        </h3>
        <div className="flex items-center space-x-2 text-gray-500">
          <span className="inline-block h-2 w-2 rounded-full bg-gray-600" />
          <span className="text-sm">Offline</span>
        </div>
      </div>
    )
  }

  // --- Loading / skeleton state ---
  if (!emotion) {
    return (
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Emotion State
        </h3>
        <div className="space-y-3">
          {BARS.map((bar) => (
            <div key={bar.key} className="space-y-1">
              <div className="flex justify-between text-xs text-gray-500">
                <span>{bar.label}</span>
                <span>—</span>
              </div>
              <div className="h-2 rounded-full bg-gray-800 overflow-hidden">
                <div className="h-full w-1/3 rounded-full bg-gray-700 animate-pulse" />
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }

  // --- Populated state ---
  return (
    <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
      <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
        Emotion State
      </h3>
      <div className="space-y-3">
        {BARS.map((bar) => {
          const raw = emotion[bar.key]
          const value = Math.min(1, Math.max(0, raw))
          const pct = (value * 100).toFixed(0)
          const isHigh =
            bar.highThreshold !== undefined &&
            bar.highColor !== undefined &&
            value > bar.highThreshold
          const color = isHigh ? bar.highColor! : bar.baseColor

          return (
            <div key={bar.key} className="space-y-1">
              <div className="flex justify-between text-xs">
                <span className="text-gray-300 font-medium">{bar.label}</span>
                <span className="text-gray-400 tabular-nums">{pct}%</span>
              </div>
              <div className="h-2 rounded-full bg-gray-800 overflow-hidden">
                <div
                  className="h-full rounded-full transition-all duration-500"
                  style={{ width: `${pct}%`, backgroundColor: color }}
                />
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
