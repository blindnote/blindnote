import { createContext, useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { cx } from './cx'
import { Icon, type IconName } from './Icon'

/**
 * Güven satırının görünümü.
 * figma: tek satır metin, tirelerle ayrılmış (Figma).
 * badges / icons: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type TaglineStyle = 'figma' | 'badges' | 'icons'
export const TaglineStyleContext = createContext<TaglineStyle>('badges')

type TaglineProps = {
  align?: 'center' | 'right'
  className?: string
}

/** Güvenlik özellikleri satırı — Figma'da butonların altında tekrar eden satır. */
export function Tagline(props: TaglineProps) {
  const t = useCopy()
  const style = useContext(TaglineStyleContext)
  const justify = props.align === 'right' ? 'justify-center md:justify-end' : 'justify-center'
  const items: ReadonlyArray<{ icon: IconName; text: string }> = [
    { icon: 'eye-off', text: t.tagline.noTrace },
    { icon: 'shield-cross', text: t.tagline.swiss },
    { icon: 'lock', text: t.tagline.e2e },
  ]

  if (style === 'badges') {
    return (
      <ul aria-label={t.tagline.aria} className={cx('flex flex-wrap gap-2', justify, props.className)}>
        {items.map((item) => (
          <li
            key={item.text}
            className="inline-flex items-center gap-1 rounded-full border border-accent/50 bg-cream px-2.5 py-1 text-xs leading-4 font-semibold whitespace-nowrap text-ink dark:border-accent/25 dark:bg-night-raised dark:text-sand"
          >
            <Icon name={item.icon} className="size-3.5 text-fur dark:text-accent" />
            {item.text}
          </li>
        ))}
      </ul>
    )
  }

  if (style === 'icons') {
    return (
      <ul aria-label={t.tagline.aria} className={cx('flex flex-wrap gap-x-3 gap-y-1', justify, props.className)}>
        {items.map((item) => (
          <li key={item.text} className="inline-flex items-center gap-1 text-xs leading-4 font-medium whitespace-nowrap text-fur dark:text-sand-muted">
            <Icon name={item.icon} className="size-3.5 text-accent" />
            {item.text}
          </li>
        ))}
      </ul>
    )
  }

  return (
    <p
      className={cx(
        'text-xs leading-[1.2] font-medium text-ink dark:text-sand-muted',
        props.align === 'right' ? 'text-center md:text-right' : 'text-center',
        props.className,
      )}
    >
      {`${t.tagline.noTrace} - ${t.tagline.swiss} - ${t.tagline.e2e}`}
    </p>
  )
}
