import { createContext, useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { cx } from './cx'
import { Icon } from './Icon'
import { Panel } from './Panel'

/**
 * Açılan notun görünümü.
 * figma: Figma'daki kutu.
 * paper / sealed: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type NoteStyle = 'figma' | 'paper' | 'sealed'
export const NoteStyleContext = createContext<NoteStyle>('sealed')

type NoteViewerProps = {
  /** Çözülmüş not. Düz metin olarak basılır (HTML yorumlanmaz). */
  text: string
  /** Not metnini kopyala (alternatif görünümlerde gösterilir). Panoya yazma işi çağıranda. */
  onCopy?: () => void
  copied?: boolean
}

const TEXT = 'break-words whitespace-pre-wrap text-ink-strong dark:text-sand'
const SCROLL = 'max-h-[55vh] overflow-y-auto rounded-md outline-none focus-visible:ring-4 focus-visible:ring-accent'

function CopyButton(props: { onCopy?: () => void; copied?: boolean }) {
  const t = useCopy()
  if (!props.onCopy) return null
  return (
    <button
      type="button"
      onClick={props.onCopy}
      className="inline-flex h-10 items-center gap-1.5 rounded-2xl border-[1.5px] border-ink md:rounded-button px-4 text-sm font-semibold text-ink outline-none transition-colors hover:bg-ink/6 focus-visible:ring-4 focus-visible:ring-accent dark:border-accent dark:text-accent dark:hover:bg-accent/10"
    >
      <Icon name={props.copied ? 'check' : 'copy'} className="size-4" />
      <span className="leading-none [text-box:trim-both_cap_alphabetic]">{props.copied ? t.noteViewer.copied : t.noteViewer.copyText}</span>
    </button>
  )
}

/**
 * Figma: Mesaj Açıldı → note-composer. Satır sonları korunur, uzun kelimeler kırılır.
 * Uzun notlarda kutu kendi içinde kayar (tasarımda yok — türetildi).
 */
export function NoteViewer(props: NoteViewerProps) {
  const t = useCopy()
  const style = useContext(NoteStyleContext)

  if (style === 'paper') {
    // A · Kâğıt not: çizgili kâğıt, üstünde bant; not "elden verilmiş" hissi verir.
    return (
      <div className="relative mx-auto w-full max-w-[720px] pt-3">
        <span aria-hidden="true" className="absolute top-1 left-6 z-10 h-6 w-20 -rotate-6 rounded-sm md:left-10 bg-accent/45 dark:bg-accent/30" />
        <div className="-rotate-[0.6deg] rounded-[20px] border border-accent/40 bg-cream shadow-step dark:border-accent/20 dark:bg-night-card">
          <div
            tabIndex={0}
            role="region"
            aria-label={t.noteViewer.secretMessage}
            className={cx(
              SCROLL,
              TEXT,
              'bg-[repeating-linear-gradient(180deg,transparent_0px,transparent_27px,color-mix(in_srgb,var(--color-accent)_35%,transparent)_27px,color-mix(in_srgb,var(--color-accent)_35%,transparent)_28px)] bg-local bg-[position:0_22px] px-6 pt-[22px] pb-3 text-sm leading-7 font-medium md:px-10 md:text-base md:leading-7',
            )}
          >
            {props.text}
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3 border-t border-dashed border-accent/50 px-6 py-3 md:px-10">
            <span className="inline-flex items-center gap-1.5 text-xs font-medium text-fur dark:text-sand-muted">
              <Icon name="eye-off" className="size-4 text-accent" />
              {t.noteViewer.disappears}
            </span>
            <CopyButton onCopy={props.onCopy} copied={props.copied} />
          </div>
        </div>
      </div>
    )
  }

  if (style === 'sealed') {
    // B · Açılmış mühür: üstte durum şeridi, altta not ve sağ altta kopyala.
    return (
      <div className="mx-auto w-full overflow-hidden rounded-2xl border-2 border-ink/10 bg-page shadow-composer md:rounded-card dark:border-night-line dark:bg-night-card">
        <div className="flex items-center justify-between gap-3 bg-ink px-4 py-3 text-white md:px-6 dark:bg-night-raised">
          <span className="inline-flex items-center gap-2 font-display text-sm font-semibold md:text-base">
            <Icon name="unlock" className="size-4 text-accent md:size-5" />
            {t.noteViewer.opened}
          </span>
        </div>
        <div
          tabIndex={0}
          role="region"
          aria-label={t.noteViewer.secretMessage}
          className={cx(SCROLL, TEXT, 'px-4 pt-4 text-[15px] leading-6 font-medium md:px-6 md:pt-6 md:text-base md:leading-7')}
        >
          {props.text}
        </div>
        {props.onCopy && (
          <div className="flex justify-end px-4 pt-4 pb-4 md:px-6 md:pb-6">
            <CopyButton onCopy={props.onCopy} copied={props.copied} />
          </div>
        )}
      </div>
    )
  }

  return (
    <Panel>
      <div
        tabIndex={0}
        role="region"
        aria-label={t.noteViewer.secretMessage}
        className={cx(SCROLL, TEXT, 'text-xs leading-[1.75] font-medium md:text-sm md:leading-7')}
      >
        {props.text}
      </div>
    </Panel>
  )
}
