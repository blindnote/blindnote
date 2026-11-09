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
  /** Screen-specific hero (ComposeScreen, LinkReadyScreen, …) */
  children: ReactNode
  homeHref: string
  /** The create button in the header. Not given at the top of the home page (Figma). */
  onCreateNote?: () => void
  /**
   * Compose screen: the create button in the header appears only once the message box scrolls off screen
   * (Figma: not at the top of Anasayfa, present in Anasayfa_Scroll).
   */
  ctaAfterScroll?: boolean
  /** Show the compact create button in the header on mobile even without scrolling (all screens except Compose) */
  ctaOnMobileAlways?: boolean
  /** The header create button's variant: filled on the home page (primary), outlined elsewhere. */
  ctaVariant?: 'primary' | 'secondary'
  menuOpen: boolean
  onMenuToggle: () => void
}

/** The header's section links point at these; scroll-spy watches them in document order. */
const SECTION_IDS = ['nedir', 'sifreleme', 'sss'] as const

/** The common skeleton for all screens: header → hero → 3 info sections → footer (all 5 Figma screens). */
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
