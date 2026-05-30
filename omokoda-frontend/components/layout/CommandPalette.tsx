'use client'

import { useEffect } from 'react'
import { Command } from 'cmdk'
import { useUIStore } from '@/lib/store/ui'
import { useRouter } from 'next/navigation'

const COMMANDS = [
  { group: 'Navigate', items: [
    { label: 'The Gate — Identity & Birth', href: '/gate', icon: '◈' },
    { label: 'The Mirror — Memory', href: '/mirror', icon: '◎' },
    { label: 'The Ocean — Swarm', href: '/ocean', icon: '≋' },
    { label: 'The Balance — Ethics', href: '/balance', icon: '⊖' },
    { label: 'The Forge — Tools', href: '/forge', icon: '⊕' },
    { label: 'The Storm — Temporal', href: '/storm', icon: '∿' },
    { label: 'The Thunder — Economy', href: '/thunder', icon: '⚡' },
    { label: 'The Vitality — Plugins', href: '/vitality', icon: '◉' },
    { label: 'The Human — Chat', href: '/human', icon: '◌' },
  ]},
  { group: 'Actions', items: [
    { label: 'Create new agent', href: '/gate?action=birth', icon: '+' },
    { label: 'Browse sessions', href: '/human?tab=sessions', icon: '◷' },
    { label: 'View receipts', href: '/thunder?tab=receipts', icon: '◑' },
    { label: 'Tool catalog', href: '/forge?tab=tools', icon: '⚙' },
    { label: 'Settings', href: '/settings', icon: '⊞' },
  ]},
]

export function CommandPalette() {
  const { commandPaletteOpen, setCommandPaletteOpen } = useUIStore()
  const router = useRouter()

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        setCommandPaletteOpen(true)
      }
      if (e.key === 'Escape') {
        setCommandPaletteOpen(false)
      }
    }
    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [setCommandPaletteOpen])

  if (!commandPaletteOpen) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[20vh]"
      onClick={() => setCommandPaletteOpen(false)}
    >
      <div
        className="w-full max-w-lg bg-surface-2 border border-border-strong rounded-xl shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <Command className="font-mono">
          <div className="flex items-center border-b border-border-DEFAULT px-4">
            <span className="text-zinc-500 mr-2">⌘</span>
            <Command.Input
              placeholder="Type a command or search..."
              className="flex-1 bg-transparent py-4 text-sm text-white placeholder-zinc-600 outline-none"
              autoFocus
            />
          </div>
          <Command.List className="max-h-80 overflow-y-auto p-2">
            <Command.Empty className="py-8 text-center text-sm text-zinc-600">
              No results found.
            </Command.Empty>

            {COMMANDS.map(({ group, items }) => (
              <Command.Group
                key={group}
                heading={
                  <span className="px-2 py-1 text-xs text-zinc-600 uppercase tracking-wider">
                    {group}
                  </span>
                }
              >
                {items.map(({ label, href, icon }) => (
                  <Command.Item
                    key={href}
                    value={label}
                    onSelect={() => {
                      router.push(href)
                      setCommandPaletteOpen(false)
                    }}
                    className="flex items-center gap-3 px-3 py-2.5 rounded-md text-sm text-zinc-300 cursor-pointer data-[selected=true]:bg-surface-4 data-[selected=true]:text-white transition-colors"
                  >
                    <span className="text-brand-400 w-5 text-center">{icon}</span>
                    <span>{label}</span>
                  </Command.Item>
                ))}
              </Command.Group>
            ))}
          </Command.List>

          <div className="border-t border-border-DEFAULT px-4 py-2 flex gap-4 text-xs text-zinc-600">
            <span>↑↓ navigate</span>
            <span>↵ select</span>
            <span>Esc close</span>
          </div>
        </Command>
      </div>
    </div>
  )
}
