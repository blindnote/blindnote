/** Koşullu class birleştirme. Paket kurmamak için küçük bir yardımcı (clsx yerine). */
export function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(' ')
}
