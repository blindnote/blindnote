import { useEffect, useState } from 'react'

/** Has the page scrolled below a given threshold? (only for a view decision; holds no state) */
export function useScrolledPast(threshold: number) {
  const [past, setPast] = useState(false)
  useEffect(() => {
    const check = () => setPast(window.scrollY > threshold)
    check()
    window.addEventListener('scroll', check, { passive: true })
    return () => window.removeEventListener('scroll', check)
  }, [threshold])
  return past
}
