/**
 * Garden Plugin Manifest — Pattern 66.
 *
 * Full JSON schema for Omo-Koda2 plugins. Extends the base GardenPlugin type
 * with agents, commands, skills (Odu modules), and hooks that a plugin bundle
 * can declare. Mirrors `.claude-plugin/plugin.json` from the Claw source.
 */

import type { HookConfig, ToolConfig } from './types';

// ── Agent declaration ────────────────────────────────────────────────────────

/** An agent bundled inside a plugin — defined by a markdown file. */
export interface AgentManifestEntry {
  /** Unique name within the plugin */
  name: string;
  /** Path to the agent's markdown definition (relative to plugin root) */
  definitionPath: string;
  /** Agent role: "assistant" | "reviewer" | "writer" | "planner" | custom */
  role: string;
  /** Preferred model: "opus" | "sonnet" | "haiku" */
  model: 'opus' | 'sonnet' | 'haiku' | string;
  /** Minimum agent tier required to activate */
  minTier: number;
  /** Tools this agent may invoke */
  tools: string[];
}

// ── Command declaration ──────────────────────────────────────────────────────

/** A slash command provided by the plugin — defined as a markdown file. */
export interface CommandManifestEntry {
  /** Command name without slash (e.g. "pr-review") */
  name: string;
  description: string;
  /** Path to the command markdown file (relative to plugin root) */
  definitionPath: string;
  /** Optional argument schema */
  arguments?: CommandArgument[];
}

export interface CommandArgument {
  name: string;
  description: string;
  required: boolean;
  type: 'string' | 'number' | 'boolean' | 'file';
}

// ── Skill (Odu) declaration ──────────────────────────────────────────────────

/** An Odu knowledge module provided by the plugin. */
export interface SkillManifestEntry {
  name: string;
  description: string;
  /** Path to the skill markdown file */
  definitionPath: string;
  /** Invocation string shown in skill pickers */
  invocation: string;
  /** Paths to linked reference documents (Pattern 72) */
  references?: string[];
}

// ── Full manifest ────────────────────────────────────────────────────────────

/** The complete plugin manifest — serialised as `garden-plugin.json` at the plugin root. */
export interface GardenPluginManifest {
  /** Plugin schema version */
  schemaVersion: '1.0';
  name: string;
  version: string;
  description: string;
  author: string;
  license?: string;

  /** Required capability permissions (e.g. "read:workspace/*", "exec:bash") */
  permissions: string[];

  /** Minimum agent tier required */
  minTier: number;

  /** Plugin type tag */
  pluginType: 'bundled' | 'external' | 'marketplace';

  /** Bundled agents */
  agents: AgentManifestEntry[];

  /** Bundled slash commands */
  commands: CommandManifestEntry[];

  /** Bundled Odu skills */
  skills: SkillManifestEntry[];

  /** Hook registrations */
  hooks: HookConfig[];

  /** Tool definitions */
  tools: ToolConfig[];

  /** Lifecycle scripts */
  lifecycle?: {
    init?: string;
    shutdown?: string;
  };

  /** IPFS CID of the plugin bundle (set by Garden on publish) */
  contentCid?: string;

  /** Sui on-chain object address (set after on-chain registration) */
  onChainAddress?: string;
}

// ── Parsing ──────────────────────────────────────────────────────────────────

/** Parse and validate a raw JSON object as a GardenPluginManifest. */
export function parseManifest(raw: unknown): GardenPluginManifest {
  if (typeof raw !== 'object' || raw === null) {
    throw new Error('Manifest must be a JSON object');
  }
  const obj = raw as Record<string, unknown>;

  if (!obj.name || typeof obj.name !== 'string') throw new Error('Manifest missing "name"');
  if (!obj.version || typeof obj.version !== 'string') throw new Error('Manifest missing "version"');

  return {
    schemaVersion: '1.0',
    name: obj.name,
    version: obj.version,
    description: (obj.description as string) ?? '',
    author: (obj.author as string) ?? '',
    license: obj.license as string | undefined,
    permissions: (obj.permissions as string[]) ?? [],
    minTier: (obj.minTier as number) ?? 0,
    pluginType: (obj.pluginType as GardenPluginManifest['pluginType']) ?? 'external',
    agents: (obj.agents as AgentManifestEntry[]) ?? [],
    commands: (obj.commands as CommandManifestEntry[]) ?? [],
    skills: (obj.skills as SkillManifestEntry[]) ?? [],
    hooks: (obj.hooks as HookConfig[]) ?? [],
    tools: (obj.tools as ToolConfig[]) ?? [],
    lifecycle: obj.lifecycle as GardenPluginManifest['lifecycle'],
    contentCid: obj.contentCid as string | undefined,
    onChainAddress: obj.onChainAddress as string | undefined,
  };
}

/** Validate a manifest, returning a list of validation errors (empty = valid). */
export function validateManifest(manifest: GardenPluginManifest): string[] {
  const errors: string[] = [];

  if (!manifest.name) errors.push('name is required');
  if (!manifest.version) errors.push('version is required');
  if (!/^\d+\.\d+\.\d+/.test(manifest.version)) {
    errors.push(`version "${manifest.version}" is not semver`);
  }

  for (const perm of manifest.permissions) {
    if (!perm.includes(':')) {
      errors.push(`permission "${perm}" must use "action:resource" format`);
    }
  }

  for (const agent of manifest.agents) {
    if (!agent.name) errors.push('agent entry missing "name"');
    if (!agent.definitionPath) errors.push(`agent "${agent.name}" missing "definitionPath"`);
  }

  for (const cmd of manifest.commands) {
    if (!cmd.name) errors.push('command entry missing "name"');
    if (!cmd.definitionPath) errors.push(`command "${cmd.name}" missing "definitionPath"`);
  }

  return errors;
}
