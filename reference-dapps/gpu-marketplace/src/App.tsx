import React, { useState } from "react";
import { ProviderList } from "./components/ProviderList.tsx";
import { JobForm } from "./components/JobForm.tsx";
import { JobStatus } from "./components/JobStatus.tsx";
import { GpuListing } from "./components/GpuListing.tsx";
import type { GpuProvider } from "./services/ucxApi.ts";
import type { PendingJob } from "../../oso-sdk-ts/src/types.ts";

type Tab = "browse" | "my-jobs" | "my-gpu";

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
  const [tab, setTab] = useState<Tab>("browse");
  const [agentId] = useState(getOrCreateAgentId);
  const [hiringProvider, setHiringProvider] = useState<GpuProvider | null>(null);
  const [jobs, setJobs] = useState<PendingJob[]>([]);

  const handleJobCreated = (job: PendingJob) => {
    setJobs((prev) => [job, ...prev]);
    setHiringProvider(null);
    setTab("my-jobs");
  };

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      {/* Header */}
      <header className="border-b border-gray-800 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">⚡</span>
          <div>
            <h1 className="font-bold text-white leading-none">GPU Marketplace</h1>
            <p className="text-xs text-gray-500 mt-0.5">Ọ̀ṢỌ́ Sovereign Compute · Phase 27.1</p>
          </div>
        </div>
        <div className="text-right">
          <p className="text-xs text-gray-500">Agent ID</p>
          <p className="font-mono text-xs text-ose-400">{agentId}</p>
        </div>
      </header>

      {/* Tabs */}
      <nav className="border-b border-gray-800 px-6 flex gap-1">
        {(["browse", "my-jobs", "my-gpu"] as Tab[]).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-4 py-3 text-sm font-medium capitalize transition-colors border-b-2 -mb-px ${
              tab === t
                ? "border-ose-500 text-ose-400"
                : "border-transparent text-gray-400 hover:text-gray-200"
            }`}
          >
            {t === "browse"
              ? "Browse"
              : t === "my-jobs"
              ? `My Jobs${jobs.length > 0 ? ` (${jobs.length})` : ""}`
              : "My GPU"}
          </button>
        ))}
      </nav>

      {/* Content */}
      <main className="max-w-5xl mx-auto px-6 py-8">
        {tab === "browse" && (
          <ProviderList onHire={setHiringProvider} />
        )}
        {tab === "my-jobs" && (
          <JobStatus
            jobs={jobs}
            agentId={agentId}
            onJobsChange={setJobs}
          />
        )}
        {tab === "my-gpu" && (
          <GpuListing agentId={agentId} />
        )}
      </main>

      {/* Job form modal */}
      {hiringProvider && (
        <JobForm
          provider={hiringProvider}
          agentId={agentId}
          onJobCreated={handleJobCreated}
          onCancel={() => setHiringProvider(null)}
        />
      )}
    </div>
  );
}
