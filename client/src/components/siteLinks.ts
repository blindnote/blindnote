export type SiteLink = { label: string; href: string }

/** Header ve mobil menüdeki bağlantılar (Figma: site-header / Menü). */
export const HEADER_LINKS: ReadonlyArray<SiteLink> = [
  { label: 'Özgürce Paylaş', href: '/' },
  { label: 'BlindNote Nedir?', href: '#nedir' },
  { label: 'Uçtan Uca Şifreleme', href: '#sifreleme' },
  { label: 'Sıkça Sorulan Sorular', href: '#sss' },
]

/** Mobil menü (Figma "Menü" frame'i): ilk madde "Gizli Mesaj Oluştur". */
export const MENU_LINKS: ReadonlyArray<SiteLink> = [
  { label: 'Gizli Mesaj Oluştur', href: '/' },
  { label: 'BlindNote Nedir?', href: '#nedir' },
  { label: 'Uçtan Uca Şifreleme', href: '#sifreleme' },
  { label: 'Sıkça Sorulan Sorular', href: '#sss' },
]

/**
 * Footer bağlantıları. Figma'da 3. madde "Nasıl Çalışır?" (header'da "Uçtan Uca Şifreleme").
 * Tasarımdaki gibi bırakıldı; hangisi doğruysa ikisi eşitlenmeli.
 */
export const FOOTER_LINKS: ReadonlyArray<SiteLink> = [
  { label: 'Özgürce Paylaş', href: '/' },
  { label: 'BlindNote Nedir?', href: '#nedir' },
  { label: 'Nasıl Çalışır?', href: '#sifreleme' },
  { label: 'Sıkça Sorulan Sorular', href: '#sss' },
]
