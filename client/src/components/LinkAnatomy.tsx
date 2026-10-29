import { createContext, useContext } from 'react'
import { Logo } from './Logo'

/** Figma: link-anatomy — örnek bağlantının parçaları (Domain / Gizli ID / Şifreleme Anahtarı). */
const LABELS = ['Domain', 'Gizli ID', 'Şifreleme Anahtarı'] as const

/**
 * Link anatomisinin görünümü.
 * figma: Figma'daki etiketli URL kutusu.
 * ticket: "Yırtılan bilet" — tasarım tuvalindeki Fikir 4 (Figma'da yok).
 */
export type AnatomyStyle = 'figma' | 'ticket'
export const AnatomyStyleContext = createContext<AnatomyStyle>('ticket')

const CAPTION =
  'Örnek bağlantı: alan adı, gizli kimlik ve şifreleme anahtarından oluşur. Şifreleme anahtarı sunucuya hiçbir zaman gönderilmez.'

/** Fikir 4 · Yırtılan bilet: büyük parça kutu numarası, koparılan küçük parça yalnızca alıcıdaki anahtar. */
function Ticket() {
  const label = 'font-display text-xs font-semibold tracking-[0.08em] uppercase md:text-sm'
  return (
    <figure className="mx-auto flex w-full max-w-[343px] flex-col items-stretch px-0 md:max-w-[480px] lg:max-w-[880px] lg:flex-row lg:items-center">
      <div className="relative flex flex-col gap-4 rounded-t-3xl bg-page p-5 shadow-step md:p-7 lg:h-[200px] lg:w-[560px] lg:justify-between lg:rounded-l-3xl lg:rounded-tr-none lg:px-9 lg:py-7 dark:bg-night-card">
        <div className="flex items-center justify-between">
          <Logo className="h-4 text-ink md:h-5 dark:text-sand" />
          <span className={`${label} text-muted`}>Tek kullanımlık</span>
        </div>
        <div>
          <p className={`${label} text-muted`}>Kutu numarası · sunucuda</p>
          <p className="mt-0.5 font-mono text-xl text-ink-strong md:text-[30px] md:leading-[38px] dark:text-sand">VDjPkmhBaXm9</p>
        </div>
        <p className="font-mono text-xs text-muted md:text-sm">blindnote.com/l#…</p>
      </div>
      <div
        aria-hidden="true"
        className="h-0.5 w-full bg-[repeating-linear-gradient(90deg,transparent_0_6px,color-mix(in_srgb,var(--color-ink)_35%,transparent)_6px_12px)] lg:h-[168px] lg:w-0.5 lg:bg-[repeating-linear-gradient(180deg,transparent_0_6px,color-mix(in_srgb,var(--color-ink)_35%,transparent)_6px_12px)] dark:opacity-60"
      />
      <div className="mt-3 mx-1 flex rotate-1 flex-col gap-3 rounded-b-3xl bg-ink p-5 text-white shadow-step md:p-7 lg:mt-0 lg:ml-7 lg:h-[200px] lg:w-[300px] lg:translate-y-2.5 lg:rotate-6 lg:justify-between lg:rounded-l-none lg:rounded-r-3xl lg:px-7 lg:py-7 dark:bg-accent dark:text-ink">
        <span className={`${label} text-accent dark:text-ink/70`}>Anahtar · sadece sende</span>
        <p className="font-mono text-lg break-all md:text-2xl md:leading-8">D936F4PyUQYhH4</p>
        <span className="text-xs opacity-80">Bu parça hiçbir zaman sunucuya gitmez.</span>
      </div>
      <figcaption className="sr-only">{CAPTION}</figcaption>
    </figure>
  )
}

export function LinkAnatomy() {
  const style = useContext(AnatomyStyleContext)
  if (style === 'ticket') return <Ticket />
  return (
    <figure className="mx-auto flex w-full max-w-[343px] flex-col gap-2 md:max-w-[734px] md:gap-4">
      <div aria-hidden="true" className="grid grid-cols-3 gap-3 md:gap-6">
        {LABELS.map((label) => (
          <span
            key={label}
            className="relative h-[27px] text-center font-display text-[11px] leading-[18px] font-semibold whitespace-nowrap text-ink md:h-[53px] md:pt-3 md:text-lg md:leading-[29px] dark:text-sand"
          >
            {label}
            <span className="absolute top-[17px] left-1/2 h-2.5 w-px bg-muted md:top-[38px] md:h-5" />
          </span>
        ))}
      </div>
      <p className="truncate rounded-xl border border-accent/40 bg-page p-3 text-center text-[11px] leading-4 text-ink-strong shadow-link md:rounded-button md:px-6 md:text-left md:text-2xl md:leading-7 dark:border-accent/20 dark:bg-night-card dark:text-sand">
        <span className="md:hidden">https://blindnote.com/l#VDjPkmhBa/D936F4PyUQY</span>
        <span className="hidden md:inline">https://blindnote.com/l#VDjPkmhBaXm9/D936F4PyUQYhH4</span>
      </p>
      <figcaption className="sr-only">{CAPTION}</figcaption>
    </figure>
  )
}
