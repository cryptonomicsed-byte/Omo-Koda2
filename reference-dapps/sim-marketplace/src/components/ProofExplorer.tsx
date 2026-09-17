import React, { useState } from "react";
import { ProofCard } from "./ProofCard.tsx";
import type { OsovmRunResult } from "../services/veilsimApi.ts";

interface Props {
  proofs: OsovmRunResult[];
  agentId: string;
}

const MINT_THRESHOLD = 0.777;

export function ProofExplorer({ proofs, agentId }: Props) {
  const [filterMintOnly, setFilterMintOnly] = useState(false);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const filtered = filterMintOnly
    ? proofs.filter((p) => p.f1_score >= MINT_THRESHOLD)
    : proofs;

  const mintCount = proofs.filter((p) => p.f1_score >= MINT_THRESHOLD).length;

  if (proofs.length === 0) {
    return (
      <p className="py-12 text-center text-gray-500">
        No proofs yet. Run a simulation in{" "}
        <span className="text-ose-400">Run Simulation</span>.
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-5">
      {/* Filters bar */}
      <div className="flex items-center gap-4 flex-wrap">
        <p className="text-sm text-gray-400">
          {proofs.length} run{proofs.length !== 1 ? "s" : ""} total ·{" "}
          <span className="text-ose-400">{mintCount} mint-eligible</span>
        </p>
        <label className="flex items-center gap-2 text-sm cursor-pointer">
          <input
            type="checkbox"
            checked={filterMintOnly}
            onChange={(e) => setFilterMintOnly(e.target.checked)}
            className="accent-ose-500"
          />
          <span className="text-gray-300">Mint-eligible only</span>
        </label>
      </div>

      {/* Proof list */}
      <div className="flex flex-col gap-3">
        {filtered.map((proof) => (
          <div key={proof.run_id}>
            {/* Collapsed row */}
            <button
              onClick={() =>
                setExpandedId((id) => (id === proof.run_id ? null : proof.run_id))
              }
              className="w-full text-left rounded-xl border border-gray-800 bg-gray-900 px-4 py-3 hover:border-gray-700 transition-colors"
            >
              <div className="flex items-center justify-between gap-3">
                <div className="flex items-center gap-3 min-w-0">
                  <span
                    className={`font-mono text-lg font-bold shrink-0 ${
                      proof.f1_score >= MINT_THRESHOLD
                        ? "text-ose-400"
                        : "text-gray-400"
                    }`}
                  >
                    {proof.f1_score.toFixed(4)}
                  </span>
                  <div className="min-w-0">
                    <p className="text-sm text-gray-300 truncate">
                      {proof.veil_id} · {proof.robot_model}
                    </p>
                    <p className="text-xs text-gray-600 font-mono truncate">
                      {proof.run_id}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  {proof.f1_score >= MINT_THRESHOLD && (
                    <span className="text-xs bg-ose-900 text-ose-300 px-2 py-0.5 rounded-full font-medium">
                      MINT
                    </span>
                  )}
                  <span className="text-xs text-gray-400">
                    {proof.trajectory_count} traj
                  </span>
                  <span className="text-gray-600">
                    {expandedId === proof.run_id ? "▲" : "▼"}
                  </span>
                </div>
              </div>
            </button>

            {/* Expanded ProofCard */}
            {expandedId === proof.run_id && (
              <div className="mt-2 pl-4">
                <ProofCard result={proof} agentId={agentId} />
              </div>
            )}
          </div>
        ))}

        {filtered.length === 0 && filterMintOnly && (
          <p className="py-8 text-center text-gray-500">
            No mint-eligible proofs yet. Aim for F1 ≥ {MINT_THRESHOLD}.
          </p>
        )}
      </div>
    </div>
  );
}
