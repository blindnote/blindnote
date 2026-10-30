import { describe, expect, it } from 'vitest'
import { buildNoteLink, parseNoteLink } from './link'

const ID = 'VDjPkmhBaXm9QsR2tLkZ4w' // 16 bytes -> 22 unpadded base64url chars
const KEY = 'D936F4PyUQYhH4tN7vKmBcR8sWxZ0eLpGqUoIaJ5dY2' // 32 bytes -> 43 chars

describe('parseNoteLink', () => {
  it('parses a well-formed fragment, with or without the leading #', () => {
    expect(parseNoteLink(`#${ID}/${KEY}`)).toEqual({ id: ID, key: KEY })
    expect(parseNoteLink(`${ID}/${KEY}`)).toEqual({ id: ID, key: KEY })
  })

  it('rejects a truncated id or key (strictness guards the burn)', () => {
    expect(parseNoteLink(`#${ID.slice(0, 21)}/${KEY}`)).toBeNull()
    expect(parseNoteLink(`#${ID}/${KEY.slice(0, 42)}`)).toBeNull()
  })

  it('rejects a fragment with the wrong number of parts', () => {
    expect(parseNoteLink(`#${ID}`)).toBeNull()
    expect(parseNoteLink(`#${ID}/${KEY}/extra`)).toBeNull()
  })

  it('rejects characters outside the url-safe alphabet', () => {
    expect(parseNoteLink(`#+${ID.slice(1)}/${KEY}`)).toBeNull()
  })

  it('returns null for an empty fragment or a plain anchor', () => {
    expect(parseNoteLink('')).toBeNull()
    expect(parseNoteLink('#sifreleme')).toBeNull()
  })
})

describe('buildNoteLink', () => {
  it('builds origin/#id/key', () => {
    expect(buildNoteLink('https://blindnote.com', ID, KEY)).toBe(`https://blindnote.com/#${ID}/${KEY}`)
  })
})
