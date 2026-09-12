/**
 * BIPON39 Agent Identity — Deterministic identity kernel for Swibe agents
 *
 * Every Swibe agent receives a BIPON39 mnemonic at birth.
 * The mnemonic deterministically generates:
 *   - agent_id (SHA-256 of seed)
 *   - odù (archetype / routing key)
 *   - elemental signature (personality vector)
 *   - master key (for message authentication)
 *
 * Identity is reproducible: same entropy = same agent forever.
 */

import crypto from 'node:crypto';
import { entropyToMnemonic, mnemonicToSeed, oduPrimaryIndex, elementalSignature, sabbathGate } from './mnemonic.js';
import { masterFromSeed, AGENT_PATH } from './derivation.js';
import { lookupMeta256, BASE256 } from './wordspace.js';
import { sha256, bufToHex, zeroize } from './crypto.js';

/**
 * Generate a complete agent identity from entropy.
 * If no entropy provided, generates 256-bit random entropy.
 *
 * @param {Uint8Array} [entropy] — 16-32 bytes of entropy
 * @param {object} [options]
 * @param {'256'|'2048'} [options.mode='256'] — wordlist mode
 * @param {string} [options.passphrase=''] — optional Ọ̀RÍ passphrase
 * @returns {Promise<AgentIdentity>}
 */
export async function generateAgentIdentity(entropy = null, options = {}) {
  const mode = options.mode || '256';
  const passphrase = options.passphrase || '';

  if (!entropy) {
    entropy = new Uint8Array(crypto.randomBytes(32));
  }

  const mnemonic = await entropyToMnemonic(entropy, mode);
  const seed = await mnemonicToSeed(mnemonic, passphrase);

  // Agent ID = SHA-256(seed) — deterministic, unique
  const idHash = await sha256(seed);
  const agentId = bufToHex(idHash);

  // Master key for HMAC-based message signing
  const master = masterFromSeed(seed, 'native');

  // Odù archetype (0-255): deterministic behavioral seed
  const odu = oduPrimaryIndex(mnemonic, mode);

  // Elemental signature: personality vector
  const elements = elementalSignature(mnemonic);

  // Dominant element
  const dominant = Object.entries(elements)
    .sort((a, b) => b[1] - a[1])[0];

  // Ritual metadata for each word
  const ritualTokens = mnemonic.map(w => {
    const slug = w.split('~')[0];
    const idx = BASE256.indexOf(slug);
    return idx >= 0 ? lookupMeta256(idx) : null;
  }).filter(Boolean);

  // Derive agent signing key path
  const paths = {
    identity: AGENT_PATH.identity(0),
    signing: AGENT_PATH.signing(0),
    comms: AGENT_PATH.comms(0),
  };

  const identity = {
    agentId,
    mnemonic,
    mode,
    odu,
    elements,
    dominantElement: dominant[0],
    ritualTokens,
    paths,
    masterKey: master.key,
    chainCode: master.chainCode,
    created: Date.now(),
  };

  // Zeroize sensitive intermediates
  zeroize(seed);

  return identity;
}

/**
 * Rebuild agent identity from mnemonic (deterministic recovery).
 * Same mnemonic always produces the same identity.
 */
export async function recoverAgentIdentity(mnemonic, options = {}) {
  const mode = options.mode || '256';
  const passphrase = options.passphrase || '';

  const words = typeof mnemonic === 'string' ? mnemonic.split(/\s+/) : mnemonic;
  const seed = await mnemonicToSeed(words, passphrase);

  const idHash = await sha256(seed);
  const agentId = bufToHex(idHash);
  const master = masterFromSeed(seed, 'native');
  const odu = oduPrimaryIndex(words, mode);
  const elements = elementalSignature(words);
  const dominant = Object.entries(elements).sort((a, b) => b[1] - a[1])[0];

  const identity = {
    agentId,
    mnemonic: words,
    mode,
    odu,
    elements,
    dominantElement: dominant[0],
    masterKey: master.key,
    chainCode: master.chainCode,
    recovered: true,
    created: Date.now(),
  };

  zeroize(seed);
  return identity;
}

/**
 * Sign a message using agent's master key (HMAC-SHA256).
 * Lightweight authentication — not a blockchain signature.
 */
export function signMessage(masterKey, message) {
  const { createHmac } = require('node:crypto');
  const mac = createHmac('sha256', Buffer.from(masterKey));
  mac.update(typeof message === 'string' ? message : Buffer.from(message));
  return new Uint8Array(mac.digest());
}

/**
 * Verify a message signature against agent's master key.
 */
export function verifyMessage(masterKey, message, signature) {
  const expected = signMessage(masterKey, message);
  const { timingSafeEqual } = require('node:crypto');
  if (expected.length !== signature.length) return false;
  return timingSafeEqual(Buffer.from(expected), Buffer.from(signature));
}

/**
 * Derive agent capabilities from Odù + elements.
 * Fire-heavy -> execution agents
 * Water-heavy -> routing / flow agents
 * Earth-heavy -> storage / persistence
 * Air-heavy -> analysis / reasoning
 * Ether-heavy -> coordination / governance
 */
export function deriveCapabilities(odu, elements) {
  const dominant = Object.entries(elements).sort((a, b) => b[1] - a[1]);
  const primary = dominant[0]?.[0] || 'Earth';
  const secondary = dominant[1]?.[0] || 'Earth';

  const affinities = {
    Fire:  { execution: 1.0, speed: 0.8,  persistence: 0.3 },
    Water: { routing: 1.0,   flow: 0.8,   adaptation: 0.7 },
    Earth: { storage: 1.0,   stability: 0.9, persistence: 0.8 },
    Air:   { analysis: 1.0,  reasoning: 0.8, communication: 0.7 },
    Ether: { coordination: 1.0, governance: 0.8, synthesis: 0.7 },
  };

  return {
    primary: { element: primary, ...affinities[primary] },
    secondary: { element: secondary, ...affinities[secondary] },
    oduArchetype: odu,
    // Sabbath-aware: agents respect temporal governance
    sabbathAware: true,
  };
}

/**
 * Generate a short agent address for network identity.
 * Format: swibe://<first 20 hex chars of agentId>
 */
export function agentAddress(agentId) {
  return `swibe://${agentId.slice(0, 40)}`;
}

/**
 * Check if agent operations should be queued (Sabbath gate).
 */
export function checkSabbath(councilOverride = false) {
  return sabbathGate(new Date(), councilOverride);
}
