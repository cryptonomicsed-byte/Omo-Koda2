# @omokoda/oso-sdk

TypeScript mirror of the Rust `oso-sdk` crate. Provides a fetch-based client for the Ọ̀ṢỌ́ sovereign job/contract/agent API.

## Install

```bash
npm install @omokoda/oso-sdk
```

## Usage

```ts
import { OsoSdk } from "@omokoda/oso-sdk";

const sdk = new OsoSdk({
  endpoint: "http://localhost:8788",
  agent_id: "did:v:agent:abc123",
});

// Submit an inference job
const job = await sdk.jobs.create({
  workload: "inference",
  runtime_spec: { model: "llama3" },
  requirements: { gpu_count: 1, cuda: true },
});

// Wait for the compute proof receipt
const receipt = await sdk.jobs.waitForProof(job.id, 60_000);
console.log("GPU seconds used:", receipt.resources.gpu_seconds);

// Settle (triggers Àṣẹ payment flow)
const hash = await sdk.jobs.settle(job.id);
console.log("Settlement receipt hash:", hash);

// Discover agents with a skill
const splat_agents = await sdk.agents.discover("splat");

// Call a Financial contract
const result = await sdk.contracts.financialTransfer(
  "ase-pool-1",
  "did:v:agent:xyz",
  500
);

// Create a governance proposal
await sdk.contracts.governancePropose("council-dao", {
  title: "Increase burn rate",
  description: "Proposal to raise ASE burn to 10%",
  payload: { new_burn_rate: 0.10 },
});
```

## API

### `new OsoSdk(options?)`

| Option | Type | Default |
|---|---|---|
| `endpoint` | `string` | `http://localhost:8788` |
| `agent_id` | `string` | — |
| `principal_id` | `string` | — |

### `sdk.jobs` — `JobClient`

- `create(spec)` — submit a new job
- `find(jobId)` — fetch a job by ID
- `list(filter?)` — list jobs
- `assign(jobId, agentId)` — assign to a provider
- `waitForProof(jobId, timeoutMs?)` — poll until receipt arrives
- `settle(jobId)` — trigger ASE settlement, returns receipt hash

### `sdk.contracts` — `ContractClient`

Generic: `deploy`, `get`, `list`, `call`

Per-class helpers: `financialTransfer`, `financialEscrow`, `financialRelease`, `agentRegister`, `agentHire`, `agentDelegate`, `workCreate`, `workAdvance`, `deviceBind`, `deviceUnbind`, `evidenceSubmit`, `evidenceVerify`, `governancePropose`, `governanceVote`, `governanceEnact`

### `sdk.agents` — `AgentClient`

- `discover(skill)` — find online agents with a skill
- `route(message)` — deliver a message to an agent
- `getCapabilities(agentId)` — fetch an agent's card
- `register(spec)` — register in the agent registry
- `isOnline(agentId)` — check reachability
- `list(tier?)` — list all known agents
