/**
 * BIPON39 Derivation — Master key + path registry
 *
 * Master key derivation via HMAC-SHA512.
 * Path registry for multi-chain support (EVM, Bitcoin, Solana, Sui).
 */

import { hmacSha512 } from './crypto.js';

/**
 * Derive master key + chain code from seed.
 * @param {Uint8Array} seed — 64-byte seed from mnemonicToSeed
 * @param {'native'|'bitcoin'} label — HMAC key domain
 * @returns {{ key: Uint8Array, chainCode: Uint8Array }}
 */
export function masterFromSeed(seed, label = 'native') {
  const hmacKey = label === 'bitcoin' ? 'Bitcoin seed' : 'BIPỌ̀N39 master';
  const I = hmacSha512(hmacKey, seed);
  return {
    key: new Uint8Array(I.slice(0, 32)),
    chainCode: new Uint8Array(I.slice(32)),
  };
}

/**
 * Standard derivation paths.
 * Purpose' / coin' / account' / change / index
 */
export const PATHS = Object.freeze({
  EVM:     (acct = 0, ch = 0, idx = 0) => `m/44'/60'/${acct}'/${ch}/${idx}`,
  Bitcoin: {
    p44: (coin = 0, acct = 0, ch = 0, idx = 0) => `m/44'/${coin}'/${acct}'/${ch}/${idx}`,
    p84: (coin = 0, acct = 0, ch = 0, idx = 0) => `m/84'/${coin}'/${acct}'/${ch}/${idx}`,
  },
  Solana:  (acct = 0, ch = 0, idx = 0) => `m/44'/501'/${acct}'/${ch}'/${idx}'`,
  Sui:     (acct = 0, ch = 0, idx = 0) => `m/44'/784'/${acct}'/${ch}'/${idx}'`,
});

/**
 * Swibe-native agent derivation path.
 * Uses purpose 86 (agent identity) with Swibe coin type 8639.
 */
export const AGENT_PATH = Object.freeze({
  identity: (agentIndex = 0) => `m/86'/8639'/0'/0/${agentIndex}`,
  signing:  (agentIndex = 0) => `m/86'/8639'/0'/1/${agentIndex}`,
  comms:    (agentIndex = 0) => `m/86'/8639'/0'/2/${agentIndex}`,
});
