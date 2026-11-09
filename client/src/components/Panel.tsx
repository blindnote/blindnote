import type { ReactNode } from 'react'
import { cx } from './cx'

type PanelProps = {
  children: ReactNode
  /** On error the border turns red (not in Figma — derived) */
  tone?: 'default' | 'danger'
  /** Extra classes for the inner white box (e.g. "mx-auto w-fit" for a centered narrow box) */
  innerClassName?: string
  className?: string
}

/**
 * Figma: note-composer (outer shell) + note-composer-box (inner white box).
 * Desktop: outer 2px border, r32, p6 / inner r24, p24. Mobile: outer r16, p3 / inner r12, p12-8.
 */
export function Panel(props: PanelProps) {
  return (
    <div
      className={cx(
        'w-full rounded-2xl border bg-page p-[3px] shadow-composer md:rounded-card md:border-2 md:p-1.5 dark:bg-night',
        props.tone === 'danger' ? 'border-danger/40 dark:border-danger-dark/50' : 'border-ink/7 dark:border-night-line',
        props.className,
      )}
    >
      <div
        className={cx('rounded-xl bg-page px-2 py-3 md:rounded-inner md:p-6 dark:bg-night-card', props.innerClassName)}
      >
        {props.children}
      </div>
    </div>
  )
}
