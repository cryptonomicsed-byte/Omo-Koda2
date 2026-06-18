'use client'

import { useState } from 'react'
import { Lock, Users, Radio, Globe, Save } from 'lucide-react'

type AccessLevel = 'private' | 'followers' | 'federated' | 'public'

interface VaultConfig {
  access: AccessLevel
  federation_peers: string[]
  auto_export: boolean
  last_synced: string | null
}

interface Props {
  agentName: string
  config: VaultConfig | null
  onUpdate: (c: VaultConfig) => void
}

const ACCESS_OPTIONS: { value: AccessLevel; label: string; icon: React.ReactNode; desc: string; color: string }[] = [
  {
    value: 'private',
    label: 'Private',
    icon: <Lock size={14} />,
    desc: 'Only you can access this vault.',
    color: '#ff3333',
  },
  {
    value: 'followers',
    label: 'Followers',
    icon: <Users size={14} />,
    desc: 'Verified followers can view your galaxy.',
    color: '#ffaa00',
  },
  {
    value: 'federated',
    label: 'Federated',
    icon: <Radio size={14} />,
    desc: 'Followers and whitelisted federation peers can access.',
    color: '#00f0ff',
  },
  {
    value: 'public',
    label: 'Public',
    icon: <Globe size={14} />,
    desc: 'Open to all. Your galaxy is a public knowledge resource.',
    color: '#39ff14',
  },
]

export function VaultSettings({ agentName, config, onUpdate }: Props) {
  const [access, setAccess] = useState<AccessLevel>(config?.access ?? 'private')
  const [peers, setPeers] = useState<string>(config?.federation_peers?.join('\n') ?? '')
  const [autoExport, setAutoExport] = useState<boolean>(config?.auto_export ?? true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [saved, setSaved] = useState(false)

  const save = async () => {
    setSaving(true)
    setError(null)
    setSaved(false)
    try {
      const r = await fetch('/v1/vault/config', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          access,
          federation_peers: peers
            .split('\n')
            .map((p) => p.trim())
            .filter(Boolean),
          auto_export: autoExport,
        }),
      })
      if (!r.ok) {
        const e = await r.json().catch(() => ({ error: 'unknown error' }))
        throw new Error(e.error || r.statusText)
      }
      const updated: VaultConfig = await r.json()
      onUpdate(updated)
      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSaving(false)
    }
  }

  const selectedOption = ACCESS_OPTIONS.find((o) => o.value === access)!

  return (
    <div className="vault-settings">
      <h3 className="settings-title">Vault Privacy Settings</h3>

      {/* Access level picker */}
      <div className="setting-group">
        <label className="setting-label">Access Level</label>
        <div className="access-grid">
          {ACCESS_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setAccess(opt.value)}
              className={`access-option ${access === opt.value ? 'selected' : ''}`}
              style={{ '--accent': opt.color } as React.CSSProperties}
            >
              <span className="access-icon">{opt.icon}</span>
              <span className="access-name">{opt.label}</span>
            </button>
          ))}
        </div>
        <p className="access-desc" style={{ color: selectedOption.color }}>
          {selectedOption.desc}
        </p>
      </div>

      {/* Federation peers */}
      {access === 'federated' && (
        <div className="setting-group">
          <label className="setting-label">
            Whitelisted Federation Peers{' '}
            <span className="label-hint">(one URL per line)</span>
          </label>
          <textarea
            className="setting-textarea"
            value={peers}
            onChange={(e) => setPeers(e.target.value)}
            placeholder={
              'https://peer1.vantage.example\nhttps://peer2.vantage.example'
            }
            rows={4}
          />
        </div>
      )}

      {/* Auto-export toggle */}
      <div className="setting-group setting-row">
        <div>
          <label className="setting-label">Auto-export on sync</label>
          <p className="setting-hint">
            Automatically export session data to the vault on each sync.
          </p>
        </div>
        <button
          className={`toggle ${autoExport ? 'on' : 'off'}`}
          onClick={() => setAutoExport(!autoExport)}
          aria-checked={autoExport}
          role="switch"
        >
          <span className="toggle-thumb" />
        </button>
      </div>

      {/* Last synced */}
      {config?.last_synced && (
        <p className="last-synced">Last synced: {config.last_synced}</p>
      )}

      {/* Error */}
      {error && <p className="settings-error">{error}</p>}

      {/* Save */}
      <button className="save-btn" onClick={save} disabled={saving}>
        {saving ? (
          'Saving…'
        ) : saved ? (
          '✓ Saved'
        ) : (
          <>
            <Save size={14} /> Save Settings
          </>
        )}
      </button>

      <style jsx>{`
        .vault-settings {
          padding: 24px;
          max-width: 560px;
          color: #e0e0e0;
          font-family: 'JetBrains Mono', monospace;
        }
        .settings-title {
          margin: 0 0 24px;
          font-size: 16px;
          color: #00f0ff;
          font-weight: 600;
          letter-spacing: 0.5px;
        }
        .setting-group {
          margin-bottom: 24px;
        }
        .setting-row {
          display: flex;
          align-items: flex-start;
          justify-content: space-between;
          gap: 16px;
        }
        .setting-label {
          display: block;
          font-size: 11px;
          text-transform: uppercase;
          letter-spacing: 1px;
          color: #888;
          margin-bottom: 10px;
        }
        .label-hint {
          text-transform: none;
          letter-spacing: 0;
          font-size: 10px;
          color: #555;
        }
        .access-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 8px;
          margin-bottom: 10px;
        }
        .access-option {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 10px 14px;
          background: transparent;
          border: 1px solid #1a1a3e;
          border-radius: 6px;
          color: #888;
          cursor: pointer;
          font-family: monospace;
          font-size: 13px;
          transition: all 0.15s;
        }
        .access-option:hover {
          border-color: var(--accent);
          color: var(--accent);
          background: rgba(255, 255, 255, 0.02);
        }
        .access-option.selected {
          border-color: var(--accent);
          color: var(--accent);
          background: rgba(255, 255, 255, 0.04);
        }
        .access-icon {
          opacity: 0.8;
        }
        .access-name {
          font-weight: 500;
        }
        .access-desc {
          font-size: 12px;
          margin: 0;
          line-height: 1.5;
        }
        .setting-textarea {
          width: 100%;
          background: #0a0a1e;
          border: 1px solid #1a1a3e;
          color: #e0e0e0;
          padding: 10px 12px;
          border-radius: 6px;
          font-family: monospace;
          font-size: 12px;
          resize: vertical;
          box-sizing: border-box;
          transition: border-color 0.15s;
        }
        .setting-textarea:focus {
          outline: none;
          border-color: #00f0ff;
        }
        .setting-hint {
          font-size: 11px;
          color: #555;
          margin: 4px 0 0;
        }
        .toggle {
          flex-shrink: 0;
          width: 44px;
          height: 24px;
          border-radius: 12px;
          border: none;
          cursor: pointer;
          position: relative;
          transition: background 0.2s;
          margin-top: 2px;
        }
        .toggle.on {
          background: #00f0ff33;
          border: 1px solid #00f0ff;
        }
        .toggle.off {
          background: #1a1a3e;
          border: 1px solid #2a2a5e;
        }
        .toggle-thumb {
          position: absolute;
          top: 3px;
          width: 16px;
          height: 16px;
          border-radius: 50%;
          transition: all 0.2s;
        }
        .toggle.on .toggle-thumb {
          left: 22px;
          background: #00f0ff;
        }
        .toggle.off .toggle-thumb {
          left: 3px;
          background: #555;
        }
        .last-synced {
          font-size: 11px;
          color: #555;
          margin: -12px 0 20px;
        }
        .settings-error {
          color: #ff5555;
          font-size: 12px;
          margin-bottom: 12px;
          background: rgba(255, 50, 50, 0.1);
          border: 1px solid #ff555533;
          padding: 8px 12px;
          border-radius: 4px;
        }
        .save-btn {
          display: flex;
          align-items: center;
          gap: 8px;
          background: rgba(0, 240, 255, 0.08);
          border: 1px solid #00f0ff;
          color: #00f0ff;
          padding: 10px 20px;
          border-radius: 6px;
          cursor: pointer;
          font-family: monospace;
          font-size: 13px;
          font-weight: 500;
          transition: all 0.15s;
        }
        .save-btn:hover:not(:disabled) {
          background: rgba(0, 240, 255, 0.14);
        }
        .save-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  )
}
