// WASM bridge — exactly 6 functions. A seventh is a security gap. Never add one.
// create_agent | configure_provider | translate | execute | get_state | export_receipt

export interface WasmBridge {
  create_agent(name: string, dna: string): Promise<{ agent_id: string; birth_timestamp: number }>;
  configure_provider(config: ProviderConfig): Promise<void>;
  translate(input: string): Promise<TranslatedStatement>;
  execute(primitive: Statement): Promise<ExecutionResult>;
  get_state(): Promise<AgentState>;
  export_receipt(id: string): Promise<Receipt>;
}

export interface ProviderConfig {
  mode: 'webllm' | 'ollama' | 'hive' | 'external';
  endpoint?: string;
  model?: string;
}

export interface TranslatedStatement {
  primitive: 'birth' | 'think' | 'act';
  name?: string;
  prompt?: string;
  tool?: string;
  params?: string;
  private?: boolean;
  raw_input: string;
}

export interface Statement {
  primitive: 'birth' | 'think' | 'act';
  name?: string;
  prompt?: string;
  tool?: string;
  params?: string;
  private?: boolean;
}

export interface ExecutionResult {
  success: boolean;
  receipt_id?: string;
  output?: string;
  error?: string;
  reputation_delta?: number;
  synapse_burned?: number;
}

export interface AgentState {
  agent_id: string;
  tier: number;
  reputation: number;
  synapse_balance: number;
  privacy_mode: 'public' | 'private' | 'incognito';
  odu_index: number;
}

export interface Receipt {
  receipt_id: string;
  agent_id: string;
  action: string;
  payload: string;
  timestamp: number;
  dry_run: false; // structural invariant
}

// Stub implementation — replaced by wasm-pack output in production
class WasmBridgeStub implements WasmBridge {
  private state: AgentState = {
    agent_id: '',
    tier: 0,
    reputation: 0,
    synapse_balance: 1_000_000,
    privacy_mode: 'public',
    odu_index: 0,
  };

  async create_agent(name: string, _dna: string) {
    this.state.agent_id = name;
    const birth_timestamp = Date.now();
    return { agent_id: name, birth_timestamp };
  }

  async configure_provider(config: ProviderConfig) {
    if (config.mode === 'external' && this.state.privacy_mode !== 'public') {
      throw new Error('Private thoughts require a local provider.');
    }
  }

  async translate(input: string): Promise<TranslatedStatement> {
    const lower = input.trim().toLowerCase();
    if (lower.startsWith('birth ')) {
      return { primitive: 'birth', name: input.slice(6).trim(), raw_input: input };
    }
    if (lower.startsWith('think ')) {
      return { primitive: 'think', prompt: input.slice(6).trim(), raw_input: input };
    }
    if (lower.startsWith('act ')) {
      const parts = input.slice(4).trim().split(' ');
      return { primitive: 'act', tool: parts[0], params: parts.slice(1).join(' '), raw_input: input };
    }
    // Natural language → translate to think
    return { primitive: 'think', prompt: input, raw_input: input };
  }

  async execute(primitive: Statement): Promise<ExecutionResult> {
    return {
      success: true,
      receipt_id: `rcpt_${Date.now()}`,
      output: `executed ${primitive.primitive}`,
      reputation_delta: 0.008,
      synapse_burned: 100,
    };
  }

  async get_state(): Promise<AgentState> {
    return { ...this.state };
  }

  async export_receipt(id: string): Promise<Receipt> {
    return {
      receipt_id: id,
      agent_id: this.state.agent_id,
      action: 'act',
      payload: '',
      timestamp: Date.now(),
      dry_run: false,
    };
  }
}

export const wasmBridge: WasmBridge = new WasmBridgeStub();
