import { Button } from './Button'
import { cx } from './cx'
import { Icon } from './Icon'
import { Logo } from './Logo'
import type { SiteLink } from './siteLinks'

type SiteHeaderProps = {
  links: ReadonlyArray<SiteLink>
  /** Mobil menü maddeleri (Figma "Menü" frame'i) */
  menuLinks: ReadonlyArray<SiteLink>
  homeHref: string
  /** Sağdaki "Gizli Mesaj Oluştur" butonu. Anasayfa'nın en üstünde Figma'da yok (Anasayfa_Scroll'da var). */
  cta?: { label: string; onClick: () => void }
  /** Sayfa kaydırıldı: başlık altına ince gölge (Figma Anasayfa_Scroll) */
  elevated?: boolean
  /** Mobilde kompakt "Oluştur" butonu görünsün mü (kaydırınca) */
  ctaOnMobile?: boolean
  /** Mobil menü açık mı (yalnız < md) */
  menuOpen: boolean
  onMenuToggle: () => void
}

/** Figma: site-header (1440) ve site-header-mobile + Menü (375). */
export function SiteHeader(props: SiteHeaderProps) {
  return (
    <header className={cx('sticky top-0 z-40 bg-page/95 backdrop-blur transition-shadow dark:bg-night/95', props.elevated && 'shadow-[0_1px_0_rgb(79_32_0/0.08),0_6px_16px_-8px_rgb(79_32_0/0.18)]')}>
      <div className="mx-auto flex h-[72px] max-w-[1280px] items-center justify-between pr-2 pl-4 md:h-[88px] md:px-8">
        <a
          href={props.homeHref}
          aria-label="BlindNote ana sayfa"
          className="rounded-md text-ink outline-none focus-visible:ring-4 focus-visible:ring-accent dark:text-sand"
        >
          <Logo className="h-[26px] md:h-8 lg:h-7 xl:h-8" />
        </a>

        <nav aria-label="Ana menü" className="hidden lg:block">
          <ul className="flex items-center gap-0.5 xl:gap-3">
            {props.links.map((link) => (
              <li key={link.href}>
                <a
                  href={link.href}
                  className="block rounded-lg px-2 py-2.5 font-display text-base leading-6 font-medium whitespace-nowrap text-ink outline-none hover:underline xl:px-3 xl:text-lg hover:decoration-accent hover:decoration-2 hover:underline-offset-4 focus-visible:ring-4 focus-visible:ring-accent dark:text-sand"
                >
                  {link.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>

        <div className="hidden justify-end lg:flex xl:w-[203px]">
          {props.cta && (
            <Button variant="secondary" onClick={props.cta.onClick}>
              {props.cta.label}
            </Button>
          )}
        </div>

        {/* Mobil: kaydırınca menünün yanında kompakt "Oluştur" butonu (Figma'da yok — türetildi) */}
        <div className="ml-auto flex items-center gap-1 lg:hidden">
          {props.cta && props.ctaOnMobile && (
            <button
              type="button"
              onClick={props.cta.onClick}
              className="inline-flex h-9 items-center gap-1.5 rounded-2xl bg-ink px-3.5 text-[13px] font-semibold text-white outline-none hover:bg-ink-hover focus-visible:ring-4 focus-visible:ring-accent dark:bg-accent dark:text-ink"
            >
              <Icon name="message-plus" className="size-[18px]" />
              <span className="leading-none [text-box:trim-both_cap_alphabetic]">Oluştur</span>
            </button>
          )}
        <button
          type="button"
          onClick={props.onMenuToggle}
          aria-expanded={props.menuOpen}
          aria-controls="mobil-menu"
          aria-label={props.menuOpen ? 'Menüyü kapat' : 'Menüyü aç'}
          className="grid size-11 place-items-center rounded-lg text-ink-strong outline-none hover:bg-accent/15 focus-visible:ring-4 focus-visible:ring-accent lg:hidden dark:text-sand"
        >
          <Icon name={props.menuOpen ? 'close' : 'menu'} className={cx(props.menuOpen ? 'size-4' : 'h-4 w-6')} />
        </button>
        </div>
      </div>

      {props.menuOpen && (
        <nav
          id="mobil-menu"
          aria-label="Mobil menü"
          className="absolute inset-x-0 top-full h-[calc(100dvh-72px)] overflow-y-auto bg-page py-6 md:h-[calc(100dvh-88px)] lg:hidden dark:bg-night"
        >
          <ul className="flex flex-col gap-2">
            {props.menuLinks.map((link) => (
              <li key={link.href}>
                <a
                  href={link.href}
                  onClick={props.onMenuToggle}
                  className="block px-4 py-3.5 font-display text-base leading-6 font-medium text-ink-strong outline-none hover:bg-accent/10 focus-visible:bg-accent/15 dark:text-sand"
                >
                  {link.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      )}
    </header>
  )
}
