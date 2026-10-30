import { cx } from './cx'
import { Monkey, type MonkeyPose } from './Monkey'

type StepCardProps = {
  pose: MonkeyPose
  title: string
  text: string
}

/** Kartın üst kenarına binme miktarı (ağzını kapatan maymunun elleri görünsün diye Figma'dakinden az) */
const OVERLAP: Record<MonkeyPose, string> = {
  eyes: '-mb-1.5 md:-mb-[7px] w-14 md:w-20',
  ears: '-mb-1.5 md:-mb-[9px] w-[72px] md:w-[106px]',
  mouth: '-mb-3.5 md:-mb-5 w-14 md:w-20',
}

/** Figma: step-card (+ üstünden bakan maymun). Maymun kartın arkasında kalır. */
export function StepCard(props: StepCardProps) {
  return (
    <li className="isolate flex w-full max-w-[343px] flex-col items-center md:max-w-[560px] lg:max-w-none">
      <Monkey pose={props.pose} className={cx('relative -z-10', OVERLAP[props.pose])} />
      <div className="flex w-full flex-1 flex-col items-center gap-2 rounded-2xl border border-accent/40 bg-[color-mix(in_srgb,var(--color-accent)_5%,var(--color-page))] p-4 text-center shadow-step md:rounded-card md:p-6 dark:border-accent/20 dark:bg-night-card dark:shadow-none">
        <h3 className="font-display text-xl leading-[30px] font-semibold text-ink md:text-2xl dark:text-sand">
          {props.title}
        </h3>
        <p className="text-sm leading-6 text-ink md:text-base md:leading-6 dark:text-sand-muted">{props.text}</p>
      </div>
    </li>
  )
}
