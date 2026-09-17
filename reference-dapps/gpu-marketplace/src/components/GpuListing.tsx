import React, { useState } from "react";
import { ContractClient } from "../../../oso-sdk-ts/src/ContractClient.ts";

interface Props {
  agentId: string;
}

interface ListingForm {
  gpu_type: string;
  vram_gb: number;
  price_per_hour: number;
  region: string;
}

export function GpuListing({ agentId }: Props) {
  const [form, setForm] = useState<ListingForm>({
    gpu_type: "",
    vram_gb: 24,
    price_per_hour: 10,
    region: "local",
  });
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const set = <K extends keyof ListingForm>(k: K, v: ListingForm[K]) =>
    setForm((f) => ({ ...f, [k]: v }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setSubmitting(true);
    try {
      const client = new ContractClient("http://localhost:3000", agentId);
      // Deploy a Financial/Marketplace contract that represents this listing
      const contract = await client.deploy({
        class: "financial",
        owner: agentId,
        metadata: {
          listing_type: "gpu",
          gpu_type: form.gpu_type,
          vram_gb: form.vram_gb,
          price_per_hour_ase: form.price_per_hour,
          region: form.region,
        },
      });
      // Call marketplace_list method on the deployed contract
      await client.call(contract.id, "marketplace_list", {
        provider_id: agentId,
        gpu_type: form.gpu_type,
        vram_gb: form.vram_gb,
        price_per_hour_ase: form.price_per_hour,
        region: form.region,
      });
      setSuccess(contract.id);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  };

  if (success) {
    return (
      <div className="rounded-xl border border-green-700 bg-green-950/30 p-6 text-center">
        <p className="text-green-300 font-semibold mb-1">GPU listed!</p>
        <p className="text-xs text-gray-400 font-mono">{success}</p>
        <button
          onClick={() => {
            setSuccess(null);
            setForm({ gpu_type: "", vram_gb: 24, price_per_hour: 10, region: "local" });
          }}
          className="mt-4 rounded bg-gray-800 px-4 py-1.5 text-sm hover:bg-gray-700"
        >
          List Another
        </button>
      </div>
    );
  }

  return (
    <div className="max-w-md mx-auto rounded-xl border border-gray-800 bg-gray-900 p-6">
      <h2 className="text-lg font-bold mb-1">List Your GPU</h2>
      <p className="text-sm text-gray-400 mb-5">
        Earn ASE by contributing compute to the network.
      </p>

      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <div>
          <label className="block text-sm text-gray-400 mb-1">GPU type</label>
          <input
            type="text"
            value={form.gpu_type}
            onChange={(e) => set("gpu_type", e.target.value)}
            placeholder="e.g. RTX4090, A100, H100"
            required
            className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
          />
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-sm text-gray-400 mb-1">VRAM (GB)</label>
            <input
              type="number"
              min={1}
              max={192}
              value={form.vram_gb}
              onChange={(e) =>
                set("vram_gb", parseInt(e.target.value, 10) || 1)
              }
              required
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            />
          </div>
          <div>
            <label className="block text-sm text-gray-400 mb-1">Price (ASE/hr)</label>
            <input
              type="number"
              min={1}
              value={form.price_per_hour}
              onChange={(e) =>
                set("price_per_hour", parseInt(e.target.value, 10) || 1)
              }
              required
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            />
          </div>
        </div>

        <div>
          <label className="block text-sm text-gray-400 mb-1">Region</label>
          <select
            value={form.region}
            onChange={(e) => set("region", e.target.value)}
            className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
          >
            <option value="local">Local / Omarchy</option>
            <option value="us-east">US East</option>
            <option value="us-west">US West</option>
            <option value="eu-west">EU West</option>
            <option value="ap-south">AP South</option>
          </select>
        </div>

        {error && (
          <p className="rounded-lg bg-red-950/50 border border-red-700 px-3 py-2 text-sm text-red-400">
            {error}
          </p>
        )}

        <button
          type="submit"
          disabled={submitting}
          className="w-full rounded-lg bg-ose-600 py-2.5 text-sm font-semibold hover:bg-ose-500 disabled:opacity-50 disabled:cursor-not-allowed mt-1"
        >
          {submitting ? "Listing…" : "List GPU"}
        </button>
      </form>
    </div>
  );
}
