import { createContext, useId, useState } from 'react'
import { useCopy } from '../lib/i18n'
import { Button } from './Button'
import { Icon } from './Icon'
import { Monkey, type MonkeyPose } from './Monkey'
import { Tagline } from './Tagline'

/**
 * Alt eylem bloğunun görünümü (Mesaj Açıldı'da "Cevap Gönder", Bulunamadı'da create).
 * figma: Figma'daki başlık + tagline + buton.
 * composer / banner / paths: tasarım incelemesi için önerilen alternatifler (Figma'da yok).
 */
export type CtaLook = 'figma' | 'composer' | 'banner' | 'paths'
/** Mesaj Açıldı → "Cevap Gönder" bloğu */
export const ReplyCtaContext = createContext<CtaLook>('banner')
/** Bulunamadı → create bloğu */
export const NotFoundCtaContext = createContext<CtaLook>('banner')

type CreateNoteCtaProps = {
  onCreate: () => void
  /** Figma: Mesaj Açıldı'da başlık var; Bulunamadı'da yok. */
  heading?: string
  buttonLabel?: string
  /** Yalnız mobilde göster (Bağlantı Oluşturuldu: masaüstünde header CTA'sı var) */
  mobileOnly?: boolean
  look?: CtaLook
  /** composer görünümünde yazılan metinle devam (ör. yazma ekranına taşı) */
  onCreateWithText?: (text: string) => void
  /** paths görünümünde "Nasıl çalışır?" bağlantısı */
  learnHref?: string
  /** banner görünümünün başlık/açıklaması */
  bannerTitle?: string
  bannerText?: string
  /** banner görünümündeki maymun (varsayılan: kulaklarını kapatan) */
  bannerMonkey?: MonkeyPose
}

/** A · Hemen yanıtla: cevabı aynı sayfada yazmaya başlat */
function ComposerLook(props: CreateNoteCtaProps) {
  const t = useCopy()
  const id = useId()
  const [text, setText] = useState('')
  return (
    <form
      noValidate
      onSubmit={(e) => {
        e.preventDefault()
        if (props.onCreateWithText) props.onCreateWithText(text)
        else props.onCreate()
      }}
      className="w-full max-w-[640px] rounded-2xl border border-accent/40 bg-cream p-3 text-left md:rounded-card md:p-5 dark:border-accent/20 dark:bg-night-raised"
    >
      <label htmlFor={id} className="font-display text-base font-semibold text-ink md:text-xl dark:text-sand">
        {props.heading ?? t.cta.replyHeading}
      </label>
      <p className="mt-0.5 text-xs text-fur md:text-sm dark:text-sand-muted">{t.cta.replyHint}</p>
      <textarea
        id={id}
        value={text}
        onChange={(e) => setText(e.target.value)}
        rows={2}
        placeholder={t.cta.replyPlaceholder}
        className="mt-3 w-full resize-none rounded-xl border border-ink/10 bg-page px-3 py-2.5 text-sm leading-6 text-ink-strong outline-none placeholder:text-muted focus-visible:ring-4 focus-visible:ring-accent md:text-base dark:border-night-line dark:bg-night-card dark:text-sand"
      />
      <div className="mt-4 flex flex-col items-center gap-3 md:flex-row md:justify-between">
        <Tagline align="right" className="order-2 md:order-1" />
        <Button type="submit" blockOnMobile className="order-1 md:order-2">
          {t.cta.replyButton}
        </Button>
      </div>
    </form>
  )
}

/** B · Şerit: maymunlu, yatay davet bandı */
function BannerLook(props: CreateNoteCtaProps) {
  const t = useCopy()
  return (
    <div className="flex w-full max-w-[848px] flex-col items-center gap-5 rounded-2xl bg-[linear-gradient(110deg,color-mix(in_srgb,var(--color-blush)_55%,var(--color-page))_0%,var(--color-cream)_55%,color-mix(in_srgb,var(--color-accent)_30%,var(--color-page))_100%)] px-5 py-7 text-center md:flex-row md:gap-6 md:rounded-card md:px-8 md:py-6 md:text-left dark:bg-[linear-gradient(110deg,var(--color-night-raised),var(--color-night-card))]">
      <Monkey
        pose={props.bannerMonkey ?? 'ears'}
        className={(props.bannerMonkey ?? 'ears') === 'ears' ? 'w-16 md:w-20' : 'w-14 md:w-16'}
      />
      <div className="flex-1">
        <p className="font-display text-lg font-semibold text-ink md:text-2xl dark:text-sand">
          {props.bannerTitle ?? props.heading ?? t.cta.bannerTitle}
        </p>
        <p className="mt-1.5 text-sm leading-5 text-ink-strong md:text-base dark:text-sand-muted">
          {props.bannerText ?? t.cta.bannerText}
        </p>
      </div>
      <Button onClick={props.onCreate} blockOnMobile>
        {props.buttonLabel ?? t.cta.createButton}
      </Button>
    </div>
  )
}

/** Yollar: ana eylem + öğren bağlantısı, iki kart */
function PathsLook(props: CreateNoteCtaProps) {
  const t = useCopy()
  const card =
    'flex flex-1 items-center gap-3 rounded-2xl border p-4 text-left outline-none transition-colors focus-visible:ring-4 focus-visible:ring-accent md:p-5'
  return (
    <div className="flex w-full max-w-[720px] flex-col gap-3 md:flex-row md:gap-4">
      <button
        type="button"
        onClick={props.onCreate}
        className={`${card} border-ink bg-ink text-white hover:bg-ink-hover dark:border-accent dark:bg-accent dark:text-ink`}
      >
        <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-white/15 dark:bg-ink/10">
          <Icon name="lock" className="size-5" />
        </span>
        <span className="flex flex-col">
          <span className="font-display text-base font-semibold md:text-lg">
            {props.buttonLabel ?? t.cta.createButton}
          </span>
          <span className="text-xs opacity-80 md:text-sm">{t.cta.pathsSub}</span>
        </span>
      </button>
      <a
        href={props.learnHref ?? '#nedir'}
        className={`${card} border-ink/15 bg-page text-ink hover:border-ink/40 dark:border-night-line dark:bg-night-card dark:text-sand`}
      >
        <span className="flex size-10 shrink-0 items-center justify-center rounded-full bg-cream dark:bg-night-raised">
          <Icon name="eye-off" className="size-5 text-fur dark:text-accent" />
        </span>
        <span className="flex flex-col">
          <span className="font-display text-base font-semibold md:text-lg">{t.cta.pathsLearnTitle}</span>
          <span className="text-xs text-fur md:text-sm dark:text-sand-muted">{t.cta.pathsLearnSub}</span>
        </span>
      </a>
    </div>
  )
}

/** Figma: Frame 513521 — "Cevap Gönder" bloğu ve create butonu + tagline. */
export function CreateNoteCta(props: CreateNoteCtaProps) {
  const t = useCopy()
  const label = props.buttonLabel ?? t.cta.createButton
  const look = props.look ?? 'figma'
  const wrap = props.mobileOnly ? 'flex w-full flex-col items-center md:hidden' : 'flex w-full flex-col items-center'

  if (look === 'composer')
    return (
      <div className={wrap}>
        <ComposerLook {...props} />
      </div>
    )
  if (look === 'banner')
    return (
      <div className={wrap}>
        <BannerLook {...props} />
      </div>
    )
  if (look === 'paths')
    return (
      <div className={wrap}>
        <PathsLook {...props} />
      </div>
    )

  return (
    <div className={wrap}>
      {props.heading ? (
        <div className="flex flex-col items-center gap-4">
          <div className="flex flex-col items-center gap-1 md:gap-2">
            <h2 className="text-center font-display text-base font-semibold tracking-[-0.02em] text-ink md:text-2xl dark:text-sand">
              {props.heading}
            </h2>
            <Tagline />
          </div>
          <Button onClick={props.onCreate}>{label}</Button>
        </div>
      ) : (
        <div className="flex w-full flex-col items-center gap-4 md:gap-3">
          <Button blockOnMobile onClick={props.onCreate}>
            {label}
          </Button>
          <Tagline />
        </div>
      )}
    </div>
  )
}
