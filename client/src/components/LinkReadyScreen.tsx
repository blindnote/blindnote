import { useCopy } from '../lib/i18n'
import { Hero } from './Hero'
import { LinkCard } from './LinkCard'

type LinkReadyScreenProps = {
  url: string
  onCopy: () => void
  copied: boolean
  copyError?: string
}

/**
 * Ekran (b) Link hazır — Figma: Bağlantı Oluşturuldu.
 * Figma'daki mobil "Yeni Gizli Mesaj Oluştur" butonu kaldırıldı: yeni mesaj, mobilde de header'daki
 * kompakt create butonundan (PageLayout → ctaOnMobileAlways) açılır.
 */
export function LinkReadyScreen(props: LinkReadyScreenProps) {
  const t = useCopy()
  return (
    <Hero
      title={t.linkReady.title}
      subtitle={t.linkReady.subtitle}
      monkey={{ pose: 'eyes', placement: 'peek' }}
      card={<LinkCard url={props.url} onCopy={props.onCopy} copied={props.copied} copyError={props.copyError} />}
    />
  )
}
