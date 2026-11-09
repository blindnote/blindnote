import { useEffect, useRef } from 'react'
import { useCopy } from '../lib/i18n'
import { prefersReducedMotion } from '../lib/motion'
import { Hero } from './Hero'
import { Panel } from './Panel'
import { Seal } from './Seal'

// Total duration of the sealing animation (halves meeting + press + check).
// Matches the delays in Seal.tsx (0.3-0.36s); when it elapses, the link is shown.
const SEAL_MS = 900

type SealingScreenProps = {
  /** The note is already created; move to the link screen once the sealing moment ends. */
  onDone: () => void
}

/**
 * Screen (between a and b) Sealing -- not in Figma (derived). The short moment
 * after a note is created and before the link is shown: the wax seal is pressed.
 * Purely visual; the note is already on the server, so this wait does not affect
 * security. Under reduced motion the moment is skipped and the link appears at once.
 */
export function SealingScreen(props: SealingScreenProps) {
  const t = useCopy()
  // onDone may be a fresh closure on every render; arm the timer once and call it through the ref.
  const onDone = useRef(props.onDone)
  onDone.current = props.onDone

  // The timer is armed once on mount; onDone is read through the ref above, so a
  // changed callback identity never restarts the seal.
  useEffect(() => {
    const ms = prefersReducedMotion() ? 0 : SEAL_MS
    const timer = window.setTimeout(() => onDone.current(), ms)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <Hero
      title={t.sealing.title}
      subtitle={t.sealing.subtitle}
      card={
        <Panel>
          <div className="flex justify-center py-5 md:py-7">
            <Seal state="sealing" className="w-[180px] md:w-[220px]" />
          </div>
        </Panel>
      }
    />
  )
}
