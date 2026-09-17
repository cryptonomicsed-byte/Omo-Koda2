import type { JobSpec, PendingJob, ComputeReceipt, JobStatus } from "./types.js";

export interface JobFilter {
  status?: JobStatus;
  agent_id?: string;
}

/**
 * JobClient — create, find, assign, wait for proof, and settle compute jobs.
 * Mirrors Rust JobClient in oso-sdk/src/job.rs.
 *
 * All calls go to `{endpoint}/api/jobs/*` via fetch.
 */
export class JobClient {
  constructor(
    private readonly endpoint: string,
    private readonly agentId?: string
  ) {}

  private url(path: string): string {
    return `${this.endpoint}/api/jobs${path}`;
  }

  private async post<T>(path: string, body: unknown): Promise<T> {
    const res = await fetch(this.url(path), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`POST ${path} failed (${res.status}): ${text}`);
    }
    return res.json() as Promise<T>;
  }

  private async get<T>(path: string): Promise<T> {
    const res = await fetch(this.url(path));
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`GET ${path} failed (${res.status}): ${text}`);
    }
    return res.json() as Promise<T>;
  }

  /**
   * Submit a new job. Returns the created PendingJob.
   */
  async create(spec: JobSpec = {}): Promise<PendingJob> {
    return this.post<PendingJob>("/create", {
      agent_id: this.agentId,
      ...spec,
    });
  }

  /**
   * Find a job by ID.
   */
  async find(jobId: string): Promise<PendingJob> {
    return this.get<PendingJob>(`/${encodeURIComponent(jobId)}`);
  }

  /**
   * List jobs, optionally filtered.
   */
  async list(filter: JobFilter = {}): Promise<PendingJob[]> {
    const params = new URLSearchParams();
    if (filter.status) params.set("status", filter.status);
    if (filter.agent_id) params.set("agent_id", filter.agent_id);
    const qs = params.toString();
    const res = await fetch(this.url(`/list${qs ? "?" + qs : ""}`));
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`GET /list failed (${res.status}): ${text}`);
    }
    return res.json() as Promise<PendingJob[]>;
  }

  /**
   * Assign a job to a provider agent.
   */
  async assign(jobId: string, agentId: string): Promise<void> {
    await this.post<void>(`/${encodeURIComponent(jobId)}/assign`, {
      provider_id: agentId,
    });
  }

  /**
   * Poll until the job has a proof receipt, or until timeout elapses.
   * Default timeout: 30 000 ms. Poll interval: 1 000 ms.
   */
  async waitForProof(
    jobId: string,
    timeoutMs: number = 30_000
  ): Promise<ComputeReceipt> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      const job = await this.find(jobId);
      if (job.status === "completed" && job.receipt) {
        return job.receipt;
      }
      if (job.status === "failed") {
        throw new Error(`Job ${jobId} failed`);
      }
      if (job.status === "cancelled") {
        throw new Error(`Job ${jobId} was cancelled`);
      }
      await new Promise((r) => setTimeout(r, 1_000));
    }
    throw new Error(`Proof timeout for job ${jobId} after ${timeoutMs}ms`);
  }

  /**
   * Settle a completed job — triggers Àṣẹ payment flow.
   * Returns the settlement receipt hash.
   */
  async settle(jobId: string): Promise<string> {
    const result = await this.post<{ receipt_hash: string }>(
      `/${encodeURIComponent(jobId)}/settle`,
      {}
    );
    return result.receipt_hash;
  }
}
