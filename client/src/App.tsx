/**
 * The whole of the application's routing and state.
 *
 * There is no router. Which screen to show follows from one question -- does
 * the URL fragment carry a note link -- and from what the user has done since.
 * A router would be a dependency, a line in the content security policy and an
 * entry in the reproducible build, for a site with five screens and no paths.
 *
 * The components below are presentational: data in through props, events out
 * through callbacks, no `fetch` and no crypto. Everything that talks to the
 * network or to WebCrypto lives in `src/lib` and is called from here.
 */

import { useEffect, useRef, useState } from 'react'
import { ComposeScreen } from './components/ComposeScreen'
import { LinkReadyScreen } from './components/LinkReadyScreen'
import { NoteNotFoundScreen } from './components/NoteNotFoundScreen'
import { NoteOpenedScreen } from './components/NoteOpenedScreen'
import { PageLayout } from './components/PageLayout'
import { RevealScreen } from './components/RevealScreen'
import { ApiError, createNote, revealNote } from './lib/api'
import { decryptNote, encryptNote, exportKey, generateKey, importKey } from './lib/crypto'
import { useCopy, type Strings } from './lib/i18n'
import { buildNoteLink, forgetFragment, parseNoteLink, type NoteLink } from './lib/link'

/** Longest note the composer will let someone type, in characters. */
const MAX_NOTE_CHARACTERS = 10_000

type Screen =
  | { name: 'compose' }
  | { name: 'link'; url: string }
  | { name: 'reveal'; link: NoteLink }
  | { name: 'opened'; text: string }
  | { name: 'notfound' }

/**
 * Decides what the fragment on arrival means.
 *
 * The site's own navigation uses fragments too -- `#nedir`, `#sifreleme`,
 * `#sss` -- so an unparseable fragment is usually just an anchor and must not
 * produce "note not found". A fragment containing a `/` was meant to be a note
 * link, though, and one that fails to parse is a broken link rather than an
 * anchor: saying so is far better than silently showing the front page to
 * somebody who was handed a truncated link.
 */
function screenForFragment(fragment: string): Screen {
  const link = parseNoteLink(fragment)
  if (link) return { name: 'reveal', link }

  const looksLikeANoteLink = fragment.replace(/^#/, '').includes('/')
  return looksLikeANoteLink ? { name: 'notfound' } : { name: 'compose' }
}

/** Turns a failure into something a person can act on. */
function messageFor(error: unknown, errors: Strings['errors']): string {
  if (error instanceof ApiError) {
    switch (error.failure.kind) {
      case 'offline':
        return errors.offline
      case 'throttled':
        return errors.throttled
      case 'rejected':
        return errors.rejected
      default:
        return errors.server
    }
  }

  if (error instanceof Error && error.message === 'note too large') return errors.tooLarge
  // Anything left is WebCrypto refusing, which on a current browser means it
  // is not there: the page is on plain HTTP, or the browser is very old.
  return errors.noCrypto
}

export function App() {
  // Read once, on the first render. The fragment is deliberately not watched:
  // clicking "Sıkça Sorulan Sorular" replaces it, and that must not throw away
  // a note the reader has not opened yet.
  const t = useCopy()

  const [screen, setScreen] = useState<Screen>(() => screenForFragment(window.location.hash))
  const [menuOpen, setMenuOpen] = useState(false)

  const [draft, setDraft] = useState('')
  const [composeStatus, setComposeStatus] = useState<'idle' | 'submitting'>('idle')
  const [composeError, setComposeError] = useState<string>()

  const [copied, setCopied] = useState(false)
  const [copyError, setCopyError] = useState<string>()
  const [noteCopied, setNoteCopied] = useState(false)

  const [revealStatus, setRevealStatus] = useState<'idle' | 'opening'>('idle')
  const [revealError, setRevealError] = useState<string>()
  // Reading a note burns it, so the reveal must fire at most once even if the
  // button and the envelope are both triggered before React disables them.
  const revealing = useRef(false)

  useEffect(() => {
    window.scrollTo({ top: 0 })
    setMenuOpen(false)
  }, [screen.name])

  // Opening a note link while the page is already loaded changes the fragment
  // without reloading anything, which is what happens when somebody pastes a
  // link into the address bar of a tab that is already here. Without this the
  // address bar would show the note and the screen would show whatever was
  // there before.
  //
  // Only a note link counts. The site's own navigation shares this fragment --
  // `#nedir`, `#sifreleme`, `#sss` -- and reacting to an anchor would throw
  // away a note the reader has not opened yet.
  useEffect(() => {
    const onFragmentChange = () => {
      const next = screenForFragment(window.location.hash)
      if (next.name === 'compose') return

      setCopied(false)
      setNoteCopied(false)
      setComposeError(undefined)
      setRevealError(undefined)
      setScreen(next)
    }

    window.addEventListener('hashchange', onFragmentChange)
    return () => window.removeEventListener('hashchange', onFragmentChange)
  }, [])

  const go = (next: Screen) => {
    setCopied(false)
    setNoteCopied(false)
    setComposeError(undefined)
    setRevealError(undefined)
    setScreen(next)
  }

  const submit = async () => {
    const text = draft.trim()
    if (text.length === 0) return // The composer shows its own prompt for this.

    setComposeError(undefined)
    setComposeStatus('submitting')
    try {
      const key = await generateKey()
      const payload = await encryptNote(key, text)
      const id = await createNote(payload)
      const url = buildNoteLink(window.location.origin, id, await exportKey(key))

      // Drop focus before the screen changes; iOS Safari otherwise keeps the
      // on-screen keyboard up over the link-ready screen.
      ;(document.activeElement as HTMLElement | null)?.blur()
      go({ name: 'link', url })
    } catch (error) {
      setComposeError(messageFor(error, t.errors))
    } finally {
      setComposeStatus('idle')
    }
  }

  const reveal = async (link: NoteLink) => {
    if (revealing.current) return
    revealing.current = true
    setRevealError(undefined)
    setRevealStatus('opening')
    try {
      // The key is checked before the request, not after. The request destroys
      // the note, so a bad key found afterwards would mean a note lost to a
      // mistyped link.
      const key = await importKey(link.key)
      const payload = await revealNote(link.id)
      const text = await decryptNote(key, payload)

      // Only now. Before the read, the fragment is the only copy of the link
      // that survives a reload.
      forgetFragment()
      go({ name: 'opened', text })
    } catch (error) {
      if (error instanceof ApiError && error.failure.kind === 'unavailable') {
        forgetFragment()
        go({ name: 'notfound' })
        return
      }
      // A note that came back but would not decrypt is already destroyed, and
      // there is nothing truthful to offer except that it is gone.
      if (!(error instanceof ApiError)) {
        forgetFragment()
        go({ name: 'notfound' })
        return
      }

      setRevealError(messageFor(error, t.errors))
    } finally {
      setRevealStatus('idle')
      revealing.current = false
    }
  }

  const copyLink = async (url: string) => {
    setCopyError(undefined)
    try {
      await navigator.clipboard.writeText(url)
      setCopied(true)
    } catch {
      setCopyError(t.errors.copyFailed)
    }
  }

  const startANote = () => {
    setDraft('')
    go({ name: 'compose' })
  }

  // On the compose screen the header button has nowhere to navigate to, so it
  // scrolls up and puts the cursor in the box instead.
  const focusTheComposer = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' })
    window.setTimeout(
      () => document.querySelector<HTMLTextAreaElement>('main textarea')?.focus({ preventScroll: true }),
      450,
    )
  }

  return (
    <PageLayout
      homeHref="/"
      onCreateNote={screen.name === 'compose' ? focusTheComposer : startANote}
      ctaAfterScroll={screen.name === 'compose'}
      ctaOnMobileAlways={screen.name !== 'compose'}
      ctaVariant={screen.name === 'compose' ? 'primary' : 'secondary'}
      menuOpen={menuOpen}
      onMenuToggle={() => setMenuOpen((open) => !open)}
    >
      {screen.name === 'compose' && (
        <ComposeScreen
          value={draft}
          onChange={setDraft}
          onSubmit={submit}
          status={composeStatus}
          error={composeError}
          maxLength={MAX_NOTE_CHARACTERS}
        />
      )}

      {screen.name === 'link' && (
        <LinkReadyScreen
          url={screen.url}
          onCopy={() => void copyLink(screen.url)}
          copied={copied}
          copyError={copyError}
        />
      )}

      {screen.name === 'reveal' && (
        <RevealScreen onReveal={() => void reveal(screen.link)} status={revealStatus} error={revealError} />
      )}

      {screen.name === 'opened' && (
        <NoteOpenedScreen
          text={screen.text}
          textCopied={noteCopied}
          onCopyText={() => {
            navigator.clipboard.writeText(screen.text).then(
              () => setNoteCopied(true),
              () => setNoteCopied(false),
            )
          }}
          onReply={startANote}
          onReplyWithText={(text) => {
            setDraft(text)
            go({ name: 'compose' })
          }}
        />
      )}

      {screen.name === 'notfound' && <NoteNotFoundScreen onCreate={startANote} />}
    </PageLayout>
  )
}
