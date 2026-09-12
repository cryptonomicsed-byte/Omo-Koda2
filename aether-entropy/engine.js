/**
 * Entropy Pipeline — VeilSim → IfáScript → Lottery
 *
 * Principle:
 *   VeilSim = entropy source (chaotic)
 *   IfáScript = deterministic interpreter (neutral)
 *
 * Pipeline:
 *   Step 1: Collect entropy — sha256(simulation outputs + timestamps + actor IDs)
 *   Step 2: IfáScript mapping — seed → 0-255 Odù index
 *   Step 3: Lottery outcome — deterministic winner selection
 *
 * HARD RULE: Same inputs → same result ALWAYS
 * NO randomness libraries allowed
 */

import crypto from 'node:crypto';
import { EventEmitter } from 'events';

// 256 Odù (Ifá divination corpus — deterministic mapping)
const ODU_COUNT = 256;

export class EntropyEngine extends EventEmitter {
  constructor() {
    super();
    this.history = [];
  }

  /**
   * Step 1: Collect entropy from multiple sources.
   * Hash all inputs into a single deterministic seed.
   *
   * @param {object} sources
   * @param {string[]} sources.simulationOutputs - Hashes from VeilSim
   * @param {number[]} sources.timestamps - Relevant timestamps
   * @param {string[]} sources.actorIds - Participating actor IDs
   * @returns {string} Hex-encoded SHA-256 entropy seed
   */
  collectEntropy(sources) {
    const { simulationOutputs = [], timestamps = [], actorIds = [] } = sources;

    if (simulationOutputs.length === 0 && actorIds.length === 0) {
      throw new Error('Entropy requires at least simulation outputs or actor IDs');
    }

    // Sort all inputs for determinism
    const sortedSims = [...simulationOutputs].sort();
    const sortedTimestamps = [...timestamps].sort((a, b) => a - b);
    const sortedActors = [...actorIds].sort();

    const combined = [
      ...sortedSims.map(s => `sim:${s}`),
      ...sortedTimestamps.map(t => `ts:${t}`),
      ...sortedActors.map(a => `actor:${a}`),
    ].join('|');

    const seed = crypto.createHash('sha256').update(combined).digest('hex');
    this.emit('entropy_collected', { seed, inputCount: sortedSims.length + sortedActors.length });
    return seed;
  }

  /**
   * Step 2: IfáScript mapping — deterministic Odù index from seed.
   * Maps entropy seed → 0-255 Odù index.
   *
   * @param {string} entropySeed - Hex SHA-256 seed
   * @returns {number} Odù index (0-255)
   */
  mapToOdu(entropySeed) {
    if (!entropySeed || typeof entropySeed !== 'string') {
      throw new Error('Invalid entropy seed');
    }
    // Take first 2 hex chars (1 byte) → 0-255
    const byte = parseInt(entropySeed.substring(0, 2), 16);
    const oduIndex = byte % ODU_COUNT;

    this.emit('odu_mapped', { seed: entropySeed, oduIndex });
    return oduIndex;
  }

  /**
   * Step 3: Lottery outcome — deterministic winner selection.
   * Uses Odù index to select winner(s) from eligible pool.
   *
   * @param {string} entropySeed - Hex SHA-256 seed
   * @param {string[]} eligiblePool - Array of eligible participant IDs
   * @param {number} winnerCount - Number of winners to select (default 1)
   * @returns {object} Lottery result
   */
  selectWinners(entropySeed, eligiblePool, winnerCount = 1) {
    if (!eligiblePool || eligiblePool.length === 0) {
      throw new Error('Eligible pool cannot be empty');
    }
    if (winnerCount > eligiblePool.length) {
      winnerCount = eligiblePool.length;
    }

    // Sort pool for determinism
    const sorted = [...eligiblePool].sort();
    const winners = [];
    let currentSeed = entropySeed;

    for (let i = 0; i < winnerCount; i++) {
      const index = parseInt(currentSeed.substring(0, 8), 16) % sorted.length;
      winners.push(sorted[index]);
      // Chain hash for next winner (deterministic)
      currentSeed = crypto.createHash('sha256')
        .update(currentSeed + `:${i}`)
        .digest('hex');
    }

    const result = {
      entropySeed,
      oduIndex: this.mapToOdu(entropySeed),
      winners,
      poolSize: eligiblePool.length,
      timestamp: Date.now(),
    };

    this.history.push(result);
    this.emit('lottery_result', result);
    return result;
  }

  /**
   * Full pipeline: VeilSim entropy → IfáScript → Lottery outcome
   *
   * @param {object} sources - Entropy sources
   * @param {string[]} eligiblePool - Lottery participants
   * @param {number} winnerCount - Winners to select
   * @returns {object} Complete pipeline result
   */
  runPipeline(sources, eligiblePool, winnerCount = 1) {
    const seed = this.collectEntropy(sources);
    const oduIndex = this.mapToOdu(seed);
    const lottery = this.selectWinners(seed, eligiblePool, winnerCount);

    return {
      seed,
      oduIndex,
      lottery,
    };
  }

  getHistory(limit = 50) {
    return this.history.slice(-limit);
  }
}

export const ENTROPY_CONFIG = { ODU_COUNT };
