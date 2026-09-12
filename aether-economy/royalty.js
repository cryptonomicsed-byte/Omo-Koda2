/**
 * Swibe ToC Royalty Engine — Phase 7
 * Creator royalty tracking at the agent layer.
 * Actual Àṣẹ royalty distribution happens at OSOVM (op_job_payment).
 * Swibe tracks creator→agent relationships and royalty percentages.
 */

import { EventEmitter } from 'events';
import { TOKEN_TYPE } from './token.js';

const SABBATH_LOCK_MS = 7 * 86_400_000;

export class RoyaltyEngine extends EventEmitter {
  constructor(walletRegistry) {
    super();
    this.registry = walletRegistry;
    this.creators = new Map();
    this.pendingPayouts = [];
    this.paidHistory = [];
  }

  registerCreator(creatorId, agentId, config = {}) {
    const percentage = config.percentage || 10;
    const token = TOKEN_TYPE.ASE;
    const vesting = config.vesting || 'sabbath';

    this.creators.set(agentId, {
      creatorId,
      agentId,
      percentage,
      token,
      vesting,
      totalEarned: 0,
      registered: Date.now(),
    });

    this.emit('register', { creatorId, agentId, percentage });
    return { creatorId, agentId, percentage, token };
  }

  processJobPayment(agentId, totalAse) {
    const creatorInfo = this.creators.get(agentId);
    if (!creatorInfo) return null;

    const royaltyAmount = Math.floor(totalAse * (creatorInfo.percentage / 100));
    const protocolBurn = Math.floor(totalAse * 0.05);
    const agentShare = totalAse - royaltyAmount - protocolBurn;

    const payout = {
      creatorId: creatorInfo.creatorId,
      agentId,
      royaltyAmount,
      protocolBurn,
      agentShare,
      token: TOKEN_TYPE.ASE,
      lockedUntil: Date.now() + SABBATH_LOCK_MS,
      status: 'locked',
      timestamp: Date.now(),
    };

    this.pendingPayouts.push(payout);
    creatorInfo.totalEarned += royaltyAmount;

    this.emit('royalty', payout);

    return {
      creator: { id: creatorInfo.creatorId, earns: royaltyAmount, token: 'ase', lockedDays: 7 },
      protocol: { burned: protocolBurn },
      agent: { receives: agentShare, convertsTo: 'dopamine' },
    };
  }

  claimUnlocked(creatorId) {
    const now = Date.now();
    const claimable = this.pendingPayouts.filter(
      p => p.creatorId === creatorId && p.status === 'locked' && now >= p.lockedUntil
    );

    let totalClaimed = 0;
    const creatorWallet = this.registry.get(creatorId);

    for (const payout of claimable) {
      payout.status = 'claimed';
      if (creatorWallet) {
        creatorWallet.receive(TOKEN_TYPE.ASE, payout.royaltyAmount);
      }
      totalClaimed += payout.royaltyAmount;
      this.paidHistory.push(payout);
    }

    if (totalClaimed > 0) {
      this.emit('claim', { creatorId, amount: totalClaimed, count: claimable.length });
    }

    return { creatorId, claimed: totalClaimed, count: claimable.length };
  }

  getPending(creatorId) {
    return this.pendingPayouts
      .filter(p => p.creatorId === creatorId && p.status === 'locked')
      .map(p => ({
        amount: p.royaltyAmount,
        lockedUntil: p.lockedUntil,
        remainingMs: Math.max(0, p.lockedUntil - Date.now()),
      }));
  }

  getCreatorInfo(agentId) {
    return this.creators.get(agentId) || null;
  }

  getTotalEarned(creatorId) {
    let total = 0;
    for (const [, info] of this.creators) {
      if (info.creatorId === creatorId) total += info.totalEarned;
    }
    return total;
  }
}
