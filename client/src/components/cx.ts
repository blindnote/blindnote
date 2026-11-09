/** Conditional class joining. A small helper to avoid adding a package (instead of clsx). */
export function cx(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(' ')
}
