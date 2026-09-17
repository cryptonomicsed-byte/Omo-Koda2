import React, { useState } from "react";
import { ContractClient } from "../../../oso-sdk-ts/src/ContractClient.ts";
import type { HireRecord } from "./HireForm.tsx";

interface Props {
  hires: HireRecord[];
  hirerId: string;
  onHiresChange: (hires: HireRecord[]) => void;
}

export function MyHires({ hires, hirerId, onHiresChange }: Props) {
  const [terminating, setTerminating] = useState<Record<string, boolean>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleTerminate = async (hire: HireRecord) => {
    setTerminating((t) => ({ ...t, [hire.contract_id]: true }));
    setErrors((e) => {
      const next = { ...e };
      delete next[hire.contract_id];
      return next;
    });
    try {
      const client = new ContractClient("http://localhost:3000", hirerId);
      // Advance work lifecycle to terminate the agreement
      await client.workAdvance(hire.contract_id, hire.contract_id, {
        action: "terminate",
        reason: "Hirer terminated",
      });
      onHiresChange(
        hires.map((h) =>
          h.contract_id === hire.contract_id ? { ...h, active: false } : h
        )
      );
    } catch (e: unknown) {
      setErrors((errs) => ({
        ...errs,
        [hire.contract_id]: e instanceof Error ? e.message : String(e),
      }));
    } finally {
      setTerminating((t) => ({ ...t, [hire.contract_id]: false }));
    }
  };

  if (hires.length === 0) {
    return (
      <p className="py-12 text-center text-gray-500">
        No active hires. Find agents in{" "}
        <span className="text-ose-400">Find Agents</span>.
      </p>
    );
  }

  const active = hires.filter((h) => h.active);
  const inactive = hires.filter((h) => !h.active);

  return (
    <div className="flex flex-col gap-6">
      {active.length > 0 && (
        <section>
          <h2 className="text-sm font-semibold text-gray-400 mb-3">
            Active ({active.length})
          </h2>
          <div className="flex flex-col gap-3">
            {active.map((hire) => (
              <HireCard
                key={hire.contract_id}
                hire={hire}
                onTerminate={handleTerminate}
                terminating={!!terminating[hire.contract_id]}
                error={errors[hire.contract_id]}
              />
            ))}
          </div>
        </section>
      )}

      {inactive.length > 0 && (
        <section>
          <h2 className="text-sm font-semibold text-gray-400 mb-3">
            Terminated ({inactive.length})
          </h2>
          <div className="flex flex-col gap-3 opacity-60">
            {inactive.map((hire) => (
              <HireCard
                key={hire.contract_id}
                hire={hire}
                onTerminate={handleTerminate}
                terminating={false}
                error={undefined}
              />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function HireCard({
  hire,
  onTerminate,
  terminating,
  error,
}: {
  hire: HireRecord;
  onTerminate: (h: HireRecord) => void;
  terminating: boolean;
  error?: string;
}) {
  const scopeColor: Record<string, string> = {
    "read-only":     "bg-blue-900 text-blue-300",
    "task-scoped":   "bg-yellow-900 text-yellow-300",
    "full-delegate": "bg-ose-900 text-ose-300",
  };

  return (
    <div className="rounded-xl border border-gray-800 bg-gray-900 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <p className="font-semibold text-white">{hire.agent.name}</p>
            <span
              className={`text-xs font-medium px-2 py-0.5 rounded-full ${
                hire.active ? "bg-green-900 text-green-300" : "bg-gray-800 text-gray-500"
              }`}
            >
              {hire.active ? "Active" : "Terminated"}
            </span>
          </div>
          <p className="text-sm text-gray-400 mt-0.5 truncate">{hire.role}</p>
          <div className="flex flex-wrap gap-2 mt-2 text-xs text-gray-500">
            <span>{hire.duration_days}d</span>
            <span>·</span>
            <span>{hire.budget_ase} ASE</span>
            <span>·</span>
            <span
              className={`font-medium px-1.5 py-0.5 rounded ${
                scopeColor[hire.delegation_scope] ?? "bg-gray-800 text-gray-400"
              }`}
            >
              {hire.delegation_scope}
            </span>
          </div>
          <p className="text-xs text-gray-600 font-mono mt-1 truncate">
            {hire.contract_id}
          </p>
        </div>

        {hire.active && (
          <button
            onClick={() => onTerminate(hire)}
            disabled={terminating}
            className="shrink-0 rounded-lg border border-red-700 text-red-400 px-3 py-1.5 text-xs font-medium hover:bg-red-950/40 disabled:opacity-50"
          >
            {terminating ? "…" : "Terminate"}
          </button>
        )}
      </div>

      {error && (
        <p className="mt-2 text-xs text-red-400 rounded bg-red-950/40 px-2 py-1">
          {error}
        </p>
      )}
    </div>
  );
}
