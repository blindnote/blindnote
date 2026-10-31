import type { ReactNode } from 'react'
import { cx } from './cx'
import { Icon } from './Icon'

type ButtonProps = {
  children: ReactNode
  onClick?: () => void
  /** form içinde gönderim butonuysa "submit" */
  type?: 'button' | 'submit'
  /** md: 44px (masaüstü hero). sm: 40px (header). Mobilde her ikisi de 38px. */
  size?: 'md' | 'sm'
  /**
   * primary: dolu kahve (sayfanın ana eylemi — her ekranda bir tane).
   * secondary: çerçeveli (header'daki "Gizli Mesaj Oluştur" gibi ikincil eylemler). Figma'da yok — türetildi.
   */
  variant?: 'primary' | 'secondary'
  /** Mobilde tam genişlik (Figma mobil hero'da buton 343px). */
  blockOnMobile?: boolean
  disabled?: boolean
  /** Bekleme durumu: spinner gösterir, tıklamayı kapatır, aria-busy verir. */
  loading?: boolean
  /** Loading sırasında gösterilecek metin, ör. "Şifreleniyor…" */
  loadingLabel?: string
  /** Metin dışında açıklama gerekiyorsa */
  ariaLabel?: string
  className?: string
}

/**
 * Ana buton — Figma "primary-button" / "header-cta".
 * Hover, focus, disabled ve loading durumları Figma'da yok; ana kahvenin tonlarından türetildi.
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
      {/* text-box: yazı tipinin üst/alt boşluğunu kırpar; metin ve ikon optik olarak ortalanır */}
      <span className="inline-flex items-center gap-1.5 leading-none [text-box:trim-both_cap_alphabetic]">
        {props.loading && props.loadingLabel ? props.loadingLabel : props.children}
      </span>
    </button>
  )
}
