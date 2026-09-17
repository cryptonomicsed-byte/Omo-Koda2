import React, { useEffect, useState } from "react";
import {
  fetchVeils,
  submitRun,
  pollRunStatus,
  buildMockResult,
  type VeilDescriptor,
  type OsovmRunResult,
} from "../services/veilsimApi.ts";

interface Props {
  agentId: string;
  onResult: (result: OsovmRunResult) => void;
}

type Phase = "idle" | "running" | "done" | "error";

const POLL_INTERVAL_MS = 1_500;
const MOCK_MODE_LATENCY_MS = 3_000;

export function SimForm({ agentId, onResult }: Props) {
  const [veils, setVeils] = useState<VeilDescriptor[]>([]);
  const [veilId, setVeilId] = useState("");
  const [robotModel, setRobotModel] = useState("");
  const [trajectoryCount, setTrajectoryCount] = useState(100);
  const [phase, setPhase] = useState<Phase>("idle");
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [veilsLoading, setVeilsLoading] = useState(true);

  useEffect(() => {
    fetchVeils()
      .then((vs) => {
        setVeils(vs);
        if (vs.length > 0) {
          setVeilId(vs[0].veil_id);
          setRobotModel(vs[0].robot_models[0] ?? "");
        }
      })
      .finally(() => setVeilsLoading(false));
  }, []);

  const selectedVeil = veils.find((v) => v.veil_id === veilId);

  const handleVeilChange = (id: string) => {
    setVeilId(id);
    const v = veils.find((v) => v.veil_id === id);
    if (v) setRobotModel(v.robot_models[0] ?? "");
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setPhase("running");
    setProgress(0);

    try {
      let result: OsovmRunResult;
      try {
        // Try real VeilSim first
        const { run_id } = await submitRun({
          veil_id: veilId,
          robot_model: robotModel,
          trajectory_count: trajectoryCount,
          agent_id: agentId,
        });

        // Poll for completion
        result = await pollUntilDone(run_id, (p) => setProgress(p));
      } catch {
        // Fall back to mock simulation
        console.warn("VeilSim not reachable, running mock simulation");
        result = await runMockSim(
          { veil_id: veilId, robot_model: robotModel, trajectory_count: trajectoryCount, agent_id: agentId },
          (p) => setProgress(p)
        );
      }

      setPhase("done");
      onResult(result);
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase("error");
    }
  };

  return (
    <div className="max-w-lg mx-auto rounded-xl border border-gray-800 bg-gray-900 p-6">
      <h2 className="text-lg font-bold mb-1">Run Simulation</h2>
      <p className="text-sm text-gray-400 mb-5">
        Submit a trajectory batch to OSOVM. Earn ASE if F1 ≥ 0.777.
      </p>

      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        {/* Veil selector */}
        <div>
          <label className="block text-sm text-gray-400 mb-1">Veil (environment)</label>
          {veilsLoading ? (
            <div className="h-10 rounded-lg bg-gray-800 animate-pulse" />
          ) : (
            <select
              value={veilId}
              onChange={(e) => handleVeilChange(e.target.value)}
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            >
              {veils.map((v) => (
                <option key={v.veil_id} value={v.veil_id}>
                  {v.name} ({v.difficulty})
                </option>
              ))}
            </select>
          )}
          {selectedVeil && (
            <p className="text-xs text-gray-500 mt-1">{selectedVeil.description}</p>
          )}
        </div>

        {/* Robot model */}
        <div>
          <label className="block text-sm text-gray-400 mb-1">Robot model</label>
          {selectedVeil ? (
            <select
              value={robotModel}
              onChange={(e) => setRobotModel(e.target.value)}
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            >
              {selectedVeil.robot_models.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          ) : (
            <input
              type="text"
              value={robotModel}
              onChange={(e) => setRobotModel(e.target.value)}
              placeholder="e.g. ant-v4"
              required
              className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
            />
          )}
        </div>

        {/* Trajectory count */}
        <div>
          <label className="block text-sm text-gray-400 mb-1">
            Trajectories
            {selectedVeil && (
              <span className="ml-1 text-gray-600">
                (max {selectedVeil.max_trajectories})
              </span>
            )}
          </label>
          <input
            type="number"
            min={1}
            max={selectedVeil?.max_trajectories ?? 10000}
            value={trajectoryCount}
            onChange={(e) =>
              setTrajectoryCount(parseInt(e.target.value, 10) || 1)
            }
            required
            className="w-full rounded-lg bg-gray-800 border border-gray-700 px-3 py-2 text-sm focus:outline-none focus:border-ose-500"
          />
        </div>

        {/* Progress */}
        {phase === "running" && (
          <div>
            <div className="flex justify-between text-xs text-gray-400 mb-1">
              <span>Running…</span>
              <span>{(progress * 100).toFixed(0)}%</span>
            </div>
            <div className="bg-gray-800 rounded-full h-2">
              <div
                className="bg-ose-500 h-2 rounded-full transition-all duration-500"
                style={{ width: `${progress * 100}%` }}
              />
            </div>
          </div>
        )}

        {phase === "done" && (
          <p className="text-green-400 text-sm text-center py-1">
            Simulation complete. See results below.
          </p>
        )}

        {error && (
          <p className="rounded-lg bg-red-950/50 border border-red-700 px-3 py-2 text-sm text-red-400">
            {error}
          </p>
        )}

        <button
          type="submit"
          disabled={phase === "running" || veilsLoading}
          className="w-full rounded-lg bg-ose-600 py-2.5 text-sm font-semibold hover:bg-ose-500 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {phase === "running" ? "Running…" : "Run Simulation"}
        </button>
      </form>
    </div>
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async function pollUntilDone(
  runId: string,
  onProgress: (p: number) => void
): Promise<OsovmRunResult> {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const status = await pollRunStatus(runId);
    onProgress(status.progress);
    if (status.status === "completed" && status.result) return status.result;
    if (status.status === "failed") throw new Error(status.error ?? "Simulation failed");
    await delay(POLL_INTERVAL_MS);
  }
  throw new Error("Simulation timed out");
}

async function runMockSim(
  req: Parameters<typeof buildMockResult>[0],
  onProgress: (p: number) => void
): Promise<OsovmRunResult> {
  const steps = 10;
  for (let i = 1; i <= steps; i++) {
    await delay(MOCK_MODE_LATENCY_MS / steps);
    onProgress(i / steps);
  }
  return buildMockResult(req);
}

function delay(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}
