/**
 * BIPON39 Module for Swibe — Deterministic Agent Identity
 *
 * Vanity-Cloakseed BIP-39 wallet standard for all Swibe agents.
 * Every agent born gets a BIPON39 mnemonic that deterministically
 * generates identity, archetype, capabilities, and signing keys.
 *
 * BIPON39 = soul. Swibe = body. This module fuses them.
 */

// Core wordspace (256 canonical tokens, 2048 expanded)
export {
  ROOTS, AFFIXES, SUBTONES,
  BASE256, EXP2048,
  WORDLIST256_MERKLE_ROOT,
  AFFIX_META,
  merkleRoot256,
  verifyWordlistIntegrity,
  lookupMeta256,
} from './wordspace.js';

// Crypto primitives
export {
  sha256,
  pbkdf2HmacSha512,
  hmacSha512,
  timingSafeEq,
  zeroize,
  bufToHex,
  hexToBytes,
} from './crypto.js';

// Mnemonic engine
export {
  SALT_PREFIX, PASS_LABEL, ALLOWED_ENT,
  bitsPerWord,
  entropyToMnemonic,
  mnemonicToEntropy,
  mnemonicToIndices,
  indicesToMnemonic,
  reencodeMnemonic,
  mnemonicToSeed,
  oduPrimaryIndex,
  elementalSignature,
  sabbathGate,
} from './mnemonic.js';

// Key derivation
export {
  masterFromSeed,
  PATHS,
  AGENT_PATH,
} from './derivation.js';

// Agent identity (main integration point)
export {
  generateAgentIdentity,
  recoverAgentIdentity,
  signMessage,
  verifyMessage,
  deriveCapabilities,
  agentAddress,
  checkSabbath,
} from './agent-identity.js';
