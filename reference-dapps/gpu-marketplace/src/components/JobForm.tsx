import React, { useState } from "react";
import { JobClient } from "../../../oso-sdk-ts/src/JobClient.ts";
import type { GpuProvider } from "../services/ucxApi.ts";
import type { PendingJob } from "../../../oso-sdk-ts/src/types.ts";

interface Props {
  provider: GpuProvider;
  agentId: string;
  onJobCreated: (job: PendingJob) => void;
  onCancel: () => void;
}

export function JobForm({ provider, agentId, onJobCreated, onCancel }: Props) {
  const [modelSpec, setModelSpec] = useState("");
  const [hours, setHours] = useState(1);
  const [budgetAse, setBudgetAse] = useState(provider.price_per_hour_ase);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const estimatedCost = hours * provider.price_per_hour_ase;
  const overBudget = budgetAse < estimatedCost;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const client = new JobClient("http://localhost:3000", agentId);
      const job = await client.create({
        workload: "inference",
        requirements: {
          vram_gb: provider.vram_gb,
          gpu_count: 1,
        },
        constraints: {
          max_price_cents: budgetAse * 100,
          regions: [provider.region],
        },
        runtime_spec: {
          model_spec: modelSpec,
          hours,
          provider_id: provider.provider_id,
        },
      });
      onJobCreated(job);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="w-full max-w-md rounded-2xl border border-gray-800 bg-gray-900 p-6">
        <h2 className="text-lg font-bold mb-1">Hire GPU</h2>
        <p className="text-sm text-gray-400 mb-5">
          {provider.name} · {provider.gpu_type} · {provider.price_per_hour_ase} ASE/hr
        </p>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Model / workload spec
            </label>
            <textarea
              value={modelSpec}
              onChange={(e) => setModelSpec(e.target.value)}
              placeholder="e.g. Llama-3-70B Q4_K_M, batch inference, 1k prompts"
              rows={3}
              required
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500 resize-none"
            />
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Hours</label>
              <input
                type="number"
                min={1}
                max={168}
                value={hours}
                onChange={(e) => {
                  const v = parseInt(e.target.value, 10);
                  setHours(isNaN(v) ? 1 : v);
                }}
                required
                className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">
                Budget (ASE)
              </label>
              <input
                type="number"
                min={1}
                value={budgetAse}
                onChange={(e) => {
                  const v = parseInt(e.target.value, 10);
                  setBudgetAse(isNaN(v) ? 1 : v);
                }}
                required
                className={`w-full rounded-lg bg-gray-800 border px-3 py-2 text-sm focus:outline-none ${
                  overBudget ? "border-red-600" : "border-gray-700 focus:border-ose-500"
                }`}
              />
            </div>
          </div>

          <div className="rounded-lg bg-gray-800/60 px-4 py-3 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Estimated cost</span>
              <span className={overBudget ? "text-red-400" : "text-ose-400"}>
                {estimatedCost} ASE
              </span>
            </div>
            {overBudget && (
              <p className="text-red-400 text-xs mt-1">
                Budget is below estimated cost
              </p>
            )}
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
              disabled={submitting || overBudget}
              className="flex-1 rounded-lg bg-ose-600 py-2 text-sm font-semibold hover:bg-ose-500 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {submitting ? "Submitting…" : "Submit Job"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
