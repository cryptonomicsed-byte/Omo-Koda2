/**
 * Swibe ToC Escrow — Phase 7+
 * Job state tracking at the agent layer.
 * Àṣẹ escrow is enforced at the VM layer (OSOVM op_job_payment).
 * Swibe tracks job lifecycle and receives Dopamine conversion signals.
 *
 * Hardening: auto-refund if job not completed within timeout.
 */

import { EventEmitter } from 'events';
import { TOKEN_TYPE } from './token.js';

const ESCROW_STATUS = {
  LOCKED: 'locked',
  RELEASED: 'released',
  DISPUTED: 'disputed',
  REFUNDED: 'refunded',
  FORFEITED: 'forfeited',
  EXPIRED: 'expired',
};

const DEFAULT_TIMEOUT_MS = 7 * 24 * 60 * 60 * 1000; // 7 days

export class EscrowEngine extends EventEmitter {
  constructor(walletRegistry, royaltyEngine = null) {
    super();
    this.registry = walletRegistry;
    this.royalty = royaltyEngine;
    this.escrows = new Map();
    this._nextId = 1;
  }

  create(humanId, agentId, aseAmount, jobDescription = '', timeoutMs = DEFAULT_TIMEOUT_MS) {
    const humanWallet = this.registry.get(humanId);
    if (!humanWallet) throw new Error(`No wallet for ${humanId}`);
    if (humanWallet.ownerType !== 'human') throw new Error('Only humans can create escrows');

    const balance = humanWallet.balance(TOKEN_TYPE.ASE);
    if (balance < aseAmount) throw new Error(`Insufficient Àṣẹ: have ${balance}, need ${aseAmount}`);

    humanWallet.lock(TOKEN_TYPE.ASE, aseAmount, 'escrow');

    const id = `escrow_${this._nextId++}`;
    const now = Date.now();
    const escrow = {
      id,
      humanId,
      agentId,
      amount: aseAmount,
      job: jobDescription,
      status: ESCROW_STATUS.LOCKED,
      created: now,
      expiresAt: now + timeoutMs,
      resolved: null,
    };

    this.escrows.set(id, escrow);
    this.emit('create', escrow);
    return escrow;
  }

  release(escrowId) {
    const escrow = this.escrows.get(escrowId);
    if (!escrow) throw new Error(`Escrow not found: ${escrowId}`);
    if (escrow.status !== ESCROW_STATUS.LOCKED) throw new Error(`Escrow is ${escrow.status}`);

    escrow.status = ESCROW_STATUS.RELEASED;
    escrow.resolved = Date.now();

    // Unlock the escrowed Àṣẹ from human wallet (it was locked, not spent)
    const _humanWallet = this.registry.get(escrow.humanId);

    let distribution = null;
    if (this.royalty) {
      distribution = this.royalty.processJobPayment(escrow.agentId, escrow.amount);
    }

    if (distribution) {
      // Agent gets their share
      const agentWallet = this.registry.get(escrow.agentId);
      if (agentWallet) {
        agentWallet.receive(TOKEN_TYPE.ASE, distribution.agent.receives);
      }
      // Protocol burn is implicit (tokens stay locked, never unlocked = burned)
    } else {
      // No royalty engine — full amount to agent
      const agentWallet = this.registry.get(escrow.agentId);
      if (agentWallet) {
        agentWallet.receive(TOKEN_TYPE.ASE, escrow.amount);
      }
    }

    this.emit('release', { escrow, distribution });
    return { escrow, distribution };
  }

  refund(escrowId) {
    const escrow = this.escrows.get(escrowId);
    if (!escrow) throw new Error(`Escrow not found: ${escrowId}`);
    if (escrow.status !== ESCROW_STATUS.LOCKED) throw new Error(`Escrow is ${escrow.status}`);

    escrow.status = ESCROW_STATUS.REFUNDED;
    escrow.resolved = Date.now();

    const humanWallet = this.registry.get(escrow.humanId);
    if (humanWallet) {
      humanWallet.unlock(TOKEN_TYPE.ASE, 'escrow');
    }

    this.emit('refund', escrow);
    return escrow;
  }

  forfeit(escrowId, reason = 'false_job') {
    const escrow = this.escrows.get(escrowId);
    if (!escrow) throw new Error(`Escrow not found: ${escrowId}`);
    if (escrow.status !== ESCROW_STATUS.LOCKED) throw new Error(`Escrow is ${escrow.status}`);

    escrow.status = ESCROW_STATUS.FORFEITED;
    escrow.resolved = Date.now();

    this.emit('forfeit', { escrow, reason });
    return { escrow, reason, burned: escrow.amount };
  }

  dispute(escrowId) {
    const escrow = this.escrows.get(escrowId);
    if (!escrow) throw new Error(`Escrow not found: ${escrowId}`);
    if (escrow.status !== ESCROW_STATUS.LOCKED) throw new Error(`Escrow is ${escrow.status}`);

    escrow.status = ESCROW_STATUS.DISPUTED;
    this.emit('dispute', escrow);
    return escrow;
  }

  /**
   * Auto-refund any escrows that have exceeded their timeout.
   * Returns array of expired escrow records.
   */
  expireStale(now = Date.now()) {
    const expired = [];
    for (const [_id, escrow] of this.escrows) {
      if (escrow.status === ESCROW_STATUS.LOCKED && now >= escrow.expiresAt) {
        escrow.status = ESCROW_STATUS.EXPIRED;
        escrow.resolved = now;

        // Refund the human
        const humanWallet = this.registry.get(escrow.humanId);
        if (humanWallet) {
          humanWallet.unlock(TOKEN_TYPE.ASE, 'escrow');
        }

        expired.push(escrow);
        this.emit('expire', escrow);
      }
    }
    return expired;
  }

  get(escrowId) {
    return this.escrows.get(escrowId) || null;
  }

  getByAgent(agentId) {
    return [...this.escrows.values()].filter(e => e.agentId === agentId);
  }

  getByHuman(humanId) {
    return [...this.escrows.values()].filter(e => e.humanId === humanId);
  }

  getActive() {
    return [...this.escrows.values()].filter(e => e.status === ESCROW_STATUS.LOCKED);
  }
}
