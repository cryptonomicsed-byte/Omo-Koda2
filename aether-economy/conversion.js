/**
 * Swibe ToC Conversion Engine — Phase 7: Burn Conversions
 * 
 * Rules:
 * 1. Dopamine → Synapse (one-way, exactly 10:1)
 * 2. Swibe handles the logic, but the actual balance change 
 *    is treated as an authorized conversion event.
 */

import { EventEmitter } from 'events';
import { TOKEN_TYPE, TOKEN_CONFIG } from './token.js';

export class ConversionEngine extends EventEmitter {
  constructor(walletRegistry) {
    super();
    this.registry = walletRegistry;
    this.history = [];
  }

  /**
   * One-way burn: 10 Dopamine → 1 Synapse.
   * @param {string} agentId - The agent performing the conversion
   * @param {number} dopamineAmount - Amount of Dopamine to burn (must be multiple of 10)
   */
  dopamineToSynapse(agentId, dopamineAmount) {
    if (dopamineAmount < 10) throw new Error('Minimum conversion is 10 Dopamine');
    
    const wallet = this.registry.get(agentId);
    if (!wallet) throw new Error(`No wallet for ${agentId}`);

    if (wallet.ownerType !== 'agent') {
      throw new Error('Only agents can perform Dopamine conversion');
    }

    const fromBalance = wallet.balance(TOKEN_TYPE.TOC_D);
    if (fromBalance < dopamineAmount) {
      throw new Error(`Insufficient Dopamine: have ${fromBalance}, need ${dopamineAmount}`);
    }

    // Burn Dopamine
    wallet.spend(TOKEN_TYPE.TOC_D, dopamineAmount, 'synapse_conversion');

    // Mint Synapse at 10:1 ratio
    const ratio = TOKEN_CONFIG[TOKEN_TYPE.TOC_S].conversionRatio;
    const synapseAmount = Math.floor(dopamineAmount * ratio);

    if (synapseAmount > 0) {
      wallet.applyConversion(TOKEN_TYPE.TOC_S, synapseAmount);
    }

    const record = {
      agentId,
      type: 'dopamine_to_synapse',
      from: TOKEN_TYPE.TOC_D,
      to: TOKEN_TYPE.TOC_S,
      burned: dopamineAmount,
      minted: synapseAmount,
      timestamp: Date.now(),
    };

    this.history.push(record);
    this.emit('conversion', record);

    return record;
  }

  getHistory(agentId = null, limit = 50) {
    const filtered = agentId
      ? this.history.filter(h => h.agentId === agentId)
      : this.history;
    return filtered.slice(-limit);
  }
}
