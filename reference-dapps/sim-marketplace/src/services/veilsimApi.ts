/**
 * VeilSim Studio API wrapper.
 * Matches the schema in aether-veilsim/backend/main.py.
 *
 * Base URL: http://localhost:8788  (proxied via Vite in dev)
 */

// ── Schema types (mirror backend/main.py Pydantic models) ────────────────────

export interface VeilDescriptor {
  veil_id: string;
  name: string;
  description: string;
  robot_models: string[];   // supported robot model IDs
  difficulty: "easy" | "medium" | "hard";
  max_trajectories: number;
}

export interface TrajectoryResult {
  trajectory_id: string;
  success: boolean;
  steps: number;
  reward: number;
  duration_ms: number;
}

export interface OsovmRunResult {
  run_id: string;
  veil_id: string;
  robot_model: string;
  trajectory_count: number;
  trajectories: TrajectoryResult[];
  f1_score: number;
  mint_eligible: boolean;   // f1_score >= 0.777
  proof_hash: string;
  agent_id: string;
  started_at: string;
  completed_at: string;
  ase_reward: number;
}

export interface RunRequest {
  veil_id: string;
  robot_model: string;
  trajectory_count: number;
  agent_id: string;
}

export interface RunStatusResponse {
  run_id: string;
  status: "queued" | "running" | "completed" | "failed";
  progress: number;   // 0.0 – 1.0
  result?: OsovmRunResult;
  error?: string;
}

// ── API calls ─────────────────────────────────────────────────────────────────

const VEILSIM_BASE = "";   // proxied by Vite in dev

async function vsGet<T>(path: string): Promise<T> {
  const res = await fetch(`${VEILSIM_BASE}${path}`);
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`VeilSim GET ${path} (${res.status}): ${text}`);
  }
  return res.json() as Promise<T>;
}

async function vsPost<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${VEILSIM_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`VeilSim POST ${path} (${res.status}): ${text}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchVeils(): Promise<VeilDescriptor[]> {
  try {
    return await vsGet<VeilDescriptor[]>("/veils");
  } catch (err) {
    console.warn("VeilSim not reachable, using mock veils:", err);
    return MOCK_VEILS;
  }
}

export async function submitRun(req: RunRequest): Promise<{ run_id: string }> {
  return vsPost<{ run_id: string }>("/run", req);
}

export async function pollRunStatus(runId: string): Promise<RunStatusResponse> {
  return vsGet<RunStatusResponse>(`/status/${encodeURIComponent(runId)}`);
}

// ── Mock veils for offline demo ───────────────────────────────────────────────

const MOCK_VEILS: VeilDescriptor[] = [
  {
    veil_id: "veil_ant_01",
    name: "Ant Navigation",
    description: "Simulated ant navigating a 2D maze. Low complexity, good for baseline.",
    robot_models: ["ant-v4", "ant-v3"],
    difficulty: "easy",
    max_trajectories: 1000,
  },
  {
    veil_id: "veil_humanoid_02",
    name: "Humanoid Locomotion",
    description: "Bipedal humanoid walking on uneven terrain. High-dimensional control.",
    robot_models: ["humanoid-v4", "humanoid-v3"],
    difficulty: "hard",
    max_trajectories: 500,
  },
  {
    veil_id: "veil_halfcheetah_03",
    name: "HalfCheetah Sprint",
    description: "Maximise forward velocity with planar cheetah body.",
    robot_models: ["half-cheetah-v4", "half-cheetah-v3"],
    difficulty: "medium",
    max_trajectories: 2000,
  },
  {
    veil_id: "veil_scarab_04",
    name: "Scarab Swarm",
    description: "100-agent ScarabSwarm collective task completion.",
    robot_models: ["scarab-v1"],
    difficulty: "hard",
    max_trajectories: 200,
  },
];

// ── Synthetic mock run for offline demo ───────────────────────────────────────

export function buildMockResult(req: RunRequest): OsovmRunResult {
  const trajectories: TrajectoryResult[] = Array.from(
    { length: req.trajectory_count },
    (_, i) => ({
      trajectory_id: `traj_${i.toString().padStart(4, "0")}`,
      success: Math.random() > 0.2,
      steps: Math.floor(Math.random() * 200) + 50,
      reward: parseFloat((Math.random() * 2).toFixed(4)),
      duration_ms: Math.floor(Math.random() * 800) + 100,
    })
  );
  const successes = trajectories.filter((t) => t.success).length;
  const f1 = parseFloat((successes / req.trajectory_count).toFixed(4));

  return {
    run_id: `run_${Math.random().toString(36).slice(2, 10)}`,
    veil_id: req.veil_id,
    robot_model: req.robot_model,
    trajectory_count: req.trajectory_count,
    trajectories,
    f1_score: f1,
    mint_eligible: f1 >= 0.777,
    proof_hash: Array.from({ length: 32 }, () =>
      Math.floor(Math.random() * 256)
        .toString(16)
        .padStart(2, "0")
    ).join(""),
    agent_id: req.agent_id,
    started_at: new Date(Date.now() - 5000).toISOString(),
    completed_at: new Date().toISOString(),
    ase_reward: parseFloat((f1 * 42).toFixed(2)),
  };
}
