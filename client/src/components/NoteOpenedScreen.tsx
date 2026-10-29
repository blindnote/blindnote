import { useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { CreateNoteCta, ReplyCtaContext } from './CreateNoteCta'
import { Hero } from './Hero'
import { NoteViewer } from './NoteViewer'

type NoteOpenedScreenProps = {
  text: string
  /** Not metnini kopyala (yalnız alternatif not görünümlerinde gösterilir) */
  onCopyText?: () => void
  textCopied?: boolean
  /** "Cevap Gönder" → Anasayfa'daki yazma ekranı */
  onReply: () => void
  /** Cevabı bu sayfada yazmaya başlayıp yazma ekranına taşı (yalnız "Hemen yanıtla" görünümünde) */
  onReplyWithText?: (text: string) => void
}

/** Ekran (c-2) Oku, içerik — Figma: Mesaj Açıldı. */
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
