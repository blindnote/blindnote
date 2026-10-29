import type { ReactNode } from 'react'
import { Logo } from './Logo'
import { Monkey } from './Monkey'

type SectionHeadingProps = {
  title: string
  description?: ReactNode
  /** SSS başlığının üstünde maymun var */
  withMonkey?: boolean
  /** h2 için id (aria-labelledby) */
  id?: string
}

/** Figma: "Heading and supporting text" — BlindNote yazısı (üst başlık) + h2 + açıklama. */
export function SectionHeading(props: SectionHeadingProps) {
  return (
    <div className="mx-auto flex max-w-[768px] flex-col items-center gap-3 px-3 text-center md:gap-4 md:px-0">
      <div className="flex flex-col items-center gap-3">
        {props.withMonkey && <Monkey pose="eyes" className="w-14 md:w-20" />}
        <Logo variant="wordmark" className="h-3.5 text-ink md:h-4 dark:text-sand" />
        <h2 id={props.id} className="font-display text-2xl leading-[1.3] font-semibold tracking-[-0.02em] text-balance text-ink md:text-[40px] dark:text-sand">
          {props.title}
        </h2>
      </div>
      {props.description && (
        <p className="text-[15px] leading-6 text-pretty text-ink md:text-xl md:leading-8 dark:text-sand-muted">{props.description}</p>
      )}
    </div>
  )
}
