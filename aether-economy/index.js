/**
 * Swibe ToC Module — Phase 7: Complete Tokenomics
 * Three-token economy: Àṣẹ (human) → Dopamine (agent) → Synapse (agent commerce)
 * Every agent born with 86B Dopamine + 86M Synapse (neural layer mapping)
 */

export { TOKEN_TYPE, TOKEN_CONFIG, TokenLedger } from './token.js';
export { generateAgentIdentity, recoverAgentIdentity, signMessage, verifyMessage, deriveCapabilities, agentAddress, checkSabbath } from '../bipon39/agent-identity.js';
export { Wallet, WalletRegistry } from './wallet.js';
export { ConversionEngine } from './conversion.js';
export { StakingEngine, GATED_PRIMITIVES, GATED_STAKE_FRACTION } from './staking.js';
export { RoyaltyEngine } from './royalty.js';
export { EscrowEngine } from './escrow.js';
export { DecayEngine, DECAY_CONFIG } from './decay.js';
export { ElegbaraRouter, IsolatedWallet, ESU_CONFIG } from './esu-wallets.js';
export { EventBridge, BRIDGE_CONFIG } from './event-bridge.js';
export { VeilSimEngine } from '../veilsim/engine.js';
export { EntropyEngine, ENTROPY_CONFIG } from '../entropy/engine.js';
export { UBIEngine, UBI_CONFIG } from '../ubi/engine.js';

import { WalletRegistry } from './wallet.js';
import { ConversionEngine } from './conversion.js';
import { StakingEngine } from './staking.js';
import { RoyaltyEngine } from './royalty.js';
import { EscrowEngine } from './escrow.js';
import { DecayEngine } from './decay.js';
import { ElegbaraRouter } from './esu-wallets.js';
import { EventBridge } from './event-bridge.js';

export class ToCEconomy {
  constructor() {
    this.wallets = new WalletRegistry();
    this.conversion = new ConversionEngine(this.wallets);
    this.staking = new StakingEngine(this.wallets);
    this.royalty = new RoyaltyEngine(this.wallets);
    this.escrow = new EscrowEngine(this.wallets, this.royalty);
    this.decay = new DecayEngine(this.wallets);
    this.elegbara = new ElegbaraRouter();
    this.bridge = new EventBridge(this, this.elegbara);
  }

  appealSlash(agentId, slashId, evidence, receiptChain) {
    return this.staking.appealSlash(agentId, slashId, evidence, receiptChain);
  }

  collectInterest(agentId) {
    return this.staking.processInterest(agentId);
  }

  getStakeStatus(agentId) {
    const wallet = this.wallets.get(agentId);
    if (!wallet) return null;
    return {
      balances: {
        dopamine: wallet.balance('toc_d'),
        synapse: wallet.balance('toc_s'),
      },
      stakes: this.staking.getAllStakes(agentId),
      slashes: this.staking.getSlashHistory(agentId),
    };
  }

  /**
   * Spawn an agent from a VM birth signal.
   * OSOVM op_agent_birth (0x3e) locks 10 Àṣẹ and emits the endowment signal.
   * Swibe creates the wallet and applies 86B Dopamine + 86M Synapse.
   * @param {string} agentId - Unique agent identifier
   * @param {string} creatorId - Creator who paid the 10 Àṣẹ birth fee
   * @param {number} royaltyPercent - Creator royalty % (default 10)
   * @param {object} vmSignal - Optional OSOVM birth signal with endowment amounts
   */
  async spawnAgent(agentId, creatorId = null, royaltyPercent = 10, vmSignal = null) {
    const wallet = await this.wallets.createAgent(agentId, vmSignal);
    if (creatorId) {
      this.royalty.registerCreator(creatorId, agentId, { percentage: royaltyPercent });
    }
    return wallet;
  }

  registerHuman(userId) {
    // Humans hold Àṣẹ at the VM layer (OSOVM), not in Swibe.
    // This registers a reference for royalty claims only.
    const wallet = this.wallets.createHuman(userId);
    return wallet;
  }

  /**
   * Handle VM conversion signal: OSOVM burned Àṣẹ, Swibe mints Dopamine.
   * This is the bridge between VM-level Àṣẹ and agent-level Dopamine.
   * Agent never holds Àṣẹ — it arrives as Dopamine.
   */
  handleVMConversion(agentId, dopamineAmount) {
    const wallet = this.wallets.get(agentId);
    if (!wallet) throw new Error(`No agent wallet for ${agentId}`);
    wallet.receiveFromVM(dopamineAmount);
    return { agentId, dopamine: dopamineAmount, source: 'osovm' };
  }

  /**
   * Handle VM job payment signal: OSOVM processed the Àṣẹ split,
   * Swibe receives the Dopamine conversion signal for the agent.
   */
  handleJobComplete(agentId, dopamineSignal) {
    return this.handleVMConversion(agentId, dopamineSignal);
  }

  getStatus() {
    return {
      supply: this.wallets.getSupply(),
      burned: this.wallets.getBurned(),
      totalStaked: {
        toc_d: this.staking.totalStaked('toc_d'),
        toc_s: this.staking.totalStaked('toc_s'),
      },
      note: 'Àṣẹ supply tracked at OSOVM layer, not here',
    };
  }
}

export function tocFromAST(_node) {
  return new ToCEconomy();
}
