import { useCopy } from '../lib/i18n'
import { Logo } from './Logo'
import { Monkey } from './Monkey'

/**
 * Figma: faq -- iki sütun: solda sabit başlık ve kısa giriş, sağda soru-cevap listesi.
 * Cevaplar açık (akordiyon yok): "hiçbir şey saklamayız" duruşuna uygun, dürüstlük
 * cevapları da dahil göz gezdirince okunur.
 */
export function Faq() {
  const t = useCopy()
  const items = t.faq.items
  return (
    <section
      id="sss"
      aria-labelledby="sss-baslik"
      className="scroll-mt-20 bg-page px-4 py-14 md:px-20 md:py-24 dark:bg-night"
    >
      <div className="mx-auto grid max-w-[1120px] gap-8 md:max-w-[720px] md:gap-10 lg:max-w-[1120px] lg:grid-cols-[minmax(0,360px)_minmax(0,1fr)] lg:gap-16">
        <div className="flex flex-col items-center gap-3 text-center lg:sticky lg:top-28 lg:items-start lg:self-start lg:text-left">
          <Monkey pose="eyes" className="w-12 md:w-16" />
          <Logo variant="wordmark" className="h-3.5 text-ink md:h-4 dark:text-sand" />
          <h2
            id="sss-baslik"
            className="font-display text-2xl leading-[1.3] font-semibold tracking-[-0.02em] md:leading-[1.25] text-ink md:text-[40px] dark:text-sand"
          >
            {t.faq.heading}
          </h2>
          <p className="text-[15px] leading-6 text-fur md:text-base md:leading-7 dark:text-sand-muted">{t.faq.intro}</p>
        </div>
        <dl className="flex flex-col gap-3 md:gap-4">
          {items.map((item, i) => (
            <div
              key={item.question}
              className="rounded-2xl border border-accent/40 bg-[color-mix(in_srgb,var(--color-accent)_5%,var(--color-page))] p-5 shadow-step md:rounded-card md:p-6 dark:border-accent/20 dark:bg-night-card dark:shadow-none"
            >
              <dt className="flex items-baseline gap-3">
                <span aria-hidden="true" className="font-display text-sm font-bold text-accent">
                  {String(i + 1).padStart(2, '0')}
                </span>
                <span className="font-display text-lg leading-snug font-semibold text-ink md:text-xl dark:text-sand">
                  {item.question}
                </span>
              </dt>
              <dd className="mt-2 pl-8 text-sm leading-6 text-ink-strong md:text-base md:leading-7 dark:text-sand-muted">
                {item.answer}
              </dd>
            </div>
          ))}
        </dl>
      </div>
    </section>
  )
}
