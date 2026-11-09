import { useEffect, useState } from 'react'
import { useCopy } from '../lib/i18n'
import { prefersReducedMotion } from '../lib/motion'
import { Button } from './Button'
import { Panel } from './Panel'
import { Seal } from './Seal'

type RevealCardProps = {
  /**
   * "Break the seal" was clicked. For security the content never opens without a
   * click, so that chat apps' link previews cannot burn the note.
   */
  onReveal: () => void
  /** opening: the note is being fetched + decrypted */
  status: 'idle' | 'opening'
  error?: string
}

// We do not start the network request (and the burn) before the break animation
// finishes, or on a fast/local server the screen would change before the moment
// is seen. Under reduced motion we open without waiting.
const BREAK_MS = 620

function ErrorText(props: { error?: string }) {
  if (!props.error) return null
  return (
    <p
      role="alert"
      className="max-w-[311px] text-center text-xs font-medium text-danger md:text-sm dark:text-danger-dark"
    >
      {props.error}
    </p>
  )
}

/**
 * Figma: Open Message → note-composer -- "Sealed envelope": a "just for you"
 * feeling, where breaking the seal = the one-time open. On click the wax seal
 * splits into two halves (Seal state="breaking"); when the animation ends the
 * real open in App begins.
 */
export function RevealCard(props: RevealCardProps) {
  const t = useCopy()
  const [breaking, setBreaking] = useState(false)
  const opening = props.status === 'opening'
  const busy = breaking || opening

  // If the open failed and we stayed on this screen (e.g. a network error), the
  // seal reassembles so the user can try again.
  useEffect(() => {
    if (props.error) setBreaking(false)
  }, [props.error])

  const breakSeal = () => {
    // The ref guard in App also prevents a double trigger; this local guard is to
    // avoid restarting the animation on a second click.
    if (busy) return
    if (prefersReducedMotion()) {
      props.onReveal()
      return
    }
    setBreaking(true)
    window.setTimeout(props.onReveal, BREAK_MS)
  }

  return (
    <Panel tone={props.error ? 'danger' : 'default'}>
      <div className="flex flex-col items-center gap-3 py-4 md:gap-4 md:py-2">
        {/* The envelope opens the note too -- the same action as the button. A real
            <button> gives keyboard access for free; the ref guard in App prevents a
            double burn. Opacity is kept while disabled so it does not fade when breaking. */}
        <button
          type="button"
          onClick={breakSeal}
          disabled={busy}
          aria-label={t.revealCard.breakSeal}
          className="group rounded-3xl outline-none transition-transform duration-150 focus-visible:ring-4 focus-visible:ring-accent enabled:cursor-pointer enabled:hover:scale-[1.03] enabled:active:scale-[0.98] disabled:cursor-default motion-reduce:transition-none motion-reduce:enabled:hover:scale-100"
        >
          <Seal state={breaking ? 'breaking' : 'idle'} className="w-[180px] md:w-[220px]" />
        </button>
        <Button onClick={breakSeal} loading={busy} loadingLabel={t.revealCard.breakingSeal}>
          {t.revealCard.breakSeal}
        </Button>
        <ErrorText error={props.error} />
      </div>
    </Panel>
  )
}
