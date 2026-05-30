'use client'

import Link from 'next/link'
import { useAgentStore } from '@/lib/store/agent'
import { useUIStore } from '@/lib/store/ui'
import { TokenGauge } from '@/components/ui/TokenGauge'

export function Header() {
  const { activeAgent, tier, tokens, privacyMode, setPrivacyMode } = useAgentStore()
  const { toggleCommandPalette } = useUIStore()

  const tierStars = '★'.repeat(Math.min(tier, 7))

  return (
    <header className="flex items-center gap-4 h-14 px-4 bg-surface-1 border-b border-border-DEFAULT flex-shrink-0">
      {/* Logo */}
      <Link href="/" className="flex items-center gap-2 mr-2">
        <span className="text-brand-400 text-lg font-mono font-bold tracking-tight">
          OMO-KODA
        </span>
      </Link>

      {/* Agent info */}
      <div className="flex items-center gap-2 text-sm">
        <span className="text-yellow-400 font-mono text-xs">{tierStars}</span>
        <span className="text-zinc-300 font-medium">
          {activeAgent ?? 'No Agent'}
        </span>
        {privacyMode && (
          <span className="text-xs bg-purple-900/40 text-purple-300 px-1.5 py-0.5 rounded font-mono">
            private
          </span>
        )}
      </div>

      {/* Context gauge */}
      <div className="flex-1 max-w-xs hidden md:block">
        <TokenGauge
          used={tokens.total}
          max={tokens.maxContext}
          showLabel={false}
        />
      </div>

      <div className="flex-1" />

      {/* Command palette trigger */}
      <button
        onClick={toggleCommandPalette}
        className="flex items-center gap-2 text-sm text-zinc-500 hover:text-zinc-300 bg-surface-3 border border-border-DEFAULT rounded-md px-3 py-1.5 transition-colors font-mono"
      >
        <span>⌘K</span>
        <span className="hidden sm:inline text-xs">Commands</span>
      </button>

      {/* Privacy toggle */}
      <button
        onClick={() => setPrivacyMode(!privacyMode)}
        className={`text-sm px-2 py-1.5 rounded-md border transition-colors font-mono ${
          privacyMode
            ? 'bg-purple-900/30 border-purple-700 text-purple-300'
            : 'bg-surface-3 border-border-DEFAULT text-zinc-500 hover:text-zinc-300'
        }`}
        title="Toggle privacy mode"
      >
        {privacyMode ? '🔒' : '🔓'}
      </button>
    </header>
  )
}
