'use client'

import { useState } from 'react'
import { Agent } from '@/types'

export function AgentProfile() {
  // Mock data - in real app, this would come from the backend
  const [agent] = useState<Agent>({
    id: 'agent-001',
    name: 'Luna',
    dnaFingerprint: 'a1b2c3d4e5f6...',
    primaryOdu: 42,
    birthTimestamp: Date.now() - 86400000, // 1 day ago
    reputation: 1250,
    tier: 2,
    owner: '0x1234...abcd',
  })

  const getTierColor = (tier: number) => {
    switch (tier) {
      case 0: return 'text-gray-400'
      case 1: return 'text-green-400'
      case 2: return 'text-blue-400'
      case 3: return 'text-purple-400'
      case 4: return 'text-yellow-400'
      default: return 'text-red-400'
    }
  }

  const getTierName = (tier: number) => {
    switch (tier) {
      case 0: return 'Initiate'
      case 1: return 'Apprentice'
      case 2: return 'Adept'
      case 3: return 'Master'
      case 4: return 'Oracle'
      default: return 'Unknown'
    }
  }

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-white">Agent Profile</h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Basic Info */}
        <div className="bg-gray-800/50 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Identity</h3>
          <div className="space-y-3">
            <div>
              <label className="text-sm text-gray-400">Name</label>
              <p className="text-white font-medium">{agent.name}</p>
            </div>
            <div>
              <label className="text-sm text-gray-400">ID</label>
              <p className="text-white font-mono text-sm">{agent.id}</p>
            </div>
            <div>
              <label className="text-sm text-gray-400">Owner</label>
              <p className="text-white font-mono text-sm">{agent.owner}</p>
            </div>
            <div>
              <label className="text-sm text-gray-400">Birth Date</label>
              <p className="text-white">
                {new Date(agent.birthTimestamp).toLocaleDateString()}
              </p>
            </div>
          </div>
        </div>

        {/* Stats */}
        <div className="bg-gray-800/50 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Statistics</h3>
          <div className="space-y-4">
            <div>
              <div className="flex justify-between items-center mb-1">
                <label className="text-sm text-gray-400">Reputation</label>
                <span className="text-white font-medium">{agent.reputation}</span>
              </div>
              <div className="w-full bg-gray-700 rounded-full h-2">
                <div
                  className="bg-gradient-to-r from-green-400 to-blue-500 h-2 rounded-full"
                  style={{ width: `${Math.min((agent.reputation / 2000) * 100, 100)}%` }}
                ></div>
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-1">
                <label className="text-sm text-gray-400">Tier</label>
                <span className={`font-medium ${getTierColor(agent.tier)}`}>
                  {getTierName(agent.tier)} ({agent.tier})
                </span>
              </div>
              <div className="flex space-x-1">
                {[0, 1, 2, 3, 4].map((tier) => (
                  <div
                    key={tier}
                    className={`h-3 flex-1 rounded ${
                      tier <= agent.tier
                        ? 'bg-gradient-to-r from-yellow-400 to-orange-500'
                        : 'bg-gray-600'
                    }`}
                  ></div>
                ))}
              </div>
            </div>

            <div>
              <label className="text-sm text-gray-400">Primary Odu</label>
              <p className="text-white font-medium">{agent.primaryOdu}/256</p>
            </div>
          </div>
        </div>
      </div>

      {/* DNA Fingerprint */}
      <div className="bg-gray-800/50 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">DNA Fingerprint</h3>
        <div className="bg-gray-900 rounded p-4">
          <code className="text-green-400 text-sm break-all">
            {agent.dnaFingerprint}
          </code>
        </div>
        <p className="text-xs text-gray-400 mt-2">
          This unique fingerprint ensures your agent&apos;s identity on the blockchain.
        </p>
      </div>
    </div>
  )
}
