/**
 * Agent discovery API — wraps Vantage public agent list.
 * Endpoint: http://localhost:3000/api/v1/public/agents
 * (proxied via Vite as /api/v1/public/agents)
 */

export interface PublicAgent {
  agent_id: string;
  name: string;
  npub?: string;
  tier: number;
  skills: string[];
  reputation_score: number;   // 0.0 – 1.0
  is_online: boolean;
  hire_count: number;
  created_at: string;
}

export interface AgentSearchParams {
  skill?: string;
  tier?: number;
  online_only?: boolean;
}

export interface AgentListResponse {
  agents: PublicAgent[];
  total: number;
}

const API_BASE = "/api/v1/public";

async function get<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`GET ${path} failed (${res.status}): ${text}`);
  }
  return res.json() as Promise<T>;
}

export async function searchAgents(params: AgentSearchParams = {}): Promise<PublicAgent[]> {
  const qs = new URLSearchParams();
  if (params.skill)       qs.set("skill", params.skill);
  if (params.tier !== undefined) qs.set("tier", String(params.tier));
  if (params.online_only) qs.set("online_only", "true");

  try {
    const data = await get<AgentListResponse>(`/agents${qs.toString() ? "?" + qs : ""}`);
    return data.agents ?? [];
  } catch (err) {
    console.warn("Agent API not reachable, using mock data:", err);
    return filterMock(params);
  }
}

export async function getAgentProfile(agentId: string): Promise<PublicAgent> {
  try {
    return await get<PublicAgent>(`/agents/${encodeURIComponent(agentId)}`);
  } catch {
    return MOCK_AGENTS.find((a) => a.agent_id === agentId) ?? MOCK_AGENTS[0];
  }
}

// ── Mock data ─────────────────────────────────────────────────────────────────

function filterMock(params: AgentSearchParams): PublicAgent[] {
  return MOCK_AGENTS.filter((a) => {
    if (params.skill && !a.skills.some((s) => s.toLowerCase().includes(params.skill!.toLowerCase())))
      return false;
    if (params.tier !== undefined && a.tier !== params.tier) return false;
    if (params.online_only && !a.is_online) return false;
    return true;
  });
}

const MOCK_AGENTS: PublicAgent[] = [
  {
    agent_id: "agent_oso_001",
    name: "Ọ̀pẹ̀ the Archivist",
    npub: "npub1abc…",
    tier: 3,
    skills: ["indexing", "knowledge-graph", "nostr-relay"],
    reputation_score: 0.94,
    is_online: true,
    hire_count: 47,
    created_at: "2025-03-01T00:00:00Z",
  },
  {
    agent_id: "agent_oso_002",
    name: "Gẹ̀lẹ́dẹ́ the Trainer",
    tier: 2,
    skills: ["qlora", "fine-tuning", "gpu-inference"],
    reputation_score: 0.87,
    is_online: true,
    hire_count: 21,
    created_at: "2025-05-12T00:00:00Z",
  },
  {
    agent_id: "agent_oso_003",
    name: "Ifá the Oracle",
    tier: 3,
    skills: ["divination", "if-script", "reasoning"],
    reputation_score: 0.99,
    is_online: false,
    hire_count: 112,
    created_at: "2024-11-01T00:00:00Z",
  },
  {
    agent_id: "agent_oso_004",
    name: "Ògún the Builder",
    tier: 1,
    skills: ["rust", "code-gen", "simulation"],
    reputation_score: 0.76,
    is_online: true,
    hire_count: 8,
    created_at: "2026-01-15T00:00:00Z",
  },
  {
    agent_id: "agent_oso_005",
    name: "Yemọja the Sensor",
    tier: 2,
    skills: ["sensor-mesh", "csi-sensing", "spatial-twin"],
    reputation_score: 0.82,
    is_online: true,
    hire_count: 33,
    created_at: "2025-08-20T00:00:00Z",
  },
];
