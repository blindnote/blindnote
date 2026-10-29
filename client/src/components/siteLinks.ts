import type { Strings } from '../lib/i18n'

export type SiteLink = { label: string; href: string }

/**
 * Header, menu and footer links. The hrefs are fixed anchors (`#nedir`,
 * `#sifreleme`, `#sss`) shared with the app's fragment routing; only the
 * labels follow the active language, so these are built from the dictionary.
 */
export function headerLinks(t: Strings): ReadonlyArray<SiteLink> {
  return [
    { label: t.nav.shareFreely, href: '/' },
    { label: t.nav.whatIs, href: '#nedir' },
    { label: t.nav.encryption, href: '#sifreleme' },
    { label: t.nav.faq, href: '#sss' },
  ]
}

/** Mobile menu: the first item is the create action rather than "share freely". */
export function menuLinks(t: Strings): ReadonlyArray<SiteLink> {
  return [
    { label: t.nav.createSecret, href: '/' },
    { label: t.nav.whatIs, href: '#nedir' },
    { label: t.nav.encryption, href: '#sifreleme' },
    { label: t.nav.faq, href: '#sss' },
  ]
}

/** Footer: the third item reads "How It Works" (Figma), pointing at the same section. */
export function footerLinks(t: Strings): ReadonlyArray<SiteLink> {
  return [
    { label: t.nav.shareFreely, href: '/' },
    { label: t.nav.whatIs, href: '#nedir' },
    { label: t.nav.howItWorks, href: '#sifreleme' },
    { label: t.nav.faq, href: '#sss' },
  ]
}
