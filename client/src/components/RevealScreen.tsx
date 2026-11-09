import { useCopy } from '../lib/i18n'
import { Hero } from './Hero'
import { RevealCard } from './RevealCard'

type RevealScreenProps = {
  onReveal: () => void
  status: 'idle' | 'opening'
  error?: string
}

/** Screen (c-1) Read, first step -- Figma: Open Message. The content opens only on a button press. */
export function RevealScreen(props: RevealScreenProps) {
  const t = useCopy()
  return (
    <Hero
      title={t.reveal.title}
      subtitle={
        <>
          {t.reveal.subtitle}
          <span className="mt-2 block text-sm leading-5 text-fur md:text-base dark:text-sand-muted">
            {t.reveal.envelopeExtra}
          </span>
        </>
      }
      monkey={{ pose: 'eyes', placement: 'peek' }}
      card={<RevealCard onReveal={props.onReveal} status={props.status} error={props.error} />}
    />
  )
}
