import { createContext, useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { cx } from './cx'
import { Icon } from './Icon'
import { Panel } from './Panel'

type LinkCardProps = {
  /** Oluşturulan tek kullanımlık bağlantı (anahtar # sonrasında, sunucuya gitmez) */
  url: string
  /** Kopyala butonuna basıldı. Panoya yazma işi çağıranda. */
  onCopy: () => void
  /** Kopyalandı geri bildirimi. Ekrandan ayrılana kadar açık kalması önerilir. */
  copied: boolean
  /** Panoya yazılamadıysa gösterilecek metin */
  copyError?: string
}

/**
 * Link kartının görünümü.
 * share (varsayılan): link + Kopyala butonu — incelemede seçildi (Figma'da yok).
 * figma: Figma'daki tek satır (karşılaştırma için).
 */
export type LinkStyle = 'share' | 'figma'
export const LinkStyleContext = createContext<LinkStyle>('share')

function Feedback(props: { copied: boolean; copyError?: string }) {
  const t = useCopy()
  return (
    <>
      <p aria-live="polite" className="sr-only">
        {props.copied ? t.linkCard.copiedAnnounce : ''}
      </p>
      {props.copyError && (
        <p role="alert" className="mt-2 text-center text-xs font-medium text-danger md:text-sm dark:text-danger-dark">
          {props.copyError}
        </p>
      )}
    </>
  )
}

/** Figma: Bağlantı Oluşturuldu → note-composer (link satırı + solar:copy-bold). */
export function LinkCard(props: LinkCardProps) {
  const t = useCopy()
  const style = useContext(LinkStyleContext)

  if (style === 'share') {
    // Paylaşım kartı: link kutusu + Kopyala butonu
    return (
      <div className="mx-auto w-full md:max-w-[816px]">
        <Panel>
            <div className="flex items-center gap-2 rounded-xl border border-accent/40 bg-cream p-1.5 pl-3 md:rounded-2xl md:p-2 md:pl-4 dark:border-accent/20 dark:bg-night-raised">
              <p title={props.url} className="min-w-0 flex-1 truncate text-xs font-medium text-ink-strong select-all md:text-sm dark:text-sand">
                <span className="sr-only">{t.linkCard.yourLink}</span>
                {props.url}
              </p>
              <Button size="sm" onClick={props.onCopy}>
                <span className="inline-flex items-center gap-1.5">
                  <Icon name={props.copied ? 'check' : 'copy'} className="size-4" />
                  {props.copied ? t.linkCard.copied : t.linkCard.copy}
                </span>
              </Button>
            </div>
        </Panel>
        <Feedback copied={props.copied} copyError={props.copyError} />
      </div>
    )
  }

  return (
    <div className="mx-auto w-full md:max-w-[816px]">
      <Panel innerClassName="py-3! md:py-6!">
        <div className="flex items-center gap-2.5">
          <p
            title={props.url}
            className="min-w-0 flex-1 truncate font-medium text-xs leading-4 text-ink-strong select-all md:text-sm md:leading-7 dark:text-sand"
          >
            <span className="sr-only">{t.linkCard.yourLink}</span>
            {props.url}
          </p>
          <button
            type="button"
            onClick={props.onCopy}
                        className={cx(
              'inline-flex shrink-0 items-center gap-1.5 rounded-lg px-2 py-1 -my-1 -mr-2 outline-none transition-colors',
              'text-ink hover:bg-accent/20 focus-visible:ring-4 focus-visible:ring-accent dark:text-accent dark:hover:bg-accent/10',
            )}
          >
            <Icon name={props.copied ? 'check' : 'copy'} className="size-4 md:size-5" />
            <span className="text-xs font-semibold md:text-sm">{props.copied ? t.linkCard.copied : t.linkCard.copy}</span>
          </button>
        </div>
      </Panel>
      <Feedback copied={props.copied} copyError={props.copyError} />
    </div>
  )
}
