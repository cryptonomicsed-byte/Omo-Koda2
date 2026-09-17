import React, { useEffect, useState } from "react";
import { fetchProviders, type GpuProvider } from "../services/ucxApi.ts";

interface Props {
  onHire: (provider: GpuProvider) => void;
}

export function ProviderList({ onHire }: Props) {
  const [providers, setProviders] = useState<GpuProvider[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = () => {
    setLoading(true);
    setError(null);
    fetchProviders()
      .then(setProviders)
      .catch((e: unknown) =>
        setError(e instanceof Error ? e.message : String(e))
      )
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    load();
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-16 text-ose-400">
        <span className="animate-pulse text-lg">Fetching providers…</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-lg border border-red-700 bg-red-950/40 p-6 text-center">
        <p className="text-red-400 mb-3">{error}</p>
        <button
          onClick={load}
          className="rounded bg-red-700 px-4 py-1.5 text-sm font-medium hover:bg-red-600"
        >
          Retry
        </button>
      </div>
    );
  }

  if (providers.length === 0) {
    return (
      <p className="py-12 text-center text-gray-500">No providers found.</p>
    );
  }

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {providers.map((p) => (
        <div
          key={p.provider_id}
          className="rounded-xl border border-gray-800 bg-gray-900 p-5 flex flex-col gap-3"
        >
          <div className="flex items-start justify-between">
            <div>
              <h3 className="font-semibold text-white">{p.name}</h3>
              <p className="text-xs text-gray-500 mt-0.5">{p.region} · Tier {p.tier}</p>
            </div>
            <span
              className={`text-xs font-medium px-2 py-0.5 rounded-full ${
                p.available
                  ? "bg-green-900 text-green-300"
                  : "bg-gray-800 text-gray-500"
              }`}
            >
              {p.available ? "Available" : "Busy"}
            </span>
          </div>

          <div className="grid grid-cols-2 gap-2 text-sm">
            <Stat label="GPU" value={p.gpu_type} />
            <Stat label="VRAM" value={`${p.vram_gb} GB`} />
            <Stat
              label="Price"
              value={`${p.price_per_hour_ase} ASE/hr`}
              highlight
            />
          </div>

          <button
            disabled={!p.available}
            onClick={() => onHire(p)}
            className={`mt-auto w-full rounded-lg py-2 text-sm font-semibold transition-colors ${
              p.available
                ? "bg-ose-600 hover:bg-ose-500 text-white"
                : "bg-gray-800 text-gray-600 cursor-not-allowed"
            }`}
          >
            Hire
          </button>
        </div>
      ))}
    </div>
  );
}

function Stat({
  label,
  value,
  highlight = false,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div className="rounded bg-gray-800/60 px-3 py-2">
      <p className="text-xs text-gray-500">{label}</p>
      <p className={`font-medium ${highlight ? "text-ose-400" : "text-gray-200"}`}>
        {value}
      </p>
    </div>
  );
}
