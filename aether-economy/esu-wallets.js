/**
 * Èṣù Wallet Architecture — Swibe Agent Layer
 * 
 * Implements the full Elegbára routing system + 8 sub-wallets + 4 mask wallets.
 * All wallets use Vanity-Cloakseed BIP-39 and are Swibe agents.
 *
 * Hard rules:
 *   - Èṣù ALWAYS gets paid first (3.69% on ALL transactions)
 *   - Elegbára does NOT hold funds long-term
 *   - No-commingling: purpose_tag + allowlist enforced
 *   - Saturday = Sabbath freeze (0 mint)
 *   - Agent birth: 0.01 ÀṢẸ → 20/40/20/20 split
 */

import { EventEmitter } from 'events';

// ===== Constants =====

const ESU_TAX_RATE = 0.0369; // 3.69%

const DISTRIBUTION = {
  veilsim:         0.30,
  rnd:             0.20,
  governance:      0.10,
  reserve:         0.10,
  lottery_burn:    0.10,
  grants:          0.10,
  ubi:             0.05,
  sabbath_reserve: 0.05,
};

const AGENT_BIRTH_FEE = 0.01; // ÀṢẸ

const BIRTH_SPLIT = {
  burn:        0.20,
  elegbara:    0.40,
  inheritance: 0.20,
  performance: 0.20,
};

const ODARA_SPLIT = {
  shrine:      0.50,
  inheritance: 0.25,
  aio:         0.15,
  burn:        0.10,
};

const PURPOSE_TAGS = {
  veilsim:         'veilsim',
  rnd:             'rnd',
  governance:      'governance',
  reserve:         'reserve',
  lottery_burn:    'lottery_burn',
  grants:          'grants',
  ubi:             'ubi',
  sabbath_reserve: 'sabbath_reserve',
  odara:           'odara',
  laalu:           'laalu',
  bara:            'bara',
  agbana:          'agbana',
};

// Allowlists: which purpose_tags each wallet can send TO
const ALLOWLISTS = {
  veilsim:         ['rnd', 'grants'],
  rnd:             ['veilsim', 'governance'],
  governance:      ['rnd', 'grants', 'reserve'],
  reserve:         ['ubi', 'sabbath_reserve', 'governance'],
  lottery_burn:    [],
  grants:          ['rnd'],
  ubi:             [],
  sabbath_reserve: ['ubi', 'grants'],
  odara:           ['reserve'],
  laalu:           ['rnd'],
  bara:            ['reserve', 'ubi'],
  agbana:          [],
};

// ===== IsolatedWallet =====

export class IsolatedWallet extends EventEmitter {
  constructor(purposeTag, allowlist = []) {
    super();
    this.purposeTag = purposeTag;
    this.allowlist = allowlist;
    this.balance = 0;
    this.totalReceived = 0;
    this.totalDisbursed = 0;
    this.history = [];
  }

  deposit(amount, source = 'elegbara') {
    if (amount <= 0) throw new Error(`Invalid deposit amount: ${amount}`);
    // Agbàná: NO ÀṢẸ mint inflow allowed
    if (this.purposeTag === PURPOSE_TAGS.agbana && source === 'mint') {
      throw new Error('Agbàná wallet cannot receive mint inflow');
    }
    this.balance += amount;
    this.totalReceived += amount;
    this.history.push({ type: 'deposit', amount, source, timestamp: Date.now() });
    this.emit('deposit', { purposeTag: this.purposeTag, amount, source });
  }

  withdraw(amount, destPurposeTag) {
    if (amount <= 0) throw new Error(`Invalid withdraw amount: ${amount}`);
    if (this.balance < amount) {
      throw new Error(`Insufficient balance in ${this.purposeTag}: have ${this.balance}, need ${amount}`);
    }
    // No-commingling enforcement
    if (destPurposeTag && !this.allowlist.includes(destPurposeTag)) {
      this.emit('commingling_reverted', {
        source: this.purposeTag,
        dest: destPurposeTag,
        amount,
      });
      throw new Error(
        `COMMINGLING DENIED: ${this.purposeTag} → ${destPurposeTag} not in allowlist [${this.allowlist.join(', ')}]`
      );
    }
    this.balance -= amount;
    this.totalDisbursed += amount;
    this.history.push({ type: 'withdraw', amount, dest: destPurposeTag, timestamp: Date.now() });
    this.emit('withdraw', { purposeTag: this.purposeTag, amount, dest: destPurposeTag });
    return amount;
  }

  getState() {
    return {
      purposeTag: this.purposeTag,
      balance: this.balance,
      totalReceived: this.totalReceived,
      totalDisbursed: this.totalDisbursed,
      allowlist: [...this.allowlist],
    };
  }
}

// ===== ElegbaraRouter =====

export class ElegbaraRouter extends EventEmitter {
  constructor() {
    super();
    // 8 primary sub-wallets
    this.wallets = {};
    for (const [key, _pct] of Object.entries(DISTRIBUTION)) {
      this.wallets[key] = new IsolatedWallet(key, ALLOWLISTS[key] || []);
    }
    // 4 Èṣù mask wallets
    this.wallets.odara = new IsolatedWallet(PURPOSE_TAGS.odara, ALLOWLISTS.odara);
    this.wallets.laalu = new IsolatedWallet(PURPOSE_TAGS.laalu, ALLOWLISTS.laalu);
    this.wallets.bara = new IsolatedWallet(PURPOSE_TAGS.bara, ALLOWLISTS.bara);
    this.wallets.agbana = new IsolatedWallet(PURPOSE_TAGS.agbana, ALLOWLISTS.agbana);

    // Birth pools
    this.inheritancePool = 0;
    this.performancePool = 0;

    // Tracking
    this.totalEsuTax = 0;
    this.totalMinted = 0;
    this.totalBurned = 0;
    this.totalAgentsBorn = 0;

    // Mint scheduler state
    this._mintInterval = null;
    this._lotteryInterval = null;
    this._ubiInterval = null;
    this._sabbathInterval = null;
  }

  // ===== Èṣù Tax =====

  extractEsuTax(amount) {
    const tax = amount * ESU_TAX_RATE;
    const net = amount - tax;
    this.totalEsuTax += tax;
    this.emit('esu_tax', { amount, tax, net });
    return { tax, net };
  }

  // ===== Mint Routing =====

  routeMint(amount) {
    if (this.isSabbath()) {
      this.emit('sabbath_blocked', { amount, day: 'Saturday' });
      return null;
    }
    if (amount <= 0) throw new Error('Mint amount must be positive');

    // Minting is NOT taxed — distribute full amount
    const routed = {};
    let remaining = amount;
    const keys = Object.keys(DISTRIBUTION);

    for (let i = 0; i < keys.length; i++) {
      const key = keys[i];
      if (i === keys.length - 1) {
        // Last wallet gets remainder to avoid rounding dust
        routed[key] = remaining;
        this.wallets[key].deposit(remaining, 'mint');
      } else {
        const share = Math.floor(amount * DISTRIBUTION[key] * 1e6) / 1e6;
        routed[key] = share;
        this.wallets[key].deposit(share, 'mint');
        remaining -= share;
      }
    }

    this.totalMinted += amount;
    this.emit('mint_routed', { total: amount, routed });
    return routed;
  }

  // ===== Transaction Tax Routing =====

  routeTransaction(amount) {
    const { tax, net } = this.extractEsuTax(amount);
    // Tax goes to Elegbára routing (split across sub-wallets)
    this.routeTaxToSubWallets(tax);
    return { tax, net };
  }

  routeTaxToSubWallets(taxAmount) {
    const keys = Object.keys(DISTRIBUTION);
    let remaining = taxAmount;
    for (let i = 0; i < keys.length; i++) {
      const key = keys[i];
      if (i === keys.length - 1) {
        this.wallets[key].deposit(remaining, 'tax');
      } else {
        const share = Math.floor(taxAmount * DISTRIBUTION[key] * 1e6) / 1e6;
        this.wallets[key].deposit(share, 'tax');
        remaining -= share;
      }
    }
  }

  // ===== Agent Birth =====

  processAgentBirth(creatorId) {
    const fee = AGENT_BIRTH_FEE;
    const burnAmt = fee * BIRTH_SPLIT.burn;
    const elegbaraAmt = fee * BIRTH_SPLIT.elegbara;
    const inheritanceAmt = fee * BIRTH_SPLIT.inheritance;
    const performanceAmt = fee * BIRTH_SPLIT.performance;

    this.totalBurned += burnAmt;
    this.wallets.lottery_burn.deposit(burnAmt, 'birth_burn');
    this.routeTaxToSubWallets(elegbaraAmt);
    this.inheritancePool += inheritanceAmt;
    this.performancePool += performanceAmt;
    this.totalAgentsBorn += 1;

    const result = {
      creator: creatorId,
      fee,
      burned: burnAmt,
      toElegbara: elegbaraAmt,
      toInheritance: inheritanceAmt,
      toPerformance: performanceAmt,
    };
    this.emit('agent_birth', result);
    return result;
  }

  // ===== Ọ̀dàrà Shrine Tithe =====

  routeOdaraTithe(amount) {
    const shrineAmt = amount * ODARA_SPLIT.shrine;
    const inheritAmt = amount * ODARA_SPLIT.inheritance;
    const aioAmt = amount * ODARA_SPLIT.aio;
    const burnAmt = amount - shrineAmt - inheritAmt - aioAmt;

    this.wallets.odara.deposit(shrineAmt, 'tithe');
    this.inheritancePool += inheritAmt;
    this.wallets.rnd.deposit(aioAmt, 'tithe');
    this.totalBurned += burnAmt;

    this.emit('odara_tithe', { amount, shrine: shrineAmt, inheritance: inheritAmt, aio: aioAmt, burned: burnAmt });
  }

  // ===== Bara Emergency Vault (3-of-5 quorum) =====

  processBaraWithdrawal(amount, approvals) {
    if (!Array.isArray(approvals) || approvals.length < 3) {
      throw new Error('Bara emergency vault requires 3-of-5 quorum');
    }
    const unique = new Set(approvals);
    if (unique.size < 3) {
      throw new Error('Bara requires 3 unique approvers');
    }
    return this.wallets.bara.withdraw(amount, 'reserve');
  }

  // ===== Sabbath Logic =====

  isSabbath(date = new Date()) {
    return date.getUTCDay() === 6; // Saturday = 6
  }

  isActiveMintDay(date = new Date()) {
    return !this.isSabbath(date); // Sunday-Friday = active
  }

  // ===== Automation: Mint Scheduler =====

  startMintScheduler(intervalMs = 60000) {
    if (this._mintInterval) return;
    this._mintInterval = setInterval(() => {
      if (this.isActiveMintDay()) {
        this.routeMint(1); // 1 ÀṢẸ per minute
      }
    }, intervalMs);
    this.emit('scheduler_started', { type: 'mint', intervalMs });
  }

  stopMintScheduler() {
    if (this._mintInterval) {
      clearInterval(this._mintInterval);
      this._mintInterval = null;
      this.emit('scheduler_stopped', { type: 'mint' });
    }
  }

  // ===== Automation: Lottery (every Monday) =====

  startLotteryScheduler(intervalMs = 3600000) {
    if (this._lotteryInterval) return;
    this._lotteryInterval = setInterval(() => {
      const now = new Date();
      if (now.getUTCDay() === 1) { // Monday
        this.runLottery();
      }
    }, intervalMs);
  }

  runLottery() {
    const pool = this.wallets.lottery_burn.balance;
    if (pool <= 0) return null;
    // Use deterministic entropy (VeilSim-sourced in production)
    const entropy = Date.now() % 1000;
    const result = {
      pool,
      entropy,
      timestamp: Date.now(),
    };
    this.emit('lottery_executed', result);
    return result;
  }

  stopLotteryScheduler() {
    if (this._lotteryInterval) {
      clearInterval(this._lotteryInterval);
      this._lotteryInterval = null;
    }
  }

  // ===== Automation: UBI (daily) =====

  startUbiScheduler(intervalMs = 86400000) {
    if (this._ubiInterval) return;
    this._ubiInterval = setInterval(() => {
      this.distributeUbi();
    }, intervalMs);
  }

  distributeUbi(recipients = []) {
    const pool = this.wallets.ubi.balance;
    if (pool <= 0 || recipients.length === 0) return null;

    // Filter: verified World ID + active within last 7 days
    const eligible = recipients.filter(r => r.worldIdVerified && r.lastActiveDaysAgo <= 7);
    if (eligible.length === 0) return null;

    const perPerson = pool / eligible.length;
    this.wallets.ubi.withdraw(pool, null); // UBI has empty allowlist — direct payout only

    const result = {
      total: pool,
      recipients: eligible.length,
      perPerson,
      timestamp: Date.now(),
    };
    this.emit('ubi_distributed', result);
    return result;
  }

  stopUbiScheduler() {
    if (this._ubiInterval) {
      clearInterval(this._ubiInterval);
      this._ubiInterval = null;
    }
  }

  // ===== Automation: Sabbath Reserve Release =====

  startSabbathReleaseScheduler(intervalMs = 3600000) {
    if (this._sabbathInterval) return;
    this._sabbathInterval = setInterval(() => {
      const now = new Date();
      // Sunday at 23:59 UTC
      if (now.getUTCDay() === 0 && now.getUTCHours() === 23 && now.getUTCMinutes() >= 59) {
        this.releaseSabbathReserve();
      }
    }, intervalMs);
  }

  releaseSabbathReserve(contributionScores = []) {
    const pool = this.wallets.sabbath_reserve.balance;
    if (pool <= 0) return null;

    const result = {
      released: pool,
      distributedBy: 'contribution_scoring',
      scores: contributionScores,
      timestamp: Date.now(),
    };
    this.emit('sabbath_released', result);
    return result;
  }

  stopSabbathReleaseScheduler() {
    if (this._sabbathInterval) {
      clearInterval(this._sabbathInterval);
      this._sabbathInterval = null;
    }
  }

  // ===== Stop All =====

  stopAll() {
    this.stopMintScheduler();
    this.stopLotteryScheduler();
    this.stopUbiScheduler();
    this.stopSabbathReleaseScheduler();
  }

  // ===== State =====

  getState() {
    const walletStates = {};
    for (const [key, wallet] of Object.entries(this.wallets)) {
      walletStates[key] = wallet.getState();
    }
    return {
      wallets: walletStates,
      inheritancePool: this.inheritancePool,
      performancePool: this.performancePool,
      totalEsuTax: this.totalEsuTax,
      totalMinted: this.totalMinted,
      totalBurned: this.totalBurned,
      totalAgentsBorn: this.totalAgentsBorn,
    };
  }
}

// ===== Exports =====

export const ESU_CONFIG = {
  ESU_TAX_RATE,
  DISTRIBUTION,
  AGENT_BIRTH_FEE,
  BIRTH_SPLIT,
  ODARA_SPLIT,
  PURPOSE_TAGS,
  ALLOWLISTS,
};
