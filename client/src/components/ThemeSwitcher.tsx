import { useCopy } from '../lib/i18n'
import { THEMES, type Theme, useTheme } from '../lib/theme'
import { cx } from './cx'

type ThemeSwitcherProps = {
  /** light: on the page. dark: on the ink footer. */
  tone?: 'light' | 'dark'
  /** md is a touch larger; sm (default) elsewhere. */
  size?: 'sm' | 'md'
  className?: string
}

/**
 * Chooses the appearance: system, light or dark. A small segmented toggle like
 * the language switcher, with the active choice marked by `aria-pressed`; the
 * choice is remembered (see `useTheme`) and applied before first paint next
 * time by the bootstrap in `index.html`.
 */
export function ThemeSwitcher(props: ThemeSwitcherProps) {
  const t = useCopy()
  const { theme, setTheme } = useTheme()
  const dark = props.tone === 'dark'
  const big = props.size === 'md'
  const label: Record<Theme, string> = {
    system: t.theme.system,
    light: t.theme.light,
    dark: t.theme.dark,
  }
  return (
    <div
      role="group"
      aria-label={t.theme.label}
      className={cx(
        'inline-flex items-center gap-0.5 rounded-full border p-0.5',
        dark ? 'border-white/25' : 'border-accent/50 dark:border-accent/25',
        props.className,
      )}
    >
      {THEMES.map((option) => {
        const active = option === theme
        return (
          <button
            key={option}
            type="button"
            onClick={() => setTheme(option)}
            aria-pressed={active}
            className={cx(
              'rounded-full font-semibold outline-none transition-colors focus-visible:ring-4 focus-visible:ring-accent',
              big ? 'px-3 py-1 text-sm' : 'px-2.5 py-0.5 text-xs',
              active
                ? dark
                  ? 'bg-white text-ink'
                  : 'bg-ink text-white dark:bg-accent dark:text-ink'
                : dark
                  ? 'text-white/70 hover:text-white'
                  : 'text-ink hover:bg-accent/15 dark:text-sand',
            )}
          >
            {label[option]}
          </button>
        )
      })}
    </div>
  )
}
