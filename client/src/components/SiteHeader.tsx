import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { cx } from './cx'
import { Icon } from './Icon'
import { LanguageSwitcher } from './LanguageSwitcher'
import { Logo } from './Logo'
import type { SiteLink } from './siteLinks'

type SiteHeaderProps = {
  links: ReadonlyArray<SiteLink>
  /** Aktif bölümün href'i (scroll-spy); eşleşen link vurgulanır. */
  activeHref?: string
  /** Mobil menü maddeleri (Figma "Menü" frame'i) */
  menuLinks: ReadonlyArray<SiteLink>
  homeHref: string
  /** Sağdaki create butonu. Anasayfa'nın en üstünde Figma'da yok (Anasayfa_Scroll'da var). */
  cta?: { label: string; onClick: () => void }
  /** Create butonunun varyantı (anasayfada dolu, diğerlerinde çerçeveli). */
  ctaVariant?: 'primary' | 'secondary'
  /** Sayfa kaydırıldı: başlık altına ince gölge (Figma Anasayfa_Scroll) */
  elevated?: boolean
  /** Mobilde kompakt create butonu görünsün mü (kaydırınca) */
  ctaOnMobile?: boolean
  /** Mobil menü açık mı (yalnız < md) */
  menuOpen: boolean
  onMenuToggle: () => void
}

/** Figma: site-header (1440) ve site-header-mobile + Menü (375). */
export function SiteHeader(props: SiteHeaderProps) {
  const t = useCopy()
  return (
    <header
      className={cx(
        'sticky top-0 z-40 transition-[background-color,box-shadow] duration-200',
        // En üstte şeffaf: gradient header'ın içinden akar, beyaz şerit olmaz.
        // Kaydırınca (veya mobil menü açıkken) frosted zemin + gölge kazanır.
        props.elevated || props.menuOpen ? 'bg-page/95 backdrop-blur dark:bg-night/95' : 'bg-transparent',
        props.elevated && 'shadow-[0_1px_0_rgb(79_32_0/0.08),0_6px_16px_-8px_rgb(79_32_0/0.18)]',
      )}
    >
      <div className="mx-auto flex h-[72px] max-w-[1280px] items-center justify-between pr-2 pl-4 md:h-[88px] md:px-8 lg:grid lg:grid-cols-[1fr_auto_1fr] lg:gap-4">
        <a
          href={props.homeHref}
          aria-label={t.header.home}
          className="rounded-md text-ink outline-none focus-visible:ring-4 focus-visible:ring-accent dark:text-sand"
        >
          <Logo className="h-[26px] md:h-8 lg:h-7 xl:h-8" />
        </a>

        <nav aria-label={t.header.mainMenu} className="hidden lg:block lg:justify-self-center">
          <ul className="flex items-center gap-0.5 xl:gap-3">
            {props.links.map((link) => {
              const isActive = link.href === props.activeHref
              return (
                <li key={link.href}>
                  <a
                    href={link.href}
                    aria-current={isActive ? 'true' : undefined}
                    className={cx(
                      'block rounded-lg px-2 py-2.5 font-display text-base leading-6 font-medium whitespace-nowrap outline-none hover:underline xl:px-3 xl:text-lg hover:decoration-accent hover:decoration-2 hover:underline-offset-4 focus-visible:ring-4 focus-visible:ring-accent',
                      isActive
                        ? 'text-ink font-semibold underline decoration-accent decoration-2 underline-offset-4 dark:text-sand'
                        : 'text-ink dark:text-sand',
                    )}
                  >
                    {link.label}
                  </a>
                </li>
              )
            })}
          </ul>
        </nav>

        <div className="hidden items-center justify-end gap-3 lg:flex lg:justify-self-end xl:w-[260px]">
          {!props.elevated && <LanguageSwitcher tone="light" size="md" />}
          {props.cta && (
            <Button variant={props.ctaVariant ?? 'secondary'} onClick={props.cta.onClick}>
              {props.cta.label}
            </Button>
          )}
        </div>

        {/* Mobil: kaydırınca menünün yanında kompakt create butonu (Figma'da yok — türetildi) */}
        <div className="ml-auto flex items-center gap-1.5 lg:hidden">
          {/* Dil seçici mobilde header'da, hamburgerin solunda. Kaydırınca
              (header yükselince) ve dar ekranda create butonuyla yer paylaşınca
              gizlenir; dil footer'da her zaman erişilebilir. */}
          {!props.elevated && !(props.cta && props.ctaOnMobile) && <LanguageSwitcher tone="light" />}
          {props.cta && props.ctaOnMobile && (
            <button
              type="button"
              onClick={props.cta.onClick}
              className="inline-flex h-9 items-center gap-1.5 rounded-2xl bg-ink px-3.5 text-[13px] font-semibold text-white outline-none hover:bg-ink-hover focus-visible:ring-4 focus-visible:ring-accent dark:bg-accent dark:text-ink"
            >
              <Icon name="message-plus" className="size-[18px]" />
              <span className="leading-none [text-box:trim-both_cap_alphabetic]">{t.header.create}</span>
            </button>
          )}
          <button
            type="button"
            onClick={props.onMenuToggle}
            aria-expanded={props.menuOpen}
            aria-controls="mobil-menu"
            aria-label={props.menuOpen ? t.header.closeMenu : t.header.openMenu}
            className="grid size-11 place-items-center rounded-lg text-ink-strong outline-none hover:bg-accent/15 focus-visible:ring-4 focus-visible:ring-accent lg:hidden dark:text-sand"
          >
            <Icon name={props.menuOpen ? 'close' : 'menu'} className={cx(props.menuOpen ? 'size-4' : 'h-4 w-6')} />
          </button>
        </div>
      </div>

      {props.menuOpen && (
        <nav
          id="mobil-menu"
          aria-label={t.header.mobileMenu}
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
