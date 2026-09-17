import React, { useState } from "react";
import { ContractClient } from "../../../oso-sdk-ts/src/ContractClient.ts";
import type { OsovmRunResult } from "../services/veilsimApi.ts";

interface Props {
  result: OsovmRunResult;
  agentId: string;
}

const MINT_THRESHOLD = 0.777;

export function ProofCard({ result, agentId }: Props) {
  const [claiming, setClaiming] = useState(false);
  const [claimed, setClaimed] = useState(false);
  const [claimError, setClaimError] = useState<string | null>(null);
  const [showTrajectories, setShowTrajectories] = useState(false);

  const mintEligible = result.f1_score >= MINT_THRESHOLD;
  const successCount = result.trajectories.filter((t) => t.success).length;

  const handleClaim = async () => {
    setClaiming(true);
    setClaimError(null);
    try {
      const client = new ContractClient("http://localhost:3000", agentId);

      // Deploy a Work contract for this run
      const contract = await client.deploy({
        class: "work",
        owner: agentId,
        metadata: { run_id: result.run_id, proof_hash: result.proof_hash },
      });

      // Verify proof
      await client.evidenceVerify(contract.id, result.proof_hash);

      // Settle reward
      await client.workAdvance(contract.id, result.run_id, {
        action: "settle",
        ase_reward: result.ase_reward,
      });

      setClaimed(true);
    } catch (e: unknown) {
      setClaimError(e instanceof Error ? e.message : String(e));
    } finally {
      setClaiming(false);
    }
  };

  return (
    <div
      className={`rounded-xl border p-5 ${
        mintEligible
          ? "border-ose-700 bg-ose-900/20"
          : "border-gray-800 bg-gray-900"
      }`}
    >
      {/* F1 score — large display */}
      <div className="flex items-start justify-between mb-4">
        <div>
          <p className="text-xs text-gray-500 mb-1">F1 Score</p>
          <p
            className={`text-5xl font-black font-mono ${
              mintEligible ? "text-ose-400" : "text-gray-400"
            }`}
          >
            {result.f1_score.toFixed(4)}
          </p>
        </div>
        <div className="text-right">
          {mintEligible ? (
            <span className="inline-block bg-ose-600 text-white text-xs font-bold px-3 py-1.5 rounded-full">
              MINT ELIGIBLE
            </span>
          ) : (
            <span className="inline-block bg-gray-800 text-gray-500 text-xs font-medium px-3 py-1.5 rounded-full">
              Below threshold ({MINT_THRESHOLD})
            </span>
          )}
          <p className="text-xs text-gray-600 mt-1">
            Threshold: {MINT_THRESHOLD}
          </p>
        </div>
      </div>

      {/* Run metadata */}
      <div className="grid grid-cols-2 gap-2 text-xs mb-4">
        <MetaRow label="Veil" value={result.veil_id} />
        <MetaRow label="Robot" value={result.robot_model} />
        <MetaRow
          label="Trajectories"
          value={`${successCount} / ${result.trajectory_count} passed`}
        />
        <MetaRow
          label="ASE Reward"
          value={`${result.ase_reward} ASE`}
          highlight={mintEligible}
        />
        <MetaRow label="Run ID" value={result.run_id} mono />
        <MetaRow
          label="Proof Hash"
          value={result.proof_hash.slice(0, 12) + "…"}
          mono
        />
      </div>

      {/* Trajectory table (collapsible) */}
      <button
        onClick={() => setShowTrajectories((v) => !v)}
        className="text-xs text-gray-500 hover:text-gray-300 mb-2 flex items-center gap-1"
      >
        <span>{showTrajectories ? "▾" : "▸"}</span>
        {showTrajectories ? "Hide" : "Show"} trajectories
      </button>

      {showTrajectories && (
        <div className="overflow-x-auto rounded-lg border border-gray-800 mb-4">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-gray-800 text-gray-500">
                <th className="px-3 py-2 text-left">ID</th>
                <th className="px-3 py-2 text-left">Status</th>
                <th className="px-3 py-2 text-right">Steps</th>
                <th className="px-3 py-2 text-right">Reward</th>
                <th className="px-3 py-2 text-right">ms</th>
              </tr>
            </thead>
            <tbody>
              {result.trajectories.slice(0, 50).map((t) => (
                <tr
                  key={t.trajectory_id}
                  className="border-b border-gray-800/50 last:border-0"
                >
                  <td className="px-3 py-1.5 font-mono text-gray-600">
                    {t.trajectory_id}
                  </td>
                  <td className="px-3 py-1.5">
                    <span
                      className={
                        t.success ? "text-green-400" : "text-red-400"
                      }
                    >
                      {t.success ? "pass" : "fail"}
                    </span>
                  </td>
                  <td className="px-3 py-1.5 text-right text-gray-400">
                    {t.steps}
                  </td>
                  <td className="px-3 py-1.5 text-right text-gray-400">
                    {t.reward.toFixed(3)}
                  </td>
                  <td className="px-3 py-1.5 text-right text-gray-500">
                    {t.duration_ms}
                  </td>
                </tr>
              ))}
              {result.trajectories.length > 50 && (
                <tr>
                  <td
                    colSpan={5}
                    className="px-3 py-2 text-center text-gray-600"
                  >
                    … {result.trajectories.length - 50} more
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {/* Claim reward */}
      {mintEligible && !claimed && (
        <>
          <button
            onClick={handleClaim}
            disabled={claiming}
            className="w-full rounded-lg bg-ose-600 py-2.5 text-sm font-semibold hover:bg-ose-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {claiming ? "Claiming…" : `Claim ${result.ase_reward} ASE Reward`}
          </button>
          {claimError && (
            <p className="mt-2 text-xs text-red-400 rounded bg-red-950/40 px-2 py-1">
              {claimError}
            </p>
          )}
        </>
      )}

      {claimed && (
        <div className="rounded-lg bg-green-950/40 border border-green-700 px-4 py-2 text-sm text-green-300 text-center">
          Reward claimed — {result.ase_reward} ASE settled on-chain.
        </div>
      )}
    </div>
  );
}

function MetaRow({
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
    <div className="rounded bg-gray-800/60 px-3 py-2">
      <p className="text-gray-500 text-xs">{label}</p>
      <p
        className={`truncate ${mono ? "font-mono" : ""} ${
          highlight ? "text-ose-400 font-semibold" : "text-gray-300"
        }`}
      >
        {value}
      </p>
    </div>
  );
}
