import { useId } from 'react'
import { cx } from './cx'

/**
 * idle: dingin, mühürlü zarf.
 * sealing: iki balmumu yarısı yanlardan gelip birleşir, mühür bir kez bastırır,
 *   halka dalgası yayılır, tik belirir (not oluşturulduktan sonra).
 * breaking: mühür iki yarıya ayrılıp savrulur, yüz silinir, içeriden yumuşak bir
 *   ışık açılır, zarf hafifçe seğirir (okuma anı).
 */
type SealState = 'idle' | 'sealing' | 'breaking'

/**
 * Mühürlü zarf illüstrasyonu (dekoratif) -- Oku ekranındaki kart ve oluşturma
 * sonrası mühürleme anı aynı çizimi paylaşır: mühür basılır, sonra kırılır.
 *
 * Animasyonların tümü SVG grupları üzerinde döner; her hareketli eleman
 * `transform-box: fill-box` taşır ki ölçek/döndürme kendi kutusuna göre olsun.
 * Hareketi azalt tercihinde `app.css` süreleri ~0'a indirir ve çağıran ekranlar
 * geçişi anında yapar, böylece bu anlar sessizce atlanır.
 */
export function Seal(props: { state: SealState; className?: string }) {
  const id = useId()
  const glowId = `${id}-glow`
  const breaking = props.state === 'breaking'
  const sealing = props.state === 'sealing'

  // Her hareketli elemanda: dönüşümler elemanın kendi kutusuna ve merkezine göre.
  const box = '[transform-box:fill-box] [transform-origin:center]'

  return (
    <svg viewBox="0 0 220 140" fill="none" aria-hidden="true" className={cx('block', props.className)}>
      <defs>
        <radialGradient id={glowId}>
          <stop offset="0%" style={{ stopColor: 'var(--color-accent)', stopOpacity: 0.9 }} />
          <stop offset="100%" style={{ stopColor: 'var(--color-accent)', stopOpacity: 0 }} />
        </radialGradient>
      </defs>

      {/* Zarf gövdesi, kapak ve iç gölge (dingin; kırılırken hafif seğirir). */}
      <g className={cx(box, breaking && 'animate-bn-lift')}>
        <rect
          x="6"
          y="10"
          width="208"
          height="124"
          rx="14"
          className="fill-cream stroke-ink dark:fill-night-raised dark:stroke-accent"
          strokeWidth="3"
        />
        <path
          d="M8 22L110 86L212 22"
          className="stroke-ink dark:stroke-accent"
          strokeWidth="3"
          strokeLinejoin="round"
        />
        <path
          d="M8 130L86 72M212 130L134 72"
          className="stroke-ink/30 dark:stroke-accent/40"
          strokeWidth="2.5"
          strokeLinecap="round"
        />
      </g>

      {/* İçeriden açılan ışık (yalnız kırılırken görünür). */}
      <circle
        cx="110"
        cy="86"
        r="38"
        fill={`url(#${glowId})`}
        className={cx(box, 'opacity-0', breaking && 'animate-bn-glow')}
      />

      {/* Mühürlemede baskı anında yayılan halka dalgası. */}
      {sealing && (
        <circle
          cx="110"
          cy="86"
          r="22"
          fill="none"
          strokeWidth="2.5"
          style={{ animationDelay: '0.36s' }}
          className={cx(box, 'opacity-0 stroke-ink animate-bn-ring dark:stroke-accent')}
        />
      )}

      {/* Balmumu mühür: iki yarı (birlikte tam daire) + yüz (kesik halka + tik). */}
      <g className={cx(box, sealing && 'animate-bn-stamp')} style={sealing ? { animationDelay: '0.3s' } : undefined}>
        <path
          d="M110 64 A22 22 0 0 0 110 108 Z"
          className={cx(
            box,
            'fill-ink dark:fill-accent',
            breaking && 'animate-bn-crack-l',
            sealing && 'animate-bn-seal-in-l',
          )}
        />
        <path
          d="M110 64 A22 22 0 0 1 110 108 Z"
          className={cx(
            box,
            'fill-ink dark:fill-accent',
            breaking && 'animate-bn-crack-r',
            sealing && 'animate-bn-seal-in-r',
          )}
        />
        <g
          className={cx(box, breaking && 'animate-bn-seal-off', sealing && 'animate-bn-draw')}
          style={sealing ? { animationDelay: '0.34s' } : undefined}
        >
          <circle
            cx="110"
            cy="86"
            r="15"
            fill="none"
            className="stroke-accent dark:stroke-ink"
            strokeWidth="2"
            strokeDasharray="3 3"
          />
          <path
            d="M103 86L108 91L118 80"
            className="stroke-accent dark:stroke-ink"
            strokeWidth="3"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </g>
      </g>
    </svg>
  )
}
