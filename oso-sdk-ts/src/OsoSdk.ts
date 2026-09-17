import { JobClient } from "./JobClient.js";
import { ContractClient } from "./ContractClient.js";
import { AgentClient } from "./AgentClient.js";
import type { OsoSdkOptions } from "./types.js";

export const DEFAULT_ENDPOINT = "http://localhost:8788";

/**
 * OsoSdk — root handle for the Ọ̀ṢỌ́ sovereign API.
 *
 * Mirrors Rust OsoSdk in oso-sdk/src/lib.rs.
 *
 * Usage:
 * ```ts
 * const sdk = new OsoSdk({ agent_id: "did:v:agent:abc" });
 * const job = await sdk.jobs.create({ workload: "inference", runtime_spec: { model: "llama3" } });
 * const receipt = await sdk.jobs.waitForProof(job.id);
 * const hash = await sdk.jobs.settle(job.id);
 * ```
 */
export class OsoSdk {
  readonly agentId: string | undefined;
  readonly principalId: string | undefined;
  readonly endpoint: string;

  private _jobs: JobClient;
  private _contracts: ContractClient;
  private _agents: AgentClient;

  constructor(options: OsoSdkOptions = {}) {
    this.endpoint = options.endpoint ?? DEFAULT_ENDPOINT;
    this.agentId = options.agent_id;
    this.principalId = options.principal_id;

    this._jobs = new JobClient(this.endpoint, this.agentId);
    this._contracts = new ContractClient(this.endpoint, this.agentId);
    this._agents = new AgentClient(this.endpoint, this.agentId);
  }

  /** Job management client. */
  get jobs(): JobClient {
    return this._jobs;
  }

  /** Contract management client (all 6 contract classes). */
  get contracts(): ContractClient {
    return this._contracts;
  }

  /** Agent discovery and routing client. */
  get agents(): AgentClient {
    return this._agents;
  }
}
