/**
 * Encryption, in the browser, with the native WebCrypto implementation.
 *
 * AES-256-GCM. No crypto library: `crypto.subtle` is in every browser this
 * site supports, it is the fastest option available, and every dependency
 * added here is another line in the content security policy and another entry
 * in the reproducible build.
 *
 * The key produced here never leaves the browser except inside the URL
 * fragment, which is not sent in an HTTP request. The server therefore holds
 * ciphertext it cannot read.
 */

import { fromBase64url, toBase64url, type Bytes } from './base64url'

/** Nonce length for AES-GCM, in bytes. 96 bits is the standard size. */
const IV_BYTES = 12

/** Raw key length in bytes. */
const KEY_BYTES = 32

/** Largest note we will encrypt, measured in UTF-8 bytes. */
export const MAX_NOTE_BYTES = 64 * 1024

/**
 * A fresh 256-bit key.
 *
 * `extractable` is true because the key has to be exported into the link.
 * That is the whole design: the link is the key.
 */
export async function generateKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt'])
}

/** Exports a key as unpadded base64url: 43 characters. */
export async function exportKey(key: CryptoKey): Promise<string> {
  const raw = await crypto.subtle.exportKey('raw', key)
  return toBase64url(new Uint8Array(raw))
}

/**
 * Reads a key back out of a link, or throws.
 *
 * The length is checked before WebCrypto sees it, so that a truncated link is
 * rejected here rather than by a failure further along -- the difference
 * matters, because by the time the ciphertext has been fetched the note is
 * already destroyed.
 */
export async function importKey(encoded: string): Promise<CryptoKey> {
  const raw = fromBase64url(encoded)
  if (raw.length !== KEY_BYTES) throw new Error('wrong key length')

  return crypto.subtle.importKey('raw', raw, { name: 'AES-GCM' }, false, ['decrypt'])
}

/**
 * Encrypts a note and returns what the server will store.
 *
 * The payload is `iv ‖ ciphertext ‖ tag`: the nonce travels with the
 * ciphertext because the server treats the whole thing as opaque and has no
 * business knowing the framing. It has its own, separate nonce for the layer
 * it adds on top.
 *
 * Throws if the note is larger than {@link MAX_NOTE_BYTES}, measured in UTF-8
 * bytes rather than characters -- a Turkish note is longer in bytes than it
 * looks, and the limit the server enforces is a byte limit.
 */
export async function encryptNote(key: CryptoKey, text: string): Promise<Bytes> {
  const plaintext = new TextEncoder().encode(text)
  if (plaintext.length > MAX_NOTE_BYTES) throw new Error('note too large')

  const iv = crypto.getRandomValues(new Uint8Array(IV_BYTES))
  const ciphertext = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, plaintext)

  const payload = new Uint8Array(iv.length + ciphertext.byteLength)
  payload.set(iv, 0)
  payload.set(new Uint8Array(ciphertext), iv.length)
  return payload
}

/**
 * Decrypts what the server returned.
 *
 * Throws if the payload is too short to contain a nonce and a tag, and if the
 * tag does not verify -- which, with the right key, means the ciphertext was
 * altered.
 */
export async function decryptNote(key: CryptoKey, payload: Bytes): Promise<string> {
  if (payload.length <= IV_BYTES) throw new Error('payload too short')

  const iv = payload.subarray(0, IV_BYTES)
  const ciphertext = payload.subarray(IV_BYTES)
  const plaintext = await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, key, ciphertext)

  return new TextDecoder().decode(plaintext)
}
