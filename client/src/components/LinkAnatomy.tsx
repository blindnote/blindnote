import { useCopy } from '../lib/i18n'
import { Logo } from './Logo'

/**
 * Link anatomy · "Torn ticket": the large piece is the box number (on the server),
 * the small torn-off piece is the key, held only by the recipient.
 */
export function LinkAnatomy() {
  const t = useCopy()
  const label = 'font-display text-xs font-semibold tracking-[0.08em] uppercase md:text-sm'
  return (
    <figure className="mx-auto flex w-full max-w-[343px] flex-col items-stretch px-0 md:max-w-[480px] lg:max-w-[880px] lg:flex-row lg:items-center">
      <div className="relative flex flex-col gap-4 rounded-t-3xl bg-page p-5 shadow-step md:p-7 lg:h-[200px] lg:w-[560px] lg:justify-between lg:rounded-l-3xl lg:rounded-tr-none lg:px-9 lg:py-7 dark:bg-night-card">
        <div className="flex items-center justify-between">
          <Logo className="h-4 text-ink md:h-5 dark:text-sand" />
          <span className={`${label} text-muted`}>{t.anatomy.oneTime}</span>
        </div>
        <div>
          <p className={`${label} text-muted`}>{t.anatomy.boxNumber}</p>
          <p className="mt-0.5 font-mono text-xl text-ink-strong md:text-[30px] md:leading-[38px] break-all dark:text-sand">
            VDjPkmhBaXm9QsR2tLkZ4w
          </p>
        </div>
        <p className="text-xs text-muted md:text-sm">{t.anatomy.boxNote}</p>
      </div>
      <div
        aria-hidden="true"
        className="h-0.5 w-full bg-[repeating-linear-gradient(90deg,transparent_0_6px,color-mix(in_srgb,var(--color-ink)_35%,transparent)_6px_12px)] lg:h-[168px] lg:w-0.5 lg:bg-[repeating-linear-gradient(180deg,transparent_0_6px,color-mix(in_srgb,var(--color-ink)_35%,transparent)_6px_12px)] dark:opacity-60"
      />
      <div className="mt-3 mx-1 flex rotate-1 flex-col gap-3 rounded-b-3xl bg-ink p-5 text-white shadow-step md:p-7 lg:mt-0 lg:ml-7 lg:h-[200px] lg:w-[300px] lg:translate-y-2.5 lg:rotate-6 lg:justify-between lg:rounded-l-none lg:rounded-r-3xl lg:px-7 lg:py-7 dark:bg-accent dark:text-ink">
        <span className={`${label} text-accent dark:text-ink/70`}>{t.anatomy.keyOnlyYou}</span>
        <p className="font-mono text-lg break-all md:text-2xl md:leading-8">
          D936F4PyUQYhH4tN7vKmBcR8sWxZ0eLpGqUoIaJ5dY2
        </p>
        <span className="text-xs opacity-80">{t.anatomy.neverServer}</span>
      </div>
      <figcaption className="sr-only">{t.anatomy.caption}</figcaption>
    </figure>
  )
}
