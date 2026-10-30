import type { ReactNode } from 'react'
import { cx } from './cx'

type PanelProps = {
  children: ReactNode
  /** Hata durumunda kenarlık kırmızılaşır (Figma'da yok — türetildi) */
  tone?: 'default' | 'danger'
  /** İç beyaz kutuya ek sınıflar (ör. ortalanmış dar kutu için "mx-auto w-fit") */
  innerClassName?: string
  className?: string
}

/**
 * Figma: note-composer (dış kap) + note-composer-box (iç beyaz kutu).
 * Masaüstü: dış 2px kenar, r32, p6 / iç r24, p24. Mobil: dış r16, p3 / iç r12, p12-8.
 */
export function Panel(props: PanelProps) {
  return (
    <div
      className={cx(
        'w-full rounded-2xl border p-[3px] shadow-composer md:rounded-card md:border-2 md:p-1.5',
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
