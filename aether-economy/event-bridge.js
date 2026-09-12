/**
 * Ọ̀ṢỌ́VM → Swibe Event Bridge
 *
 * Cross-system flow:
 *   Human pays 0.01 ÀṢẸ →
 *   AIO triggers agent birth event →
 *   Ọ̀ṢỌ́VM mints agent identity + ToC-D + ToC-S →
 *   Event emitted →
 *   Swibe listener detects event →
 *   Agent wallet created →
 *   ToC tokens assigned →
 *   Agent becomes active
 *
 * Hard constraints:
 *   - Agent NEVER holds ÀṢẸ
 *   - ToC minted ONLY at agent birth
 *   - Per agent: 86B Dopamine + 86M Synapse
 */

import { EventEmitter } from 'events';

const AGENT_BIRTH_FEE = 0.01; // ÀṢẸ

export class EventBridge extends EventEmitter {
  constructor(tocEconomy, elegbaraRouter = null) {
    super();
    this.economy = tocEconomy;
    this.router = elegbaraRouter;
    this.pendingEvents = [];
    this.processedEvents = [];
    this._listener = null;
  }

  /**
   * Process an agent birth event from Ọ̀ṢỌ́VM.
   * This is the complete cross-system flow.
   *
   * @param {object} vmEvent - Event from Ọ̀ṢỌ́VM
   * @param {string} vmEvent.agentId - New agent identifier
   * @param {string} vmEvent.creatorId - Human who paid 0.01 ÀṢẸ
   * @param {string} vmEvent.txHash - On-chain transaction hash
   * @param {number} vmEvent.timestamp - Block timestamp
   * @param {object} vmEvent.tocEndowment - Optional override endowments
   */
  async processAgentBirth(vmEvent) {
    const { agentId, creatorId, txHash, timestamp, tocEndowment } = vmEvent;

    if (!agentId || !creatorId) {
      throw new Error('Agent birth event missing agentId or creatorId');
    }

    // Step 1: Process birth fee split through Elegbára (if router available)
    let birthSplit = null;
    if (this.router) {
      birthSplit = this.router.processAgentBirth(creatorId);
    }

    // Step 2: Create agent wallet + mint ToC endowment
    const vmSignal = tocEndowment || {
      dopamine_endowment: 86_000_000_000,
      synapse_endowment: 86_000_000,
    };
    const _wallet = await this.economy.spawnAgent(agentId, creatorId, 10, vmSignal);

    // Step 3: Record processed event
    const processed = {
      type: 'agent_birth',
      agentId,
      creatorId,
      txHash: txHash || null,
      timestamp: timestamp || Date.now(),
      birthSplit,
      tocEndowment: vmSignal,
      walletCreated: true,
    };
    this.processedEvents.push(processed);
    this.emit('agent_birth_complete', processed);

    return processed;
  }

  /**
   * Queue an event for processing (async bridge mode).
   */
  queueEvent(vmEvent) {
    this.pendingEvents.push({
      ...vmEvent,
      queued: Date.now(),
    });
    this.emit('event_queued', vmEvent);
  }

  /**
   * Process all pending events.
   */
  async processQueue() {
    const results = [];
    while (this.pendingEvents.length > 0) {
      const event = this.pendingEvents.shift();
      try {
        const result = await this.processAgentBirth(event);
        results.push(result);
      } catch (err) {
        this.emit('event_error', { event, error: err.message });
        results.push({ event, error: err.message });
      }
    }
    return results;
  }

  /**
   * Start polling listener (simulates blockchain event subscription).
   */
  startListener(pollIntervalMs = 5000) {
    if (this._listener) return;
    this._listener = setInterval(() => {
      if (this.pendingEvents.length > 0) {
        this.processQueue();
      }
    }, pollIntervalMs);
    this.emit('listener_started', { pollIntervalMs });
  }

  stopListener() {
    if (this._listener) {
      clearInterval(this._listener);
      this._listener = null;
      this.emit('listener_stopped');
    }
  }

  getProcessedEvents(limit = 50) {
    return this.processedEvents.slice(-limit);
  }

  getPendingCount() {
    return this.pendingEvents.length;
  }
}

export const BRIDGE_CONFIG = { AGENT_BIRTH_FEE };
