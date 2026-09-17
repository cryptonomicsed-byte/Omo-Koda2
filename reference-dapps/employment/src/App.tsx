import React, { useState } from "react";
import { AgentSearch } from "./components/AgentSearch.tsx";
import { HireForm, type HireRecord } from "./components/HireForm.tsx";
import { MyHires } from "./components/MyHires.tsx";
import { AgentProfile } from "./components/AgentProfile.tsx";
import type { PublicAgent } from "./services/agentApi.ts";

type Tab = "find" | "my-hires" | "profile";

const AGENT_ID_KEY = "oso_agent_id";

function getOrCreateAgentId(): string {
  let id = localStorage.getItem(AGENT_ID_KEY);
  if (!id) {
    id = `agent_${Math.random().toString(36).slice(2, 10)}`;
    localStorage.setItem(AGENT_ID_KEY, id);
  }
  return id;
}

export default function App() {
  const [tab, setTab] = useState<Tab>("find");
  const [agentId] = useState(getOrCreateAgentId);
  const [hiringAgent, setHiringAgent] = useState<PublicAgent | null>(null);
  const [hires, setHires] = useState<HireRecord[]>([]);

  const handleHired = (record: HireRecord) => {
    setHires((prev) => [record, ...prev]);
    setHiringAgent(null);
    setTab("my-hires");
  };

  const activeCount = hires.filter((h) => h.active).length;

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      {/* Header */}
      <header className="border-b border-gray-800 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">🤝</span>
          <div>
            <h1 className="font-bold text-white leading-none">Agent Employment</h1>
            <p className="text-xs text-gray-500 mt-0.5">Ọ̀ṢỌ́ Sovereign Labour · Phase 27.2</p>
          </div>
        </div>
        <div className="text-right">
          <p className="text-xs text-gray-500">Agent ID</p>
          <p className="font-mono text-xs text-ose-400">{agentId}</p>
        </div>
      </header>

      {/* Tabs */}
      <nav className="border-b border-gray-800 px-6 flex gap-1">
        {(["find", "my-hires", "profile"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
              tab === t
                ? "border-ose-500 text-ose-400"
                : "border-transparent text-gray-400 hover:text-gray-200"
            }`}
          >
            {t === "find"
              ? "Find Agents"
              : t === "my-hires"
              ? `My Hires${activeCount > 0 ? ` (${activeCount})` : ""}`
              : "My Profile"}
          </button>
        ))}
      </nav>

      {/* Content */}
      <main className="max-w-5xl mx-auto px-6 py-8">
        {tab === "find" && (
          <AgentSearch onHire={setHiringAgent} />
        )}
        {tab === "my-hires" && (
          <MyHires
            hires={hires}
            hirerId={agentId}
            onHiresChange={setHires}
          />
        )}
        {tab === "profile" && (
          <AgentProfile agentId={agentId} hires={hires} />
        )}
      </main>

      {/* Hire form modal */}
      {hiringAgent && (
        <HireForm
          agent={hiringAgent}
          hirerId={agentId}
          onHired={handleHired}
          onCancel={() => setHiringAgent(null)}
        />
      )}
    </div>
  );
}
