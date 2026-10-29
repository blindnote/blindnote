/**
 * Unpadded base64url, the only encoding that crosses the wire or sits in a
 * link.
 *
 * Unpadded because `=` is reserved in a query string and ugly in a fragment,
 * and every value here has a fixed length anyway. The server refuses padded
 * input, so these two functions and the Rust side have to agree exactly.
 */

/**
 * A byte array backed by a plain `ArrayBuffer`.
 *
 * Spelled out because `Uint8Array` is generic over its buffer since TypeScript
 * 5.7, and the unparameterised form admits a `SharedArrayBuffer`, which
 * WebCrypto will not accept. Naming the type here keeps that explanation in
 * one place instead of at every call site.
 */
export type Bytes = Uint8Array<ArrayBuffer>

/** Encodes bytes as unpadded base64url. */
export function toBase64url(bytes: Uint8Array): string {
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

/**
 * Decodes unpadded base64url, or throws.
 *
 * Rejects anything outside the url-safe alphabet rather than letting `atob`
 * be lenient about it, so that one note id has exactly one spelling.
 */
export function fromBase64url(value: string): Bytes {
  if (!/^[A-Za-z0-9_-]*$/.test(value)) throw new Error('not base64url')

  const padded = value.replace(/-/g, '+').replace(/_/g, '/')
  const binary = atob(padded)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i += 1) bytes[i] = binary.charCodeAt(i)
  return bytes
}
