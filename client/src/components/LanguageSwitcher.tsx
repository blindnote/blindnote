import { cx } from './cx'
import { DICTS, SUPPORTED, useLocale } from '../lib/i18n'

type LanguageSwitcherProps = {
  /** light: on the page/header. dark: on the ink footer. */
  tone?: 'light' | 'dark'
  /** md is a touch larger, for the header; sm (default) elsewhere. */
  size?: 'sm' | 'md'
  className?: string
}

/**
 * Switches the interface language. With two languages a small segmented toggle
 * is clearer than a dropdown; the active one is marked with `aria-pressed`.
 * The choice is remembered (see `useLocale`), so it survives a reload.
 */
export function LanguageSwitcher(props: LanguageSwitcherProps) {
  const { locale, setLocale } = useLocale()
  const dark = props.tone === 'dark'
  const big = props.size === 'md'
  return (
    <div
      role="group"
      aria-label={DICTS[locale].header.language}
      className={cx(
        'inline-flex items-center gap-0.5 rounded-full border p-0.5',
        dark ? 'border-white/25' : 'border-accent/50 dark:border-accent/25',
        props.className,
      )}
    >
      {SUPPORTED.map((option) => {
        const active = option === locale
        return (
          <button
            key={option}
            type="button"
            onClick={() => setLocale(option)}
            aria-pressed={active}
            aria-label={DICTS[option].langName}
            className={cx(
              'rounded-full font-semibold uppercase outline-none transition-colors focus-visible:ring-4 focus-visible:ring-accent',
              big ? 'px-3 py-3 text-sm lg:py-1' : 'px-2.5 py-1.5 text-xs',
              active
                ? dark
                  ? 'bg-white text-ink'
                  : 'bg-ink text-white dark:bg-accent dark:text-ink'
                : dark
                  ? 'text-white/70 hover:text-white'
                  : 'text-ink hover:bg-accent/15 dark:text-sand',
            )}
          >
            {option}
          </button>
        )
      })}
    </div>
  )
}
