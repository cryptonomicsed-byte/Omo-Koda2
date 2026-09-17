import React, { useState } from "react";
import { ContractClient } from "../../../oso-sdk-ts/src/ContractClient.ts";
import type { PublicAgent } from "../services/agentApi.ts";

export interface HireRecord {
  agent: PublicAgent;
  contract_id: string;
  role: string;
  duration_days: number;
  budget_ase: number;
  delegation_scope: string;
  hired_at: string;
  active: boolean;
}

interface Props {
  agent: PublicAgent;
  hirerId: string;
  onHired: (record: HireRecord) => void;
  onCancel: () => void;
}

const DELEGATION_SCOPES = [
  { value: "read-only",    label: "Read-only — can read data, no actions" },
  { value: "task-scoped",  label: "Task-scoped — act within this job only" },
  { value: "full-delegate",label: "Full delegate — act on your behalf" },
];

export function HireForm({ agent, hirerId, onHired, onCancel }: Props) {
  const [role, setRole] = useState("");
  const [durationDays, setDurationDays] = useState(7);
  const [budgetAse, setBudgetAse] = useState(100);
  const [delegationScope, setDelegationScope] = useState("task-scoped");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const client = new ContractClient("http://localhost:3000", hirerId);

      // 1. Deploy an Agent contract
      const contract = await client.deploy({
        class: "agent",
        owner: hirerId,
        metadata: { role, duration_days: durationDays, budget_ase: budgetAse },
      });

      // 2. Call hire
      await client.agentHire(contract.id, agent.agent_id, {
        role,
        duration_days: durationDays,
        budget_ase: budgetAse,
      });

      // 3. Delegate capability
      await client.agentDelegate(contract.id, agent.agent_id, delegationScope);

      onHired({
        agent,
        contract_id: contract.id,
        role,
        duration_days: durationDays,
        budget_ase: budgetAse,
        delegation_scope: delegationScope,
        hired_at: new Date().toISOString(),
        active: true,
      });
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="w-full max-w-md rounded-2xl border border-gray-800 bg-gray-900 p-6">
        <h2 className="text-lg font-bold mb-1">Hire Agent</h2>
        <p className="text-sm text-gray-400 mb-5">
          {agent.name} · Tier {agent.tier} · Rep{" "}
          {(agent.reputation_score * 100).toFixed(0)}
        </p>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Role / task description</label>
            <input
              type="text"
              value={role}
              onChange={(e) => setRole(e.target.value)}
              placeholder="e.g. Index my knowledge graph, run nightly fine-tune"
              required
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Duration (days)</label>
              <input
                type="number"
                min={1}
                max={365}
                value={durationDays}
                onChange={(e) =>
                  setDurationDays(parseInt(e.target.value, 10) || 1)
                }
                required
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Budget (ASE)</label>
              <input
                type="number"
                min={1}
                value={budgetAse}
                onChange={(e) =>
                  setBudgetAse(parseInt(e.target.value, 10) || 1)
                }
                required
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Delegation scope</label>
            <select
              value={delegationScope}
              onChange={(e) => setDelegationScope(e.target.value)}
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            >
              {DELEGATION_SCOPES.map((s) => (
                <option key={s.value} value={s.value}>
                  {s.label}
                </option>
              ))}
            </select>
          </div>

          {error && (
            <p className="rounded-lg bg-red-950/50 border border-red-700 px-3 py-2 text-sm text-red-400">
              {error}
            </p>
          )}

          <div className="flex gap-3 pt-1">
            <button
              type="button"
              onClick={onCancel}
              className="flex-1 rounded-lg border border-gray-700 py-2 text-sm hover:bg-gray-800"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              className="flex-1 rounded-lg bg-ose-600 py-2 text-sm font-semibold hover:bg-ose-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submitting ? "Hiring…" : "Hire & Delegate"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
