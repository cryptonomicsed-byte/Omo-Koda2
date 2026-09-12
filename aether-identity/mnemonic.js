/**
 * BIPON39 Mnemonic Engine — entropy <-> mnemonic <-> seed
 *
 * Core protocol logic:
 * - entropy -> checksum -> bit packing -> word indices -> mnemonic
 * - mnemonic -> seed via PBKDF2-HMAC-SHA512 with "BIPỌ̀N39 seed" salt
 * - Odù primary index: XOR reduction of word indices
 * - Elemental signature: affix-based element counting
 * - Sabbath gate: queue irreversible writes on Saturday
 */

import { BASE256, EXP2048, AFFIX_META } from './wordspace.js';
import { sha256, pbkdf2HmacSha512, timingSafeEq, zeroize } from './crypto.js';

export const SALT_PREFIX = 'BIPỌ̀N39 seed';  // NFC preserved
export const PASS_LABEL  = ' Ọ̀RÍ:';         // leading space is normative

export const ALLOWED_ENT = [128, 160, 192, 224, 256];

function toNFKD(s) { return s.normalize('NFKD'); }
function _toNFC(s)  { return s.normalize('NFC'); }

export function bitsPerWord(mode) {
  return mode === '256' ? 8 : 11;
}

/**
 * Convert entropy bytes to mnemonic word array.
 */
export async function entropyToMnemonic(entropy, mode = '256') {
  const ENT = entropy.length * 8;
  if (!ALLOWED_ENT.includes(ENT)) {
    throw new Error(`Invalid ENT ${ENT} for mode ${mode}`);
  }

  const hash = await sha256(entropy);
  const csBits = ENT / 32;
  const csByte = hash[0];

  const bits = [];
  for (const b of entropy) {
    for (let i = 7; i >= 0; i--) bits.push((b >>> i) & 1);
  }
  // Extract checksum: top csBits bits of hash[0], MSB first (BIP39 standard)
  for (let i = 7; i > 7 - csBits; i--) {
    bits.push((csByte >>> i) & 1);
  }

  const bpw = bitsPerWord(mode);
  const total = ENT + csBits;
  const pad = (bpw - (total % bpw)) % bpw;
  for (let i = 0; i < pad; i++) bits.push(0);

  const indices = [];
  for (let i = 0; i < bits.length; i += bpw) {
    let v = 0;
    for (let j = 0; j < bpw; j++) v = (v << 1) | bits[i + j];
    indices.push(v);
  }

  return indicesToMnemonic(indices, mode);
}

export function indicesToMnemonic(indices, mode) {
  if (mode === '256') return indices.map(i => BASE256[i]);
  return indices.map(i => EXP2048[i]);
}

function validateNormalizedSlug(input) {
  if (!input) throw new Error('Empty token');
  if (!/^[a-z0-9~-]+$/.test(input)) throw new Error('Invalid characters in token');
}

export function mnemonicToIndices(words, mode) {
  const list = mode === '256' ? BASE256 : EXP2048;
  return words.map(w => {
    const slug = toNFKD(w).toLowerCase();
    validateNormalizedSlug(slug);
    const idx = list.indexOf(slug);
    if (idx < 0) throw new Error(`Unknown token: ${slug}`);
    return idx;
  });
}

/**
 * Recover entropy from mnemonic words. Validates checksum.
 */
export async function mnemonicToEntropy(words, mode) {
  const bpw = bitsPerWord(mode);
  const idx = mnemonicToIndices(words, mode);
  const totalBits = idx.length * bpw;

  const bits = [];
  for (const v of idx) {
    for (let i = bpw - 1; i >= 0; i--) bits.push((v >>> i) & 1);
  }

  const allowed = [...ALLOWED_ENT].sort((a, b) => b - a);
  for (const ENT of allowed) {
    const csBits = ENT / 32;
    if (ENT + csBits > totalBits) continue;

    const entBits = bits.slice(0, ENT);
    const csGot = bits.slice(ENT, ENT + csBits);

    const bytes = [];
    for (let i = 0; i < ENT; i += 8) {
      let b = 0;
      for (let j = 0; j < 8; j++) b = (b << 1) | entBits[i + j];
      bytes.push(b);
    }
    const entropy = new Uint8Array(bytes);
    const hash = await sha256(entropy);
    const csExp = hash[0] >>> (8 - csBits);
    const csVal = csGot.reduce((a, b) => (a << 1) | b, 0);

    const aBuf = new Uint8Array([csVal & 0xff]);
    const eBuf = new Uint8Array([csExp & 0xff]);
    if (timingSafeEq(aBuf, eBuf)) return entropy;
  }

  throw new Error('Checksum mismatch or unsupported parameters');
}

/**
 * Re-encode mnemonic between modes via entropy roundtrip.
 */
export async function reencodeMnemonic(words, from, to) {
  const ent = await mnemonicToEntropy(words, from);
  return entropyToMnemonic(ent, to);
}

/**
 * Derive 64-byte seed from mnemonic phrase + optional passphrase.
 * Uses PBKDF2-HMAC-SHA512, 2048 iterations.
 * Salt: "BIPỌ̀N39 seed" [+ " Ọ̀RÍ:" + passphrase]
 */
export async function mnemonicToSeed(mnemonic, passphrase = '') {
  const m = toNFKD(typeof mnemonic === 'string' ? mnemonic : mnemonic.join(' '));
  const saltStr = SALT_PREFIX + (passphrase ? PASS_LABEL + passphrase : '');
  const salt = toNFKD(saltStr);
  const mBytes = new TextEncoder().encode(m);
  const sBytes = new TextEncoder().encode(salt);
  try {
    return await pbkdf2HmacSha512(mBytes, sBytes, 2048, 64);
  } finally {
    zeroize(mBytes);
    zeroize(sBytes);
  }
}

// --- Odù + Elemental ---

/**
 * XOR reduction of word indices -> 0..255.
 * Deterministic archetype / routing key.
 */
export function oduPrimaryIndex(words, mode) {
  const idx = mnemonicToIndices(words, mode);
  return idx.reduce((a, b) => (a ^ b) & 0xff, 0);
}

/**
 * Count elements across mnemonic words based on affix mapping.
 */
export function elementalSignature(words) {
  const counts = { Fire: 0, Water: 0, Earth: 0, Air: 0, Ether: 0 };
  for (const w of words) {
    const slug = w.split('~')[0]; // strip subtone if present
    const id = BASE256.indexOf(slug);
    if (id < 0) continue;
    const affix = slug.split('-')[1];
    const meta = AFFIX_META[affix];
    if (meta) counts[meta.element]++;
  }
  return counts;
}

/**
 * Sabbath gate: queue irreversible writes on Saturday.
 * Returns "allow" or "queue".
 */
export function sabbathGate(now = new Date(), councilOverride = false, trustedUtcDay = undefined) {
  const day = (typeof trustedUtcDay === 'number') ? trustedUtcDay : now.getUTCDay();
  if (day === 6 && !councilOverride) return 'queue';
  return 'allow';
}
