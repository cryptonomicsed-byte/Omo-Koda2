/**
 * Toolkit Bundles — Pattern 74.
 *
 * A PluginToolkit is a pre-packaged ensemble of agents + commands + skills
 * assembled for a specific workflow (e.g. "PR Review Toolkit" = 6 agents + 1 command).
 *
 * Mirrors Claw's `pr-review-toolkit/` pattern.
 */

import type { AgentManifestEntry, CommandManifestEntry, SkillManifestEntry } from './manifest';
import type { GardenPlugin } from './types';

// ── Toolkit manifest ─────────────────────────────────────────────────────────

export interface ToolkitManifest {
  name: string;
  version: string;
  description: string;
  /** Workflow category (e.g. "code-review", "documentation", "testing") */
  category: string;
  /** Ordered list of agents in the ensemble */
  agents: AgentManifestEntry[];
  /** Commands exposed by the toolkit */
  commands: CommandManifestEntry[];
  /** Skills bundled with the toolkit */
  skills: SkillManifestEntry[];
  /** Suggested orchestration strategy */
  orchestration: 'sequential' | 'parallel' | 'hierarchical' | 'pipeline';
  /** Entry command — run this to invoke the full toolkit */
  entryCommand?: string;
}

// ── Toolkit instance ─────────────────────────────────────────────────────────

export interface ToolkitRunOptions {
  /** Override the orchestration strategy */
  orchestration?: ToolkitManifest['orchestration'];
  /** Arguments forwarded to the entry command */
  args?: Record<string, string>;
  /** Agents to exclude from this run */
  excludeAgents?: string[];
}

export interface ToolkitRunResult {
  toolkitName: string;
  agentsActivated: string[];
  commandsRun: string[];
  status: 'ok' | 'partial' | 'error';
  errors: string[];
}

// ── PluginToolkit ─────────────────────────────────────────────────────────────

/**
 * PluginToolkit — loads a toolkit manifest and provides helpers for activating
 * the ensemble, resolving agents, and composing commands.
 */
export class PluginToolkit {
  constructor(private readonly manifest: ToolkitManifest) {}

  get name(): string {
    return this.manifest.name;
  }

  get category(): string {
    return this.manifest.category;
  }

  get agentCount(): number {
    return this.manifest.agents.length;
  }

  get commandCount(): number {
    return this.manifest.commands.length;
  }

  /** Return agents filtered by role. */
  agentsByRole(role: string): AgentManifestEntry[] {
    return this.manifest.agents.filter((a) => a.role === role);
  }

  /** Return agents matching a minimum tier. */
  agentsByMinTier(tier: number): AgentManifestEntry[] {
    return this.manifest.agents.filter((a) => a.minTier <= tier);
  }

  /** Find a command by name. */
  findCommand(name: string): CommandManifestEntry | undefined {
    return this.manifest.commands.find((c) => c.name === name);
  }

  /** Check whether a named skill is included. */
  hasSkill(name: string): boolean {
    return this.manifest.skills.some((s) => s.name === name);
  }

  /**
   * Build an activation plan: the ordered list of agents to run given the
   * orchestration strategy and any exclusions.
   */
  buildActivationPlan(opts: ToolkitRunOptions = {}): AgentManifestEntry[] {
    const strategy = opts.orchestration ?? this.manifest.orchestration;
    const excluded = new Set(opts.excludeAgents ?? []);
    const eligible = this.manifest.agents.filter((a) => !excluded.has(a.name));

    switch (strategy) {
      case 'sequential':
      case 'pipeline':
        return eligible;

      case 'parallel':
        // All eligible agents run concurrently — return in alphabetical order for determinism
        return [...eligible].sort((a, b) => a.name.localeCompare(b.name));

      case 'hierarchical': {
        // Lead agent first (role === "lead"), workers after
        const leads = eligible.filter((a) => a.role === 'lead');
        const workers = eligible.filter((a) => a.role !== 'lead');
        return [...leads, ...workers];
      }

      default:
        return eligible;
    }
  }

  /**
   * Simulate a toolkit run (returns a result stub; real execution drives the swarm).
   */
  run(opts: ToolkitRunOptions = {}): ToolkitRunResult {
    const plan = this.buildActivationPlan(opts);
    const commands = this.manifest.entryCommand ? [this.manifest.entryCommand] : [];

    return {
      toolkitName: this.manifest.name,
      agentsActivated: plan.map((a) => a.name),
      commandsRun: commands,
      status: plan.length > 0 ? 'ok' : 'partial',
      errors: plan.length === 0 ? ['No eligible agents in activation plan'] : [],
    };
  }

  /** Serialise the manifest to JSON. */
  toJSON(): ToolkitManifest {
    return { ...this.manifest };
  }
}

// ── ToolkitRegistry ───────────────────────────────────────────────────────────

/** In-memory registry of loaded toolkits. */
export class ToolkitRegistry {
  private toolkits: Map<string, PluginToolkit> = new Map();

  register(toolkit: PluginToolkit): void {
    this.toolkits.set(toolkit.name, toolkit);
  }

  get(name: string): PluginToolkit | undefined {
    return this.toolkits.get(name);
  }

  listByCategory(category: string): PluginToolkit[] {
    return Array.from(this.toolkits.values()).filter((t) => t.category === category);
  }

  list(): PluginToolkit[] {
    return Array.from(this.toolkits.values()).sort((a, b) => a.name.localeCompare(b.name));
  }

  /** Convert a GardenPlugin to a minimal ToolkitManifest and register it. */
  registerFromPlugin(plugin: GardenPlugin): PluginToolkit {
    const manifest: ToolkitManifest = {
      name: plugin.name,
      version: plugin.version,
      description: plugin.description,
      category: 'general',
      agents: [],
      commands: [],
      skills: [],
      orchestration: 'sequential',
    };
    const toolkit = new PluginToolkit(manifest);
    this.register(toolkit);
    return toolkit;
  }

  count(): number {
    return this.toolkits.size;
  }
}
