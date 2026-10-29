import { createContext, useContext, type ReactNode } from 'react'
import { cx } from './cx'
import { Monkey, type MonkeyPose } from './Monkey'

type HeroProps = {
  title: ReactNode
  subtitle?: ReactNode
  /**
   * peek: maymun kartın arkasından bakar (Yaz, Bağlantı Oluşturuldu, Mesaj Aç, Mesaj Açıldı).
   * top: maymun başlığın üstünde durur (Mesaj Bulunamadı).
   */
  monkey?: { pose: MonkeyPose; placement: 'peek' | 'top' }
  /** Başlığın altındaki ana kart (composer, link kartı, görüntüle kartı, not) */
  card?: ReactNode
  /** Kartın altındaki içerik (Cevap Gönder, CTA) */
  footer?: ReactNode
  /**
   * Masaüstünde başlığın üst boşluğu (Figma): default 128px (Yaz, Link, Mesaj Aç),
   * compact 88px (Mesaj Açıldı), roomy 167px (Bulunamadı). Mobilde hepsi 24px.
   */
  space?: 'default' | 'compact' | 'roomy'
}

const TOP_SPACE = { default: 'md:pt-32', compact: 'md:pt-[88px]', roomy: 'md:pt-[167px]' } as const

/**
 * Hero arka planı.
 * figma: Figma'daki görünüm (soldan pembe, ortası beyaz, sağı krem); keskin şerit kenarları yumuşatıldı.
 * glow / cream: tasarım incelemesi için önerilen iki alternatif (Figma'da yok).
 */
export type HeroBackdrop = 'figma' | 'glow' | 'cream'
const BACKDROPS: Record<HeroBackdrop, string> = {
  figma: [
    'bg-[radial-gradient(45%_110%_at_0%_45%,color-mix(in_srgb,var(--color-blush)_30%,var(--color-page))_0%,transparent_75%),radial-gradient(40%_110%_at_100%_55%,color-mix(in_srgb,var(--color-accent)_8%,var(--color-page))_0%,transparent_75%),linear-gradient(180deg,var(--color-page)_0%,color-mix(in_srgb,var(--color-blush)_8%,var(--color-page))_100%)]',
    'md:bg-[radial-gradient(30%_130%_at_0%_45%,color-mix(in_srgb,var(--color-blush)_38%,var(--color-page))_0%,color-mix(in_srgb,var(--color-blush)_12%,var(--color-page))_35%,transparent_80%),radial-gradient(35%_120%_at_100%_55%,color-mix(in_srgb,var(--color-accent)_9%,var(--color-page))_0%,transparent_80%),radial-gradient(40%_60%_at_60%_100%,color-mix(in_srgb,var(--color-blush)_6%,var(--color-page))_0%,transparent_80%)]',
    'dark:bg-[radial-gradient(60%_70%_at_8%_10%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%),radial-gradient(55%_65%_at_95%_25%,color-mix(in_srgb,var(--color-accent)_7%,transparent)_0%,transparent_70%)]',
    'md:dark:bg-[radial-gradient(60%_70%_at_8%_10%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%),radial-gradient(55%_65%_at_95%_25%,color-mix(in_srgb,var(--color-accent)_7%,transparent)_0%,transparent_70%)]',
  ].join(' '),
  glow: [
    'bg-[radial-gradient(70%_55%_at_0%_10%,color-mix(in_srgb,var(--color-blush)_85%,var(--color-page))_0%,transparent_70%),radial-gradient(70%_50%_at_100%_45%,color-mix(in_srgb,var(--color-accent)_45%,var(--color-page))_0%,transparent_70%),radial-gradient(90%_40%_at_40%_100%,color-mix(in_srgb,var(--color-blush)_45%,var(--color-page))_0%,transparent_75%)]',
    'md:bg-[radial-gradient(45%_85%_at_5%_15%,color-mix(in_srgb,var(--color-blush)_85%,var(--color-page))_0%,transparent_70%),radial-gradient(40%_80%_at_98%_40%,color-mix(in_srgb,var(--color-accent)_45%,var(--color-page))_0%,transparent_70%),radial-gradient(55%_55%_at_45%_105%,color-mix(in_srgb,var(--color-blush)_45%,var(--color-page))_0%,transparent_75%)]',
    'dark:bg-[radial-gradient(60%_70%_at_8%_10%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%),radial-gradient(55%_65%_at_95%_25%,color-mix(in_srgb,var(--color-accent)_7%,transparent)_0%,transparent_70%)]',
    'md:dark:bg-[radial-gradient(60%_70%_at_8%_10%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%),radial-gradient(55%_65%_at_95%_25%,color-mix(in_srgb,var(--color-accent)_7%,transparent)_0%,transparent_70%)]',
  ].join(' '),
  cream: [
    'bg-[linear-gradient(165deg,color-mix(in_srgb,var(--color-blush)_70%,var(--color-cream))_0%,var(--color-cream)_45%,color-mix(in_srgb,var(--color-accent)_35%,var(--color-cream))_100%)]',
    'dark:bg-[radial-gradient(60%_70%_at_8%_10%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%),radial-gradient(55%_65%_at_95%_25%,color-mix(in_srgb,var(--color-accent)_7%,transparent)_0%,transparent_70%)]',
    'md:dark:bg-[radial-gradient(60%_70%_at_8%_10%,color-mix(in_srgb,var(--color-accent)_10%,transparent)_0%,transparent_70%),radial-gradient(55%_65%_at_95%_25%,color-mix(in_srgb,var(--color-accent)_7%,transparent)_0%,transparent_70%)]',
  ].join(' '),
}

/** Önizleme/inceleme için arka plan seçimi; ürün varsayılanı Figma görünümüdür. */
export const HeroBackdropContext = createContext<HeroBackdrop>('glow')

/** Kartın arkasından bakan maymunun boyutu ve karta binme miktarı (Figma ölçüleri) */
const PEEK = {
  eyes: { gap: 'mt-[72px] md:mt-[100px]', monkey: 'w-14 translate-y-[18px] md:w-[90px] md:translate-y-[27px]' },
  ears: { gap: 'mt-[76px] md:mt-[67px]', monkey: 'w-[72px] translate-y-2 md:w-[106px] md:translate-y-3' },
  mouth: { gap: 'mt-[72px] md:mt-[100px]', monkey: 'w-14 translate-y-[18px] md:w-20 md:translate-y-7' },
} as const

/**
 * Figma: hero (Frame 513516). Arka plan Figma'da raster görsel; burada aynı geçiş tema
 * renklerinden CSS gradyanıyla kuruldu. Masaüstünde soldan pembe → ortada beyaz → sağda krem,
 * mobilde yukarıdan aşağı hafif pembe; renk kartın altına kadar iner, en altta sayfa rengine söner.
 */
export function Hero(props: HeroProps) {
  const backdrop = useContext(HeroBackdropContext)
  const peek = props.monkey?.placement === 'peek' ? PEEK[props.monkey.pose] : undefined
  return (
    <section className="relative isolate overflow-hidden">
      <div aria-hidden="true" className={cx('absolute inset-0 -z-10', BACKDROPS[backdrop])} />
      {/* Figma "section-fade": en altta sayfa rengine yumuşak geçiş */}
      <div aria-hidden="true" className="absolute inset-x-0 bottom-0 -z-10 h-8 bg-gradient-to-b from-transparent to-page md:h-[51px] dark:to-night" />

      <div className={cx('mx-auto flex max-w-[880px] flex-col items-center px-4 pt-8 pb-12 md:pb-[62px]', TOP_SPACE[props.space ?? 'default'])}>
        {props.monkey?.placement === 'top' && (
          <Monkey
            pose={props.monkey.pose}
            className={props.monkey.pose === 'ears' ? 'mb-3 w-[72px] md:-mb-1.5 md:w-[106px]' : 'mb-3 w-14 md:-mb-1.5 md:w-20'}
          />
        )}

        <h1 className="max-w-[343px] text-center font-display text-[32px] leading-[1.2] font-bold tracking-[-0.02em] text-balance text-ink md:max-w-[848px] md:text-5xl md:leading-[1.2] dark:text-sand">
          {props.title}
        </h1>
        {props.subtitle && (
          <p
            className={cx(
              'max-w-[343px] text-center text-base leading-[1.45] text-pretty text-ink md:max-w-[848px] md:text-2xl md:leading-[1.35] dark:text-sand-muted',
              props.monkey?.placement === 'top' ? 'mt-3 md:mt-3' : 'mt-3 md:mt-4',
            )}
          >
            {props.subtitle}
          </p>
        )}

        {props.card && (
          <div className={cx('relative isolate w-full', peek ? peek.gap : 'mt-8 md:mt-[62px]')}>
            {peek && props.monkey && (
              <Monkey pose={props.monkey.pose} className={cx('absolute bottom-full left-1/2 -z-10 -translate-x-1/2', peek.monkey)} />
            )}
            {props.card}
          </div>
        )}

        {props.footer && (
          <div className={cx('flex w-full flex-col items-center', props.card ? 'mt-8 md:mt-9' : 'mt-10 md:mt-[30px]')}>{props.footer}</div>
        )}
      </div>
    </section>
  )
}
