import type { ReactNode } from 'react'
import { useCopy } from '../lib/i18n'
import { BackToTop } from './BackToTop'
import { EncryptionExplainer } from './EncryptionExplainer'
import { Faq } from './Faq'
import { HowItWorks } from './HowItWorks'
import { SiteFooter } from './SiteFooter'
import { SiteHeader } from './SiteHeader'
import { footerLinks, headerLinks, menuLinks } from './siteLinks'
import { useActiveSection } from './useActiveSection'
import { useScrolledPast } from './useScrollY'

type PageLayoutProps = {
  /** Ekrana özel hero (ComposeScreen, LinkReadyScreen, …) */
  children: ReactNode
  homeHref: string
  /** Header'daki create butonu. Anasayfa'nın en üstünde verilmez (Figma). */
  onCreateNote?: () => void
  /**
   * Yaz ekranı: header'daki create butonu yalnız mesaj kutusu ekrandan çıkınca görünür
   * (Figma: Anasayfa en üstte yok, Anasayfa_Scroll'da var).
   */
  ctaAfterScroll?: boolean
  /** Mobilde header'daki kompakt create butonu kaydırmadan da görünsün (Yaz dışındaki tüm ekranlar) */
  ctaOnMobileAlways?: boolean
  /** Header create butonunun varyantı: anasayfada dolu (primary), diğerlerinde çerçeveli. */
  ctaVariant?: 'primary' | 'secondary'
  menuOpen: boolean
  onMenuToggle: () => void
}

/** The header's section links point at these; scroll-spy watches them in document order. */
const SECTION_IDS = ['nedir', 'sifreleme', 'sss'] as const

/** Tüm ekranlarda ortak iskelet: header → hero → 3 bilgi bölümü → footer (Figma'daki 5 ekranın hepsi). */
export function PageLayout(props: PageLayoutProps) {
  const t = useCopy()
  const scrolled = useScrolledPast(8)
  const pastHero = useScrolledPast(520)
  const showCta = props.onCreateNote && (!props.ctaAfterScroll || pastHero)
  const activeSection = useActiveSection(SECTION_IDS)
  const activeHref = activeSection ? `#${activeSection}` : '/'
  return (
    <div className="min-h-dvh bg-page dark:bg-night">
      <SiteHeader
        links={headerLinks(t)}
        activeHref={activeHref}
        menuLinks={menuLinks(t)}
        homeHref={props.homeHref}
        cta={showCta && props.onCreateNote ? { label: t.nav.createSecret, onClick: props.onCreateNote } : undefined}
        ctaVariant={props.ctaVariant}
        elevated={scrolled}
        ctaOnMobile={props.ctaOnMobileAlways || pastHero}
        menuOpen={props.menuOpen}
        onMenuToggle={props.onMenuToggle}
      />
      <main>
        {props.children}
        <HowItWorks />
        <EncryptionExplainer />
        <Faq />
      </main>
      <SiteFooter links={footerLinks(t)} homeHref={props.homeHref} />
      <BackToTop />
    </div>
  )
}
