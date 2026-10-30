import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { Panel } from './Panel'

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
    <p
      role="alert"
      className="max-w-[311px] text-center text-xs font-medium text-danger md:text-sm dark:text-danger-dark"
    >
      {props.error}
    </p>
  )
}

/** Mühürlü zarf illüstrasyonu (dekoratif) */
function Envelope() {
  return (
    <svg viewBox="0 0 220 140" fill="none" aria-hidden="true" className="block w-[180px] md:w-[220px]">
      <rect
        x="6"
        y="10"
        width="208"
        height="124"
        rx="14"
        className="fill-cream stroke-ink dark:fill-night-raised dark:stroke-accent"
        strokeWidth="3"
      />
      <path d="M8 22L110 86L212 22" className="stroke-ink dark:stroke-accent" strokeWidth="3" strokeLinejoin="round" />
      <path
        d="M8 130L86 72M212 130L134 72"
        className="stroke-ink/30 dark:stroke-accent/40"
        strokeWidth="2.5"
        strokeLinecap="round"
      />
      <circle cx="110" cy="86" r="22" className="fill-ink dark:fill-accent" />
      <circle
        cx="110"
        cy="86"
        r="15"
        className="fill-none stroke-accent dark:stroke-ink"
        strokeWidth="2"
        strokeDasharray="3 3"
      />
      <path
        d="M103 86L108 91L118 80"
        className="stroke-accent dark:stroke-ink"
        strokeWidth="3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

/**
 * Figma: Mesaj Aç → note-composer -- "Mühürlü zarf": "sana özel" duygusu, mühürü kırmak = tek seferlik açmak.
 */
export function RevealCard(props: RevealCardProps) {
  const t = useCopy()
  const opening = props.status === 'opening'
  return (
    <Panel tone={props.error ? 'danger' : 'default'}>
      <div className="flex flex-col items-center gap-3 py-4 md:gap-4 md:py-2">
        {/* The envelope opens the note too -- same action as the button. A real
            <button> gives keyboard access for free; the ref guard in App keeps
            a double trigger from firing two burns. */}
        <button
          type="button"
          onClick={props.onReveal}
          disabled={opening}
          aria-label={t.revealCard.breakSeal}
          className="group rounded-3xl outline-none transition-transform duration-150 focus-visible:ring-4 focus-visible:ring-accent enabled:cursor-pointer enabled:hover:scale-[1.03] enabled:active:scale-[0.98] disabled:opacity-60 motion-reduce:transition-none motion-reduce:enabled:hover:scale-100"
        >
          <Envelope />
        </button>
        <Button onClick={props.onReveal} loading={opening} loadingLabel={t.revealCard.breakingSeal}>
          {t.revealCard.breakSeal}
        </Button>
        <ErrorText error={props.error} />
      </div>
    </Panel>
  )
}
