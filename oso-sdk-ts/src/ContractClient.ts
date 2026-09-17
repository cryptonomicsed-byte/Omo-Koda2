import type {
  NativeContract,
  ContractSpec,
  ContractClass,
  CallResult,
} from "./types.js";

/**
 * ContractClient — interface to the 6 native contract classes.
 * Mirrors Rust ContractClient in oso-sdk/src/contract.rs.
 *
 * The 6 classes:
 *   Financial  — AsePool, Payment, Escrow, Marketplace, Treasury
 *   Agent      — AgentRegistry, Hiring, Delegation, SkillRegistry, Reputation
 *   Work       — JobContract (13-step lifecycle), WorkAgreement
 *   Device     — DeviceRegistry, DeviceManifest, SensorPolicy
 *   Evidence   — ZàngbétòProof, EvidenceBundle, WitnessAttestation
 *   Governance — Council, DAO, SectorGovernance
 */
export class ContractClient {
  constructor(
    private readonly endpoint: string,
    private readonly agentId?: string
  ) {}

  private url(path: string): string {
    return `${this.endpoint}/api/contracts${path}`;
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

  // ── Generic contract operations ───────────────────────────────────────────

  /** Deploy a new native contract. */
  async deploy(spec: ContractSpec): Promise<NativeContract> {
    return this.post<NativeContract>("/deploy", {
      agent_id: this.agentId,
      ...spec,
    });
  }

  /** Look up a contract by ID. */
  async get(contractId: string): Promise<NativeContract> {
    return this.get<NativeContract>(`/${encodeURIComponent(contractId)}`);
  }

  /** List contracts, optionally filtered by class. */
  async list(contractClass?: ContractClass): Promise<NativeContract[]> {
    const qs = contractClass ? `?class=${contractClass}` : "";
    const res = await fetch(this.url(`/list${qs}`));
    if (!res.ok) throw new Error(`list contracts failed (${res.status})`);
    return res.json() as Promise<NativeContract[]>;
  }

  /** Call a method on a deployed contract. */
  async call(
    contractId: string,
    method: string,
    args: Record<string, unknown> = {}
  ): Promise<CallResult> {
    return this.post<CallResult>(`/${encodeURIComponent(contractId)}/call`, {
      method,
      args,
      caller: this.agentId,
    });
  }

  // ── Financial contracts ───────────────────────────────────────────────────

  /** Transfer ASE from caller to recipient via an AsePool contract. */
  async financialTransfer(
    contractId: string,
    to: string,
    amount: number
  ): Promise<CallResult> {
    return this.call(contractId, "transfer", { to, amount });
  }

  /** Open an escrow for a work agreement. */
  async financialEscrow(
    contractId: string,
    jobId: string,
    amount: number
  ): Promise<CallResult> {
    return this.call(contractId, "escrow", { job_id: jobId, amount });
  }

  /** Release escrowed funds after job settlement. */
  async financialRelease(
    contractId: string,
    jobId: string
  ): Promise<CallResult> {
    return this.call(contractId, "release", { job_id: jobId });
  }

  // ── Agent contracts ───────────────────────────────────────────────────────

  /** Register an agent in the AgentRegistry. */
  async agentRegister(
    contractId: string,
    agentSpec: { agent_id: string; tier: number; skills: string[]; npub?: string }
  ): Promise<CallResult> {
    return this.call(contractId, "register", agentSpec);
  }

  /** Hire an agent for a task (creates delegation). */
  async agentHire(
    contractId: string,
    agentId: string,
    taskSpec: Record<string, unknown>
  ): Promise<CallResult> {
    return this.call(contractId, "hire", { agent_id: agentId, task: taskSpec });
  }

  /** Delegate capability to another agent. */
  async agentDelegate(
    contractId: string,
    toAgentId: string,
    capability: string
  ): Promise<CallResult> {
    return this.call(contractId, "delegate", {
      to: toAgentId,
      capability,
    });
  }

  // ── Work contracts ────────────────────────────────────────────────────────

  /** Create a work agreement (starts 13-step lifecycle). */
  async workCreate(
    contractId: string,
    spec: {
      provider_id: string;
      job_id: string;
      ase_amount: number;
      deadline_secs?: number;
    }
  ): Promise<CallResult> {
    return this.call(contractId, "create_agreement", spec);
  }

  /** Advance the work lifecycle to the next state. */
  async workAdvance(
    contractId: string,
    agreementId: string,
    evidence?: Record<string, unknown>
  ): Promise<CallResult> {
    return this.call(contractId, "advance", {
      agreement_id: agreementId,
      evidence,
    });
  }

  // ── Device contracts ──────────────────────────────────────────────────────

  /** Register a device manifest. */
  async deviceBind(
    contractId: string,
    manifest: {
      device_id: string;
      firmware_hash: string;
      capabilities: string[];
    }
  ): Promise<CallResult> {
    return this.call(contractId, "bind", manifest);
  }

  /** Revoke a device binding. */
  async deviceUnbind(
    contractId: string,
    deviceId: string
  ): Promise<CallResult> {
    return this.call(contractId, "unbind", { device_id: deviceId });
  }

  // ── Evidence contracts ────────────────────────────────────────────────────

  /** Submit a Zàngbétò proof or evidence bundle. */
  async evidenceSubmit(
    contractId: string,
    bundle: {
      receipt_hash: string;
      evidence_type: string;
      payload: Record<string, unknown>;
    }
  ): Promise<CallResult> {
    return this.call(contractId, "submit", bundle);
  }

  /** Verify a submitted evidence bundle. */
  async evidenceVerify(
    contractId: string,
    receiptHash: string
  ): Promise<CallResult> {
    return this.call(contractId, "verify", { receipt_hash: receiptHash });
  }

  // ── Governance contracts ──────────────────────────────────────────────────

  /** Create a governance proposal. */
  async governancePropose(
    contractId: string,
    proposal: {
      title: string;
      description: string;
      payload: Record<string, unknown>;
    }
  ): Promise<CallResult> {
    return this.call(contractId, "propose", {
      proposer: this.agentId,
      ...proposal,
    });
  }

  /** Cast a vote on a proposal. */
  async governanceVote(
    contractId: string,
    proposalId: string,
    vote: "yes" | "no" | "abstain"
  ): Promise<CallResult> {
    return this.call(contractId, "vote", {
      proposal_id: proposalId,
      vote,
      voter: this.agentId,
    });
  }

  /** Enact a passed proposal. */
  async governanceEnact(
    contractId: string,
    proposalId: string
  ): Promise<CallResult> {
    return this.call(contractId, "enact", { proposal_id: proposalId });
  }
}
