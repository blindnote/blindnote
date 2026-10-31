import { useEffect, useState } from 'react'

/** Sayfa belirli bir eşiğin altına kaydırıldı mı? (yalnızca görünüm kararı için; durum taşımaz) */
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
