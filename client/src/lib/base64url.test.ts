import { describe, expect, it } from 'vitest'
import { fromBase64url, toBase64url } from './base64url'

describe('base64url', () => {
  it('round-trips arbitrary bytes at every length', () => {
    for (let len = 0; len <= 48; len += 1) {
      const bytes = new Uint8Array(len)
      for (let i = 0; i < len; i += 1) bytes[i] = (i * 37 + 11) & 0xff
      const back = fromBase64url(toBase64url(bytes))
      expect(Array.from(back)).toEqual(Array.from(bytes))
    }
  })

  it('emits no padding and only url-safe characters', () => {
    const encoded = toBase64url(new Uint8Array([0xfb, 0xff, 0xbf, 0x00, 0x10, 0x83]))
    expect(encoded).not.toContain('=')
    expect(encoded).not.toContain('+')
    expect(encoded).not.toContain('/')
    expect(encoded).toMatch(/^[A-Za-z0-9_-]*$/)
  })

  it('uses - and _ where standard base64 would use + and /', () => {
    expect(toBase64url(new Uint8Array([0xff, 0xff]))).toBe('__8')
    expect(toBase64url(new Uint8Array([0xfb, 0xf0]))).toBe('-_A')
  })

  it('rejects anything outside the url-safe alphabet', () => {
    expect(() => fromBase64url('abc+')).toThrow()
    expect(() => fromBase64url('abc/')).toThrow()
    expect(() => fromBase64url('ab=')).toThrow()
    expect(() => fromBase64url('aa aa')).toThrow()
  })

  it('decodes an empty string to no bytes', () => {
    expect(fromBase64url('').length).toBe(0)
  })
})
