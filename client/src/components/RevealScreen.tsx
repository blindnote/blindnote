import { useCopy } from '../lib/i18n'
import { Hero } from './Hero'
import { RevealCard } from './RevealCard'

type RevealScreenProps = {
  onReveal: () => void
  status: 'idle' | 'opening'
  error?: string
}

/** Ekran (c-1) Oku, ilk adım -- Figma: Mesaj Aç. İçerik ancak butona basınca açılır. */
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
