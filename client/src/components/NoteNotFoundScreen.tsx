import { createContext, useContext } from 'react'
import { CreateNoteCta, NotFoundCtaContext } from './CreateNoteCta'
import { Hero } from './Hero'
import { Monkey } from './Monkey'

/**
 * Bulunamadı ekranının görünümü.
 * figma: Figma'daki sade hali.
 * reasons / poof: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type NotFoundStyle = 'figma' | 'reasons' | 'poof'
export const NotFoundStyleContext = createContext<NotFoundStyle>('poof')

type NoteNotFoundScreenProps = {
  onCreate: () => void
}

const REASONS = [
  { title: 'Mesaj zaten açılmış olabilir', text: 'Tek kullanımlık bağlantılar açıldığı anda kendini yok eder.' },
  { title: 'Bağlantı eksik kopyalanmış olabilir', text: '# işaretinden sonraki kısım dahil, bağlantının tamamını kontrol et.' },
  { title: 'Gönderene haber ver', text: 'Mesajı yeniden oluşturup sana yeni bir bağlantı gönderebilir.' },
] as const

/** Ekran (d) Yandı / bulunamadı — Figma: Mesaj Bulunamadı ("yoketmiş" → "yok etmiş" düzeltildi) */
export function NoteNotFoundScreen(props: NoteNotFoundScreenProps) {
  const style = useContext(NotFoundStyleContext)
  const look = useContext(NotFoundCtaContext)
  const cta = (
    <CreateNoteCta
      onCreate={props.onCreate}
      look={look}
      bannerTitle="Yeni bir mesaj mı paylaşacaksın?"
      bannerText="Saniyeler içinde tek kullanımlık, şifreli bir bağlantı oluştur."
      bannerMonkey="eyes"
    />
  )

  if (style === 'reasons') {
    // A · Ne oldu? — kullanıcıya olası nedenleri ve ne yapabileceğini anlatan kart
    return (
      <Hero
        title="Mesajı Bulamadık!"
        subtitle="Bağlantı kendini yok etmiş olabilir."
        monkey={{ pose: 'mouth', placement: 'peek' }}
        card={
          <div className="mx-auto w-full max-w-[640px] rounded-2xl border border-accent/40 bg-page p-4 text-left shadow-step md:rounded-card md:p-8 dark:border-accent/20 dark:bg-night-card">
            <p className="font-display text-base font-semibold text-ink md:text-xl dark:text-sand">Ne olmuş olabilir?</p>
            <ul className="mt-3 flex flex-col gap-3 md:mt-4 md:gap-4">
              {REASONS.map((r, i) => (
                <li key={r.title} className="flex gap-3">
                  <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-cream font-display text-xs font-bold text-fur md:size-7 md:text-sm dark:bg-night-raised dark:text-accent">
                    {i + 1}
                  </span>
                  <span className="flex flex-col">
                    <span className="text-sm font-semibold text-ink-strong md:text-base dark:text-sand">{r.title}</span>
                    <span className="text-xs leading-5 text-fur md:text-sm dark:text-sand-muted">{r.text}</span>
                  </span>
                </li>
              ))}
            </ul>
          </div>
        }
        footer={cta}
      />
    )
  }

  if (style === 'poof') {
    // B · Puf! — büyük, oyunbaz tipografi; maymun dumanın içinden bakar
    return (
      <section className="relative isolate overflow-hidden bg-[radial-gradient(60%_70%_at_50%_45%,color-mix(in_srgb,var(--color-blush)_45%,var(--color-page))_0%,var(--color-page)_70%)] dark:bg-[radial-gradient(60%_70%_at_50%_45%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%)]">
        <div className="mx-auto flex max-w-[880px] flex-col items-center px-4 pt-10 pb-12 text-center md:pt-[120px] md:pb-20">
          <div className="relative flex items-center justify-center">
            <span
              aria-hidden="true"
              className="font-display text-[120px] leading-none font-bold tracking-[-0.04em] text-accent/35 select-none md:text-[220px] dark:text-accent/15"
            >
              Puf!
            </span>
            <span aria-hidden="true" className="absolute inset-x-6 bottom-2 h-10 rounded-full bg-page/70 blur-xl md:bottom-6 md:h-16 dark:bg-night/70" />
            <Monkey pose="mouth" className="absolute bottom-0 w-16 md:w-28" />
          </div>
          {/*
            Başlık tasarımda "Bu mesaj hiç olmadı." idi. Sunucu, notun hiç var
            olmamasıyla çoktan okunmuş olmasını AYIRT EDEMEZ (ve bilerek
            edemez), üstelik yaygın durum ikincisi. Başlık bu yüzden taraf
            seçmiyor; alt metin iki olasılığı da söylüyor ve beklenen bir
            mesajın yanmış görünmesinin ne anlama geldiğini ekliyor.
          */}
          <h1 className="mt-4 font-display text-[32px] leading-[1.2] font-bold tracking-[-0.02em] text-ink md:mt-6 md:text-5xl dark:text-sand">
            Bu mesaj burada değil.
          </h1>
          <p className="mt-3 max-w-[343px] text-base leading-[1.45] text-pretty text-ink md:mt-4 md:max-w-[600px] md:text-2xl md:leading-[1.35] dark:text-sand-muted">
            Belki çoktan okundu, belki de hiç var olmadı. Tek kullanımlık mesajlar iz bırakmaz.
            <span className="mt-2 block text-sm leading-5 text-fur md:text-base dark:text-sand-muted">
              Beklediğin bir mesajsa, biri senden önce açmış olabilir. Gönderene haber ver.
            </span>
          </p>
          <div className="mt-8 flex w-full flex-col items-center gap-3 md:mt-9">
            {cta}
          </div>
        </div>
      </section>
    )
  }

  return (
    <Hero
      title="Mesajı Bulamadık!"
      subtitle="Bağlantı kendini yok etmiş olabilir."
      monkey={{ pose: 'mouth', placement: 'top' }}
      space="roomy"
      footer={cta}
    />
  )
}
