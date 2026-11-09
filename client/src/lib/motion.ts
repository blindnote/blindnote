/**
 * Whether the viewer asked the system to minimise motion.
 *
 * `app.css` already neutralises CSS animation and transition durations under
 * `prefers-reduced-motion: reduce`; this is the matching check for the few
 * places where JavaScript paces a transition (waiting for the seal to break or
 * to stamp before moving on). When motion is reduced those waits collapse to
 * zero, so the moment is skipped rather than sat through with nothing visible.
 */
export function prefersReducedMotion(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-reduced-motion: reduce)').matches
  )
}
