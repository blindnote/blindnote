import { useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { CreateNoteCta, ReplyCtaContext } from './CreateNoteCta'
import { Hero } from './Hero'
import { NoteViewer } from './NoteViewer'

type NoteOpenedScreenProps = {
  text: string
  /** Copy the note text (shown only in the alternative note views) */
  onCopyText?: () => void
  textCopied?: boolean
  /** "Send Reply" → the compose screen on the home page */
  onReply: () => void
  /** Start writing the reply on this page and carry it to the compose screen (only in the "Reply now" view) */
  onReplyWithText?: (text: string) => void
}

/** Screen (c-2) Read, content — Figma: Message Opened. */
export function NoteOpenedScreen(props: NoteOpenedScreenProps) {
  const t = useCopy()
  const look = useContext(ReplyCtaContext)
  return (
    <Hero
      title={t.opened.title}
      subtitle={t.opened.subtitle}
      monkey={{ pose: 'ears', placement: 'peek' }}
      space="compact"
      card={<NoteViewer text={props.text} onCopy={props.onCopyText} copied={props.textCopied} />}
      footer={
        <CreateNoteCta
          heading={t.openedCta.heading}
          onCreate={props.onReply}
          look={look}
          onCreateWithText={props.onReplyWithText}
          bannerTitle={t.openedCta.bannerTitle}
          bannerText={t.openedCta.bannerText}
          bannerMonkey="eyes"
          buttonLabel={look === 'banner' ? t.openedCta.button : undefined}
        />
      }
    />
  )
}
