/**
 * BIPON39 Crypto — SHA-256, PBKDF2-HMAC-SHA512, constant-time compare, zeroize.
 * Node.js implementation for Swibe runtime.
 */

import { createHash, createHmac, pbkdf2Sync, timingSafeEqual } from 'node:crypto';

export async function sha256(data) {
  const h = createHash('sha256');
  h.update(Buffer.from(data));
  return new Uint8Array(h.digest());
}

export async function pbkdf2HmacSha512(password, salt, iters = 2048, dkLen = 64) {
  const out = pbkdf2Sync(
    Buffer.from(password),
    Buffer.from(salt),
    iters,
    dkLen,
    'sha512'
  );
  return new Uint8Array(out);
}

export function hmacSha512(key, data) {
  const mac = createHmac('sha512', Buffer.from(key));
  mac.update(Buffer.from(data));
  return new Uint8Array(mac.digest());
}

export function timingSafeEq(a, b) {
  if (a.length !== b.length) return false;
  return timingSafeEqual(Buffer.from(a), Buffer.from(b));
}

export function zeroize(buf) {
  buf.fill(0);
}

export function bufToHex(b) {
  return [...b].map(x => x.toString(16).padStart(2, '0')).join('');
}

export function hexToBytes(h) {
  const clean = h.startsWith('0x') ? h.slice(2) : h;
  return new Uint8Array(clean.match(/../g).map(x => parseInt(x, 16)));
}

export function randomEntropy(bytes) {
  const { randomBytes } = require('node:crypto');
  return new Uint8Array(randomBytes(bytes));
}
