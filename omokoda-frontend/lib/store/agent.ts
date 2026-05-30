'use client'

import { create } from 'zustand'

export type StreamingState = 'idle' | 'thinking' | 'acting' | 'done'

export interface TokenUsage {
  input: number
  output: number
  cache: number
  total: number
  maxContext: number
}

export interface AgentState {
  activeAgent: string | null
  sessionId: string | null
  privacyMode: boolean
  tier: number
  reputation: number
  streamingState: StreamingState
  tokens: TokenUsage
  // Actions
  setActiveAgent: (name: string) => void
  setSessionId: (id: string) => void
  setPrivacyMode: (enabled: boolean) => void
  setTier: (tier: number) => void
  setReputation: (rep: number) => void
  setStreamingState: (state: StreamingState) => void
  updateTokens: (tokens: Partial<TokenUsage>) => void
  reset: () => void
}

const defaultTokens: TokenUsage = {
  input: 0,
  output: 0,
  cache: 0,
  total: 0,
  maxContext: 200000,
}

export const useAgentStore = create<AgentState>((set) => ({
  activeAgent: null,
  sessionId: null,
  privacyMode: false,
  tier: 1,
  reputation: 0,
  streamingState: 'idle',
  tokens: defaultTokens,

  setActiveAgent: (name) => set({ activeAgent: name }),
  setSessionId: (id) => set({ sessionId: id }),
  setPrivacyMode: (enabled) => set({ privacyMode: enabled }),
  setTier: (tier) => set({ tier }),
  setReputation: (reputation) => set({ reputation }),
  setStreamingState: (streamingState) => set({ streamingState }),
  updateTokens: (tokens) =>
    set((state) => ({ tokens: { ...state.tokens, ...tokens } })),
  reset: () =>
    set({
      activeAgent: null,
      sessionId: null,
      privacyMode: false,
      tier: 1,
      reputation: 0,
      streamingState: 'idle',
      tokens: defaultTokens,
    }),
}))
