/**
 * Swibe ToC Token Definitions — Phase 7: Tokenomics
 * Agent-layer tokens: Dopamine/ToC-D (internal), Synapse/ToC-S (commerce)
 * 
 * Rules:
 * 1. Minted ONLY by ỌSỌVM at agent birth (triggered by AIO).
 * 2. Swibe handles post-birth conversion logic only.
 * 3. No minting in Swibe except from authorized VM signals.
 */

export const TOKEN_TYPE = {
  ASE: 'ase',     // Àṣẹ (human entry token, tracked at VM layer but referenced here for escrow)
  TOC_D: 'toc_d', // Dopamine
  TOC_S: 'toc_s', // Synapse
};

export const TOKEN_CONFIG = {
  [TOKEN_TYPE.TOC_D]: {
    name: 'Dopamine',
    symbol: 'TOC-D',
    description: 'Internal life-force — agent motivation and action fuel',
    holders: 'agents_only',
    birthEndowment: 86_000_000_000,
    transferable: false,
    dailyDecay: 0.01, // 1% daily
  },
  [TOKEN_TYPE.TOC_S]: {
    name: 'Synapse',
    symbol: 'TOC-S',
    description: 'Inter-agent commerce — agent-to-agent speech',
    holders: 'agents_only',
    birthEndowment: 86_000_000,
    transferable: true,
    conversionRatio: 0.1, // 10:1 (Dopamine to Synapse)
  },
};

export class TokenLedger {
  constructor() {
    this.balances = new Map();
    this.totalSupply = {
      [TOKEN_TYPE.ASE]: 0,
      [TOKEN_TYPE.TOC_D]: 0,
      [TOKEN_TYPE.TOC_S]: 0,
    };
    this.burnedTotal = {
      [TOKEN_TYPE.ASE]: 0,
      [TOKEN_TYPE.TOC_D]: 0,
      [TOKEN_TYPE.TOC_S]: 0,
    };
    this.transactions = [];
  }

  _key(holder, token) {
    return `${holder}:${token}`;
  }

  balance(holder, token) {
    return this.balances.get(this._key(holder, token)) || 0;
  }

  /**
   * Mint tokens. In Swibe, this MUST only be called by authorized VM signals.
   */
  mint(holder, token, amount, source = 'vm_signal') {
    if (amount <= 0) throw new Error('Mint amount must be positive');
    
    // Strict Source Control: only OSOVM authorized events
    const authorizedSources = ['vm_birth', 'vm_conversion'];
    if (!authorizedSources.includes(source)) {
      throw new Error(`Unauthorized minting attempt in Swibe. Source: ${source}`);
    }

    const key = this._key(holder, token);
    const prev = this.balances.get(key) || 0;
    this.balances.set(key, prev + amount);
    this.totalSupply[token] += amount;
    
    this.transactions.push({ 
      type: 'mint', 
      holder, 
      token, 
      amount, 
      source, 
      timestamp: Date.now() 
    });
  }

  burn(holder, token, amount, reason = 'action', receiptHash = null) {
    if (amount <= 0) return;
    const key = this._key(holder, token);
    const prev = this.balances.get(key) || 0;
    if (prev < amount) throw new Error(`Insufficient ${token}: have ${prev}, need ${amount}`);

    this.balances.set(key, prev - amount);
    this.totalSupply[token] -= amount;
    this.burnedTotal[token] += amount;

    const tx = {
      type: 'burn',
      holder,
      token,
      amount,
      reason,
      timestamp: Date.now(),
    };

    // Burn audit trail: Dopamine burns must link to a receipt
    if (token === TOKEN_TYPE.TOC_D) {
      tx.receiptHash = receiptHash || this._generateBurnReceipt(holder, amount, reason);
    }

    this.transactions.push(tx);
    return tx;
  }

  /**
   * Generate a burn receipt hash for audit trail.
   * Every Dopamine burn is sealed with a deterministic receipt.
   */
  _generateBurnReceipt(holder, amount, reason) {
    const payload = `${holder}:${amount}:${reason}:${Date.now()}`;
    // Simple deterministic hash — in production this would be SHA-256
    let hash = 0;
    for (let i = 0; i < payload.length; i++) {
      hash = ((hash << 5) - hash + payload.charCodeAt(i)) | 0;
    }
    return `burn_${Math.abs(hash).toString(16).padStart(8, '0')}`;
  }

  transfer(from, to, token, amount) {
    const config = TOKEN_CONFIG[token];
    if (!config.transferable) throw new Error(`${token} is non-transferable`);
    this.burn(from, token, amount, 'transfer');
    // Note: transfer 'mint' is just a balance move, not creation of new supply
    // But for the ledger, we track it as authorized balance increase
    const key = this._key(to, token);
    const prev = this.balances.get(key) || 0;
    this.balances.set(key, prev + amount);
    this.totalSupply[token] += amount; // burn already subtracted it
    
    this.transactions.push({ type: 'transfer', from, to, token, amount, timestamp: Date.now() });
  }

  getSupply() {
    return { ...this.totalSupply };
  }

  getBurned() {
    return { ...this.burnedTotal };
  }
}
