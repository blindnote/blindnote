import { describe, expect, it } from 'vitest'
import { MAX_NOTE_BYTES, decryptNote, encryptNote, exportKey, generateKey, importKey } from './crypto'

describe('crypto', () => {
  it('exports a fresh key as 43 base64url characters (256 bits)', async () => {
    const encoded = await exportKey(await generateKey())
    expect(encoded).toHaveLength(43)
    expect(encoded).toMatch(/^[A-Za-z0-9_-]+$/)
  })

  it('round-trips a note, re-importing the key as the recipient would', async () => {
    const key = await generateKey()
    const text = 'gizli mesaj — çok özel 🔐'
    const payload = await encryptNote(key, text)
    const recipientKey = await importKey(await exportKey(key))
    expect(await decryptNote(recipientKey, payload)).toBe(text)
  })

  it('rejects a key of the wrong length', async () => {
    await expect(importKey('AAAA')).rejects.toThrow()
  })

  it('fails to decrypt if the ciphertext was altered', async () => {
    const key = await generateKey()
    const payload = await encryptNote(key, 'tamper check')
    const last = payload.length - 1
    payload[last] = (payload[last] ?? 0) ^ 0x01
    await expect(decryptNote(key, payload)).rejects.toThrow()
  })

  it('refuses a note larger than the maximum', async () => {
    const key = await generateKey()
    await expect(encryptNote(key, 'a'.repeat(MAX_NOTE_BYTES + 1))).rejects.toThrow()
  })

  it('throws on a payload too short to hold a nonce', async () => {
    const key = await generateKey()
    await expect(decryptNote(key, new Uint8Array(8))).rejects.toThrow()
  })
})
