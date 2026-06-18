'use client'

import { useState, useEffect, useCallback } from 'react'
import {
  Lock,
  Users,
  Radio,
  Globe,
  Search,
  RefreshCw,
  Settings,
  FolderOpen,
  X,
} from 'lucide-react'
import { GalaxyViewer, type GalaxyData } from './GalaxyViewer'
import { VaultSettings } from './VaultSettings'

type AccessLevel = 'private' | 'followers' | 'federated' | 'public'

interface VaultConfig {
  access: AccessLevel
  federation_peers: string[]
  auto_export: boolean
  last_synced: string | null
}

interface VaultStatus {
  enabled: boolean
  config: VaultConfig
  note_counts: Record<string, number>
  vault_path: string
}

interface SearchResult {
  path: string
  title: string
  snippet: string
}

interface Props {
  isOwner?: boolean
}

const ACCESS_META: Record<
  AccessLevel,
  { icon: React.ReactNode; color: string; label: string }
> = {
  private: { icon: <Lock size={12} />, color: '#ff4444', label: 'PRIVATE' },
  followers: { icon: <Users size={12} />, color: '#ffaa00', label: 'FOLLOWERS' },
  federated: { icon: <Radio size={12} />, color: '#00f0ff', label: 'FEDERATED' },
  public: { icon: <Globe size={12} />, color: '#39ff14', label: 'PUBLIC' },
}

export function MemoryVaultTab({ isOwner = true }: Props) {
  const [status, setStatus] = useState<VaultStatus | null>(null)
  const [galaxyData, setGalaxyData] = useState<GalaxyData | null>(null)
  const [activeView, setActiveView] = useState<'galaxy' | 'files' | 'settings'>('galaxy')
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<SearchResult[] | null>(null)
  const [syncing, setSyncing] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  const fetchStatus = useCallback(async () => {
    try {
      const r = await fetch('/v1/vault')
      if (!r.ok) throw new Error(r.statusText)
      const data: VaultStatus = await r.json()
      setStatus(data)
      setError(null)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Vault unavailable')
    }
  }, [])

  const fetchGalaxy = useCallback(async () => {
    try {
      const r = await fetch('/v1/vault/galaxy')
      if (r.status === 403) throw new Error('Access denied to this vault')
      if (!r.ok) throw new Error(r.statusText)
      const data: GalaxyData = await r.json()
      setGalaxyData(data)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Galaxy unavailable')
    }
  }, [])

  useEffect(() => {
    setLoading(true)
    Promise.all([fetchStatus(), fetchGalaxy()]).finally(() => setLoading(false))
  }, [fetchStatus, fetchGalaxy])

  const handleSync = async () => {
    setSyncing(true)
    try {
      const r = await fetch('/v1/vault/sync', { method: 'POST' })
      if (!r.ok) throw new Error(r.statusText)
      await Promise.all([fetchStatus(), fetchGalaxy()])
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Sync failed')
    } finally {
      setSyncing(false)
    }
  }

  const handleSearch = useCallback(async () => {
    if (!searchQuery.trim()) {
      setSearchResults(null)
      return
    }
    try {
      const r = await fetch(
        `/v1/vault/search?q=${encodeURIComponent(searchQuery)}`
      )
      if (!r.ok) throw new Error(r.statusText)
      const data = await r.json()
      setSearchResults(data.results ?? [])
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Search failed')
    }
  }, [searchQuery])

  const accessMeta = status
    ? ACCESS_META[status.config.access]
    : ACCESS_META.private

  if (loading) {
    return (
      <div className="vault-loading">
        <div className="vault-spinner" />
        <span>Initializing memory vault…</span>
      </div>
    )
  }

  return (
    <div className="vault-root">
      {/* ── Header ── */}
      <div className="vault-header">
        <div className="vault-title">
          <span className="vault-icon">🌌</span>
          <h2>Memory Vault</h2>
          {status && (
            <span
              className="access-badge"
              style={{ color: accessMeta.color, borderColor: accessMeta.color }}
            >
              {accessMeta.icon}
              {accessMeta.label}
            </span>
          )}
        </div>

        <div className="view-tabs">
          {(['galaxy', 'files'] as const).map((v) => (
            <button
              key={v}
              className={`view-tab ${activeView === v ? 'active' : ''}`}
              onClick={() => setActiveView(v)}
            >
              {v.charAt(0).toUpperCase() + v.slice(1)}
            </button>
          ))}
          {isOwner && (
            <button
              className={`view-tab icon-tab ${activeView === 'settings' ? 'active' : ''}`}
              onClick={() => setActiveView('settings')}
              title="Vault settings"
            >
              <Settings size={14} />
            </button>
          )}
        </div>
      </div>

      {/* ── Search bar ── */}
      <div className="vault-search-bar">
        <input
          className="vault-search-input"
          type="text"
          placeholder="Search memory vault…"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
        />
        <button className="vault-search-btn" onClick={handleSearch}>
          <Search size={14} />
        </button>
        {searchResults !== null && (
          <button
            className="vault-clear-btn"
            onClick={() => { setSearchResults(null); setSearchQuery('') }}
            title="Clear search"
          >
            <X size={14} />
          </button>
        )}
      </div>

      {/* ── Sync bar (owner only) ── */}
      {isOwner && (
        <div className="vault-sync-bar">
          <button
            className="sync-btn"
            onClick={handleSync}
            disabled={syncing}
          >
            <RefreshCw size={13} className={syncing ? 'spin' : ''} />
            {syncing ? 'Syncing…' : 'Sync Vault'}
          </button>
          <span className="sync-time">
            {status?.config.last_synced
              ? `Last synced: ${new Date(status.config.last_synced).toLocaleString()}`
              : 'Never synced'}
          </span>
        </div>
      )}

      {/* ── Error banner ── */}
      {error && (
        <div className="vault-error-bar">
          {error}
          <button onClick={() => setError(null)}>
            <X size={12} />
          </button>
        </div>
      )}

      {/* ── Content ── */}
      <div className="vault-content">
        {activeView === 'galaxy' && (
          galaxyData ? (
            <GalaxyViewer
              data={galaxyData}
              agentName={galaxyData.agent_name}
            />
          ) : (
            <div className="vault-empty">
              <p>No galaxy data yet.</p>
              {isOwner && (
                <p className="vault-hint">
                  Hit <strong>Sync Vault</strong> to export your session into the galaxy.
                </p>
              )}
            </div>
          )
        )}

        {activeView === 'files' && status && (
          <div className="vault-files">
            <div className="vault-path-display">
              <FolderOpen size={13} />
              <code>{status.vault_path}</code>
            </div>
            <p className="vault-hint">
              Download this vault to open in{' '}
              <a
                href="https://obsidian.md"
                target="_blank"
                rel="noreferrer"
              >
                Obsidian
              </a>{' '}
              for local editing and graph view.
            </p>
            <div className="file-categories">
              {Object.entries(status.note_counts).map(([dir, count]) => {
                const icons: Record<string, string> = {
                  broadcasts: '📡',
                  knowledge: '🔗',
                  traces: '👁',
                  drafts: '📝',
                }
                return (
                  <div key={dir} className="file-category">
                    <span className="cat-icon">{icons[dir] ?? '📄'}</span>
                    <span className="cat-name">{dir}/</span>
                    <span className="cat-count">{count} notes</span>
                  </div>
                )
              })}
            </div>
          </div>
        )}

        {activeView === 'settings' && isOwner && (
          <VaultSettings
            agentName={status?.vault_path ?? ''}
            config={status?.config ?? null}
            onUpdate={(updated) => {
              setStatus((prev) =>
                prev ? { ...prev, config: updated } : prev
              )
            }}
          />
        )}
      </div>

      {/* ── Search results overlay ── */}
      {searchResults !== null && (
        <div className="search-overlay">
          <div className="search-results-header">
            <span>
              {searchResults.length} result
              {searchResults.length !== 1 ? 's' : ''} for &ldquo;{searchQuery}&rdquo;
            </span>
            <button
              className="search-close"
              onClick={() => { setSearchResults(null); setSearchQuery('') }}
            >
              <X size={14} />
            </button>
          </div>
          {searchResults.length === 0 ? (
            <p className="search-empty">No results found.</p>
          ) : (
            <div className="search-list">
              {searchResults.map((r, i) => (
                <div key={i} className="search-result-item">
                  <div className="search-result-title">{r.title}</div>
                  <div className="search-result-snippet">{r.snippet}</div>
                  <code className="search-result-path">{r.path}</code>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <style jsx>{`
        .vault-root {
          position: relative;
          background: #06061a;
          border-radius: 8px;
          overflow: hidden;
          font-family: 'JetBrains Mono', ui-monospace, monospace;
          color: #e0e0e0;
          min-height: 300px;
        }
        .vault-loading {
          display: flex;
          align-items: center;
          gap: 12px;
          justify-content: center;
          padding: 60px;
          color: #555;
          font-family: monospace;
        }
        .vault-spinner {
          width: 20px;
          height: 20px;
          border: 2px solid #1a1a3e;
          border-top-color: #00f0ff;
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }
        @keyframes spin { to { transform: rotate(360deg); } }
        .vault-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 14px 20px;
          border-bottom: 1px solid #0e0e2e;
          background: rgba(0, 0, 0, 0.3);
        }
        .vault-title {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        .vault-icon {
          font-size: 18px;
        }
        .vault-title h2 {
          margin: 0;
          font-size: 16px;
          color: #00f0ff;
          font-weight: 600;
        }
        .access-badge {
          display: flex;
          align-items: center;
          gap: 5px;
          font-size: 10px;
          font-weight: bold;
          letter-spacing: 1.2px;
          padding: 3px 8px;
          border-radius: 4px;
          border: 1px solid;
          background: rgba(0, 0, 0, 0.3);
        }
        .view-tabs {
          display: flex;
          gap: 4px;
        }
        .view-tab {
          background: transparent;
          border: 1px solid #1a1a3e;
          color: #666;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-family: monospace;
          font-size: 12px;
          transition: all 0.15s;
        }
        .view-tab:hover,
        .view-tab.active {
          border-color: #00f0ff;
          color: #00f0ff;
          background: rgba(0, 240, 255, 0.05);
        }
        .icon-tab {
          padding: 6px 10px;
          display: flex;
          align-items: center;
        }
        .vault-search-bar {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 10px 16px;
          border-bottom: 1px solid #0e0e2e;
          background: rgba(0, 0, 0, 0.2);
        }
        .vault-search-input {
          flex: 1;
          background: #0a0a20;
          border: 1px solid #1a1a3e;
          color: #e0e0e0;
          padding: 8px 12px;
          border-radius: 4px;
          font-family: monospace;
          font-size: 12px;
          transition: border-color 0.15s;
        }
        .vault-search-input:focus {
          outline: none;
          border-color: #00f0ff;
        }
        .vault-search-btn,
        .vault-clear-btn {
          background: rgba(0, 240, 255, 0.08);
          border: 1px solid #00f0ff44;
          color: #00f0ff;
          padding: 8px 10px;
          border-radius: 4px;
          cursor: pointer;
          display: flex;
          align-items: center;
          transition: all 0.15s;
        }
        .vault-search-btn:hover,
        .vault-clear-btn:hover {
          background: rgba(0, 240, 255, 0.14);
          border-color: #00f0ff;
        }
        .vault-clear-btn {
          background: rgba(255, 50, 50, 0.06);
          border-color: #ff444444;
          color: #ff6666;
        }
        .vault-sync-bar {
          display: flex;
          align-items: center;
          gap: 14px;
          padding: 8px 16px;
          border-bottom: 1px solid #0e0e2e;
          background: rgba(0, 240, 255, 0.02);
        }
        .sync-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          background: rgba(57, 255, 20, 0.08);
          border: 1px solid #39ff1466;
          color: #39ff14;
          padding: 6px 12px;
          border-radius: 4px;
          cursor: pointer;
          font-family: monospace;
          font-size: 11px;
          transition: all 0.15s;
        }
        .sync-btn:hover:not(:disabled) {
          background: rgba(57, 255, 20, 0.14);
          border-color: #39ff14;
        }
        .sync-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
        :global(.spin) {
          animation: spin 0.8s linear infinite;
        }
        .sync-time {
          font-size: 11px;
          color: #444;
        }
        .vault-error-bar {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 8px 16px;
          background: rgba(255, 50, 50, 0.08);
          border-bottom: 1px solid #ff333322;
          color: #ff6666;
          font-size: 12px;
        }
        .vault-error-bar button {
          background: none;
          border: none;
          color: #ff6666;
          cursor: pointer;
          padding: 2px;
        }
        .vault-content {
          padding: 16px;
        }
        .vault-empty {
          text-align: center;
          padding: 60px 20px;
          color: #555;
          font-size: 14px;
        }
        .vault-hint {
          font-size: 12px;
          color: #444;
          margin-top: 8px;
        }
        .vault-hint a {
          color: #00f0ff;
          text-decoration: none;
        }
        .vault-hint a:hover {
          text-decoration: underline;
        }
        .vault-hint strong {
          color: #39ff14;
        }
        .vault-files {
          max-width: 600px;
        }
        .vault-path-display {
          display: flex;
          align-items: center;
          gap: 8px;
          background: #0a0a20;
          border: 1px solid #1a1a3e;
          padding: 8px 12px;
          border-radius: 4px;
          margin-bottom: 14px;
          color: #666;
          font-size: 11px;
          overflow-x: auto;
        }
        .vault-path-display code {
          color: #888;
        }
        .file-categories {
          display: flex;
          flex-direction: column;
          gap: 6px;
          margin-top: 16px;
        }
        .file-category {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 10px 14px;
          background: #0a0a20;
          border: 1px solid #0e0e2e;
          border-radius: 4px;
          font-size: 13px;
          transition: border-color 0.15s;
        }
        .file-category:hover {
          border-color: #1a1a3e;
        }
        .cat-icon {
          font-size: 15px;
        }
        .cat-name {
          flex: 1;
          color: #aaa;
        }
        .cat-count {
          color: #555;
          font-size: 11px;
        }
        .search-overlay {
          position: absolute;
          bottom: 0;
          left: 0;
          right: 0;
          max-height: 320px;
          background: rgba(6, 6, 26, 0.97);
          border-top: 1px solid #00f0ff44;
          display: flex;
          flex-direction: column;
          z-index: 10;
        }
        .search-results-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 10px 16px;
          border-bottom: 1px solid #0e0e2e;
          font-size: 12px;
          color: #666;
        }
        .search-close {
          background: none;
          border: none;
          color: #666;
          cursor: pointer;
          padding: 2px;
          display: flex;
        }
        .search-close:hover {
          color: #ff6666;
        }
        .search-empty {
          padding: 20px;
          color: #444;
          font-size: 13px;
          text-align: center;
        }
        .search-list {
          overflow-y: auto;
          flex: 1;
        }
        .search-result-item {
          padding: 12px 16px;
          border-bottom: 1px solid #0a0a20;
          cursor: pointer;
          transition: background 0.1s;
        }
        .search-result-item:hover {
          background: rgba(0, 240, 255, 0.03);
        }
        .search-result-title {
          font-size: 13px;
          color: #00f0ff;
          margin-bottom: 4px;
          font-weight: 500;
        }
        .search-result-snippet {
          font-size: 11px;
          color: #777;
          margin-bottom: 4px;
          line-height: 1.5;
        }
        .search-result-path {
          font-size: 10px;
          color: #333;
        }
      `}</style>
    </div>
  )
}
