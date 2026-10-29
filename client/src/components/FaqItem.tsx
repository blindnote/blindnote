import type { ReactNode } from 'react'
import { Icon } from './Icon'

type FaqItemProps = {
  question: string
  answer: ReactNode
  /** Aynı name'e sahip maddelerden aynı anda yalnız biri açık kalır (yerel <details name>). */
  group?: string
}

/**
 * Figma: faq-item (kapalı hâl). Açık hâl tasarımda yok — aynı kenarlık içinde cevap
 * ve dönen ok olarak türetildi. JS state'i yok: yerel <details>/<summary>.
 */
export function FaqItem(props: FaqItemProps) {
  return (
    <details
      name={props.group}
      className="group w-full rounded-[22px] border border-ink/30 transition-colors open:border-ink/50 md:rounded-card dark:border-night-line dark:open:border-accent/30"
    >
      <summary className="flex cursor-pointer list-none items-center gap-1 rounded-[22px] min-h-12 px-4 py-3 text-ink outline-none md:min-h-[52px] md:px-5 select-none focus-visible:ring-4 focus-visible:ring-accent md:rounded-card dark:text-sand [&::-webkit-details-marker]:hidden">
        <span className="flex-1 text-[15px] leading-6 font-medium md:text-lg md:leading-7">{props.question}</span>
        <Icon name="chevron-down" className="size-5 text-ink/30 transition-transform group-open:rotate-180 md:size-6 dark:text-sand/40" />
      </summary>
      <div className="px-4 pb-4 text-[15px] leading-6 text-ink md:px-5 md:pb-5 md:text-base md:leading-7 dark:text-sand-muted">{props.answer}</div>
    </details>
  )
}
