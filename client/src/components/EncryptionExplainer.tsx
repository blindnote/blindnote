import { LinkAnatomy } from './LinkAnatomy'
import { SectionHeading } from './SectionHeading'

/** Figma: encryption-explainer — krem zemin, açıklama + bağlantı anatomisi. Statik içerik. */
export function EncryptionExplainer() {
  return (
    <section
      id="sifreleme"
      aria-labelledby="sifreleme-baslik"
      className="relative scroll-mt-20 bg-cream py-14 md:py-24 dark:bg-night-raised"
    >
      {/* Figma "section-fade": üst ve alt kenarda beyaza yumuşak geçiş */}
      <div aria-hidden="true" className="absolute inset-x-0 top-0 h-[13px] bg-gradient-to-b from-page to-transparent md:h-12 dark:from-night" />
      <div aria-hidden="true" className="absolute inset-x-0 bottom-0 h-[13px] bg-gradient-to-t from-page to-transparent md:h-12 dark:from-night" />
      <div className="relative flex flex-col items-center gap-10 md:gap-[60px]">
        <SectionHeading
          id="sifreleme-baslik"
          title="Uçtan Uca Şifreleme"
          description="Şifreleme anahtarı hiçbir zaman sunucularda saklanmaz; yalnızca oluşturulan bağlantının bir parçası olarak çalışır. Bu bağlantı olmadan — biz dahil — hiç kimse paylaştığınız bilginin şifresini çözemez."
        />
        <LinkAnatomy />
      </div>
    </section>
  )
}
