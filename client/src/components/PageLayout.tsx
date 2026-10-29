import type { ReactNode } from 'react'
import { BackToTop } from './BackToTop'
import { EncryptionExplainer } from './EncryptionExplainer'
import { Faq, type FaqEntry } from './Faq'
import { HowItWorks } from './HowItWorks'
import { SiteFooter } from './SiteFooter'
import { SiteHeader } from './SiteHeader'
import { FOOTER_LINKS, HEADER_LINKS, MENU_LINKS } from './siteLinks'
import { useScrolledPast } from './useScrollY'

type PageLayoutProps = {
  /** Ekrana özel hero (ComposeScreen, LinkReadyScreen, …) */
  children: ReactNode
  homeHref: string
  /** Header'daki "Gizli Mesaj Oluştur". Anasayfa'nın en üstünde verilmez (Figma). */
  onCreateNote?: () => void
  /**
   * Yaz ekranı: header'daki "Gizli Mesaj Oluştur" yalnız mesaj kutusu ekrandan çıkınca görünür
   * (Figma: Anasayfa en üstte yok, Anasayfa_Scroll'da var).
   */
  ctaAfterScroll?: boolean
  /** Mobilde header'daki kompakt "Oluştur" butonu kaydırmadan da görünsün (Yaz dışındaki tüm ekranlar) */
  ctaOnMobileAlways?: boolean
  menuOpen: boolean
  onMenuToggle: () => void
  faqItems: ReadonlyArray<FaqEntry>
}

/** Tüm ekranlarda ortak iskelet: header → hero → 3 bilgi bölümü → footer (Figma'daki 5 ekranın hepsi). */
export function PageLayout(props: PageLayoutProps) {
  const scrolled = useScrolledPast(8)
  const pastHero = useScrolledPast(520)
  const showCta = props.onCreateNote && (!props.ctaAfterScroll || pastHero)
  return (
    <div className="min-h-dvh bg-page dark:bg-night">
      <SiteHeader
        links={HEADER_LINKS}
        menuLinks={MENU_LINKS}
        homeHref={props.homeHref}
        cta={showCta && props.onCreateNote ? { label: 'Gizli Mesaj Oluştur', onClick: props.onCreateNote } : undefined}
        elevated={scrolled}
        ctaOnMobile={props.ctaOnMobileAlways || pastHero}
        menuOpen={props.menuOpen}
        onMenuToggle={props.onMenuToggle}
      />
      <main>
        {props.children}
        <HowItWorks />
        <EncryptionExplainer />
        <Faq items={props.faqItems} />
      </main>
      <SiteFooter links={FOOTER_LINKS} homeHref={props.homeHref} />
      <BackToTop />
    </div>
  )
}
