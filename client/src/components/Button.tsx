import type { ReactNode } from 'react'
import { cx } from './cx'
import { Icon } from './Icon'

type ButtonProps = {
  children: ReactNode
  onClick?: () => void
  /** "submit" if it is the submit button inside a form */
  type?: 'button' | 'submit'
  /** md: 44px (desktop hero). sm: 40px (header). On mobile both are 38px. */
  size?: 'md' | 'sm'
  /**
   * primary: solid brown (the page's main action — one per screen).
   * secondary: outlined (secondary actions like the header's create button). Not in Figma — derived.
   */
  variant?: 'primary' | 'secondary'
  /** Full width on mobile (in the Figma mobile hero the button is 343px). */
  blockOnMobile?: boolean
  disabled?: boolean
  /** Waiting state: shows a spinner, disables clicking, sets aria-busy. */
  loading?: boolean
  /** Text to show while loading, e.g. "Encrypting…" */
  loadingLabel?: string
  /** If a description beyond the text is needed */
  ariaLabel?: string
  className?: string
}

/**
 * Ana buton — Figma "primary-button" / "header-cta".
 * Hover, focus, disabled and loading states are not in Figma; derived from the main brown's tones.
 */
export function Button(props: ButtonProps) {
  const size = props.size ?? 'md'
  const inactive = props.disabled || props.loading
  const secondary = props.variant === 'secondary'
  return (
    <button
      type={props.type ?? 'button'}
      onClick={props.onClick}
      disabled={inactive}
      aria-busy={props.loading || undefined}
      aria-label={props.ariaLabel}
      className={cx(
        'inline-flex shrink-0 items-center justify-center gap-2 font-sans font-semibold whitespace-nowrap',
        'h-11 rounded-2xl px-4 text-sm md:rounded-button md:px-5',
        size === 'md' ? 'md:h-11' : 'md:h-10',
        'transition-colors',
        secondary
          ? 'border-[1.5px] border-ink bg-transparent text-ink hover:bg-ink/6 dark:border-accent dark:text-accent dark:hover:bg-accent/10'
          : 'bg-ink text-white hover:bg-ink-hover dark:bg-accent dark:text-ink dark:hover:bg-accent-hover',
        'outline-none focus-visible:ring-4 focus-visible:ring-accent focus-visible:ring-offset-2 focus-visible:ring-offset-page',
        'dark:focus-visible:ring-accent/60 dark:focus-visible:ring-offset-night',
        'disabled:cursor-not-allowed disabled:opacity-50',
        !secondary && 'disabled:hover:bg-ink dark:disabled:hover:bg-accent',
        props.loading && 'disabled:opacity-80',
        props.blockOnMobile && 'w-full md:w-auto',
        props.className,
      )}
    >
      {props.loading && <Icon name="spinner" className="size-4" />}
      {/* text-box: trims the font's top/bottom spacing; text and icon are optically centered */}
      <span className="inline-flex items-center gap-1.5 leading-none [text-box:trim-both_cap_alphabetic]">
        {props.loading && props.loadingLabel ? props.loadingLabel : props.children}
      </span>
    </button>
  )
}
