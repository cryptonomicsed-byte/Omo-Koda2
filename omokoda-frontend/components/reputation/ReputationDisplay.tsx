'use client';

interface ReputationChange {
  amount: number;
  reason: string;
  timestamp: number;
}

interface ReputationDisplayProps {
  reputation: number;
  tier: number;
  changes?: ReputationChange[];
}

const TIER_COLORS = ['text-gray-400', 'text-blue-400', 'text-green-400', 'text-yellow-400', 'text-orange-400', 'text-purple-400'];
const TIER_NAMES = ['Newborn', 'Curious', 'Creator', 'Builder', 'Architect', 'Sovereign'];

export function ReputationDisplay({ reputation, tier, changes = [] }: ReputationDisplayProps) {
  const safeTier = Math.min(Math.max(tier, 0), 5);
  const reputationPct = (reputation / 100) * 100;

  return (
    <div className="bg-gray-900 border border-gray-700 rounded-lg p-4 space-y-3">
      <div className="flex justify-between items-center">
        <span className="text-sm text-gray-400">Reputation</span>
        <span className={`text-lg font-mono font-bold ${TIER_COLORS[safeTier]}`}>
          {reputation.toFixed(3)}
        </span>
      </div>

      <div className="w-full bg-gray-800 rounded-full h-2">
        <div
          className={`h-2 rounded-full transition-all duration-500 bg-current ${TIER_COLORS[safeTier]}`}
          style={{ width: `${Math.min(reputationPct, 100)}%` }}
        />
      </div>

      <div className="flex justify-between text-xs text-gray-500">
        <span>Tier {safeTier} — {TIER_NAMES[safeTier]}</span>
        <span>{reputation.toFixed(3)} / 100.000</span>
      </div>

      {changes.length > 0 && (
        <div className="space-y-1 max-h-32 overflow-y-auto">
          <div className="text-xs text-gray-500 font-medium">Change Log</div>
          {changes.slice(-10).reverse().map((change, i) => (
            <div key={i} className="flex justify-between text-xs font-mono">
              <span className="text-gray-500 truncate">{change.reason}</span>
              <span className={change.amount >= 0 ? 'text-green-400' : 'text-red-400'}>
                {change.amount >= 0 ? '+' : ''}{change.amount.toFixed(3)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
