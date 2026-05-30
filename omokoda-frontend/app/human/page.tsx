'use client'

import { useState, useRef, useEffect } from 'react'

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

export default function HumanPage() {
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

    // Simulate streaming agent response
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

  return (
    <div className="flex flex-col h-[calc(100vh-56px)] max-w-3xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between px-6 py-3 border-b border-border-DEFAULT">
        <div className="flex items-center gap-3">
          <span className="text-xl text-brand-400">◎</span>
          <div>
            <h1 className="text-sm font-bold text-white">The Human</h1>
            <p className="text-xs text-zinc-600">Agent chat · streaming · context-aware</p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <button
            onClick={() => setPrivacy((v) => !v)}
            className="text-xs px-2 py-1 rounded border border-border-DEFAULT text-zinc-400 hover:text-white transition-colors"
          >
            {privacy ? '🔒 private' : '🔓 public'}
          </button>
          {streaming && (
            <button
              onClick={() => setStreaming(false)}
              className="text-xs px-2 py-1 rounded bg-red-900/30 border border-red-800 text-red-400 hover:bg-red-900/50 transition-colors"
            >
              ■ stop
            </button>
          )}
        </div>
      </div>

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
        <div className="flex gap-4 mt-1 text-xs text-zinc-700 font-mono">
          <span>In: {totalInput.toLocaleString()}</span>
          <span>Out: {totalOutput.toLocaleString()}</span>
          <span>Cost: $0.04</span>
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
    </div>
  )
}
