'use client';

import { useState, useEffect } from 'react';

interface GardenReceipt {
  receipt_id: string;
  agent_id: string;
  action: string;
  timestamp: number;
  tip_total_sui?: number;
}

export default function GardenPage() {
  const [receipts, setReceipts] = useState<GardenReceipt[]>([]);
  const [filter, setFilter] = useState('');

  useEffect(() => {
    // Stub: in production, fetch from Garden index (Walrus + Sui events)
    setReceipts([
      { receipt_id: 'rcpt_001', agent_id: 'oracle-prime', action: 'think', timestamp: Date.now() - 3600000, tip_total_sui: 0.05 },
      { receipt_id: 'rcpt_002', agent_id: 'weaver-7', action: 'act:code_runner', timestamp: Date.now() - 7200000, tip_total_sui: 0.12 },
      { receipt_id: 'rcpt_003', agent_id: 'oracle-prime', action: 'act:web_search', timestamp: Date.now() - 10800000 },
    ]);
  }, []);

  const filtered = receipts.filter(r =>
    !filter || r.agent_id.includes(filter) || r.action.includes(filter)
  );

  return (
    <main className="min-h-screen bg-black text-white p-8 font-mono">
      <div className="max-w-3xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-bold text-purple-400">The Garden</h1>
          <div className="text-xs text-gray-600">public cognition marketplace</div>
        </div>

        <input
          className="w-full bg-gray-950 border border-gray-800 rounded px-3 py-2 text-sm text-white placeholder-gray-700 outline-none focus:border-purple-500"
          placeholder="filter by agent or action..."
          value={filter}
          onChange={e => setFilter(e.target.value)}
        />

        <div className="space-y-2">
          {filtered.map(r => (
            <div key={r.receipt_id} className="border border-gray-800 rounded p-3 hover:border-gray-600 transition-colors">
              <div className="flex justify-between items-start">
                <div className="space-y-0.5">
                  <div className="text-sm text-green-400">{r.agent_id}</div>
                  <div className="text-xs text-gray-500">{r.action}</div>
                  <div className="text-xs text-gray-700 font-mono">{r.receipt_id}</div>
                </div>
                <div className="text-right space-y-0.5">
                  {r.tip_total_sui && (
                    <div className="text-xs text-yellow-400">{r.tip_total_sui} SUI</div>
                  )}
                  <div className="text-xs text-gray-700">
                    {new Date(r.timestamp).toLocaleTimeString()}
                  </div>
                </div>
              </div>
            </div>
          ))}
          {filtered.length === 0 && (
            <div className="text-center text-gray-700 py-8">The Garden is empty.</div>
          )}
        </div>
      </div>
    </main>
  );
}
