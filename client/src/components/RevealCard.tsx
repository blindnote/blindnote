import { useEffect, useState } from 'react'
import { useCopy } from '../lib/i18n'
import { prefersReducedMotion } from '../lib/motion'
import { Button } from './Button'
import { Panel } from './Panel'
import { Seal } from './Seal'

type RevealCardProps = {
  /**
   * "Mührü Kır" tıklandı. Güvenlik gereği içerik tıklamadan açılmaz: sohbet
   * uygulamalarının link önizlemesi notu yakmasın.
   */
  onReveal: () => void
  /** opening: not çekiliyor + çözülüyor */
  status: 'idle' | 'opening'
  error?: string
}

// Mührün kırılma animasyonu bitmeden ağ isteğini (ve notu yakmayı) başlatmayız,
// yoksa hızlı/yerel sunucuda an görünmeden ekran değişir. Hareketi azalt
// tercihinde beklemeden açarız.
const BREAK_MS = 620

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

/**
 * Figma: Mesaj Aç → note-composer -- "Mühürlü zarf": "sana özel" duygusu, mührü
 * kırmak = tek seferlik açmak. Tıklayınca balmumu mühür iki yarıya ayrılır
 * (Seal state="breaking"); animasyon bitince App'teki asıl açma başlar.
 */
export function RevealCard(props: RevealCardProps) {
  const t = useCopy()
  const [breaking, setBreaking] = useState(false)
  const opening = props.status === 'opening'
  const busy = breaking || opening

  // Açma başarısız olup bu ekranda kalındıysa (ör. ağ hatası), mühür geri
  // toparlanır ki kullanıcı yeniden deneyebilsin.
  useEffect(() => {
    if (props.error) setBreaking(false)
  }, [props.error])

  const breakSeal = () => {
    // App'teki ref koruması da çift tetiklemeyi engeller; bu yerel koruma ikinci
    // tıkta animasyonu baştan başlatmamak için.
    if (busy) return
    if (prefersReducedMotion()) {
      props.onReveal()
      return
    }
    setBreaking(true)
    window.setTimeout(props.onReveal, BREAK_MS)
  }

  return (
    <Panel tone={props.error ? 'danger' : 'default'}>
      <div className="flex flex-col items-center gap-3 py-4 md:gap-4 md:py-2">
        {/* Zarf da notu açar -- butonla aynı eylem. Gerçek bir <button> klavye
            erişimini bedavaya verir; App'teki ref koruması çift yanmayı önler.
            Kırılırken soluklaşmasın diye disabled'da opaklık korunur. */}
        <button
          type="button"
          onClick={breakSeal}
          disabled={busy}
          aria-label={t.revealCard.breakSeal}
          className="group rounded-3xl outline-none transition-transform duration-150 focus-visible:ring-4 focus-visible:ring-accent enabled:cursor-pointer enabled:hover:scale-[1.03] enabled:active:scale-[0.98] disabled:cursor-default motion-reduce:transition-none motion-reduce:enabled:hover:scale-100"
        >
          <Seal state={breaking ? 'breaking' : 'idle'} className="w-[180px] md:w-[220px]" />
        </button>
        <Button onClick={breakSeal} loading={busy} loadingLabel={t.revealCard.breakingSeal}>
          {t.revealCard.breakSeal}
        </Button>
        <ErrorText error={props.error} />
      </div>
    </Panel>
  )
}
