import { useEffect, useState } from 'react'

/**
 * Scroll-spy: returns the id of the section currently under the header, or
 * `null` while the page is above the first section. Fragment-only — it reads
 * the DOM by id and never touches history, so it leaves hard rule 2/3 alone.
 *
 * `ids` must be stable (a module constant), in document order; the topmost
 * section intersecting the trigger band wins. The band starts below the sticky
 * header (`-96px` top) and ends past the viewport middle (`-55%` bottom), so a
 * section goes active as its heading settles under the header.
 */
export function useActiveSection(ids: readonly string[]): string | null {
  const [active, setActive] = useState<string | null>(null)
  useEffect(() => {
    const els = ids
      .map((id) => document.getElementById(id))
      .filter((el): el is HTMLElement => el !== null)
    if (els.length === 0) return
    const visible = new Set<string>()
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) visible.add(entry.target.id)
          else visible.delete(entry.target.id)
        }
        // Bottommost section touching the band is the one just scrolled into;
        // the previous section's lower edge can still graze the band's top.
        let next: string | null = null
        for (const id of ids) if (visible.has(id)) next = id
        setActive(next)
      },
      { rootMargin: '-96px 0px -55% 0px', threshold: 0 },
    )
    els.forEach((el) => observer.observe(el))
    return () => observer.disconnect()
  }, [ids])
  return active
}
