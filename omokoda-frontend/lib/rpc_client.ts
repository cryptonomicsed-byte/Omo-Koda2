/**
 * RustRpcClient — WebSocket bridge to the Ọmọ Kọ́dà Steward (Rust backend).
 *
 * All three primitives (birth / think / act) are routed through this client.
 * Privacy mode changes are sent before the next think/act call.
 *
 * Three primitives only — no other methods on this interface.
 */

export type PrivacyMode = 'public' | 'private' | 'incognito';

export interface BirthParams {
  name: string;
  metadata?: Record<string, string>;
}

export interface ThinkOptions {
  private?: boolean;
  provider?: string;
}

export interface ActOptions {
  sandbox?: boolean;
}

export interface BirthReceipt {
  agent_id: string;
  name: string;
  birth_timestamp: number;
  mnemonic: string;
  dna_fingerprint: string;
  tier: number;
  reputation: number;
  synapse_balance: number;
  pet: string;
}

export interface ThoughtReceipt {
  thought: string;
  reputation_delta: number;
  synapse_burned: number;
  provider: string;
  private: boolean;
}

export interface ActReceipt {
  tool: string;
  output: string;
  success: boolean;
  reputation_delta: number;
  synapse_burned: number;
  sandboxed: boolean;
}

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
  timer: ReturnType<typeof setTimeout>;
}

export class RustRpcClient {
  private ws: WebSocket | null = null;
  private pending = new Map<string, PendingRequest>();
  private reqCounter = 0;
  private privacyMode: PrivacyMode = 'public';

  constructor(private readonly url: string = 'ws://localhost:8765') {}

  private getSocket(): WebSocket {
    if (!this.ws || this.ws.readyState === WebSocket.CLOSED) {
      this.ws = new WebSocket(this.url);
      this.ws.onmessage = (event) => this.handleMessage(event);
      this.ws.onclose = () => {
        this.pending.forEach((req, id) => {
          req.reject(new Error('WebSocket closed'));
          clearTimeout(req.timer);
          this.pending.delete(id);
        });
      };
    }
    return this.ws;
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const msg = JSON.parse(event.data as string);
      const req = this.pending.get(msg.id);
      if (!req) return;
      clearTimeout(req.timer);
      this.pending.delete(msg.id);
      if (msg.error) {
        req.reject(new Error(msg.error));
      } else {
        req.resolve(msg.result);
      }
    } catch {
      // Malformed message — ignore
    }
  }

  private sendRequest(method: string, params: unknown, timeoutMs = 30_000): Promise<unknown> {
    const id = String(++this.reqCounter);
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`Request timed out: ${method}`));
      }, timeoutMs);

      this.pending.set(id, { resolve, reject, timer });

      const socket = this.getSocket();
      const send = () => socket.send(JSON.stringify({ id, method, params }));

      if (socket.readyState === WebSocket.OPEN) {
        send();
      } else {
        socket.addEventListener('open', send, { once: true });
      }
    });
  }

  // --- The Three Primitives ---

  async birth(params: BirthParams): Promise<BirthReceipt> {
    return this.sendRequest('birth', params) as Promise<BirthReceipt>;
  }

  async think(prompt: string, options: ThinkOptions = {}): Promise<ThoughtReceipt> {
    return this.sendRequest('think', {
      prompt,
      private: options.private ?? (this.privacyMode !== 'public'),
      provider: options.provider,
    }) as Promise<ThoughtReceipt>;
  }

  async act(tool: string, args: Record<string, unknown>, options: ActOptions = {}): Promise<ActReceipt> {
    return this.sendRequest('act', {
      tool,
      args,
      sandbox: options.sandbox ?? true,
    }) as Promise<ActReceipt>;
  }

  // --- Privacy control ---

  async setPrivacyMode(mode: PrivacyMode): Promise<void> {
    this.privacyMode = mode;
    await this.sendRequest('set_privacy_mode', { mode });
  }

  getPrivacyMode(): PrivacyMode {
    return this.privacyMode;
  }

  // --- Memory ---

  async forget(): Promise<void> {
    await this.sendRequest('forget', {});
  }

  // --- Lifecycle ---

  close(): void {
    this.ws?.close();
  }
}

export const rpcClient = new RustRpcClient();
