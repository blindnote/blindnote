import { useCopy } from '../lib/i18n'
import { Hero } from './Hero'
import { LinkCard } from './LinkCard'

type LinkReadyScreenProps = {
  url: string
  onCopy: () => void
  copied: boolean
  copyError?: string
}

/**
 * Screen (b) Link ready — Figma: Link Ready.
 * The mobile "new secret message" button from Figma was removed: a new message opens, on mobile too,
 * from the compact create button in the header (PageLayout → ctaOnMobileAlways).
 */
export function LinkReadyScreen(props: LinkReadyScreenProps) {
  const t = useCopy()
  return (
    <Hero
      title={t.linkReady.title}
      subtitle={t.linkReady.subtitle}
      monkey={{ pose: 'eyes', placement: 'peek' }}
      card={<LinkCard url={props.url} onCopy={props.onCopy} copied={props.copied} copyError={props.copyError} />}
    />
  )
}
