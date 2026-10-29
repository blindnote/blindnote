/**
 * The three calls this client makes.
 *
 * Every one is a `POST` and every id travels in the body. No note id ever
 * appears in a request line, because a request line reaches logs, proxies and
 * error reports -- and reading a note destroys it, so a `GET` that did the
 * reading could be triggered by anything that follows links.
 */

import { fromBase64url, toBase64url, type Bytes } from './base64url'

/** What went wrong, in terms the interface can act on. */
export type ApiFailure =
  /** The note does not exist, has expired, or has already been read. */
  | { kind: 'unavailable' }
  /** Too many requests from this address. */
  | { kind: 'throttled' }
  /** The note was refused: too large, or malformed. */
  | { kind: 'rejected' }
  /** The request never arrived, or the answer never came back. */
  | { kind: 'offline' }
  /** Something broke on the server. */
  | { kind: 'server' }

/** A failure the interface can show, thrown by the calls below. */
export class ApiError extends Error {
  readonly failure: ApiFailure

  constructor(failure: ApiFailure) {
    super(failure.kind)
    this.name = 'ApiError'
    this.failure = failure
  }
}

/** Maps a status onto a failure. One shared mapping, so none of it drifts. */
function failureFor(status: number): ApiFailure {
  if (status === 404) return { kind: 'unavailable' }
  if (status === 429) return { kind: 'throttled' }
  if (status === 400 || status === 413) return { kind: 'rejected' }
  return { kind: 'server' }
}

/** Sends one request, turning everything that can go wrong into an `ApiError`. */
async function post(path: string, body: unknown): Promise<Response> {
  let response: Response
  try {
    response = await fetch(path, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(body),
      // Nothing here is authenticated and there is nothing to send: no
      // cookies, no credentials, ever.
      credentials: 'omit',
      cache: 'no-store',
    })
  } catch {
    throw new ApiError({ kind: 'offline' })
  }

  if (!response.ok) throw new ApiError(failureFor(response.status))
  return response
}

/** Stores an encrypted note and returns its id. */
export async function createNote(payload: Uint8Array): Promise<string> {
  const response = await post('/api/notes', { ciphertext: toBase64url(payload) })
  const body = (await response.json()) as { id?: unknown }

  if (typeof body.id !== 'string') throw new ApiError({ kind: 'server' })
  return body.id
}

/**
 * Reads a note and destroys it.
 *
 * There is no second chance. By the time this resolves the note's key has been
 * deleted on the server, so whatever comes back here is the only copy there
 * will ever be.
 */
export async function revealNote(id: string): Promise<Bytes> {
  const response = await post('/api/reveal', { id })
  const body = (await response.json()) as { ciphertext?: unknown }

  if (typeof body.ciphertext !== 'string') throw new ApiError({ kind: 'server' })
  try {
    return fromBase64url(body.ciphertext)
  } catch {
    throw new ApiError({ kind: 'server' })
  }
}

/** Destroys a note without reading it, on an abuse report. */
export async function reportNote(id: string): Promise<void> {
  await post('/api/report', { id })
}
