'use client'

import { useState, useRef, useEffect } from 'react'
import { vaultInsertKnowledge, vaultGetConfig, vaultSetConfig, type VaultConfig } from '@/lib/api'

interface Message {
  role: 'user' | 'agent' | 'tool'
  content: string
  tokens?: number
  tool?: string
  ms?: number
}

const INITIAL: Message[] = [
  { role: 'agent', content: 'Hello. I am your Omo-Koda agent. How can I help you today?' },
]

type Tab = 'chat' | 'knowledge' | 'settings'

export default function HumanPage() {
  const [tab, setTab] = useState<Tab>('chat')

  // ── Chat state ────────────────────────────────────────────────────────────
  const [messages, setMessages] = useState<Message[]>(INITIAL)
  const [input, setInput] = useState('')
  const [streaming, setStreaming] = useState(false)
  const [privacy, setPrivacy] = useState(false)
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  function send() {
    const text = input.trim()
    if (!text || streaming) return
    setInput('')
    const userMsg: Message = { role: 'user', content: text }
    setMessages((prev) => [...prev, userMsg])
    setStreaming(true)

    const response = `Understood. Processing your request: "${text}". This is a simulated response demonstrating the streaming chat interface. In production, tokens would arrive via SSE from the Axum backend.`
    let idx = 0
    const agentMsg: Message = { role: 'agent', content: '', tokens: response.length }
    setMessages((prev) => [...prev, agentMsg])
    const iv = setInterval(() => {
      idx++
      setMessages((prev) => {
        const updated = [...prev]
        updated[updated.length - 1] = { ...agentMsg, content: response.slice(0, idx) }
        return updated
      })
      if (idx >= response.length) {
        clearInterval(iv)
        setStreaming(false)
      }
    }, 18)
  }

  const totalInput = 8240
  const totalOutput = 3120
  const maxCtx = 200000
  const ctxUsed = totalInput + totalOutput
  const ctxPct = Math.round((ctxUsed / maxCtx) * 100)

  // ── Knowledge form state ──────────────────────────────────────────────────
  const [subject, setSubject] = useState('')
  const [predicate, setPredicate] = useState('')
  const [obj, setObj] = useState('')
  const [confidence, setConfidence] = useState('1.0')
  const [knowledgeStatus, setKnowledgeStatus] = useState<'idle' | 'saving' | 'saved' | 'error'>('idle')

  async function submitKnowledge(e: React.FormEvent) {
    e.preventDefault()
    if (!subject || !predicate || !obj) return
    setKnowledgeStatus('saving')
    try {
      await vaultInsertKnowledge({
        subject: subject.trim(),
        predicate: predicate.trim(),
        object: obj.trim(),
        confidence: parseFloat(confidence) || 1.0,
      })
      setKnowledgeStatus('saved')
      setSubject('')
      setPredicate('')
      setObj('')
      setConfidence('1.0')
      setTimeout(() => setKnowledgeStatus('idle'), 2000)
    } catch {
      setKnowledgeStatus('error')
      setTimeout(() => setKnowledgeStatus('idle'), 3000)
    }
  }

  // ── Settings state ────────────────────────────────────────────────────────
  const [vaultCfg, setVaultCfg] = useState<VaultConfig | null>(null)
  const [cfgLoading, setCfgLoading] = useState(false)
  const [cfgSaved, setCfgSaved] = useState(false)

  useEffect(() => {
    if (tab === 'settings') {
      vaultGetConfig()
        .then(setVaultCfg)
        .catch(() => setVaultCfg({ access_level: 'private', auto_export: false }))
    }
  }, [tab])

  async function saveVaultCfg() {
    if (!vaultCfg) return
    setCfgLoading(true)
    try {
      await vaultSetConfig(vaultCfg)
      setCfgSaved(true)
      setTimeout(() => setCfgSaved(false), 2000)
    } catch {}
    finally { setCfgLoading(false) }
  }

  return (
    <div className="flex flex-col h-[calc(100vh-56px)] max-w-3xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-3 border-b border-border-DEFAULT">
        <div className="flex items-center gap-3">
          <span className="text-xl text-brand-400">◎</span>
          <div>
            <h1 className="text-sm font-bold text-white">The Human</h1>
            <p className="text-xs text-zinc-600">Agent interface · vault · settings</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {(['chat', 'knowledge', 'settings'] as Tab[]).map((t) => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`text-xs px-3 py-1.5 rounded-lg border transition-colors capitalize ${
                tab === t
                  ? 'bg-brand-600/20 border-brand-600/40 text-brand-400'
                  : 'border-border-DEFAULT text-zinc-400 hover:text-white'
              }`}
            >
              {t}
            </button>
          ))}
        </div>
      </div>

      {/* ── Chat Tab ──────────────────────────────────────────────────────── */}
      {tab === 'chat' && (
        <>
          {/* Context gauge */}
          <div className="px-6 py-2 border-b border-border-subtle">
            <div className="flex items-center justify-between text-xs text-zinc-600 mb-1">
              <span>Context window</span>
              <span className="font-mono">
                {ctxUsed.toLocaleString()} / {maxCtx.toLocaleString()} tokens · {ctxPct}%
              </span>
            </div>
            <div className="h-1 w-full bg-surface-4 rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full transition-all ${
                  ctxPct > 85 ? 'bg-red-500' : ctxPct > 60 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${ctxPct}%` }}
              />
            </div>
          </div>

          {/* Messages */}
          <div className="flex-1 overflow-y-auto px-6 py-4 space-y-4">
            {messages.map((msg, i) => (
              <div key={i} className={`flex gap-3 ${msg.role === 'user' ? 'justify-end' : ''}`}>
                {msg.role !== 'user' && (
                  <div className="w-6 h-6 rounded-full bg-brand-600 flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-xs text-black font-bold">A</span>
                  </div>
                )}
                <div
                  className={`max-w-[80%] rounded-xl px-4 py-2.5 text-sm leading-relaxed ${
                    msg.role === 'user'
                      ? 'bg-brand-600 text-black ml-auto'
                      : msg.role === 'tool'
                      ? 'bg-surface-3 border border-border-subtle font-mono text-xs text-green-400'
                      : 'bg-surface-2 border border-border-DEFAULT text-zinc-200'
                  }`}
                >
                  {msg.role === 'tool' && (
                    <div className="text-zinc-500 mb-1">⚙ {msg.tool} · {msg.ms}ms</div>
                  )}
                  {msg.content}
                  {i === messages.length - 1 && streaming && (
                    <span className="inline-block w-0.5 h-4 bg-brand-400 ml-0.5 animate-pulse align-text-bottom" />
                  )}
                </div>
                {msg.role === 'user' && (
                  <div className="w-6 h-6 rounded-full bg-zinc-700 flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-xs text-zinc-300">U</span>
                  </div>
                )}
              </div>
            ))}
            <div ref={bottomRef} />
          </div>

          {/* Input */}
          <div className="px-6 py-4 border-t border-border-DEFAULT">
            <div className="flex gap-2">
              <button
                onClick={() => setPrivacy((v) => !v)}
                className="text-xs px-2 py-1 rounded border border-border-DEFAULT text-zinc-400 hover:text-white transition-colors"
              >
                {privacy ? '🔒' : '🔓'}
              </button>
              <input
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && send()}
                placeholder={streaming ? 'Agent is responding…' : 'Message the agent…'}
                disabled={streaming}
                className="flex-1 bg-surface-2 border border-border-DEFAULT rounded-lg px-4 py-2.5 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-brand-600 disabled:opacity-50 transition-colors"
              />
              <button
                onClick={send}
                disabled={!input.trim() || streaming}
                className="px-4 py-2.5 rounded-lg bg-brand-600 text-black text-sm font-medium hover:bg-brand-400 disabled:opacity-40 transition-colors"
              >
                Send
              </button>
            </div>
            <div className="flex justify-between mt-1.5 text-xs text-zinc-700">
              <span>Enter to send · Shift+Enter for newline</span>
              <span className="font-mono">{input.length} chars</span>
            </div>
          </div>
        </>
      )}

      {/* ── Knowledge Tab ─────────────────────────────────────────────────── */}
      {tab === 'knowledge' && (
        <div className="flex-1 overflow-y-auto px-6 py-6">
          <div className="max-w-lg">
            <h2 className="text-sm font-semibold text-zinc-300 mb-1">Add Knowledge Triple</h2>
            <p className="text-xs text-zinc-600 mb-6">
              Teach the agent a fact. Subject → Predicate → Object triples are stored in the vault knowledge graph.
            </p>

            <form onSubmit={submitKnowledge} className="space-y-4">
              <div>
                <label className="block text-xs text-zinc-500 mb-1.5">Subject</label>
                <input
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  placeholder="e.g. Ọmọ Kọ́dà"
                  required
                  className="w-full bg-surface-2 border border-border-DEFAULT rounded-lg px-4 py-2.5 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-brand-600"
                />
              </div>

              <div>
                <label className="block text-xs text-zinc-500 mb-1.5">Predicate</label>
                <input
                  value={predicate}
                  onChange={(e) => setPredicate(e.target.value)}
                  placeholder="e.g. is-a, governs, related-to"
                  required
                  className="w-full bg-surface-2 border border-border-DEFAULT rounded-lg px-4 py-2.5 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-brand-600"
                />
              </div>

              <div>
                <label className="block text-xs text-zinc-500 mb-1.5">Object</label>
                <input
                  value={obj}
                  onChange={(e) => setObj(e.target.value)}
                  placeholder="e.g. sovereign agent operating system"
                  required
                  className="w-full bg-surface-2 border border-border-DEFAULT rounded-lg px-4 py-2.5 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-brand-600"
                />
              </div>

              <div>
                <label className="block text-xs text-zinc-500 mb-1.5">Confidence (0–1)</label>
                <input
                  type="number"
                  value={confidence}
                  onChange={(e) => setConfidence(e.target.value)}
                  min="0"
                  max="1"
                  step="0.1"
                  className="w-full bg-surface-2 border border-border-DEFAULT rounded-lg px-4 py-2.5 text-sm text-white placeholder-zinc-600 focus:outline-none focus:border-brand-600"
                />
              </div>

              <button
                type="submit"
                disabled={knowledgeStatus === 'saving'}
                className="w-full py-2.5 rounded-lg bg-brand-600 text-black text-sm font-medium hover:bg-brand-400 disabled:opacity-40 transition-colors"
              >
                {knowledgeStatus === 'saving' ? 'Saving…' :
                 knowledgeStatus === 'saved' ? 'Saved ✓' :
                 knowledgeStatus === 'error' ? 'Error — try again' :
                 'Add to Knowledge Graph'}
              </button>
            </form>
          </div>
        </div>
      )}

      {/* ── Settings Tab ──────────────────────────────────────────────────── */}
      {tab === 'settings' && (
        <div className="flex-1 overflow-y-auto px-6 py-6">
          <div className="max-w-lg">
            <h2 className="text-sm font-semibold text-zinc-300 mb-1">Vault Settings</h2>
            <p className="text-xs text-zinc-600 mb-6">
              Configure vault access and auto-export behavior.
            </p>

            {!vaultCfg && (
              <p className="text-xs text-zinc-600">Loading…</p>
            )}

            {vaultCfg && (
              <div className="space-y-4">
                <div>
                  <label className="block text-xs text-zinc-500 mb-1.5">Access Level</label>
                  <select
                    value={vaultCfg.access_level}
                    onChange={(e) => setVaultCfg({ ...vaultCfg, access_level: e.target.value })}
                    className="w-full bg-surface-2 border border-border-DEFAULT rounded-lg px-4 py-2.5 text-sm text-white focus:outline-none focus:border-brand-600"
                  >
                    <option value="private">Private</option>
                    <option value="followers">Followers only</option>
                    <option value="public">Public</option>
                  </select>
                </div>

                <div className="flex items-center justify-between p-4 bg-surface-3 rounded-lg">
                  <div>
                    <div className="text-sm text-zinc-300">Auto-export</div>
                    <div className="text-xs text-zinc-600 mt-0.5">
                      Append each completed thought to vault traces automatically
                    </div>
                  </div>
                  <button
                    onClick={() => setVaultCfg({ ...vaultCfg, auto_export: !vaultCfg.auto_export })}
                    className={`w-10 h-6 rounded-full transition-colors ${
                      vaultCfg.auto_export ? 'bg-brand-600' : 'bg-surface-4'
                    }`}
                  >
                    <span
                      className={`block w-4 h-4 rounded-full bg-white shadow transition-transform mx-1 ${
                        vaultCfg.auto_export ? 'translate-x-4' : 'translate-x-0'
                      }`}
                    />
                  </button>
                </div>

                <button
                  onClick={saveVaultCfg}
                  disabled={cfgLoading}
                  className="w-full py-2.5 rounded-lg bg-brand-600 text-black text-sm font-medium hover:bg-brand-400 disabled:opacity-40 transition-colors"
                >
                  {cfgLoading ? 'Saving…' : cfgSaved ? 'Saved ✓' : 'Save Settings'}
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
