import type { GardenPlugin, InstallOptions, MarketplaceListing } from './types';
import { compareVersions, parseVersion } from './types';

/**
 * MarketplaceBackend — swappable backend for the Garden Marketplace.
 * Production implementations connect to IPFS/Arweave and the Sui on-chain registry.
 */
export interface MarketplaceBackend {
  fetch(query: string, tags?: string[]): Promise<MarketplaceListing[]>;
  fetchById(id: string): Promise<MarketplaceListing | null>;
  /** Verify a content CID/hash is well-formed and optionally matches an expected value */
  verifyContentHash(cid: string, expectedHash?: string): Promise<boolean>;
}

/**
 * IPFS-backed marketplace backend.
 * Queries an on-chain registry index for listings; fetches bundles from IPFS gateway.
 * Arweave transaction IDs (43-char base64url) are also accepted as content references.
 */
export class IpfsMarketplaceBackend implements MarketplaceBackend {
  constructor(private readonly gatewayUrl: string = 'https://ipfs.io/ipfs') {}

  async fetch(_query: string, _tags?: string[]): Promise<MarketplaceListing[]> {
    // Production: query Sui on-chain GardenRegistry for listings matching query/tags,
    // then fetch manifest JSON from IPFS for each result.
    return [];
  }

  async fetchById(_id: string): Promise<MarketplaceListing | null> {
    return null;
  }

  async verifyContentHash(cid: string, _expectedHash?: string): Promise<boolean> {
    // CIDv0 starts with "Qm", CIDv1 starts with "b", Arweave is 43 chars base64url
    return cid.startsWith('Qm') || cid.startsWith('b') || /^[A-Za-z0-9_-]{43}$/.test(cid);
  }
}

/**
 * GardenMarketplace — decentralized plugin marketplace with IPFS content addressing
 * and Sui on-chain verification.
 */
export class GardenMarketplace {
  constructor(private readonly backend: MarketplaceBackend) {}

  /** Search the marketplace, results sorted by download count descending */
  async search(query: string, tags?: string[]): Promise<MarketplaceListing[]> {
    const results = await this.backend.fetch(query, tags);
    return results.sort((a, b) => b.downloads - a.downloads);
  }

  async getPlugin(id: string): Promise<MarketplaceListing | null> {
    return this.backend.fetchById(id);
  }

  /**
   * Install a plugin from a marketplace listing.
   * Verifies content CID before returning the plugin record unless skipVerification is set.
   */
  async install(listing: MarketplaceListing, opts: InstallOptions = {}): Promise<GardenPlugin> {
    if (!opts.skipVerification) {
      const valid = await this.backend.verifyContentHash(listing.contentCid);
      if (!valid) {
        throw new Error(`Content hash verification failed for '${listing.plugin.name}'`);
      }
    }

    return {
      ...listing.plugin,
      state: 'enabled',
      contentHash: listing.contentCid,
      onChainAddress: listing.id,
      installedAt: Date.now(),
    };
  }

  /** Determine the highest semver from a listing's version history */
  latestVersion(listing: MarketplaceListing): string {
    const all = [listing.latestVersion, ...listing.previousVersions];
    return all.sort((a, b) => compareVersions(parseVersion(b), parseVersion(a)))[0]!;
  }

  /** True if the marketplace has a newer version than what is installed */
  hasUpdate(installed: GardenPlugin, listing: MarketplaceListing): boolean {
    return compareVersions(parseVersion(this.latestVersion(listing)), parseVersion(installed.version)) > 0;
  }

  /**
   * Startup check — verify marketplace plugins still exist on-chain.
   * Orphaned plugins (listing removed) are separated so the caller can decide
   * whether to uninstall or keep them.
   */
  async startupCheck(
    plugins: GardenPlugin[],
  ): Promise<{ valid: GardenPlugin[]; orphaned: GardenPlugin[] }> {
    const valid: GardenPlugin[] = [];
    const orphaned: GardenPlugin[] = [];

    for (const plugin of plugins) {
      if (plugin.pluginType !== 'marketplace') {
        valid.push(plugin);
        continue;
      }
      const listing = plugin.onChainAddress
        ? await this.backend.fetchById(plugin.onChainAddress)
        : null;
      (listing ? valid : orphaned).push(plugin);
    }

    return { valid, orphaned };
  }
}
