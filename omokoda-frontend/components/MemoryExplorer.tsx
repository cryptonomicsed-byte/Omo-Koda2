'use client'

import { useState, useEffect } from 'react'

interface MemCellEmotion {
  score: number
  valence: 'positive' | 'negative' | 'neutral'
}

interface MemCellView {
  id: string
  text: string
  importance: number
  emotion: MemCellEmotion
}

interface MemSceneView {
  theme: string
  summary: string
  strength: number
}

interface MemoryResponse {
  cells: MemCellView[]
  scenes: MemSceneView[]
}

interface MemoryExplorerProps {
  agentId: string
}

function importanceDot(importance: number): string {
  if (importance >= 0.7) return '#4ade80'   // green — high
  if (importance >= 0.4) return '#facc15'   // yellow — medium
  return '#f87171'                           // red — low
}

function valenceEmoji(valence: 'positive' | 'negative' | 'neutral'): string {
  if (valence === 'positive') return '✓'
  if (valence === 'negative') return '✗'
  return '◦'
}

function truncate(text: string, max = 80): string {
  return text.length > max ? text.slice(0, max - 1) + '…' : text
}

export default function MemoryExplorer({ agentId }: MemoryExplorerProps) {
  const [data, setData] = useState<MemoryResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    setLoading(true)
    setError(null)
    setData(null)

    // The kernel exposes memory via the vault galaxy (there is no per-agent SOMA
    // HTTP endpoint). Adapt galaxy stars → memory cells and nebulae → scenes.
    fetch('/v1/vault/galaxy')
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      })
      .then((galaxy) => {
        if (cancelled) return
        const stars: { id: string; label: string; kind?: string }[] = galaxy?.stars ?? []
        const nebulae: string[] = galaxy?.nebulae ?? []
        const adapted: MemoryResponse = {
          cells: stars.map((s) => ({
            id: s.id,
            text: s.label,
            importance: 0.5,
            emotion: { score: 0, valence: 'neutral' as const },
          })),
          scenes: nebulae.map((n) => ({ theme: n, summary: '', strength: 1 })),
        }
        setData(adapted)
        setLoading(false)
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message ?? 'Failed to load memory')
          setLoading(false)
        }
      })

    return () => { cancelled = true }
  }, [agentId])

  // --- Loading ---
  if (loading) {
    return (
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <p className="text-sm text-gray-400">Loading memory…</p>
      </div>
    )
  }

  // --- Error ---
  if (error || !data) {
    return (
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <p className="text-sm text-red-400">{error ?? 'Unknown error'}</p>
      </div>
    )
  }

  const topCells = [...data.cells]
    .sort((a, b) => b.importance - a.importance)
    .slice(0, 10)

  const sortedScenes = [...data.scenes].sort((a, b) => b.strength - a.strength)

  const noContent = topCells.length === 0 && sortedScenes.length === 0

  if (noContent) {
    return (
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <p className="text-sm text-gray-500">No memories yet</p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Memory Cells */}
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Memory Cells
        </h3>
        {topCells.length === 0 ? (
          <p className="text-sm text-gray-500">No memories yet</p>
        ) : (
          <ul className="space-y-2">
            {topCells.map((cell) => (
              <li
                key={cell.id}
                className="flex items-start space-x-2 text-sm"
              >
                {/* Importance dot */}
                <span
                  className="mt-1.5 h-2 w-2 flex-shrink-0 rounded-full"
                  style={{ backgroundColor: importanceDot(cell.importance) }}
                  title={`Importance: ${(cell.importance * 100).toFixed(0)}%`}
                />
                {/* Text */}
                <span className="flex-1 text-gray-300 leading-snug">
                  {truncate(cell.text)}
                </span>
                {/* Valence */}
                <span
                  className="flex-shrink-0 text-gray-400 font-mono"
                  title={`Valence: ${cell.emotion.valence}`}
                >
                  {valenceEmoji(cell.emotion.valence)}
                </span>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* Active Scenes */}
      <div className="rounded-lg bg-gray-900/50 border border-gray-700 p-4">
        <h3 className="text-sm font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Active Scenes
        </h3>
        {sortedScenes.length === 0 ? (
          <p className="text-sm text-gray-500">No memories yet</p>
        ) : (
          <ul className="space-y-4">
            {sortedScenes.map((scene, idx) => (
              <li key={idx} className="space-y-1">
                <div className="flex items-center justify-between">
                  <span className="text-sm font-bold text-white">{scene.theme}</span>
                  <span className="text-xs font-mono bg-gray-800 text-gray-300 px-2 py-0.5 rounded-full border border-gray-700">
                    {scene.strength.toFixed(2)}
                  </span>
                </div>
                <p className="text-sm text-gray-400 leading-snug">{scene.summary}</p>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}
