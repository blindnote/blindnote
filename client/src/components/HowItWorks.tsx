import { createContext, useContext } from 'react'
import { Monkey, type MonkeyPose } from './Monkey'
import { SectionHeading } from './SectionHeading'
import { StepCard } from './StepCard'

/**
 * "Tek Kullanımlık Güvenli Notlar" bölümünün görünümü.
 * figma: 3 adım kartı. timeline / numbers: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type HowStyle = 'figma' | 'timeline' | 'numbers'
export const HowStyleContext = createContext<HowStyle>('figma')

const STEPS: ReadonlyArray<{ pose: MonkeyPose; title: string; text: string }> = [
  { pose: 'eyes', title: 'Yaz', text: 'Paylaşmak istediğiniz hassas bilgiyi mesaj alanına yazın.' },
  { pose: 'ears', title: 'Bağlantı Oluştur', text: 'Sistem, içeriği yalnızca bir kez görüntülenebilen tek kullanımlık güvenli bir link oluşturur.' },
  { pose: 'mouth', title: 'Paylaş', text: 'Alıcı linki bir kez açabilir; açıldığı anda içerik otomatik olarak yok edilir ve bir daha erişilemez.' },
]

/** A · Zaman çizgisi: maymunlar kesikli yol üzerinde durak; odak başlıkta */
function Timeline() {
  return (
    <ol className="relative mx-auto mt-10 grid max-w-[1100px] gap-8 md:mt-14 md:grid-cols-3 md:gap-6">
      <span
        aria-hidden="true"
        className="absolute top-0 bottom-0 left-[31px] w-0.5 bg-[repeating-linear-gradient(180deg,color-mix(in_srgb,var(--color-accent)_70%,transparent)_0_8px,transparent_8px_14px)] md:top-[31px] md:right-[16%] md:bottom-auto md:left-[16%] md:h-0.5 md:w-auto md:bg-[repeating-linear-gradient(90deg,color-mix(in_srgb,var(--color-accent)_70%,transparent)_0_8px,transparent_8px_14px)]"
      />
      {STEPS.map((s, i) => (
        <li key={s.title} className="relative flex gap-4 md:flex-col md:items-center md:gap-5 md:text-center">
          <span className="relative z-10 flex size-16 shrink-0 items-center justify-center rounded-full bg-cream ring-4 ring-page dark:bg-night-raised dark:ring-night">
            <Monkey pose={s.pose} className={s.pose === 'ears' ? 'w-12' : 'w-9'} />
          </span>
          <div className="flex flex-col gap-1.5 md:items-center">
            <span className="text-xs font-semibold tracking-wide text-muted uppercase">{i + 1}. adım</span>
            <h3 className="font-display text-2xl leading-tight font-semibold text-ink md:text-[28px] dark:text-sand">{s.title}</h3>
            <p className="max-w-[320px] text-sm leading-6 text-ink md:text-base dark:text-sand-muted">{s.text}</p>
          </div>
        </li>
      ))}
    </ol>
  )
}

/** B · Büyük numaralar: kartsız, editoryal düzen; numara geri planda, odak başlıkta */
function Numbers() {
  return (
    <ol className="mx-auto mt-10 grid max-w-[1120px] md:mt-14 md:grid-cols-3">
      {STEPS.map((s, i) => (
        <li
          key={s.title}
          className="flex flex-col gap-2 border-t border-accent/50 px-2 py-7 first:border-t-0 md:border-t-0 md:border-l md:px-8 md:py-2 md:first:border-l-0 dark:border-accent/25"
        >
          <span aria-hidden="true" className="font-display text-5xl leading-none font-bold tracking-[-0.04em] text-accent/30 md:text-6xl dark:text-accent/20">
            {String(i + 1).padStart(2, '0')}
          </span>
          <h3 className="mt-1 font-display text-2xl leading-tight font-semibold text-ink md:text-[28px] dark:text-sand">{s.title}</h3>
          <p className="text-sm leading-6 text-ink md:text-base md:leading-7 dark:text-sand-muted">{s.text}</p>
        </li>
      ))}
    </ol>
  )
}

/** Figma: how-it-works — "Tek Kullanımlık Güvenli Notlar" + 3 adım kartı. Statik içerik. */
export function HowItWorks() {
  const style = useContext(HowStyleContext)
  return (
    <section id="nedir" aria-labelledby="nedir-baslik" className="scroll-mt-20 bg-page px-1.5 py-14 md:px-8 md:py-24 dark:bg-night">
      <SectionHeading
        id="nedir-baslik"
        title="Tek Kullanımlık Güvenli Notlar"
        description="Yalnızca bir kez görüntülenebilen hassas bilgileri güvenle paylaşmanın en doğru yolu."
      />
      {style === 'timeline' ? (
        <div className="px-4 md:px-0"><Timeline /></div>
      ) : style === 'numbers' ? (
        <div className="px-4 md:px-0"><Numbers /></div>
      ) : (
        <ol className="mx-auto mt-10 flex max-w-[1216px] flex-col items-center gap-10 md:mt-10 lg:flex-row lg:items-stretch lg:gap-6">
          {STEPS.map((s) => (
            <StepCard key={s.title} pose={s.pose} title={s.title} text={s.text} />
          ))}
        </ol>
      )}
    </section>
  )
}
