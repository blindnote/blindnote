import { Hero } from './Hero'
import { useContext } from 'react'
import { RevealCard, RevealStyleContext } from './RevealCard'

type RevealScreenProps = {
  onReveal: () => void
  status: 'idle' | 'opening'
  error?: string
}

/** Ekran (c-1) Oku, ilk adım — Figma: Mesaj Aç. İçerik ancak butona basınca açılır. */
export function RevealScreen(props: RevealScreenProps) {
  const style = useContext(RevealStyleContext)
  return (
    <Hero
      title="Tak Tak"
      subtitle={
        style === 'envelope' ? (
          <>
            Sana özel gizli bir mesaj var!
            <span className="mt-2 block text-sm leading-5 text-fur md:text-base dark:text-sand-muted">
              Mühür bir kez kırılır. Açtığın anda mesaj sunucudan silinir.
            </span>
          </>
        ) : (
          'Sana özel gizli bir mesaj var!'
        )
      }
      monkey={{ pose: 'eyes', placement: 'peek' }}
      card={<RevealCard onReveal={props.onReveal} status={props.status} error={props.error} />}
    />
  )
}
