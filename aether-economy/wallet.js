/**
 * Swibe ToC Wallet — Phase 7: Agent Wallets
 * Every agent born receives 86B Dopamine + 86M Synapse (neural layer mapping)
 *
 * Birth endowment is triggered by OSOVM op_agent_birth (0x3e).
 * The VM locks Àṣẹ from the creator and emits the endowment signal.
 * Swibe applies the signal — agents never mint their own tokens.
 */

import { EventEmitter } from 'events';
import { TOKEN_TYPE, TOKEN_CONFIG, TokenLedger } from './token.js';
import { generateAgentIdentity, recoverAgentIdentity, deriveCapabilities, agentAddress } from '../bipon39/agent-identity.js';

const BIRTH_ENDOWMENT = {
  [TOKEN_TYPE.TOC_D]: 86_000_000_000,
  [TOKEN_TYPE.TOC_S]: 86_000_000,
};

export class Wallet extends EventEmitter {
  constructor(ownerId, ownerType = 'agent', ledger = null) {
    super();
    this.ownerId = ownerId;
    this.ownerType = ownerType;
    this.ledger = ledger || new TokenLedger();
    this.identity = null; // BIPON39 identity (set at birth)
    this.created = Date.now();
    this.lastDecay = Date.now();
  }

  /**
   * Apply birth endowment from OSOVM signal.
   * Generates BIPON39 identity for the agent.
   * Called when OSOVM birth event is received.
   */
  async birth(vmSignal = null) {
    if (this.ownerType !== 'agent') {
      throw new Error('Only agents receive birth endowment');
    }

    // Generate BIPON39 identity for this agent
    const entropy = vmSignal?.entropy || null;
    this.identity = await generateAgentIdentity(entropy, {
      mode: vmSignal?.mode || '256',
      passphrase: vmSignal?.passphrase || '',
    });
    this.identity.capabilities = deriveCapabilities(
      this.identity.odu,
      this.identity.elements
    );
    this.identity.address = agentAddress(this.identity.agentId);

    const dopamine = vmSignal?.dopamine_endowment || BIRTH_ENDOWMENT[TOKEN_TYPE.TOC_D];
    const synapse = vmSignal?.synapse_endowment || BIRTH_ENDOWMENT[TOKEN_TYPE.TOC_S];

    // MINT ONLY from VM signal
    this.ledger.mint(this.ownerId, TOKEN_TYPE.TOC_D, dopamine, 'vm_birth');
    this.ledger.mint(this.ownerId, TOKEN_TYPE.TOC_S, synapse, 'vm_birth');

    this.emit('birth', {
      ownerId: this.ownerId,
      dopamine,
      synapse,
      bipon39: {
        agentId: this.identity.agentId,
        odu: this.identity.odu,
        elements: this.identity.elements,
        dominantElement: this.identity.dominantElement,
        address: this.identity.address,
      },
    });
    return this;
  }

  /**
   * Recover agent identity from saved mnemonic.
   */
  async recover(mnemonic, options = {}) {
    this.identity = await recoverAgentIdentity(mnemonic, options);
    this.identity.capabilities = deriveCapabilities(
      this.identity.odu,
      this.identity.elements
    );
    this.identity.address = agentAddress(this.identity.agentId);
    return this;
  }

  /**
   * Get the agent's BIPON39 mnemonic (for backup/recovery).
   */
  getMnemonic() {
    return this.identity?.mnemonic || null;
  }

  /**
   * Get the agent's network address.
   */
  getAddress() {
    return this.identity?.address || null;
  }

  /**
   * Handle Dopamine increment from VM signal (Àṣẹ → Dopamine conversion).
   */
  receiveFromVM(dopamineAmount) {
    this.ledger.mint(this.ownerId, TOKEN_TYPE.TOC_D, dopamineAmount, 'vm_conversion');
    this.emit('vm_conversion', { dopamine: dopamineAmount });
  }

  /**
   * Handle authorized conversion within Swibe.
   * Uses 'vm_conversion' source as it represents an authorized economic event.
   */
  applyConversion(token, amount) {
    this.ledger.mint(this.ownerId, token, amount, 'vm_conversion');
    this.emit('receive', { token, amount, source: 'vm_conversion' });
  }

  /**
   * Lock tokens for staking or escrow. Moves balance to a locked pool.
   */
  lock(token, amount, purpose = 'stake') {
    if (!this._locked) this._locked = new Map();
    const bal = this.ledger.balance(this.ownerId, token);
    if (bal < amount) throw new Error(`Insufficient ${token} to lock: have ${bal}, need ${amount}`);
    this.ledger.burn(this.ownerId, token, amount, `lock_${purpose}`);
    const key = `${token}:${purpose}`;
    const prev = this._locked.get(key) || 0;
    this._locked.set(key, prev + amount);
    return amount;
  }

  /**
   * Unlock previously locked tokens (returns to balance).
   */
  unlock(token, purpose = 'stake') {
    if (!this._locked) return 0;
    const key = `${token}:${purpose}`;
    const amount = this._locked.get(key) || 0;
    if (amount > 0) {
      this.ledger.mint(this.ownerId, token, amount, 'vm_conversion');
      this._locked.set(key, 0);
    }
    return amount;
  }

  /**
   * Receive tokens from external source (e.g., escrow release).
   */
  receive(token, amount) {
    this.ledger.mint(this.ownerId, token, amount, 'vm_conversion');
  }

  balance(token) {
    return this.ledger.balance(this.ownerId, token);
  }

  balances() {
    return {
      toc_d: this.ledger.balance(this.ownerId, TOKEN_TYPE.TOC_D),
      toc_s: this.ledger.balance(this.ownerId, TOKEN_TYPE.TOC_S),
    };
  }

  spend(token, amount, reason = 'action') {
    this.ledger.burn(this.ownerId, token, amount, reason);
    this.emit('spend', { token, amount, reason });
  }

  transfer(to, token, amount) {
    this.ledger.transfer(this.ownerId, to, token, amount);
    this.emit('transfer', { to, token, amount });
  }

  applyDecay() {
    const now = Date.now();
    const dayMs = 86_400_000;
    const elapsed = now - this.lastDecay;
    if (elapsed < dayMs) return 0;

    const days = Math.floor(elapsed / dayMs);
    const bal = this.balance(TOKEN_TYPE.TOC_D);
    const decayRate = TOKEN_CONFIG[TOKEN_TYPE.TOC_D].dailyDecay;
    const decayAmount = Math.floor(bal * decayRate * days);

    if (decayAmount > 0) {
      this.ledger.burn(this.ownerId, TOKEN_TYPE.TOC_D, decayAmount, 'daily_decay');
      this.emit('decay', { amount: decayAmount, days });
    }

    this.lastDecay = now;
    return decayAmount;
  }
}

export class WalletRegistry {
  constructor() {
    this.ledger = new TokenLedger();
    this.wallets = new Map();
  }

  async createAgent(agentId, vmSignal = null) {
    if (this.wallets.has(agentId)) {
      return this.wallets.get(agentId);
    }
    const wallet = new Wallet(agentId, 'agent', this.ledger);
    await wallet.birth(vmSignal);
    this.wallets.set(agentId, wallet);
    return wallet;
  }

  createHuman(userId) {
    if (this.wallets.has(userId)) {
      return this.wallets.get(userId);
    }
    const wallet = new Wallet(userId, 'human', this.ledger);
    this.wallets.set(userId, wallet);
    return wallet;
  }

  get(id) {
    return this.wallets.get(id) || null;
  }

  getSupply() {
    return this.ledger.getSupply();
  }

  getBurned() {
    return this.ledger.getBurned();
  }
}
