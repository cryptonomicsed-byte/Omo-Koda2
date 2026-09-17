/**
 * UCX adapter list — fetches GPU providers from the UCX broker.
 * Endpoint: http://localhost:7790/providers
 * (proxied via Vite as /providers in dev)
 */

export type GpuType =
  | "A40"
  | "A100"
  | "H100"
  | "RTX4090"
  | "RTX3090"
  | "V100"
  | string;

export interface GpuProvider {
  provider_id: string;
  name: string;
  gpu_type: GpuType;
  vram_gb: number;
  price_per_hour_ase: number;
  available: boolean;
  region: string;
  tier: number;
}

export interface ListProvidersResponse {
  providers: GpuProvider[];
  total: number;
}

const UCX_BASE =
  typeof window !== "undefined" && window.location.hostname === "localhost"
    ? ""            // Vite proxy rewrites /providers → localhost:7790
    : "http://localhost:7790";

async function ucxGet<T>(path: string): Promise<T> {
  const res = await fetch(`${UCX_BASE}${path}`);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`UCX ${path} failed (${res.status}): ${text}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchProviders(): Promise<GpuProvider[]> {
  try {
    const data = await ucxGet<ListProvidersResponse>("/providers");
    return data.providers ?? [];
  } catch (err) {
    // Surface mock data when UCX is not running so the UI is still demo-able
    console.warn("UCX not reachable, using mock providers:", err);
    return MOCK_PROVIDERS;
  }
}

// ── Mock data for offline demo ────────────────────────────────────────────────

const MOCK_PROVIDERS: GpuProvider[] = [
  {
    provider_id: "prov_001",
    name: "SolarNode Alpha",
    gpu_type: "A40",
    vram_gb: 48,
    price_per_hour_ase: 12,
    available: true,
    region: "us-east",
    tier: 2,
  },
  {
    provider_id: "prov_002",
    name: "MeshRig Omega",
    gpu_type: "RTX4090",
    vram_gb: 24,
    price_per_hour_ase: 8,
    available: true,
    region: "eu-west",
    tier: 1,
  },
  {
    provider_id: "prov_003",
    name: "SovereignCloud H1",
    gpu_type: "H100",
    vram_gb: 80,
    price_per_hour_ase: 28,
    available: false,
    region: "ap-south",
    tier: 3,
  },
  {
    provider_id: "prov_004",
    name: "OmarchyBox-7",
    gpu_type: "A100",
    vram_gb: 40,
    price_per_hour_ase: 18,
    available: true,
    region: "local",
    tier: 2,
  },
];
