'use client'

import { useEffect, useState } from 'react'

// Live capability discovery via the kernel's read-only `skills` tool
// (POST /v1/act). Returns the agent's registered skills/tools as text or JSON.
export function AgentTools() {
  const [output, setOutput] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let alive = true
    fetch('/v1/act', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ tool: 'skills', params: '{}' }),
    })
      .then(async (r) => {
        const data = await r.json().catch(() => ({}))
        if (!r.ok) throw new Error(data?.error || `kernel ${r.status}`)
        return data
      })
      .then((data) => { if (alive) setOutput(data.tool_output ?? '(no tools reported)') })
      .catch((e) => { if (alive) setError(String(e.message || e)) })
      .finally(() => { if (alive) setLoading(false) })
    return () => { alive = false }
  }, [])

  // Try to render tool_output as a structured list; fall back to preformatted text.
  let items: { name: string; description?: string; tier?: number }[] | null = null
  if (output) {
    try {
      const parsed = JSON.parse(output)
      if (Array.isArray(parsed)) items = parsed
      else if (Array.isArray(parsed?.skills)) items = parsed.skills
      else if (Array.isArray(parsed?.tools)) items = parsed.tools
    } catch { /* not JSON — show raw */ }
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold text-white">Agent Tools & Skills</h2>
      <p className="text-gray-300">Live capabilities reported by the Omo-Koda kernel.</p>

      {loading && <p className="text-gray-400">Discovering capabilities…</p>}
      {error && <p className="text-red-300 text-sm">Could not load tools: {error}</p>}

      {items && (
        <div className="grid gap-4 md:grid-cols-2">
          {items.map((tool, i) => (
            <div key={tool.name ?? i} className="rounded-2xl bg-white/10 p-4 border border-white/10 shadow-sm">
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-semibold text-white">{tool.name ?? `tool ${i + 1}`}</h3>
                {tool.tier != null && (
                  <span className="rounded-full bg-blue-600 px-3 py-1 text-xs uppercase tracking-wide text-white">Tier {tool.tier}</span>
                )}
              </div>
              {tool.description && <p className="text-gray-300 text-sm">{tool.description}</p>}
            </div>
          ))}
        </div>
      )}

      {output && !items && (
        <pre className="whitespace-pre-wrap break-words bg-gray-900/60 rounded-lg p-4 text-sm text-gray-200 max-h-96 overflow-y-auto">{output}</pre>
      )}
    </div>
  )
}
