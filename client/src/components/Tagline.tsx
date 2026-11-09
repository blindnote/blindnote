import { useCopy } from '../lib/i18n'
import { cx } from './cx'
import { Icon, type IconName } from './Icon'

type TaglineProps = {
  align?: 'center' | 'right'
  className?: string
}

/** The security-features line -- badges (eye-off / server-off / lock). */
export function Tagline(props: TaglineProps) {
  const t = useCopy()
  const justify = props.align === 'right' ? 'justify-center md:justify-end' : 'justify-center'
  const items: ReadonlyArray<{ icon: IconName; text: string }> = [
    { icon: 'eye-off', text: t.tagline.oneTime },
    { icon: 'server-off', text: t.tagline.zeroKnowledge },
    { icon: 'lock', text: t.tagline.e2e },
  ]
  return (
    <ul aria-label={t.tagline.aria} className={cx('flex flex-wrap gap-2', justify, props.className)}>
      {items.map((item) => (
        <li
          key={item.text}
          className="inline-flex items-center gap-1 rounded-full border border-accent/50 bg-cream px-2.5 py-1 text-[13px] leading-5 font-semibold whitespace-nowrap text-ink dark:border-accent/25 dark:bg-night-raised dark:text-sand"
        >
          <Icon name={item.icon} className="size-3.5 text-fur dark:text-accent" />
          {item.text}
        </li>
      ))}
    </ul>
  )
}
