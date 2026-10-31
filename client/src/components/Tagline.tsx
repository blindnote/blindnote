import { createContext, useContext } from 'react'
import { cx } from './cx'
import { Icon, type IconName } from './Icon'

/**
 * Güven satırının görünümü.
 * figma: tek satır metin, tirelerle ayrılmış (Figma).
 * badges / icons: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type TaglineStyle = 'figma' | 'badges' | 'icons'
export const TaglineStyleContext = createContext<TaglineStyle>('badges')

const ITEMS: ReadonlyArray<{ icon: IconName; text: string }> = [
  { icon: 'eye-off', text: 'İz Bırakmadan' },
  { icon: 'shield-cross', text: 'İsviçre Korumalı' },
  { icon: 'lock', text: 'Uçtan Uca Şifrelenmiş' },
]

type TaglineProps = {
  align?: 'center' | 'right'
  className?: string
}

/** "İz Bırakmadan - İsviçre Korumalı - Uçtan Uca Şifrelenmiş" — Figma'da butonların altında tekrar eden satır. */
export function Tagline(props: TaglineProps) {
  const style = useContext(TaglineStyleContext)
  const justify = props.align === 'right' ? 'justify-center md:justify-end' : 'justify-center'

  if (style === 'badges') {
    return (
      <ul aria-label="Güvenlik özellikleri" className={cx('flex flex-wrap gap-2', justify, props.className)}>
        {ITEMS.map((item) => (
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
      <ul aria-label="Güvenlik özellikleri" className={cx('flex flex-wrap gap-x-3 gap-y-1', justify, props.className)}>
        {ITEMS.map((item) => (
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
      İz Bırakmadan - İsviçre Korumalı - Uçtan Uca Şifrelenmiş
    </p>
  )
}
