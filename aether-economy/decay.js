/**
 * ToC Dopamine Decay Engine
 * Exactly 1% daily decay on Dopamine (ToC-D) balances.
 * Deterministic, testable, runs once per day per agent.
 */

import { EventEmitter } from 'events';
import { TOKEN_TYPE } from './token.js';

const DECAY_RATE = 0.01; // Exactly 1% per day
const DAY_MS = 86_400_000;

export const DECAY_CONFIG = { DECAY_RATE, DAY_MS };

export class DecayEngine extends EventEmitter {
  constructor(walletRegistry) {
    super();
    this.registry = walletRegistry;
    this.lastDecayRun = new Map(); // agentId → timestamp
    this.history = [];
    this._interval = null;
  }

  /**
   * Apply 1% daily decay to a single agent's Dopamine.
   */
  decayAgent(agentId, nowMs = Date.now()) {
    const wallet = this.registry.get(agentId);
    if (!wallet) throw new Error(`No wallet for ${agentId}`);
    if (wallet.ownerType !== 'agent') return 0;

    const lastRun = this.lastDecayRun.get(agentId) || wallet.created;
    const elapsed = nowMs - lastRun;
    if (elapsed < DAY_MS) return 0;

    const days = Math.floor(elapsed / DAY_MS);
    const currentBalance = wallet.balance(TOKEN_TYPE.TOC_D);
    if (currentBalance <= 0) return 0;

    // Exact compound decay: remaining = balance * (1 - rate)^days
    const retainFactor = Math.pow(1 - DECAY_RATE, days);
    const remaining = Math.floor(currentBalance * retainFactor);
    const decayAmount = currentBalance - remaining;

    if (decayAmount > 0) {
      wallet.spend(TOKEN_TYPE.TOC_D, decayAmount, 'daily_decay');
    }

    this.lastDecayRun.set(agentId, nowMs);

    const record = {
      agentId,
      days,
      previousBalance: currentBalance,
      decayed: decayAmount,
      newBalance: remaining,
      timestamp: nowMs,
    };
    this.history.push(record);
    this.emit('decay', record);

    return decayAmount;
  }

  decayAll(nowMs = Date.now()) {
    const results = [];
    for (const [agentId, wallet] of this.registry.wallets) {
      if (wallet.ownerType === 'agent') {
        const decayed = this.decayAgent(agentId, nowMs);
        if (decayed > 0) {
          results.push({ agentId, decayed });
        }
      }
    }
    return results;
  }

  startScheduler(intervalMs = DAY_MS) {
    if (this._interval) return;
    this._interval = setInterval(() => {
      this.decayAll();
    }, intervalMs);
  }

  stopScheduler() {
    if (this._interval) {
      clearInterval(this._interval);
      this._interval = null;
    }
  }
}
