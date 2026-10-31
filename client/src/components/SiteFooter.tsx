import { Logo } from './Logo'
import type { SiteLink } from './siteLinks'

type SiteFooterProps = {
  links: ReadonlyArray<SiteLink>
  homeHref: string
}

/** Figma: site-footer — koyu kahve zemin, beyaz logo + bağlantılar. */
export function SiteFooter(props: SiteFooterProps) {
  return (
    <footer className="bg-ink px-4 py-10 text-white md:px-10 md:py-12 lg:px-[90px] dark:bg-night-footer">
      <div className="mx-auto flex max-w-[1280px] flex-col items-center gap-6 md:gap-8 lg:items-stretch lg:gap-10">
        <a
          href={props.homeHref}
          aria-label="BlindNote ana sayfa"
          className="w-fit rounded-md outline-none focus-visible:ring-4 focus-visible:ring-accent"
        >
          <Logo className="h-[26px] md:h-8" />
        </a>
        <nav aria-label="Alt menü">
          <ul className="flex flex-col items-center gap-1 md:flex-row md:flex-wrap md:justify-center md:gap-x-4 lg:flex-nowrap lg:justify-between lg:gap-6 xl:gap-12">
            {props.links.map((link) => (
              <li key={link.label}>
                <a
                  href={link.href}
                  className="block rounded-md px-3 py-2.5 font-display text-base leading-[26px] font-semibold whitespace-nowrap outline-none hover:text-accent xl:w-[200px] xl:text-center focus-visible:ring-4 focus-visible:ring-accent md:text-xl md:leading-8"
                >
                  {link.label}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </div>
    </footer>
  )
}
