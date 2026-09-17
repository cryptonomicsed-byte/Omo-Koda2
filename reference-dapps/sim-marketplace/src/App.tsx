import React, { useState, useEffect } from "react";
import { SimForm } from "./components/SimForm.tsx";
import { ProofExplorer } from "./components/ProofExplorer.tsx";
import { ProofCard } from "./components/ProofCard.tsx";
import type { OsovmRunResult } from "./services/veilsimApi.ts";

type Tab = "run" | "explorer";

const AGENT_ID_KEY = "oso_agent_id";
const PROOFS_STORAGE_KEY = "oso_sim_proofs";

function getOrCreateAgentId(): string {
  let id = localStorage.getItem(AGENT_ID_KEY);
  if (!id) {
    id = `agent_${Math.random().toString(36).slice(2, 10)}`;
    localStorage.setItem(AGENT_ID_KEY, id);
  }
  return id;
}

function loadProofs(): OsovmRunResult[] {
  try {
    const raw = localStorage.getItem(PROOFS_STORAGE_KEY);
    return raw ? (JSON.parse(raw) as OsovmRunResult[]) : [];
  } catch {
    return [];
  }
}

function saveProofs(proofs: OsovmRunResult[]): void {
  try {
    localStorage.setItem(PROOFS_STORAGE_KEY, JSON.stringify(proofs));
  } catch {
    // storage full — drop oldest
  }
}

export default function App() {
  const [tab, setTab] = useState<Tab>("run");
  const [agentId] = useState(getOrCreateAgentId);
  const [proofs, setProofs] = useState<OsovmRunResult[]>(loadProofs);
  const [latestResult, setLatestResult] = useState<OsovmRunResult | null>(null);

  // Persist proofs to localStorage on change
  useEffect(() => {
    saveProofs(proofs);
  }, [proofs]);

  const handleResult = (result: OsovmRunResult) => {
    setLatestResult(result);
    setProofs((prev) => [result, ...prev]);
  };

  const mintEligibleCount = proofs.filter((p) => p.f1_score >= 0.777).length;

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      {/* Header */}
      <header className="border-b border-gray-800 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">🔬</span>
          <div>
            <h1 className="font-bold text-white leading-none">Simulation Marketplace</h1>
            <p className="text-xs text-gray-500 mt-0.5">Ọ̀ṢỌ́ OSOVM Proofs · Phase 27.3</p>
          </div>
        </div>
        <div className="text-right">
          <p className="text-xs text-gray-500">Agent ID</p>
          <p className="font-mono text-xs text-ose-400">{agentId}</p>
        </div>
      </header>

      {/* Tabs */}
      <nav className="border-b border-gray-800 px-6 flex gap-1">
        {(["run", "explorer"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
              tab === t
                ? "border-ose-500 text-ose-400"
                : "border-transparent text-gray-400 hover:text-gray-200"
            }`}
          >
            {t === "run"
              ? "Run Simulation"
              : `Proof Explorer${mintEligibleCount > 0 ? ` (${mintEligibleCount} mint)` : proofs.length > 0 ? ` (${proofs.length})` : ""}`}
          </button>
        ))}
      </nav>

      {/* Content */}
      <main className="max-w-4xl mx-auto px-6 py-8 flex flex-col gap-8">
        {tab === "run" && (
          <>
            <SimForm agentId={agentId} onResult={handleResult} />

            {latestResult && (
              <div>
                <h2 className="text-sm font-semibold text-gray-400 mb-3">
                  Latest Result
                </h2>
                <ProofCard result={latestResult} agentId={agentId} />
              </div>
            )}
          </>
        )}

        {tab === "explorer" && (
          <ProofExplorer proofs={proofs} agentId={agentId} />
        )}
      </main>
    </div>
  );
}
