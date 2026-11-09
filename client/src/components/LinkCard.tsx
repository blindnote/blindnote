import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { Icon } from './Icon'
import { Panel } from './Panel'

type LinkCardProps = {
  /** The generated one-time link (the key is after the #, never sent to the server) */
  url: string
  /** The copy button was pressed. Writing to the clipboard is the caller's job. */
  onCopy: () => void
  /** The "copied" feedback. It is recommended to keep it on until the screen is left. */
  copied: boolean
  /** Text to show if writing to the clipboard failed */
  copyError?: string
}

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

/** Splits a note URL into its prefix (`…/#`), id and key -- for display only. */
function splitNoteUrl(url: string): { prefix: string; id: string; key: string } | null {
  const hash = url.indexOf('#')
  if (hash === -1) return null
  const slash = url.indexOf('/', hash + 1)
  if (slash === -1) return null
  const prefix = url.slice(0, hash + 1)
  const id = url.slice(hash + 1, slash)
  const key = url.slice(slash + 1)
  if (id === '' || key === '') return null
  return { prefix, id, key }
}

/**
 * Figma: Link Ready → note-composer. The WHOLE link is shown (no truncation);
 * the key part is highlighted -- that is the truly valuable part. Copy writes the whole URL
 * (props.url) in any case; splitting into segments is purely visual, select-all still gives the full URL.
 */
export function LinkCard(props: LinkCardProps) {
  const t = useCopy()
  const parts = splitNoteUrl(props.url)
  return (
    <div className="mx-auto w-full md:max-w-[640px]">
      <Panel>
        <div className="rounded-xl border border-accent/40 bg-cream p-4 md:rounded-2xl md:p-5 dark:border-accent/20 dark:bg-night-raised">
          <p title={props.url} className="font-mono text-sm leading-7 break-all select-all md:text-base md:leading-8">
            <span className="sr-only">{t.linkCard.yourLink}</span>
            {parts ? (
              <>
                <span className="text-muted">{parts.prefix}</span>
                <span className="text-ink-strong dark:text-sand">{parts.id}</span>
                <span className="text-muted">/</span>
                <span className="rounded bg-accent/25 px-1 font-semibold text-ink-strong box-decoration-clone dark:bg-accent/25 dark:text-sand">
                  {parts.key}
                </span>
              </>
            ) : (
              <span className="text-ink-strong dark:text-sand">{props.url}</span>
            )}
          </p>
        </div>
        <p className="mt-3 text-center text-[13px] text-muted md:text-sm">{t.linkCard.keyHint}</p>
        <div className="mt-4 flex justify-center">
          <Button onClick={props.onCopy}>
            <Icon name={props.copied ? 'check' : 'copy'} className="size-4 md:size-5" />
            {props.copied ? t.linkCard.copied : t.linkCard.copy}
          </Button>
        </div>
      </Panel>
      <Feedback copied={props.copied} copyError={props.copyError} />
    </div>
  )
}
