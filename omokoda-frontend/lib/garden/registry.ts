import type { GardenPlugin, PluginState, ReconcileResult } from './types';

/**
 * GardenRegistry — in-process plugin lifecycle manager.
 *
 * Tracks the installed set, handles enable/disable/error state transitions,
 * and reconciles against a desired-state list (install missing, update stale,
 * remove undesired).
 */
export class GardenRegistry {
  private plugins: Map<string, GardenPlugin> = new Map();

  register(plugin: GardenPlugin): void {
    this.plugins.set(plugin.name, { ...plugin });
  }

  unregister(name: string): boolean {
    return this.plugins.delete(name);
  }

  get(name: string): GardenPlugin | undefined {
    return this.plugins.get(name);
  }

  list(): GardenPlugin[] {
    return Array.from(this.plugins.values());
  }

  listByState(state: PluginState): GardenPlugin[] {
    return this.list().filter((p) => p.state === state);
  }

  enable(name: string): boolean {
    const p = this.plugins.get(name);
    if (!p) return false;
    this.plugins.set(name, { ...p, state: 'enabled' });
    return true;
  }

  disable(name: string): boolean {
    const p = this.plugins.get(name);
    if (!p) return false;
    this.plugins.set(name, { ...p, state: 'disabled' });
    return true;
  }

  setError(name: string, _error: string): void {
    const p = this.plugins.get(name);
    if (p) this.plugins.set(name, { ...p, state: 'error' });
  }

  /**
   * Reconcile installed plugins against a desired state list.
   * - Install plugins that are missing.
   * - Update plugins whose version has changed.
   * - Remove plugins not in the desired list.
   */
  reconcile(desired: GardenPlugin[]): ReconcileResult {
    const result: ReconcileResult = { installed: [], updated: [], removed: [], errors: [] };
    const desiredNames = new Set(desired.map((p) => p.name));

    for (const plugin of desired) {
      const existing = this.plugins.get(plugin.name);
      if (!existing) {
        this.register(plugin);
        result.installed.push(plugin.name);
      } else if (existing.version !== plugin.version) {
        this.plugins.set(plugin.name, { ...plugin });
        result.updated.push(plugin.name);
      }
    }

    for (const name of this.plugins.keys()) {
      if (!desiredNames.has(name)) {
        this.plugins.delete(name);
        result.removed.push(name);
      }
    }

    return result;
  }

  /** Collect all hook event names declared by enabled plugins */
  registeredHookEvents(): Set<string> {
    const events = new Set<string>();
    for (const p of this.plugins.values()) {
      if (p.state === 'enabled') {
        for (const h of p.hooks) events.add(h.event);
      }
    }
    return events;
  }

  /** Discover all tools provided by enabled plugins */
  registeredTools(): Array<{ plugin: string; tool: GardenPlugin['tools'][number] }> {
    const tools: Array<{ plugin: string; tool: GardenPlugin['tools'][number] }> = [];
    for (const p of this.plugins.values()) {
      if (p.state === 'enabled') {
        for (const t of p.tools) tools.push({ plugin: p.name, tool: t });
      }
    }
    return tools;
  }
}
