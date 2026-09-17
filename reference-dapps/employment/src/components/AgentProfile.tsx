import React, { useEffect, useState } from "react";
import { getAgentProfile, type PublicAgent } from "../services/agentApi.ts";
import { AgentClient } from "../../../oso-sdk-ts/src/AgentClient.ts";
import type { HireRecord } from "./HireForm.tsx";

interface Props {
  agentId: string;
  hires: HireRecord[];
}

export function AgentProfile({ agentId, hires }: Props) {
  const [profile, setProfile] = useState<PublicAgent | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [registering, setRegistering] = useState(false);
  const [registered, setRegistered] = useState(false);

  const load = () => {
    setLoading(true);
    setError(null);
    getAgentProfile(agentId)
      .then(setProfile)
      .catch((e: unknown) =>
        setError(e instanceof Error ? e.message : String(e))
      )
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    load();
  }, [agentId]);

  const handleRegister = async () => {
    if (!profile) return;
    setRegistering(true);
    try {
      const client = new AgentClient("http://localhost:3000", agentId);
      await client.register({ agent_id: agentId, tier: profile.tier, skills: profile.skills });
      setRegistered(true);
    } catch {
      // best-effort
    } finally {
      setRegistering(false);
    }
  };

  if (loading) {
    return <p className="py-12 text-center text-ose-400 animate-pulse">Loading profile…</p>;
  }

  if (error) {
    return (
      <div className="rounded-lg border border-red-700 bg-red-950/40 p-6 text-center">
        <p className="text-red-400 mb-3">{error}</p>
        <button
          onClick={load}
          className="rounded bg-red-700 px-4 py-1.5 text-sm hover:bg-red-600"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!profile) return null;

  const completedHires = hires.filter((h) => !h.active).length;
  const activeHires = hires.filter((h) => h.active).length;

  return (
    <div className="max-w-lg mx-auto flex flex-col gap-5">
      {/* Identity card */}
      <div className="rounded-xl border border-gray-800 bg-gray-900 p-6">
        <div className="flex items-start justify-between mb-4">
          <div>
            <h2 className="text-xl font-bold text-white">{profile.name}</h2>
            <p className="text-sm text-gray-400 mt-0.5">Tier {profile.tier}</p>
            {profile.npub && (
              <p className="font-mono text-xs text-ose-400 mt-1">{profile.npub}</p>
            )}
          </div>
          <span
            className={`text-xs font-medium px-2.5 py-1 rounded-full ${
              profile.is_online ? "bg-green-900 text-green-300" : "bg-gray-800 text-gray-500"
            }`}
          >
            {profile.is_online ? "Online" : "Offline"}
          </span>
        </div>

        {/* Reputation bar */}
        <div className="mb-4">
          <div className="flex justify-between text-xs text-gray-400 mb-1">
            <span>Reputation</span>
            <span className="font-mono font-semibold text-ose-400">
              {(profile.reputation_score * 100).toFixed(1)} / 100
            </span>
          </div>
          <div className="bg-gray-800 rounded-full h-2">
            <div
              className="bg-ose-500 h-2 rounded-full transition-all"
              style={{ width: `${profile.reputation_score * 100}%` }}
            />
          </div>
        </div>

        {/* Skills */}
        <div className="flex flex-wrap gap-1.5">
          {profile.skills.map((s) => (
            <span key={s} className="text-xs bg-gray-800 text-gray-300 px-2 py-0.5 rounded">
              {s}
            </span>
          ))}
        </div>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-3">
        <StatCard label="Total Hires" value={String(profile.hire_count)} />
        <StatCard label="Active" value={String(activeHires)} highlight />
        <StatCard label="Completed" value={String(completedHires)} />
      </div>

      {/* Hire history */}
      {hires.length > 0 && (
        <div className="rounded-xl border border-gray-800 bg-gray-900 p-4">
          <h3 className="text-sm font-semibold text-gray-400 mb-3">Hire History</h3>
          <div className="flex flex-col gap-2">
            {hires.map((h) => (
              <div
                key={h.contract_id}
                className="flex items-center justify-between text-sm"
              >
                <span className="text-gray-300 truncate flex-1">{h.role}</span>
                <span
                  className={`ml-3 text-xs ${
                    h.active ? "text-green-400" : "text-gray-500"
                  }`}
                >
                  {h.active ? "active" : "done"}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Register button */}
      {!registered && (
        <button
          onClick={handleRegister}
          disabled={registering}
          className="w-full rounded-lg border border-ose-700 text-ose-400 py-2.5 text-sm font-medium hover:bg-ose-900/30 disabled:opacity-50"
        >
          {registering ? "Registering…" : "Register in Agent Registry"}
        </button>
      )}
      {registered && (
        <p className="text-center text-sm text-green-400">
          Registered in agent registry.
        </p>
      )}
    </div>
  );
}

function StatCard({
  label,
  value,
  highlight = false,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div className="rounded-lg bg-gray-900 border border-gray-800 px-4 py-3 text-center">
      <p className="text-xs text-gray-500 mb-1">{label}</p>
      <p className={`text-xl font-bold ${highlight ? "text-ose-400" : "text-white"}`}>
        {value}
      </p>
    </div>
  );
}
