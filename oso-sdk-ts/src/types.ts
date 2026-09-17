/**
 * TypeScript types mirroring the Rust oso-sdk structs.
 * Matches: job.rs, contract.rs, agent.rs, omokoda-core/src/oso_ir.rs
 */

// ── Job types ─────────────────────────────────────────────────────────────────

export type WorkloadType =
  | "generic"
  | "inference"
  | "training"
  | "simulation"
  | "rendering"
  | "indexing";

export type JobStatus =
  | "pending"
  | "allocated"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

export interface WorkloadRequirements {
  vram_gb?: number;
  ram_gb?: number;
  cpu_cores?: number;
  gpu_count?: number;
  fp16?: boolean;
  bf16?: boolean;
  cuda?: boolean;
  min_tier?: number;
}

export interface ComputeConstraints {
  max_price_cents?: number;
  max_queue_secs?: number;
  privacy?: "standard" | "confidential" | "sovereign";
  regions?: string[];
  allow_external?: boolean;
}

export interface JobSpec {
  workload?: WorkloadType;
  requirements?: WorkloadRequirements;
  constraints?: ComputeConstraints;
  runtime_spec?: Record<string, unknown>;
}

export interface ResourceUsage {
  gpu_seconds: number;
  cpu_seconds: number;
  ram_gb_seconds: number;
  storage_gb: number;
  egress_gb: number;
}

export interface BillingRecord {
  amount_cents: number;
  currency: "usd" | "ase" | "sui";
  line_items: Array<{ label: string; amount_cents: number }>;
}

export interface VerificationProof {
  artifact_hash?: string;
  runtime_attestation?: string;
  execution_hash?: string;
}

/** Mirrors Rust ComputeReceipt from ucx-protocol */
export interface ComputeReceipt {
  job_id: string;
  provider_id: string;
  completed_at: string; // ISO-8601
  resources: ResourceUsage;
  billing: BillingRecord;
  verification: VerificationProof;
  zangbeto_anchor?: string;
  /** SHA-256 hex hash of the canonical receipt fields */
  hash?: string;
}

/** A submitted job tracked by the SDK — mirrors Rust PendingJob */
export interface PendingJob {
  id: string;
  agent_id: string;
  workload: WorkloadType;
  requirements: WorkloadRequirements;
  constraints: ComputeConstraints;
  runtime_spec: Record<string, unknown>;
  status: JobStatus;
  created_at: string;
  receipt?: ComputeReceipt;
}

// ── Contract types ────────────────────────────────────────────────────────────

export type ContractClass =
  | "financial"
  | "agent"
  | "work"
  | "device"
  | "evidence"
  | "governance";

export interface NativeContract {
  id: string;
  class: ContractClass;
  owner: string;
  metadata: Record<string, unknown>;
}

export interface CallResult {
  contract_id: string;
  method: string;
  output: unknown;
  arp_payload?: Record<string, unknown>;
}

export interface ContractSpec {
  class: ContractClass;
  owner?: string;
  metadata?: Record<string, unknown>;
}

// ── Agent types ───────────────────────────────────────────────────────────────

/** Mirrors Rust AgentCard */
export interface AgentCard {
  agent_id: string;
  npub?: string;
  tier: number;
  skills: string[];
  is_online: boolean;
}

export interface AgentSpec {
  agent_id?: string;
  npub?: string;
  tier?: number;
  skills?: string[];
}

export interface RouteMessage {
  to: string;
  from?: string;
  payload: unknown;
  kind?: string;
}

export interface RouteResult {
  delivered: boolean;
  recipient: string;
  relay?: string;
  error?: string;
}

// ── SDK root types ────────────────────────────────────────────────────────────

export interface OsoSdkOptions {
  endpoint?: string;
  agent_id?: string;
  principal_id?: string;
}

/** Result of executing an OSO-IR program (from oso-simulator) */
export interface OsoRunResult {
  success: boolean;
  program_id?: string;
  ase_spent: number;
  ase_earned: number;
  receipts_emitted: string[];
  final_state: Record<string, unknown>;
  error?: string;
}

// ── OSO-IR types ──────────────────────────────────────────────────────────────

export interface IrInstruction {
  opcode: number;
  opcode_name: string;
  args: Record<string, unknown>;
  line: number;
}

export type IrProgram = IrInstruction[];

export interface LintResult {
  passed: boolean;
  warnings: LintDiagnostic[];
  errors: LintDiagnostic[];
}

export interface LintDiagnostic {
  rule: string;
  message: string;
  line?: number;
}
