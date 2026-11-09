import { useContext } from 'react'
import { useCopy } from '../lib/i18n'
import { CreateNoteCta, NotFoundCtaContext } from './CreateNoteCta'
import { Monkey } from './Monkey'

type NoteNotFoundScreenProps = {
  onCreate: () => void
}

/** Screen (d) Burned / not found -- "Poof!": large playful typography, the monkey peeks through the smoke. */
export function NoteNotFoundScreen(props: NoteNotFoundScreenProps) {
  const t = useCopy()
  const look = useContext(NotFoundCtaContext)
  const cta = (
    <CreateNoteCta
      onCreate={props.onCreate}
      look={look}
      bannerTitle={t.notFound.bannerTitle}
      bannerText={t.notFound.bannerText}
      bannerMonkey="eyes"
    />
  )
  return (
    <section className="relative isolate overflow-hidden bg-[radial-gradient(60%_70%_at_50%_45%,color-mix(in_srgb,var(--color-blush)_45%,var(--color-page))_0%,var(--color-page)_70%)] dark:bg-[radial-gradient(60%_70%_at_50%_45%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%)]">
      <div className="mx-auto flex max-w-[880px] flex-col items-center px-4 pt-10 pb-12 text-center md:pt-[120px] md:pb-20">
        <div className="relative flex items-center justify-center">
          <span
            aria-hidden="true"
            className="font-display text-[120px] leading-none font-bold tracking-[-0.04em] text-accent/35 select-none md:text-[220px] dark:text-accent/15"
          >
            {t.notFound.poofTitle}
          </span>
          <span
            aria-hidden="true"
            className="absolute inset-x-6 bottom-2 h-10 rounded-full bg-page/70 blur-xl md:bottom-6 md:h-16 dark:bg-night/70"
          />
          <Monkey pose="mouth" className="absolute bottom-0 w-16 md:w-28" />
        </div>
        <h1 className="mt-4 font-display text-[32px] leading-[1.2] font-bold tracking-[-0.02em] text-ink md:mt-6 md:text-5xl dark:text-sand">
          {t.notFound.poofHeadline}
        </h1>
        <p className="mt-3 max-w-[343px] text-base leading-[1.45] text-pretty text-ink md:mt-4 md:max-w-[600px] md:text-2xl md:leading-[1.35] dark:text-sand-muted">
          {t.notFound.poofBody}
          <span className="mt-2 block text-sm leading-5 text-fur md:text-base dark:text-sand-muted">
            {t.notFound.poofHint}
          </span>
        </p>
        <div className="mt-8 flex w-full flex-col items-center gap-3 md:mt-9">{cta}</div>
      </div>
    </section>
  )
}
