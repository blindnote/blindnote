import { cx } from './cx'

/* Figma'daki ikon bileşenlerinden (chevron-down, solar:copy-bold, menu-01, x-close) dışa aktarıldı.
   Spinner ve check: tasarımda yok, loading/kopyalandı durumları için türetildi. Renk currentColor. */
const ICONS = {
  'chevron-down': { viewBox: '0 0 24 24', stroke: true, d: 'M6 9L12 15L18 9' },
  menu: { viewBox: '10 13 20 14', stroke: true, d: 'M11 20H29M11 14H29M11 26H29' },
  close: { viewBox: '13 13 14 14', stroke: true, d: 'M26 14L14 26M14 14L26 26' },
  check: { viewBox: '0 0 20 20', stroke: true, d: 'M4.5 10.5L8 14L15.5 6.5' },
  // Güven satırı alternatifleri için (Figma'da yok — türetildi)
  'eye-off': {
    viewBox: '0 0 24 24',
    stroke: true,
    d: 'M3 3L21 21M10.6 6.1A9.8 9.8 0 0 1 12 6C17 6 21 12 21 12A15.5 15.5 0 0 1 17.8 15.7M6.6 7.6C4.3 9.2 3 12 3 12S7 18 12 18A8.8 8.8 0 0 0 16.4 16.8M9.9 9.9A3 3 0 0 0 14.1 14.1',
  },
  'server-off': {
    viewBox: '0 0 24 24',
    stroke: true,
    d: 'M6 5H18A2 2 0 0 1 20 7V8A2 2 0 0 1 18 10H6A2 2 0 0 1 4 8V7A2 2 0 0 1 6 5Z M6 14H18A2 2 0 0 1 20 16V17A2 2 0 0 1 18 19H6A2 2 0 0 1 4 17V16A2 2 0 0 1 6 14Z M7.5 7.5H7.51 M7.5 16.5H7.51 M3 3L21 21',
  },
  lock: {
    viewBox: '0 0 24 24',
    stroke: true,
    d: 'M7 11V8A5 5 0 0 1 17 8V11M6 11H18A1 1 0 0 1 19 12V19A1 1 0 0 1 18 20H6A1 1 0 0 1 5 19V12A1 1 0 0 1 6 11ZM12 14.5V16.5',
  },
  unlock: {
    viewBox: '0 0 24 24',
    stroke: true,
    d: 'M7 11V8A5 5 0 0 1 16.6 6.2M6 11H18A1 1 0 0 1 19 12V19A1 1 0 0 1 18 20H6A1 1 0 0 1 5 19V12A1 1 0 0 1 6 11ZM12 14.5V16.5',
  },
  'message-plus': {
    viewBox: '0 0 24 24',
    stroke: true,
    d: 'M5 4.5H19A2 2 0 0 1 21 6.5V15A2 2 0 0 1 19 17H10L6 20.5V17H5A2 2 0 0 1 3 15V6.5A2 2 0 0 1 5 4.5ZM12 8V13.5M9.25 10.75H14.75',
  },
  copy: {
    viewBox: '0 0 20 20',
    stroke: false,
    d: 'M12.7 1.6H9.4C7.9 1.6 6.8 1.6 5.9 1.7C4.9 1.9 4.2 2.1 3.6 2.7C3.0 3.3 2.7 4.1 2.6 5.0C2.5 6.0 2.5 7.1 2.5 8.6V13.5C2.5 14.7 3.2 15.8 4.3 16.2C4.3 15.5 4.3 14.4 4.3 13.5V9.4C4.3 8.3 4.3 7.4 4.3 6.6C4.5 5.9 4.7 5.1 5.3 4.5C5.9 3.9 6.7 3.6 7.5 3.5C8.2 3.4 9.1 3.4 10.2 3.4H12.7C13.8 3.4 14.7 3.4 15.4 3.5C15.2 3.0 14.8 2.5 14.3 2.1C13.8 1.8 13.3 1.6 12.7 1.6Z M5.5 9.4C5.5 7.2 5.5 6.0 6.2 5.3C6.9 4.6 8.0 4.6 10.3 4.6H12.7C14.9 4.6 16.0 4.6 16.7 5.3C17.5 6.0 17.5 7.2 17.5 9.4V13.5C17.5 15.7 17.5 16.9 16.7 17.6C16.0 18.3 14.9 18.3 12.7 18.3H10.3C8.0 18.3 6.9 18.3 6.2 17.6C5.5 16.9 5.5 15.7 5.5 13.5V9.4Z',
  },
} as const

export type IconName = keyof typeof ICONS | 'spinner'

type IconProps = {
  name: IconName
  /** Boyut ve renk, ör. "size-5 text-ink". */
  className?: string
}

/** Dekoratif ikon. Anlamı her zaman çevreleyen buton/etiket taşır (aria-hidden). */
export function Icon(props: IconProps) {
  if (props.name === 'spinner') {
    return (
      <svg
        viewBox="0 0 24 24"
        fill="none"
        aria-hidden="true"
        focusable="false"
        className={cx('block shrink-0 animate-spin', props.className)}
      >
        <circle cx="12" cy="12" r="9" stroke="currentColor" strokeOpacity="0.25" strokeWidth="3" />
        <path d="M21 12a9 9 0 0 0-9-9" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
      </svg>
    )
  }
  const icon = ICONS[props.name]
  return (
    <svg
      viewBox={icon.viewBox}
      fill="none"
      aria-hidden="true"
      focusable="false"
      className={cx('block shrink-0', props.className)}
    >
      {icon.stroke ? (
        <path d={icon.d} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
      ) : (
        <path d={icon.d} fill="currentColor" />
      )}
    </svg>
  )
}
