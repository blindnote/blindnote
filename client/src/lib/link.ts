/**
 * The link: `https://host/#<id>/<key>`.
 *
 * Both halves live in the fragment, which browsers never send in an HTTP
 * request. That is the whole reason the id is there too rather than in the
 * path: a path id would reach the server's logs, any proxy in between, and
 * `Referer` -- and a link-preview bot following it could burn the note before
 * the recipient ever saw it.
 */

import { fromBase64url } from './base64url'

/** A note id is 16 bytes, which is 22 unpadded base64url characters. */
const ID_LENGTH = 22

/** A key is 32 bytes, which is 43. */
const KEY_LENGTH = 43

/** What a note link carries. */
export type NoteLink = {
  /** The note id, still encoded, as the API wants it. */
  id: string
  /** The key, still encoded, for `importKey`. */
  key: string
}

/**
 * Reads a note link out of a fragment, or returns `null`.
 *
 * Strict on purpose, and this is the strictness that matters most in the whole
 * client: reading a note *destroys* it, so a truncated or mistyped link must
 * be rejected **before** the request that burns it. A link that fails here
 * costs the user a trip back to wherever they got it. A link that passed here
 * and failed after the fetch would cost them the note.
 *
 * The separator is `/`, which is what the design's own documentation specifies.
 */
export function parseNoteLink(fragment: string): NoteLink | null {
  const raw = fragment.startsWith('#') ? fragment.slice(1) : fragment
  const [id, key, ...rest] = raw.split('/')
  if (id === undefined || key === undefined || rest.length > 0) return null
  if (id.length !== ID_LENGTH || key.length !== KEY_LENGTH) return null

  try {
    // Both halves have to decode, not merely look right. `fromBase64url`
    // refuses anything outside the url-safe alphabet.
    fromBase64url(id)
    fromBase64url(key)
  } catch {
    return null
  }

  return { id, key }
}

/** Builds the link to hand to the recipient. */
export function buildNoteLink(origin: string, id: string, key: string): string {
  return `${origin}/#${id}/${key}`
}

/**
 * Removes the fragment from the address bar.
 *
 * Called only after a note has been read, never before. Before the read, the
 * fragment is the only copy of the link the page has that survives a reload --
 * and an in-app browser inside a chat application reloads more often than you
 * would think. Afterwards the key opens nothing, and leaving it in the address
 * bar means leaving it in screenshots, in a shared screen, and in whatever the
 * browser syncs.
 */
export function forgetFragment(): void {
  try {
    window.history.replaceState(null, '', window.location.pathname)
  } catch {
    // A browser that refuses to rewrite history is not a reason to fail a read
    // that already succeeded.
  }
}
