'use client'

import { useState, useEffect } from 'react'
import { vaultListFiles, vaultReadFile, type VaultFileEntry } from '@/lib/api'

export default function OceanPage() {
  const [files, setFiles] = useState<VaultFileEntry[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selected, setSelected] = useState<string | null>(null)
  const [content, setContent] = useState<string | null>(null)
  const [contentLoading, setContentLoading] = useState(false)

  useEffect(() => {
    vaultListFiles()
      .then(setFiles)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false))
  }, [])

  async function openFile(path: string) {
    setSelected(path)
    setContent(null)
    setContentLoading(true)
    try {
      const result = await vaultReadFile(path)
      setContent(result.content)
    } catch (e) {
      setContent(`Error: ${e instanceof Error ? e.message : 'unknown'}`)
    } finally {
      setContentLoading(false)
    }
  }

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <div className="mb-8">
        <div className="flex items-center gap-3 mb-2">
          <span className="text-2xl text-ocean">≋</span>
          <h1 className="text-2xl font-bold text-white">The Ocean</h1>
        </div>
        <p className="text-zinc-500 text-sm">
          Vault file browser — knowledge, traces, and broadcast templates.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* File list */}
        <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
          <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
            Vault Files
            {!loading && <span className="ml-2 text-zinc-600 normal-case font-normal">({files.length})</span>}
          </h2>

          {loading && (
            <p className="text-xs text-zinc-600">Loading vault…</p>
          )}

          {error && (
            <p className="text-xs text-red-400">{error}</p>
          )}

          {!loading && !error && files.length === 0 && (
            <p className="text-xs text-zinc-600">
              No vault files yet. Birth an agent to create the vault.
            </p>
          )}

          <div className="space-y-1">
            {files.map((f) => (
              <button
                key={f.path}
                onClick={() => openFile(f.path)}
                className={`w-full text-left px-3 py-2 rounded-lg text-sm font-mono transition-colors ${
                  selected === f.path
                    ? 'bg-brand-600/20 border border-brand-600/40 text-brand-400'
                    : 'hover:bg-surface-3 text-zinc-300 border border-transparent'
                }`}
              >
                <div className="truncate">{f.path}</div>
                <div className="text-xs text-zinc-600 mt-0.5">
                  {Math.ceil(f.size_bytes / 1024)}KB
                </div>
              </button>
            ))}
          </div>
        </div>

        {/* File content viewer */}
        <div className="bg-surface-2 border border-border-DEFAULT rounded-xl p-6">
          <h2 className="text-sm font-semibold text-zinc-400 mb-4 uppercase tracking-wider">
            {selected ? selected : 'File Viewer'}
          </h2>

          {!selected && (
            <p className="text-xs text-zinc-600">Select a file to view its contents.</p>
          )}

          {contentLoading && (
            <p className="text-xs text-zinc-600">Loading…</p>
          )}

          {content !== null && !contentLoading && (
            <pre className="text-xs text-zinc-300 whitespace-pre-wrap font-mono overflow-auto max-h-96 leading-relaxed">
              {content}
            </pre>
          )}
        </div>
      </div>
    </div>
  )
}
