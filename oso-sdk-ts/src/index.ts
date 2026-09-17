/**
 * @omokoda/oso-sdk — TypeScript mirror of the Rust oso-sdk.
 *
 * Exports the root OsoSdk handle plus all sub-clients and types.
 */

export { OsoSdk, DEFAULT_ENDPOINT } from "./OsoSdk.js";
export { JobClient } from "./JobClient.js";
export { ContractClient } from "./ContractClient.js";
export { AgentClient } from "./AgentClient.js";

export type {
  // Job
  JobSpec,
  JobFilter,
  PendingJob,
  JobStatus,
  WorkloadType,
  WorkloadRequirements,
  ComputeConstraints,
  ComputeReceipt,
  ResourceUsage,
  BillingRecord,
  VerificationProof,
  // Contract
  NativeContract,
  ContractSpec,
  ContractClass,
  CallResult,
  // Agent
  AgentCard,
  AgentSpec,
  RouteMessage,
  RouteResult,
  // SDK
  OsoSdkOptions,
  OsoRunResult,
  // IR / linting
  IrInstruction,
  IrProgram,
  LintResult,
  LintDiagnostic,
} from "./types.js";
