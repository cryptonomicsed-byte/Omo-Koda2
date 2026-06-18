'use client'

import { useState } from 'react'
import { AgentChat } from './AgentChat'
import { AgentProfile } from './AgentProfile'
import { AgentTools } from './AgentTools'
import { AgentStats } from './AgentStats'
import { MemoryVaultTab } from './MemoryVaultTab'

type TabId = 'chat' | 'profile' | 'tools' | 'stats' | 'vault'

export function AgentDashboard() {
  const [activeTab, setActiveTab] = useState<TabId>('chat')

  const tabs: { id: TabId; label: string }[] = [
    { id: 'chat', label: 'Chat' },
    { id: 'profile', label: 'Profile' },
    { id: 'tools', label: 'Tools' },
    { id: 'stats', label: 'Statistics' },
    { id: 'vault', label: '🌌 Memory Vault' },
  ]

  return (
    <div className="max-w-6xl mx-auto">
      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-white/10 rounded-lg p-1 backdrop-blur-sm">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 py-2 px-4 rounded-md text-sm font-medium transition-all ${
              activeTab === tab.id
                ? 'bg-white text-gray-900 shadow-lg'
                : 'text-gray-300 hover:text-white hover:bg-white/10'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className={activeTab === 'vault' ? '' : 'bg-white/10 backdrop-blur-sm rounded-lg p-6'}>
        {activeTab === 'chat' && <AgentChat />}
        {activeTab === 'profile' && <AgentProfile />}
        {activeTab === 'tools' && <AgentTools />}
        {activeTab === 'stats' && <AgentStats />}
        {activeTab === 'vault' && <MemoryVaultTab isOwner={true} />}
      </div>
    </div>
  )
}