import { useContext } from 'react'
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

/**
 * Ekran (c-2) Oku, içerik — Figma: Mesaj Açıldı.
 * Alt başlık masaüstü metninden alındı, yazım hataları düzeltildi
 * ("ihma" → "imha", "görüntülemeyecek" → "görüntülenemeyecek").
 * Mobil frame'de farklı bir metin var: "Sayfadan ayrıldığın anda mesaj kendini yokeder."
 */
export function NoteOpenedScreen(props: NoteOpenedScreenProps) {
  const look = useContext(ReplyCtaContext)
  return (
    <Hero
      title="Bu mesajı bir kez görüntüleyebilirsin!"
      subtitle="Mesajı açtığın anda bağlantı imha edildi. Bu mesaj tekrar görüntülenemeyecek."
      monkey={{ pose: 'ears', placement: 'peek' }}
      space="compact"
      card={<NoteViewer text={props.text} onCopy={props.onCopyText} copied={props.textCopied} />}
      footer={
        <CreateNoteCta
          heading="Cevap Gönder"
          onCreate={props.onReply}
          look={look}
          onCreateWithText={props.onReplyWithText}
          bannerTitle="Cevap vermek ister misin?"
          bannerText="Cevabın da aynı şekilde: tek kullanımlık, iz bırakmadan."
          bannerMonkey="eyes"
          buttonLabel={look === 'banner' ? 'Gizli Cevap Oluştur' : undefined}
        />
      }
    />
  )
}
