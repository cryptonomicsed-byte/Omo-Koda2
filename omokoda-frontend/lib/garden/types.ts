export type PluginType = 'bundled' | 'external' | 'marketplace';
export type PluginState = 'enabled' | 'disabled' | 'error' | 'installing' | 'uninstalling';

export interface PluginVersion {
  major: number;
  minor: number;
  patch: number;
}

export function parseVersion(v: string): PluginVersion {
  const parts = v.split('.').map(Number);
  return { major: parts[0] ?? 0, minor: parts[1] ?? 0, patch: parts[2] ?? 0 };
}

export function compareVersions(a: PluginVersion, b: PluginVersion): number {
  if (a.major !== b.major) return a.major - b.major;
  if (a.minor !== b.minor) return a.minor - b.minor;
  return a.patch - b.patch;
}

export interface HookConfig {
  event: string;
  command: string;
  blocking: boolean;
}

export interface ToolConfig {
  name: string;
  description: string;
  command: string;
  requiredTier: number;
  isWrite: boolean;
}

export interface GardenPlugin {
  name: string;
  version: string;
  description: string;
  author: string;
  pluginType: PluginType;
  state: PluginState;
  permissions: string[];
  hooks: HookConfig[];
  tools: ToolConfig[];
  installPath?: string;
  installedAt?: number;
  sourceUrl?: string;
  /** IPFS CID or Arweave transaction ID of the plugin content bundle */
  contentHash?: string;
  /** Sui object address of the on-chain plugin record */
  onChainAddress?: string;
}

export interface MarketplaceListing {
  id: string;
  plugin: GardenPlugin;
  /** Sui address of the publisher */
  publisher: string;
  publishedAt: number;
  /** IPFS CIDv1 of the plugin bundle */
  contentCid: string;
  downloads: number;
  rating: number;
  tags: string[];
  latestVersion: string;
  previousVersions: string[];
  onChainVerified: boolean;
}

export interface InstallOptions {
  skipVerification?: boolean;
  targetPath?: string;
}

export interface ReconcileResult {
  installed: string[];
  updated: string[];
  removed: string[];
  errors: Array<{ name: string; error: string }>;
}
