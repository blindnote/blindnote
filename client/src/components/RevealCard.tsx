import { createContext, useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { Icon } from './Icon'
import { Panel } from './Panel'

/**
 * "Mesaj Aç" kartının görünümü.
 * figma: Figma'daki dar kutu.
 * envelope / frosted: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type RevealStyle = 'figma' | 'envelope' | 'frosted'
export const RevealStyleContext = createContext<RevealStyle>('envelope')

type RevealCardProps = {
  /**
   * "Mesajı Görüntüle" tıklandı. Güvenlik gereği içerik tıklamadan açılmaz:
   * sohbet uygulamalarının link önizlemesi notu yakmasın.
   */
  onReveal: () => void
  /** opening: not çekiliyor + çözülüyor */
  status: 'idle' | 'opening'
  error?: string
}

function ErrorText(props: { error?: string }) {
  if (!props.error) return null
  return (
    <p role="alert" className="max-w-[311px] text-center text-xs font-medium text-danger md:text-sm dark:text-danger-dark">
      {props.error}
    </p>
  )
}

/** Mühürlü zarf illüstrasyonu (dekoratif) */
function Envelope() {
  return (
    <svg viewBox="0 0 220 140" fill="none" aria-hidden="true" className="block w-[180px] md:w-[220px]">
      <rect x="6" y="10" width="208" height="124" rx="14" className="fill-cream stroke-ink dark:fill-night-raised dark:stroke-accent" strokeWidth="3" />
      <path d="M8 22L110 86L212 22" className="stroke-ink dark:stroke-accent" strokeWidth="3" strokeLinejoin="round" />
      <path d="M8 130L86 72M212 130L134 72" className="stroke-ink/30 dark:stroke-accent/40" strokeWidth="2.5" strokeLinecap="round" />
      <circle cx="110" cy="86" r="22" className="fill-ink dark:fill-accent" />
      <circle cx="110" cy="86" r="15" className="fill-none stroke-accent dark:stroke-ink" strokeWidth="2" strokeDasharray="3 3" />
      <path d="M103 86L108 91L118 80" className="stroke-accent dark:stroke-ink" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

/** Figma: Mesaj Aç → note-composer (ortada dar beyaz kutu, buton + uyarı). */
export function RevealCard(props: RevealCardProps) {
  const t = useCopy()
  const style = useContext(RevealStyleContext)
  const opening = props.status === 'opening'

  if (style === 'envelope') {
    // A · Mühürlü zarf: "sana özel" duygusunu güçlendirir; mühürü kırmak = tek seferlik açmak
    return (
      <Panel tone={props.error ? 'danger' : 'default'}>
        <div className="flex flex-col items-center gap-3 py-4 md:gap-4 md:py-2">
          <Envelope />
          <Button onClick={props.onReveal} loading={opening} loadingLabel={t.revealCard.breakingSeal}>
            {t.revealCard.breakSeal}
          </Button>
          <ErrorText error={props.error} />
        </div>
      </Panel>
    )
  }

  if (style === 'frosted') {
    // B · Buzlu cam: mesajın orada olduğunu ama henüz okunamadığını gösterir
    return (
      <Panel tone={props.error ? 'danger' : 'default'}>
        <div className="relative overflow-hidden rounded-lg">
          <div aria-hidden="true" className="flex flex-col gap-3 px-2 py-6 blur-[3px] md:px-6 md:py-8">
            {['w-11/12', 'w-9/12', 'w-10/12', 'w-6/12'].map((w) => (
              <span key={w} className={`h-3 rounded-full bg-ink/15 md:h-3.5 dark:bg-sand/15 ${w}`} />
            ))}
          </div>
          <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 bg-page/55 backdrop-blur-[2px] dark:bg-night-card/55">
            <span className="flex size-9 items-center justify-center rounded-full bg-ink text-accent dark:bg-accent dark:text-ink">
              <Icon name="lock" className="size-5" />
            </span>
            <Button onClick={props.onReveal} loading={opening} loadingLabel={t.revealCard.opening}>
              {t.revealCard.view}
            </Button>
            <p className="text-center text-xs font-semibold text-ink md:text-sm dark:text-sand-muted">{t.revealCard.onlyOnceShort}</p>
          </div>
        </div>
        <div className="flex justify-center pt-2">
          <ErrorText error={props.error} />
        </div>
      </Panel>
    )
  }

  return (
    <Panel tone={props.error ? 'danger' : 'default'} innerClassName="mx-auto w-full md:w-fit md:min-w-[359px]">
      <div className="flex flex-col items-center gap-3 py-6 pb-8 md:py-0 md:pb-2">
        <Button onClick={props.onReveal} loading={opening} loadingLabel={t.revealCard.opening}>
          {t.revealCard.view}
        </Button>
        <p className="text-center text-xs font-semibold text-ink md:text-sm dark:text-sand-muted">
          {t.revealCard.onlyOnceLong}
        </p>
        <ErrorText error={props.error} />
      </div>
    </Panel>
  )
}
