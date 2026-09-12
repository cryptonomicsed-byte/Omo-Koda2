/**
 * BIPON39 Wordspace — Base-256 canon (v1.0)
 * 16 ROOTS x 16 AFFIXES = 256 tokens. Frozen slug order is canonical.
 * Deterministic expansion to 2048 via 8 subtones per base word.
 *
 * Ported from bipon39 TypeScript for Swibe agent identity.
 */

import { sha256, bufToHex } from './crypto.js';

export const ROOTS = Object.freeze([
  'esu', 'sango', 'ogun', 'oya', 'yemoja', 'osun', 'obatala', 'orunmila',
  'egungun', 'ori', 'ile', 'omi', 'ina', 'afeefe', 'igi', 'irawo',
]);

export const AFFIXES = Object.freeze([
  'gate', 'volt', 'forge', 'stream', 'tide', 'veil', 'crown', 'mirror',
  'path', 'seal', 'code', 'sigil', 'drum', 'thunder', 'river', 'dawn',
]);

export const SUBTONES = Object.freeze([
  'alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta',
]);

// 256 canonical tokens
export const BASE256 = Object.freeze(
  ROOTS.flatMap(r => AFFIXES.map(a => `${r}-${a}`))
);

// 2048 expanded tokens: index = (baseId << 3) | subIndex
export const EXP2048 = Object.freeze(
  BASE256.flatMap(base => SUBTONES.map(st => `${base}~${st}`))
);

// Pinned Merkle root (SHA-256 leaves of ASCII slugs)
export const WORDLIST256_MERKLE_ROOT =
  '0ab1fafa074065c86877877f828499a8649b558457000f0ede436730eeb9da89';

/**
 * Compute SHA-256 Merkle root of the BASE256 wordlist.
 * Pairwise concat+SHA-256; duplicate last leaf if odd count.
 */
export async function merkleRoot256(slugs = BASE256) {
  let level = [];
  for (const s of slugs) {
    level.push(await sha256(new TextEncoder().encode(s)));
  }
  if (level.length === 0) throw new Error('empty wordlist');

  while (level.length > 1) {
    if (level.length % 2 === 1) level = [...level, level[level.length - 1]];
    const next = [];
    for (let i = 0; i < level.length; i += 2) {
      const concat = new Uint8Array(level[i].length + level[i + 1].length);
      concat.set(level[i], 0);
      concat.set(level[i + 1], level[i].length);
      next.push(await sha256(concat));
    }
    level = next;
  }
  return bufToHex(level[0]);
}

export async function verifyWordlistIntegrity() {
  return (await merkleRoot256(BASE256)) === WORDLIST256_MERKLE_ROOT;
}

// Affix -> element/ritual/ethic/sigil metadata (frozen)
export const AFFIX_META = Object.freeze({
  gate:    { element: 'Earth', ritual_cue: 'draw crossroads',    ethical_tag: 'threshold',       sigil_seed: 'cross+dot' },
  volt:    { element: 'Fire',  ritual_cue: 'clap thunder',       ethical_tag: 'righteous-force', sigil_seed: 'lightning-jaw' },
  forge:   { element: 'Fire',  ritual_cue: 'strike iron',        ethical_tag: 'craft',           sigil_seed: 'anvil-rune' },
  stream:  { element: 'Water', ritual_cue: 'pour libation',      ethical_tag: 'flow',            sigil_seed: 'wave-sigil' },
  tide:    { element: 'Water', ritual_cue: 'offer to ocean',     ethical_tag: 'returning',       sigil_seed: 'tide-knot' },
  veil:    { element: 'Air',   ritual_cue: 'burn incense',       ethical_tag: 'reveal/conceal',  sigil_seed: 'smoke-spiral' },
  crown:   { element: 'Ether', ritual_cue: 'white cloth prayer', ethical_tag: 'mercy-law',       sigil_seed: 'halo-arc' },
  mirror:  { element: 'Air',   ritual_cue: 'polish mirror',      ethical_tag: 'truth',           sigil_seed: 'twin-glyph' },
  path:    { element: 'Earth', ritual_cue: 'mark footprints',    ethical_tag: 'journey',         sigil_seed: 'line-run' },
  seal:    { element: 'Ether', ritual_cue: 'close the circle',   ethical_tag: 'binding',         sigil_seed: 'ring-lock' },
  code:    { element: 'Air',   ritual_cue: 'write glyph',        ethical_tag: 'syntax',          sigil_seed: 'hex-grid' },
  sigil:   { element: 'Ether', ritual_cue: 'trace sigil',        ethical_tag: 'intent',          sigil_seed: 'sigil-star' },
  drum:    { element: 'Earth', ritual_cue: 'beat dundun',         ethical_tag: 'rhythm',          sigil_seed: 'pulse-mark' },
  thunder: { element: 'Fire',  ritual_cue: 'speak justice',      ethical_tag: 'judgment',        sigil_seed: 'bolt-mark' },
  river:   { element: 'Water', ritual_cue: 'wash hands',         ethical_tag: 'cleansing',       sigil_seed: 'delta-sign' },
  dawn:    { element: 'Ether', ritual_cue: 'face sunrise',       ethical_tag: 'begin',           sigil_seed: 'east-ray' },
});

/**
 * Look up full ritual metadata for a BASE256 token by index.
 */
export function lookupMeta256(id) {
  const word = BASE256[id];
  if (!word) return null;
  const [root, affix] = word.split('-');
  const m = AFFIX_META[affix];
  return { id, word, root, affix, ...m };
}
