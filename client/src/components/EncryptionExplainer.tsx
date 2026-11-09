import { useCopy } from '../lib/i18n'
import { LinkAnatomy } from './LinkAnatomy'
import { SectionHeading } from './SectionHeading'

/** Figma: encryption-explainer — cream background, explanation + link anatomy. Static content. */
export function EncryptionExplainer() {
  const t = useCopy()
  return (
    <section
      id="sifreleme"
      aria-labelledby="sifreleme-baslik"
      className="relative scroll-mt-20 bg-cream py-14 md:py-24 dark:bg-night-raised"
    >
      {/* Figma "section-fade": a soft fade to white at the top and bottom edges */}
      <div
        aria-hidden="true"
        className="absolute inset-x-0 top-0 h-[13px] bg-gradient-to-b from-page to-transparent md:h-12 dark:from-night"
      />
      <div
        aria-hidden="true"
        className="absolute inset-x-0 bottom-0 h-[13px] bg-gradient-to-t from-page to-transparent md:h-12 dark:from-night"
      />
      <div className="relative flex flex-col items-center gap-10 md:gap-[60px]">
        <SectionHeading id="sifreleme-baslik" title={t.encryption.title} description={t.encryption.description} />
        <LinkAnatomy />
      </div>
    </section>
  )
}
