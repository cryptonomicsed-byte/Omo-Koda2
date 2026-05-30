'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useUIStore } from '@/lib/store/ui'

const NAV_ITEMS = [
  { href: '/gate', label: 'The Gate', icon: '◈', color: 'text-gate', desc: 'Identity & Birth' },
  { href: '/mirror', label: 'The Mirror', icon: '◎', color: 'text-mirror', desc: 'Memory & Context' },
  { href: '/ocean', label: 'The Ocean', icon: '≋', color: 'text-ocean', desc: 'Swarm & Network' },
  { href: '/balance', label: 'The Balance', icon: '⊖', color: 'text-balance', desc: 'Ethics & Gates' },
  { href: '/forge', label: 'The Forge', icon: '⊕', color: 'text-forge', desc: 'Tools & Execution' },
  { href: '/storm', label: 'The Storm', icon: '∿', color: 'text-storm', desc: 'Time & Flow' },
  { href: '/thunder', label: 'The Thunder', icon: '⚡', color: 'text-thunder', desc: 'Economy & Justice' },
  { href: '/vitality', label: 'The Vitality', icon: '◉', color: 'text-brand-400', desc: 'Plugins & Nodes' },
  { href: '/human', label: 'The Human', icon: '◌', color: 'text-zinc-300', desc: 'Chat & Settings' },
]

export function Sidebar() {
  const pathname = usePathname()
  const { sidebarOpen, toggleSidebar } = useUIStore()

  return (
    <aside
      className={`flex flex-col bg-surface-1 border-r border-border-DEFAULT transition-all duration-200 ${
        sidebarOpen ? 'w-56' : 'w-14'
      } min-h-screen flex-shrink-0`}
    >
      {/* Toggle */}
      <button
        onClick={toggleSidebar}
        className="flex items-center justify-center h-12 w-full border-b border-border-DEFAULT text-zinc-500 hover:text-brand-400 transition-colors"
        title={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
      >
        {sidebarOpen ? '◀' : '▶'}
      </button>

      {/* Nav items */}
      <nav className="flex flex-col gap-0.5 p-2 flex-1">
        {NAV_ITEMS.map(({ href, label, icon, color, desc }) => {
          const active = pathname === href || pathname.startsWith(href + '/')
          return (
            <Link
              key={href}
              href={href}
              className={`flex items-center gap-3 rounded-md px-2 py-2 text-sm transition-colors group ${
                active
                  ? 'bg-surface-3 text-white'
                  : 'text-zinc-400 hover:bg-surface-3 hover:text-white'
              }`}
              title={!sidebarOpen ? label : undefined}
            >
              <span className={`text-base flex-shrink-0 ${color}`}>{icon}</span>
              {sidebarOpen && (
                <div className="flex flex-col min-w-0">
                  <span className="font-medium truncate">{label}</span>
                  <span className="text-xs text-zinc-600 truncate">{desc}</span>
                </div>
              )}
            </Link>
          )
        })}
      </nav>

      {/* Settings link */}
      {sidebarOpen && (
        <div className="border-t border-border-DEFAULT p-2">
          <Link
            href="/settings"
            className="flex items-center gap-3 rounded-md px-2 py-2 text-sm text-zinc-500 hover:text-white hover:bg-surface-3 transition-colors"
          >
            <span>⚙</span>
            <span>Settings</span>
          </Link>
        </div>
      )}
    </aside>
  )
}
