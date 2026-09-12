/**
 * VeilSim Payout Engine — Activates the 30% VeilSim Treasury
 *
 * Simulation scoring model:
 *   - Actors submit simulation results with score (0.0-1.0) and determinism proof
 *   - Scores normalized, rewards distributed proportionally from pool
 *   - Deterministic ordering: sorted by actor + job_id
 *
 * Hard rules:
 *   - Reject invalid scores (>1 or <0)
 *   - Require proof placeholder
 *   - Deterministic output ordering
 */

import { EventEmitter } from 'events';
import crypto from 'node:crypto';

export class VeilSimEngine extends EventEmitter {
  constructor() {
    super();
    this.epochs = [];
    this.currentSubmissions = [];
  }

  /**
   * Submit a simulation result for the current epoch.
   * @param {object} submission
   * @param {string} submission.actor - Actor/runner ID
   * @param {string} submission.job_id - Simulation job ID
   * @param {number} submission.score - Quality score 0.0 → 1.0
   * @param {string} submission.determinism_proof - Proof of deterministic execution
   * @param {number} submission.timestamp - Submission time
   */
  submit(submission) {
    const { actor, job_id, score, determinism_proof, timestamp } = submission;

    if (!actor || !job_id) {
      throw new Error('Submission requires actor and job_id');
    }
    if (typeof score !== 'number' || !Number.isFinite(score) || score < 0 || score > 1) {
      throw new Error(`Invalid score: ${score}. Must be 0.0 → 1.0`);
    }
    if (!determinism_proof) {
      throw new Error('Submission requires determinism_proof');
    }

    const record = {
      actor,
      job_id,
      score,
      determinism_proof,
      timestamp: timestamp || Date.now(),
    };

    this.currentSubmissions.push(record);
    this.emit('submission', record);
    return record;
  }

  /**
   * Distribute the VeilSim pool for the current epoch.
   * Rewards proportional to normalized scores.
   * Deterministic ordering: sorted by actor + job_id.
   *
   * @param {number} pool - Total ÀṢẸ available for this epoch
   * @returns {object} Epoch result with payouts
   */
  distributeEpoch(pool) {
    if (pool <= 0) throw new Error('Pool must be positive');
    if (this.currentSubmissions.length === 0) {
      return { epoch: this.epochs.length, pool, payouts: [], totalDistributed: 0 };
    }

    // Deterministic ordering: sort by actor, then job_id
    const sorted = [...this.currentSubmissions].sort((a, b) => {
      const actorCmp = a.actor.localeCompare(b.actor);
      if (actorCmp !== 0) return actorCmp;
      return a.job_id.localeCompare(b.job_id);
    });

    // Normalize scores
    const totalScore = sorted.reduce((sum, s) => sum + s.score, 0);
    if (totalScore === 0) {
      return { epoch: this.epochs.length, pool, payouts: [], totalDistributed: 0 };
    }

    // Distribute proportionally, last actor gets remainder (no dust loss)
    const payouts = [];
    let distributed = 0;

    for (let i = 0; i < sorted.length; i++) {
      const s = sorted[i];
      let reward;
      if (i === sorted.length - 1) {
        reward = pool - distributed; // Remainder to last
      } else {
        reward = Math.floor((s.score / totalScore) * pool * 1e6) / 1e6;
      }
      payouts.push({
        actor: s.actor,
        job_id: s.job_id,
        score: s.score,
        normalizedWeight: s.score / totalScore,
        reward,
      });
      distributed += reward;
    }

    const epoch = {
      epoch: this.epochs.length,
      pool,
      totalScore,
      payouts,
      totalDistributed: distributed,
      timestamp: Date.now(),
    };

    this.epochs.push(epoch);
    this.currentSubmissions = []; // Reset for next epoch
    this.emit('epoch_distributed', epoch);

    return epoch;
  }

  /**
   * Generate entropy from current submissions (used by entropy pipeline).
   * Hash of all simulation outputs + timestamps + actor IDs.
   */
  generateEntropy() {
    if (this.currentSubmissions.length === 0 && this.epochs.length === 0) {
      throw new Error('No simulation data available for entropy generation');
    }

    const source = this.currentSubmissions.length > 0
      ? this.currentSubmissions
      : this.epochs[this.epochs.length - 1].payouts;

    // Deterministic ordering before hashing
    const sorted = [...source].sort((a, b) => {
      const actorCmp = a.actor.localeCompare(b.actor);
      if (actorCmp !== 0) return actorCmp;
      return (a.job_id || '').localeCompare(b.job_id || '');
    });

    const data = sorted.map(s =>
      `${s.actor}:${s.job_id}:${s.score}:${s.timestamp || s.reward || 0}`
    ).join('|');

    const hash = crypto.createHash('sha256').update(data).digest('hex');
    return hash;
  }

  getEpochHistory(limit = 50) {
    return this.epochs.slice(-limit);
  }

  getCurrentSubmissions() {
    return [...this.currentSubmissions];
  }
}
