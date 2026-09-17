import React, { useEffect, useRef, useState } from "react";
import { JobClient } from "../../../oso-sdk-ts/src/JobClient.ts";
import type { PendingJob } from "../../../oso-sdk-ts/src/types.ts";

interface Props {
  jobs: PendingJob[];
  agentId: string;
  onJobsChange: (jobs: PendingJob[]) => void;
}

const MINT_THRESHOLD = 0.777;
const POLL_INTERVAL_MS = 3_000;

export function JobStatus({ jobs, agentId, onJobsChange }: Props) {
  const [minting, setMinting] = useState<Record<string, boolean>>({});
  const [mintedBadges, setMintedBadges] = useState<Record<string, string>>({});
  const clientRef = useRef(new JobClient("http://localhost:3000", agentId));

  // Poll all non-terminal jobs
  useEffect(() => {
    const active = jobs.filter(
      (j) => j.status === "pending" || j.status === "allocated" || j.status === "running"
    );
    if (active.length === 0) return;

    const interval = setInterval(async () => {
      const updated = await Promise.all(
        jobs.map(async (j) => {
          if (
            j.status === "completed" ||
            j.status === "failed" ||
            j.status === "cancelled"
          )
            return j;
          try {
            return await clientRef.current.find(j.id);
          } catch {
            return j;
          }
        })
      );
      onJobsChange(updated);
    }, POLL_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [jobs, onJobsChange]);

  const handleMintBadge = async (job: PendingJob) => {
    if (!job.receipt) return;
    setMinting((m) => ({ ...m, [job.id]: true }));
    // Simulate badge mint (WorkContract.verify + settle would go here)
    await new Promise((r) => setTimeout(r, 1_200));
    const badgeId = `badge_${job.id.slice(0, 8)}`;
    setMintedBadges((b) => ({ ...b, [job.id]: badgeId }));
    setMinting((m) => ({ ...m, [job.id]: false }));
  };

  if (jobs.length === 0) {
    return (
      <p className="py-12 text-center text-gray-500">
        No jobs yet. Go to <span className="text-ose-400">Browse</span> to hire a GPU.
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {jobs.map((job) => {
        const f1 = extractF1(job);
        const mintEligible = f1 !== null && f1 >= MINT_THRESHOLD;
        const badge = mintedBadges[job.id];

        return (
          <div
            key={job.id}
            className="rounded-xl border border-gray-800 bg-gray-900 p-5"
          >
            <div className="flex items-center justify-between mb-3">
              <div>
                <p className="font-mono text-xs text-gray-500">{job.id}</p>
                <p className="text-sm text-gray-300 mt-0.5 capitalize">
                  {(job.runtime_spec?.model_spec as string | undefined) ?? "—"}
                </p>
              </div>
              <StatusBadge status={job.status} />
            </div>

            {job.receipt && (
              <div className="rounded-lg bg-gray-800/60 p-3 text-xs space-y-1 mb-3">
                <Row label="Provider" value={job.receipt.provider_id} />
                <Row
                  label="GPU seconds"
                  value={String(job.receipt.resources.gpu_seconds)}
                />
                <Row
                  label="Cost"
                  value={`${job.receipt.billing.amount_cents / 100} ${job.receipt.billing.currency.toUpperCase()}`}
                />
                {job.receipt.verification.execution_hash && (
                  <Row
                    label="Proof hash"
                    value={job.receipt.verification.execution_hash.slice(0, 16) + "…"}
                    mono
                  />
                )}
                {f1 !== null && (
                  <Row
                    label="F1 score"
                    value={f1.toFixed(4)}
                    highlight={mintEligible}
                  />
                )}
              </div>
            )}

            {mintEligible && !badge && (
              <button
                onClick={() => handleMintBadge(job)}
                disabled={minting[job.id]}
                className="w-full rounded-lg bg-ose-600 py-2 text-sm font-semibold hover:bg-ose-500 disabled:opacity-50"
              >
                {minting[job.id] ? "Minting…" : "Mint Proof Badge"}
              </button>
            )}

            {badge && (
              <div className="rounded-lg bg-ose-900/40 border border-ose-700 px-4 py-2 text-sm text-ose-300 text-center">
                Badge minted: <span className="font-mono">{badge}</span>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

function extractF1(job: PendingJob): number | null {
  if (!job.receipt?.verification) return null;
  const f1 = (job.receipt.verification as Record<string, unknown>)["f1_score"];
  return typeof f1 === "number" ? f1 : null;
}

function StatusBadge({ status }: { status: string }) {
  const cls: Record<string, string> = {
    pending:   "bg-yellow-900 text-yellow-300",
    allocated: "bg-blue-900 text-blue-300",
    running:   "bg-cyan-900 text-cyan-300",
    completed: "bg-green-900 text-green-300",
    failed:    "bg-red-900 text-red-300",
    cancelled: "bg-gray-800 text-gray-400",
  };
  return (
    <span
      className={`text-xs font-medium px-2.5 py-0.5 rounded-full capitalize ${cls[status] ?? "bg-gray-800 text-gray-400"}`}
    >
      {status}
    </span>
  );
}

function Row({
  label,
  value,
  mono = false,
  highlight = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
  highlight?: boolean;
}) {
  return (
    <div className="flex justify-between gap-2">
      <span className="text-gray-500">{label}</span>
      <span
        className={`${mono ? "font-mono" : ""} ${
          highlight ? "text-ose-400 font-semibold" : "text-gray-300"
        }`}
      >
        {value}
      </span>
    </div>
  );
}
