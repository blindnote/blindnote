import { useCopy } from '../lib/i18n'
import { cx } from './cx'
import { Icon } from './Icon'

type NoteViewerProps = {
  /** Çözülmüş not. Düz metin olarak basılır (HTML yorumlanmaz). */
  text: string
  /** Not metnini kopyala. Panoya yazma işi çağıranda. */
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
 * Figma: Mesaj Açıldı → note-composer -- "Açılmış mühür": üstte durum şeridi, altta not,
 * sağ altta kopyala. Satır sonları korunur, uzun kelimeler kırılır; uzun notlarda kutu kendi içinde kayar.
 */
export function NoteViewer(props: NoteViewerProps) {
  const t = useCopy()
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
