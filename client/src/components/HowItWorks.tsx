import { useCopy } from '../lib/i18n'
import type { MonkeyPose } from './Monkey'
import { SectionHeading } from './SectionHeading'
import { StepCard } from './StepCard'

/** Step illustrations, paired with the translated steps by position. */
const POSES: readonly MonkeyPose[] = ['eyes', 'ears', 'mouth']

/** Figma: how-it-works -- "Tek Kullanımlık Güvenli Notlar" + 3 adım kartı. Statik içerik. */
export function HowItWorks() {
  const t = useCopy()
  return (
    <section
      id="nedir"
      aria-labelledby="nedir-baslik"
      className="scroll-mt-20 bg-page px-1.5 py-14 md:px-8 md:py-24 dark:bg-night"
    >
      <SectionHeading id="nedir-baslik" title={t.how.title} description={t.how.description} />
      <ol className="mx-auto mt-10 flex max-w-[1216px] flex-col items-center gap-10 md:mt-10 lg:flex-row lg:items-stretch lg:gap-6">
        {t.how.steps.map((s, i) => (
          <StepCard key={s.title} pose={POSES[i] ?? 'eyes'} title={s.title} text={s.text} />
        ))}
      </ol>
    </section>
  )
}
