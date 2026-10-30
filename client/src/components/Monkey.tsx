import type { ReactNode } from 'react'

/*
 * Üç maymun illüstrasyonu (gözler / kulaklar / ağız) — Figma'daki maymunların yeniden çizilmiş, temizlenmiş hâli.
 * Oranlar Figma ile aynı (90×76, 106×69, 80×88), bu yüzden yerleşim ölçüleri değişmedi.
 * Renkler illüstrasyona ait sabit palet (kürk #7A573F, yüz #FABE82, kontur #4F2000); temayla değişmez.
 */
const MONKEY_POSES = {
  eyes: {
    viewBox: '0 0 120 101',
    art: (
      <>
      <circle cx="24" cy="48" r="13" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <circle cx="24" cy="48" r="7" fill="#FABE82" />
      <circle cx="96" cy="48" r="13" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <circle cx="96" cy="48" r="7" fill="#FABE82" />
      <circle cx="60" cy="50" r="37" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <path d="M54 14 q4 -9 10 -5 q-4 1 -2 6" fill="#7A573F" stroke="#4F2000" strokeWidth="3" strokeLinejoin="round" strokeLinecap="round" />
      <path d="M60 36 C54 24 30 26 31 46 C31 56 33 60 36 64 C36 78 48 82 60 82 C72 82 84 78 84 64 C87 60 89 56 89 46 C90 26 66 24 60 36 Z" fill="#FABE82" />
      <circle cx="57" cy="59" r="1.6" fill="#4F2000" />
      <circle cx="63" cy="59" r="1.6" fill="#4F2000" />
      <path d="M51 66 Q60 78 69 66 Z" fill="#4F2000" stroke="#4F2000" strokeWidth="2" strokeLinejoin="round" />
      <path d="M55.5 72.5 Q60 69.5 64.5 72.5 Q60 75.5 55.5 72.5Z" fill="#FF9EC4" />
      <ellipse cx="41" cy="59" rx="5" ry="3" fill="#FF9EC4" opacity=".45" />
      <ellipse cx="79" cy="59" rx="5" ry="3" fill="#FF9EC4" opacity=".45" />
      <path d="M24 106 Q24 84 40 66" fill="none" stroke="#4F2000" strokeWidth="15" strokeLinecap="round" />
      <path d="M24 106 Q24 84 40 66" fill="none" stroke="#7A573F" strokeWidth="9" strokeLinecap="round" />
      <path d="M96 106 Q96 84 80 66" fill="none" stroke="#4F2000" strokeWidth="15" strokeLinecap="round" />
      <path d="M96 106 Q96 84 80 66" fill="none" stroke="#7A573F" strokeWidth="9" strokeLinecap="round" />
      <g transform="translate(46 44) rotate(-6) scale(0.95 0.95)">
        <path d="M-12 10 V-5 A3 3 0 0 1 -6 -5 V-8 A3 3 0 0 1 0 -8 A3 3 0 0 1 6 -8 V-5 A3 3 0 0 1 12 -5 V10 Q12 19 0 19 Q-12 19 -12 10 Z" fill="#FABE82" stroke="#4F2000" strokeWidth="3.1578947368421053" strokeLinejoin="round" />
        <path d="M-6 -5 V1 M0 -8 V1 M6 -5 V1" stroke="#4F2000" strokeWidth="2.1052631578947367" strokeLinecap="round" />
      </g>
      <g transform="translate(74 44) rotate(6) scale(-0.95 0.95)">
        <path d="M-12 10 V-5 A3 3 0 0 1 -6 -5 V-8 A3 3 0 0 1 0 -8 A3 3 0 0 1 6 -8 V-5 A3 3 0 0 1 12 -5 V10 Q12 19 0 19 Q-12 19 -12 10 Z" fill="#FABE82" stroke="#4F2000" strokeWidth="3.1578947368421053" strokeLinejoin="round" />
        <path d="M-6 -5 V1 M0 -8 V1 M6 -5 V1" stroke="#4F2000" strokeWidth="2.1052631578947367" strokeLinecap="round" />
      </g>
      </>
    ) as ReactNode,
  },
  ears: {
    viewBox: '0 0 150 98',
    art: (
      <>
      <circle cx="39" cy="50" r="13" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <circle cx="39" cy="50" r="7" fill="#FABE82" />
      <circle cx="111" cy="50" r="13" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <circle cx="111" cy="50" r="7" fill="#FABE82" />
      <circle cx="75" cy="52" r="37" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <path d="M69 16 q4 -9 10 -5 q-4 1 -2 6" fill="#7A573F" stroke="#4F2000" strokeWidth="3" strokeLinejoin="round" strokeLinecap="round" />
      <path d="M75 38 C69 26 45 28 46 48 C46 58 48 62 51 66 C51 80 63 84 75 84 C87 84 99 80 99 66 C102 62 104 58 104 48 C105 28 81 26 75 38 Z" fill="#FABE82" />
      <ellipse cx="64" cy="49" rx="4" ry="4.6" fill="#4F2000" />
      <circle cx="65.4" cy="47.4" r="1.3" fill="#fff" />
      <ellipse cx="86" cy="49" rx="4" ry="4.6" fill="#4F2000" />
      <circle cx="87.4" cy="47.4" r="1.3" fill="#fff" />
      <circle cx="72" cy="61" r="1.6" fill="#4F2000" />
      <circle cx="78" cy="61" r="1.6" fill="#4F2000" />
      <path d="M66 68 Q75 80 84 68 Z" fill="#4F2000" stroke="#4F2000" strokeWidth="2" strokeLinejoin="round" />
      <path d="M70.5 74.5 Q75 71.5 79.5 74.5 Q75 77.5 70.5 74.5Z" fill="#FF9EC4" />
      <ellipse cx="56" cy="61" rx="5" ry="3" fill="#FF9EC4" opacity=".45" />
      <ellipse cx="94" cy="61" rx="5" ry="3" fill="#FF9EC4" opacity=".45" />
      <path d="M22 104 Q16 88 36 68" fill="none" stroke="#4F2000" strokeWidth="15" strokeLinecap="round" />
      <path d="M22 104 Q16 88 36 68" fill="none" stroke="#7A573F" strokeWidth="9" strokeLinecap="round" />
      <path d="M128 104 Q134 88 114 68" fill="none" stroke="#4F2000" strokeWidth="15" strokeLinecap="round" />
      <path d="M128 104 Q134 88 114 68" fill="none" stroke="#7A573F" strokeWidth="9" strokeLinecap="round" />
      <g transform="translate(38 50) rotate(-4) scale(1.05 1.05)">
        <path d="M-12 10 V-5 A3 3 0 0 1 -6 -5 V-8 A3 3 0 0 1 0 -8 A3 3 0 0 1 6 -8 V-5 A3 3 0 0 1 12 -5 V10 Q12 19 0 19 Q-12 19 -12 10 Z" fill="#FABE82" stroke="#4F2000" strokeWidth="2.857142857142857" strokeLinejoin="round" />
        <path d="M-6 -5 V1 M0 -8 V1 M6 -5 V1" stroke="#4F2000" strokeWidth="1.9047619047619047" strokeLinecap="round" />
      </g>
      <g transform="translate(112 50) rotate(4) scale(-1.05 1.05)">
        <path d="M-12 10 V-5 A3 3 0 0 1 -6 -5 V-8 A3 3 0 0 1 0 -8 A3 3 0 0 1 6 -8 V-5 A3 3 0 0 1 12 -5 V10 Q12 19 0 19 Q-12 19 -12 10 Z" fill="#FABE82" stroke="#4F2000" strokeWidth="2.857142857142857" strokeLinejoin="round" />
        <path d="M-6 -5 V1 M0 -8 V1 M6 -5 V1" stroke="#4F2000" strokeWidth="1.9047619047619047" strokeLinecap="round" />
      </g>
      </>
    ) as ReactNode,
  },
  mouth: {
    viewBox: '0 0 110 121',
    art: (
      <>
      <circle cx="19" cy="48" r="13" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <circle cx="19" cy="48" r="7" fill="#FABE82" />
      <circle cx="91" cy="48" r="13" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <circle cx="91" cy="48" r="7" fill="#FABE82" />
      <circle cx="55" cy="50" r="37" fill="#7A573F" stroke="#4F2000" strokeWidth="3" />
      <path d="M49 14 q4 -9 10 -5 q-4 1 -2 6" fill="#7A573F" stroke="#4F2000" strokeWidth="3" strokeLinejoin="round" strokeLinecap="round" />
      <path d="M55 36 C49 24 25 26 26 46 C26 56 28 60 31 64 C31 78 43 82 55 82 C67 82 79 78 79 64 C82 60 84 56 84 46 C85 26 61 24 55 36 Z" fill="#FABE82" />
      <ellipse cx="44" cy="47" rx="4" ry="4.6" fill="#4F2000" />
      <circle cx="45.4" cy="45.4" r="1.3" fill="#fff" />
      <ellipse cx="66" cy="47" rx="4" ry="4.6" fill="#4F2000" />
      <circle cx="67.4" cy="45.4" r="1.3" fill="#fff" />
      <ellipse cx="36" cy="59" rx="5" ry="3" fill="#FF9EC4" opacity=".45" />
      <ellipse cx="74" cy="59" rx="5" ry="3" fill="#FF9EC4" opacity=".45" />
      <path d="M28 126 Q26 112 44 94" fill="none" stroke="#4F2000" strokeWidth="15" strokeLinecap="round" />
      <path d="M28 126 Q26 112 44 94" fill="none" stroke="#7A573F" strokeWidth="9" strokeLinecap="round" />
      <path d="M82 126 Q84 112 66 94" fill="none" stroke="#4F2000" strokeWidth="15" strokeLinecap="round" />
      <path d="M82 126 Q84 112 66 94" fill="none" stroke="#7A573F" strokeWidth="9" strokeLinecap="round" />
      <g transform="translate(46 74) rotate(-6) scale(0.95 0.95)">
        <path d="M-12 10 V-5 A3 3 0 0 1 -6 -5 V-8 A3 3 0 0 1 0 -8 A3 3 0 0 1 6 -8 V-5 A3 3 0 0 1 12 -5 V10 Q12 19 0 19 Q-12 19 -12 10 Z" fill="#FABE82" stroke="#4F2000" strokeWidth="3.1578947368421053" strokeLinejoin="round" />
        <path d="M-6 -5 V1 M0 -8 V1 M6 -5 V1" stroke="#4F2000" strokeWidth="2.1052631578947367" strokeLinecap="round" />
      </g>
      <g transform="translate(64 74) rotate(6) scale(-0.95 0.95)">
        <path d="M-12 10 V-5 A3 3 0 0 1 -6 -5 V-8 A3 3 0 0 1 0 -8 A3 3 0 0 1 6 -8 V-5 A3 3 0 0 1 12 -5 V10 Q12 19 0 19 Q-12 19 -12 10 Z" fill="#FABE82" stroke="#4F2000" strokeWidth="3.1578947368421053" strokeLinejoin="round" />
        <path d="M-6 -5 V1 M0 -8 V1 M6 -5 V1" stroke="#4F2000" strokeWidth="2.1052631578947367" strokeLinecap="round" />
      </g>
      </>
    ) as ReactNode,
  },
} as const

export type MonkeyPose = keyof typeof MONKEY_POSES

type MonkeyProps = {
  /** eyes: gözlerini kapatan (Yaz, hero), ears: kulaklarını kapatan (Bağlantı, Mesaj Açıldı), mouth: ağzını kapatan (Paylaş, Bulunamadı) */
  pose: MonkeyPose
  /** Boyut için Tailwind sınıfları, ör. "w-[90px]". Yükseklik oranla gelir. */
  className?: string
}

/** Dekoratif maymun illüstrasyonu. Ekran okuyuculardan gizlidir. */
export function Monkey(props: MonkeyProps) {
  const pose = MONKEY_POSES[props.pose]
  return (
    <svg
      viewBox={pose.viewBox}
      fill="none"
      aria-hidden="true"
      focusable="false"
      className={['block h-auto shrink-0', props.className].filter(Boolean).join(' ')}
    >
      {pose.art}
    </svg>
  )
}
