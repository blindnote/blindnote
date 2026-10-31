import { cx } from './cx'
import { Icon } from './Icon'
import { useScrolledPast } from './useScrollY'

/** Yukarı çık butonu (Figma'da yok — türetildi). Bir ekran boyu kaydırınca sağ altta belirir. */
export function BackToTop() {
  const visible = useScrolledPast(600)
  return (
    <button
      type="button"
      aria-label="Sayfanın başına dön"
      tabIndex={visible ? 0 : -1}
      aria-hidden={!visible || undefined}
      onClick={() => {
        const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches
        window.scrollTo({ top: 0, behavior: reduce ? 'auto' : 'smooth' })
      }}
      className={cx(
        'fixed right-4 bottom-[calc(1rem+env(safe-area-inset-bottom))] z-40 grid size-12 place-items-center rounded-full border border-ink/10 bg-page text-ink shadow-composer outline-none transition-all duration-200 hover:bg-cream focus-visible:ring-4 focus-visible:ring-accent md:right-8 md:bottom-8 md:size-14 dark:border-night-line dark:bg-night-card dark:text-sand dark:hover:bg-night-raised',
        visible ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-3 opacity-0',
      )}
    >
      <Icon name="chevron-down" className="size-6 rotate-180" />
    </button>
  )
}
