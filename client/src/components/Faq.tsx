import { createContext, useContext, type ReactNode } from 'react'
import { FaqItem } from './FaqItem'
import { Monkey } from './Monkey'
import { SectionHeading } from './SectionHeading'
import { Logo } from './Logo'

export type FaqEntry = { question: string; answer: ReactNode }

/**
 * SSS bölümünün görünümü.
 * figma: tek sütun akordiyon. split / cards: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type FaqStyle = 'figma' | 'split' | 'cards'
export const FaqStyleContext = createContext<FaqStyle>('split')

type FaqProps = {
  /** Sorular Figma'dan; cevap metinleri tasarımda yok, çağıran verir. */
  items: ReadonlyArray<FaqEntry>
}

/** Figma: faq — maymun + "Sıkça Sorulan Sorular" + akordiyon listesi. */
export function Faq(props: FaqProps) {
  const style = useContext(FaqStyleContext)
  const section = 'scroll-mt-20 bg-page px-4 py-14 md:px-20 md:py-24 dark:bg-night'

  if (style === 'split') {
    // A · İki sütun: solda sabit başlık ve kısa giriş, sağda akordiyon
    return (
      <section id="sss" aria-labelledby="sss-baslik" className={section}>
        <div className="mx-auto grid max-w-[1120px] gap-8 md:max-w-[720px] md:gap-10 lg:max-w-[1120px] lg:grid-cols-[minmax(0,360px)_minmax(0,1fr)] lg:gap-16">
          <div className="flex flex-col items-center gap-3 text-center lg:sticky lg:top-28 lg:items-start lg:self-start lg:text-left">
            <Monkey pose="eyes" className="w-12 md:w-16" />
            <Logo variant="wordmark" className="h-3.5 text-ink md:h-4 dark:text-sand" />
            <h2 id="sss-baslik" className="font-display text-2xl leading-[1.3] font-semibold tracking-[-0.02em] md:leading-[1.25] text-ink md:text-[40px] dark:text-sand">
              Sıkça Sorulan Sorular
            </h2>
            <p className="text-[15px] leading-6 text-fur md:text-base md:leading-7 dark:text-sand-muted">
              Merak ettiklerinin cevabı burada. Kısaca: mesajın tarayıcında şifrelenir, bir kez açılır ve yok olur.
            </p>
          </div>
          <div className="flex flex-col gap-3 md:gap-4">
            {props.items.map((item) => (
              <FaqItem key={item.question} question={item.question} answer={item.answer} group="sss" />
            ))}
          </div>
        </div>
      </section>
    )
  }

  if (style === 'cards') {
    // B · Kartlar: soru sayısı az olduğu için hepsi açık; tıklamadan okunur
    return (
      <section id="sss" aria-labelledby="sss-baslik" className={section}>
        <SectionHeading id="sss-baslik" title="Sıkça Sorulan Sorular" withMonkey />
        <dl className="mx-auto mt-10 grid max-w-[1062px] gap-4 md:mt-[60px] md:grid-cols-2 md:gap-6">
          {props.items.map((item, i) => (
            <div
              key={item.question}
              className="flex flex-col gap-2 rounded-2xl border border-accent/40 bg-[color-mix(in_srgb,var(--color-accent)_5%,var(--color-page))] p-5 shadow-step md:rounded-card md:p-7 dark:border-accent/20 dark:bg-night-card dark:shadow-none"
            >
              <span aria-hidden="true" className="font-display text-sm font-bold text-accent">{String(i + 1).padStart(2, '0')}</span>
              <dt className="font-display text-lg leading-snug font-semibold text-ink md:text-xl dark:text-sand">{item.question}</dt>
              <dd className="text-sm leading-6 text-ink-strong md:text-base md:leading-7 dark:text-sand-muted">{item.answer}</dd>
            </div>
          ))}
        </dl>
      </section>
    )
  }

  return (
    <section id="sss" aria-labelledby="sss-baslik" className={section}>
      <SectionHeading id="sss-baslik" title="Sıkça Sorulan Sorular" withMonkey />
      <div className="mx-auto mt-10 flex max-w-[1062px] flex-col gap-4 md:mt-[60px]">
        {props.items.map((item) => (
          <FaqItem key={item.question} question={item.question} answer={item.answer} group="sss" />
        ))}
      </div>
    </section>
  )
}
