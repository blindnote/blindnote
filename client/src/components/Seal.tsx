import { useId } from 'react'
import { cx } from './cx'

/**
 * idle: a calm, sealed envelope.
 * sealing: the two wax halves come in from the sides and meet, the seal presses
 *   down once, a ring ripples out, the check appears (after a note is created).
 * breaking: the seal splits into two halves and flings apart, the face wipes
 *   away, a soft light opens from within, the envelope twitches (the read moment).
 */
type SealState = 'idle' | 'sealing' | 'breaking'

/**
 * The sealed-envelope illustration (decorative) -- the card on the read screen
 * and the sealing moment after creation share one drawing: the seal is pressed,
 * then broken.
 *
 * Every animation runs on SVG groups; each moving element carries
 * `transform-box: fill-box` so scaling/rotation is relative to its own box.
 * Under reduced motion `app.css` collapses the durations to ~0 and the calling
 * screens make the transition instant, so these moments are silently skipped.
 */
export function Seal(props: { state: SealState; className?: string }) {
  const id = useId()
  const glowId = `${id}-glow`
  const breaking = props.state === 'breaking'
  const sealing = props.state === 'sealing'

  // On every moving element: transforms are relative to the element's own box and center.
  const box = '[transform-box:fill-box] [transform-origin:center]'

  return (
    <svg viewBox="0 0 220 140" fill="none" aria-hidden="true" className={cx('block', props.className)}>
      <defs>
        <radialGradient id={glowId}>
          <stop offset="0%" style={{ stopColor: 'var(--color-accent)', stopOpacity: 0.9 }} />
          <stop offset="100%" style={{ stopColor: 'var(--color-accent)', stopOpacity: 0 }} />
        </radialGradient>
      </defs>

      {/* Envelope body, flap and inner shadow (calm; twitches slightly while breaking). */}
      <g className={cx(box, breaking && 'animate-bn-lift')}>
        <rect
          x="6"
          y="10"
          width="208"
          height="124"
          rx="14"
          className="fill-cream stroke-ink dark:fill-night-raised dark:stroke-accent"
          strokeWidth="3"
        />
        <path
          d="M8 22L110 86L212 22"
          className="stroke-ink dark:stroke-accent"
          strokeWidth="3"
          strokeLinejoin="round"
        />
        <path
          d="M8 130L86 72M212 130L134 72"
          className="stroke-ink/30 dark:stroke-accent/40"
          strokeWidth="2.5"
          strokeLinecap="round"
        />
      </g>

      {/* Light opening from within (only visible while breaking). */}
      <circle
        cx="110"
        cy="86"
        r="38"
        fill={`url(#${glowId})`}
        className={cx(box, 'opacity-0', breaking && 'animate-bn-glow')}
      />

      {/* Ring ripple that spreads at the moment of pressing, while sealing. */}
      {sealing && (
        <circle
          cx="110"
          cy="86"
          r="22"
          fill="none"
          strokeWidth="2.5"
          style={{ animationDelay: '0.36s' }}
          className={cx(box, 'opacity-0 stroke-ink animate-bn-ring dark:stroke-accent')}
        />
      )}

      {/* Wax seal: two halves (a full circle together) + face (dashed ring + check). */}
      <g className={cx(box, sealing && 'animate-bn-stamp')} style={sealing ? { animationDelay: '0.3s' } : undefined}>
        <path
          d="M110 64 A22 22 0 0 0 110 108 Z"
          className={cx(
            box,
            'fill-ink dark:fill-accent',
            breaking && 'animate-bn-crack-l',
            sealing && 'animate-bn-seal-in-l',
          )}
        />
        <path
          d="M110 64 A22 22 0 0 1 110 108 Z"
          className={cx(
            box,
            'fill-ink dark:fill-accent',
            breaking && 'animate-bn-crack-r',
            sealing && 'animate-bn-seal-in-r',
          )}
        />
        <g
          className={cx(box, breaking && 'animate-bn-seal-off', sealing && 'animate-bn-draw')}
          style={sealing ? { animationDelay: '0.34s' } : undefined}
        >
          <circle
            cx="110"
            cy="86"
            r="15"
            fill="none"
            className="stroke-accent dark:stroke-ink"
            strokeWidth="2"
            strokeDasharray="3 3"
          />
          <path
            d="M103 86L108 91L118 80"
            className="stroke-accent dark:stroke-ink"
            strokeWidth="3"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </g>
      </g>
    </svg>
  )
}
