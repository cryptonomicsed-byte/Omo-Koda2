export interface BirthReceipt {
  agent_id: string;
  birth_timestamp: number;
}

export interface ThoughtReceipt {
  thought_id: string;
  hermetic_score: number;
}

export interface ActReceipt {
  act_id: string;
  tool: string;
  quality: string;
  synapse_cost: number;
}

type Resolve<T> = (value: T) => void;
type Reject = (reason: unknown) => void;

interface PendingRequest<T = unknown> {
  resolve: Resolve<T>;
  reject: Reject;
  timer: ReturnType<typeof setTimeout>;
}

let _nextId = 1;

export class RustRpcClient {
  private ws: WebSocket;
  private pending = new Map<string, PendingRequest>();
  private privacyMode: "public" | "private" | "incognito" = "public";

  constructor(url: string) {
    this.ws = new WebSocket(url);
    this.ws.addEventListener("message", (ev) => this.handleMessage(ev));
  }

  birth(params: { name: string; metadata?: Record<string, string> }): Promise<BirthReceipt> {
    return this.sendRequest<BirthReceipt>("birth", params);
  }

  think(prompt: string, options?: { private?: boolean }): Promise<ThoughtReceipt> {
    return this.sendRequest<ThoughtReceipt>("think", { prompt, ...options });
  }

  act(tool: string, args: Record<string, unknown>, options?: { sandbox?: boolean }): Promise<ActReceipt> {
    return this.sendRequest<ActReceipt>("act", { tool, params: JSON.stringify(args), ...options });
  }

  setPrivacyMode(mode: "public" | "private" | "incognito"): Promise<void> {
    this.privacyMode = mode;
    return this.sendRequest<void>("set_privacy_mode", { mode });
  }

  forget(): Promise<void> {
    return this.sendRequest<void>("forget", {});
  }

  private sendRequest<T>(method: string, params: unknown, timeoutMs = 30_000): Promise<T> {
    return new Promise<T>((resolve, reject) => {
      const id = String(_nextId++);
      const timer = setTimeout(() => {
        this.pending.delete(id);
        reject(new Error(`RPC timeout: ${method}`));
      }, timeoutMs);
      this.pending.set(id, { resolve: resolve as Resolve<unknown>, reject, timer });
      this.ws.send(JSON.stringify({ id, method, params }));
    });
  }

  private handleMessage(ev: MessageEvent): void {
    let msg: { id: string; result?: unknown; error?: string };
    try {
      msg = JSON.parse(ev.data as string);
    } catch {
      return;
    }
    const pending = this.pending.get(msg.id);
    if (!pending) return;
    clearTimeout(pending.timer);
    this.pending.delete(msg.id);
    if (msg.error) {
      pending.reject(new Error(msg.error));
    } else {
      pending.resolve(msg.result);
    }
  }
}
