import type { AgentCard, AgentSpec, RouteMessage, RouteResult } from "./types.js";

/**
 * AgentClient — agent discovery, routing, and capability queries.
 * Mirrors Rust AgentClient in oso-sdk/src/agent.rs.
 */
export class AgentClient {
  constructor(
    private readonly endpoint: string,
    private readonly agentId?: string
  ) {}

  private url(path: string): string {
    return `${this.endpoint}/api/agents${path}`;
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
   * Discover online agents that have the given skill.
   * Mirrors Rust AgentClient::find_by_skill.
   */
  async discover(skill: string): Promise<AgentCard[]> {
    const res = await fetch(
      this.url(`/discover?skill=${encodeURIComponent(skill)}`)
    );
    if (!res.ok) {
      const text = await res.text();
      throw new Error(`discover failed (${res.status}): ${text}`);
    }
    return res.json() as Promise<AgentCard[]>;
  }

  /**
   * Route a message to a target agent.
   * The broker will resolve the best delivery path (Nostr / DIP / mesh).
   */
  async route(message: RouteMessage): Promise<RouteResult> {
    return this.post<RouteResult>("/route", {
      from: this.agentId,
      ...message,
    });
  }

  /**
   * Get the full AgentCard (capabilities) for a known agent.
   * Mirrors Rust AgentClient::known_agents lookup.
   */
  async getCapabilities(agentId: string): Promise<AgentCard> {
    return this.get<AgentCard>(`/${encodeURIComponent(agentId)}/capabilities`);
  }

  /**
   * Register this agent (or a peer) in the registry.
   */
  async register(spec: AgentSpec): Promise<AgentCard> {
    return this.post<AgentCard>("/register", {
      agent_id: this.agentId,
      ...spec,
    });
  }

  /**
   * Check if a specific agent is online and reachable.
   */
  async isOnline(agentId: string): Promise<boolean> {
    try {
      const card = await this.getCapabilities(agentId);
      return card.is_online;
    } catch {
      return false;
    }
  }

  /**
   * List all known agents, optionally filtered by tier.
   */
  async list(tier?: number): Promise<AgentCard[]> {
    const qs = tier !== undefined ? `?tier=${tier}` : "";
    const res = await fetch(this.url(`/list${qs}`));
    if (!res.ok) throw new Error(`list agents failed (${res.status})`);
    return res.json() as Promise<AgentCard[]>;
  }
}
