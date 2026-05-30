'use client'

import { create } from 'zustand'

export type Principle =
  | 'gate'
  | 'mirror'
  | 'ocean'
  | 'balance'
  | 'forge'
  | 'storm'
  | 'thunder'
  | 'vitality'
  | 'human'

export interface UIState {
  sidebarOpen: boolean
  commandPaletteOpen: boolean
  activePrincipal: Principle
  // Actions
  setSidebarOpen: (open: boolean) => void
  toggleSidebar: () => void
  setCommandPaletteOpen: (open: boolean) => void
  toggleCommandPalette: () => void
  setActivePrincipal: (p: Principle) => void
}

export const useUIStore = create<UIState>((set) => ({
  sidebarOpen: true,
  commandPaletteOpen: false,
  activePrincipal: 'gate',

  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
  toggleCommandPalette: () =>
    set((s) => ({ commandPaletteOpen: !s.commandPaletteOpen })),
  setActivePrincipal: (activePrincipal) => set({ activePrincipal }),
}))
