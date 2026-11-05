import { useEffect, useRef } from 'react'
import { useCopy } from '../lib/i18n'
import { prefersReducedMotion } from '../lib/motion'
import { Hero } from './Hero'
import { Panel } from './Panel'
import { Seal } from './Seal'

// Mühürleme animasyonunun toplam süresi (yarıların birleşmesi + baskı + tik).
// Seal.tsx'teki gecikmelerle (0.3-0.36s) uyumlu; bu süre dolunca bağlantı gösterilir.
const SEAL_MS = 900

type SealingScreenProps = {
  /** Not zaten oluşturuldu; mühürleme anı bitince bağlantı ekranına geç. */
  onDone: () => void
}

/**
 * Ekran (a→b arası) Mühürleme -- Figma'da yok (türetildi). Not oluşturulduktan
 * sonra, bağlantı gösterilmeden önceki kısa an: balmumu mühür basılır. Yalnızca
 * görseldir; not çoktan sunucuda, bu bekleme güvenliği etkilemez. Hareketi azalt
 * tercihinde an atlanır ve bağlantı anında gelir.
 */
export function SealingScreen(props: SealingScreenProps) {
  const t = useCopy()
  // onDone her render'da yeni kapanış olabilir; süreci bir kez kurup ref'ten çağır.
  const onDone = useRef(props.onDone)
  onDone.current = props.onDone

  // The timer is armed once on mount; onDone is read through the ref above, so a
  // changed callback identity never restarts the seal.
  useEffect(() => {
    const ms = prefersReducedMotion() ? 0 : SEAL_MS
    const timer = window.setTimeout(() => onDone.current(), ms)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <Hero
      title={t.sealing.title}
      subtitle={t.sealing.subtitle}
      card={
        <Panel>
          <div className="flex justify-center py-5 md:py-7">
            <Seal state="sealing" className="w-[180px] md:w-[220px]" />
          </div>
        </Panel>
      }
    />
  )
}
