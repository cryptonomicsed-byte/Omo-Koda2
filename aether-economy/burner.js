import crypto from 'node:crypto';

/**
 * REPL-Aware Token Tracking
 * Track Dopamine/Synapse burns in REPL sessions
 */
class ToCBurner {
  constructor(identity) {
    this.identity = identity;
    this.sessionBurns = new Map(); // thoughtId -> amount
  }
  
  burnDopamine(amount, thoughtId) {
    // Log burn for audit
    this.sessionBurns.set(thoughtId, {
      amount,
      timestamp: Date.now(),
      merkle: this._computeMerkle(thoughtId, amount)
    });
    
    // Emit receipt chain event
    // Note: in a real environment, this might use a proper event bus
    console.log(`[ToC:BURN] identity=${this.identity} amount=${amount} thoughtId=${thoughtId}`);
  }
  
  _computeMerkle(thoughtId, amount) {
    return crypto.createHash('sha256')
      .update(`${thoughtId}:${amount}:${Date.now()}`)
      .digest('hex');
  }

  _computeSessionMerkle() {
    const hashes = Array.from(this.sessionBurns.values()).map(b => b.merkle);
    if (hashes.length === 0) return '0'.repeat(64);
    return crypto.createHash('sha256')
      .update(hashes.join(''))
      .digest('hex');
  }
  
  getSessionSummary() {
    return {
      totalBurned: Array.from(this.sessionBurns.values())
        .reduce((sum, entry) => sum + entry.amount, 0),
      transactionCount: this.sessionBurns.size,
      merkleRoot: this._computeSessionMerkle()
    };
  }
}

export { ToCBurner };
export default ToCBurner;
