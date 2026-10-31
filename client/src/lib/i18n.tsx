/**
 * Interface language: the copy a person reads, in the languages we ship.
 *
 * This is content, so it keeps its own language -- unlike everything else in
 * the repository, which is English. Both dictionaries live here; components
 * read the active one through `useCopy()`.
 *
 * Detection is client-side on purpose. The honest, private default is the
 * browser's own language preference (`navigator.languages`), never the IP
 * address: an IP lookup needs a third party or server-side geolocation, it is
 * wrong behind a VPN, and this service logs nothing about who loaded the page.
 * English is the fallback; Turkish is served when the browser asks for it. A
 * manual choice is remembered in `localStorage` and wins over detection.
 *
 * The answers are written against what the service actually does, not the
 * strongest claim it could make. The 2021 version called itself end-to-end
 * encrypted while storing plaintext; being careful here is why this one exists.
 */

import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'

export type Locale = 'en' | 'tr'

export const SUPPORTED: readonly Locale[] = ['en', 'tr']
const DEFAULT_LOCALE: Locale = 'en'
const STORAGE_KEY = 'blindnote.locale'

/** One reason/step/label pair, used by the list sections. */
type TitledText = { title: string; text: string }
type FaqEntryText = { question: string; answer: string }

/** Every string the interface shows, in one shape both languages implement. */
export interface Strings {
  /** The language's own name, for the switcher. */
  langName: string
  /** Document title for this locale. */
  metaTitle: string
  nav: {
    shareFreely: string
    whatIs: string
    encryption: string
    howItWorks: string
    faq: string
    createSecret: string
  }
  header: {
    home: string
    mainMenu: string
    mobileMenu: string
    create: string
    openMenu: string
    closeMenu: string
    language: string
  }
  footer: { footerMenu: string }
  theme: { label: string; system: string; light: string; dark: string }
  compose: { title: string; subtitle: string }
  composer: {
    button: string
    loading: string
    label: string
    placeholder: string
    empty: string
  }
  linkReady: { title: string; subtitle: string }
  linkCard: { yourLink: string; copy: string; copied: string; copiedAnnounce: string; keyHint: string }
  reveal: { title: string; subtitle: string; envelopeExtra: string }
  revealCard: {
    view: string
    opening: string
    breakSeal: string
    breakingSeal: string
    onlyOnceShort: string
    onlyOnceLong: string
  }
  opened: { title: string; subtitle: string }
  noteViewer: {
    copyText: string
    copied: string
    secretMessage: string
    disappears: string
    opened: string
  }
  notFound: {
    title: string
    subtitle: string
    poofTitle: string
    poofHeadline: string
    poofBody: string
    poofHint: string
    bannerTitle: string
    bannerText: string
    reasonsHeading: string
    reasons: readonly TitledText[]
  }
  openedCta: { heading: string; bannerTitle: string; bannerText: string; button: string }
  cta: {
    replyHeading: string
    replyHint: string
    replyPlaceholder: string
    replyButton: string
    bannerTitle: string
    bannerText: string
    createButton: string
    pathsSub: string
    pathsLearnTitle: string
    pathsLearnSub: string
  }
  how: {
    title: string
    description: string
    stepLabel: (n: number) => string
    steps: readonly TitledText[]
  }
  encryption: { title: string; description: string }
  anatomy: {
    labels: readonly [string, string, string]
    caption: string
    oneTime: string
    boxNumber: string
    keyOnlyYou: string
    neverServer: string
    boxNote: string
  }
  tagline: { aria: string; oneTime: string; zeroKnowledge: string; e2e: string }
  faq: { heading: string; intro: string; items: readonly FaqEntryText[] }
  backToTop: string
  errors: {
    tooLarge: string
    offline: string
    throttled: string
    rejected: string
    server: string
    noCrypto: string
    copyFailed: string
  }
}

const en: Strings = {
  langName: 'English',
  metaTitle: 'BlindNote — One-time encrypted notes',
  nav: {
    shareFreely: 'Share Freely',
    whatIs: 'What is BlindNote?',
    encryption: 'End-to-End Encryption',
    howItWorks: 'How It Works',
    faq: 'FAQ',
    createSecret: 'Create a Secret Message',
  },
  header: {
    home: 'BlindNote home',
    mainMenu: 'Main menu',
    mobileMenu: 'Mobile menu',
    create: 'Create',
    openMenu: 'Open menu',
    closeMenu: 'Close menu',
    language: 'Language',
  },
  footer: { footerMenu: 'Footer menu' },
  theme: { label: 'Theme', system: 'System', light: 'Light', dark: 'Dark' },
  compose: {
    title: 'Share Freely',
    subtitle: "Create a link that destroys itself once it's opened.",
  },
  composer: {
    button: 'Create a Secret Message',
    loading: 'Encrypting…',
    label: 'Your secret message',
    placeholder: 'For their eyes only…',
    empty: 'Write your message first.',
  },
  linkReady: {
    title: 'Your Link Is Ready!',
    subtitle: 'The message in this link can be opened only once.',
  },
  linkCard: {
    yourLink: 'Your link: ',
    copy: 'Copy',
    copied: 'Copied',
    copiedAnnounce: 'Link copied to clipboard.',
    keyHint: 'The highlighted part is your key — it never reaches our server.',
  },
  reveal: {
    title: 'Knock Knock',
    subtitle: "There's a secret message just for you!",
    envelopeExtra: 'The seal breaks only once. The moment you open it, the message is deleted from the server.',
  },
  revealCard: {
    view: 'View the Message',
    opening: 'Opening…',
    breakSeal: 'Break the Seal and Read',
    breakingSeal: 'Breaking the seal…',
    onlyOnceShort: 'Can be viewed only once.',
    onlyOnceLong: 'You can view the message only once.',
  },
  opened: {
    title: 'You can view this message only once!',
    subtitle: 'The moment you opened it, the link was destroyed. This message cannot be viewed again.',
  },
  noteViewer: {
    copyText: 'Copy text',
    copied: 'Copied',
    secretMessage: 'Secret message',
    disappears: 'This note disappears when you leave the page',
    opened: 'Your message is open',
  },
  notFound: {
    title: "We Couldn't Find the Message!",
    subtitle: 'The link may have already destroyed itself.',
    poofTitle: 'Poof!',
    poofHeadline: "This message isn't here.",
    poofBody: 'Maybe it was already read, maybe it never existed. One-time messages leave no trace.',
    poofHint: 'If you were expecting a message, someone may have opened it before you. Let the sender know.',
    bannerTitle: 'Sharing a new message?',
    bannerText: 'Create a one-time, encrypted link in seconds.',
    reasonsHeading: 'What might have happened?',
    reasons: [
      {
        title: 'The message may have already been opened',
        text: "One-time links destroy themselves the moment they're opened.",
      },
      {
        title: 'The link may have been copied incompletely',
        text: 'Check the whole link, including the part after the # sign.',
      },
      {
        title: 'Let the sender know',
        text: 'They can recreate the message and send you a new link.',
      },
    ],
  },
  openedCta: {
    heading: 'Send a Reply',
    bannerTitle: 'Want to reply?',
    bannerText: 'Your reply works the same way: one-time, no trace.',
    button: 'Create a Secret Reply',
  },
  cta: {
    replyHeading: 'Send a Reply',
    replyHint: 'Your reply is sent with a one-time link too.',
    replyPlaceholder: 'Write your reply…',
    replyButton: 'Create a Secret Reply',
    bannerTitle: 'Your turn',
    bannerText: 'Send a one-time message of your own, without a trace.',
    createButton: 'Create a Secret Message',
    pathsSub: 'One-time, end-to-end encrypted.',
    pathsLearnTitle: 'How does BlindNote work?',
    pathsLearnSub: 'Why do messages destroy themselves?',
  },
  how: {
    title: 'One-Time Secure Notes',
    description: 'The right way to share sensitive information that can be viewed only once.',
    stepLabel: (n) => `Step ${n}`,
    steps: [
      { title: 'Write', text: 'Type the sensitive information you want to share into the message box.' },
      {
        title: 'Create a Link',
        text: 'The system creates a one-time secure link whose contents can be viewed only once.',
      },
      {
        title: 'Share',
        text: "The recipient can open the link once; the moment it's opened, the content is destroyed automatically and can never be reached again.",
      },
    ],
  },
  encryption: {
    title: 'End-to-End Encryption',
    description:
      'The encryption key is never stored on our servers; it works only as part of the link you create. Without that link — including us — no one can decrypt the information you shared.',
  },
  anatomy: {
    labels: ['Domain', 'Secret ID', 'Encryption Key'],
    caption:
      "Example link: it's made of a domain, a secret ID, and an encryption key. The encryption key is never sent to the server.",
    oneTime: 'One-time',
    boxNumber: 'Box number · on the server',
    keyOnlyYou: 'Key · only with you',
    neverServer: 'This part never goes to the server.',
    boxNote: 'Useless without the key.',
  },
  tagline: {
    aria: 'Security features',
    oneTime: 'One-Time Read',
    zeroKnowledge: 'Zero-Knowledge',
    e2e: 'End-to-End Encrypted',
  },
  faq: {
    heading: 'Frequently Asked Questions',
    intro:
      'The answers you are looking for are here. In short: your message is encrypted in your browser, opened once, and then gone.',
    items: [
      {
        question: 'Why should I use this service?',
        answer:
          "To share a password, a key, or a private note without leaving a lasting trace in your chat history. The link opens only once; the moment it is opened, the note is deleted from the server. You don't need an account, and we never ask for your email address.",
      },
      {
        question: 'How does the service work?',
        answer:
          'Your note is encrypted in your browser before it is sent; only the encrypted form reaches the server. The key that decrypts it sits in the part of the link after the # sign, and browsers never add that part to any request. So the key never reaches us.',
      },
      {
        question: 'Can you (or anyone else) read my note?',
        answer:
          'No. Because the key never reaches us, all we hold is encrypted text we cannot open. We keep no access logs and store nothing tied to you — only anonymous totals, like how many notes are made and read and how many visits the site gets. Your IP address is used in memory only, to limit abuse and to estimate visitor numbers, and is never written anywhere.',
      },
      {
        question: 'Can a deleted note be recovered? Do you keep backups?',
        answer:
          "When a note is read, we delete its 32-byte key; once the key is gone, the encrypted text can no longer be opened by anyone. We keep no backups and take no snapshots — what is deleted does not come back. We don't claim the bytes are physically erased from the disk; on a modern SSD no one can prove that. Our guarantee is mathematical: the key is gone.",
      },
      {
        question: 'What does this service not protect against?',
        answer:
          "Let's be honest: we can't protect the screen of the person reading the note. Malware on the recipient's device, a screenshot, or the recipient keeping a copy after reading are beyond our reach. The protection covers the note's journey through us — not the screen at the other end.",
      },
      {
        question: 'What if someone else gets hold of the link?',
        answer:
          "The link itself is the key: whoever opens it first reads the note. We can't prevent that, and we won't claim we can. But there is an upside: someone who sees their expected note as “not found” learns that someone got there first. If the link never reached the other side, don't assume it reached no one.",
      },
      {
        question: 'What if I never open the note?',
        answer: 'The note and its key are deleted after 24 hours, even if it was never read.',
      },
    ],
  },
  backToTop: 'Back to top',
  errors: {
    tooLarge: "A note can't be larger than 64 KB. Could you shorten it and try again?",
    offline: 'Could not reach the server. Check your connection and try again?',
    throttled: "You're going too fast. Wait a moment and try again.",
    rejected: "The note couldn't be sent. Could you shorten it and try again?",
    server: 'Something went wrong. Could you try again?',
    noCrypto: "Your browser can't encrypt. A current browser is required.",
    copyFailed: 'Could not copy to the clipboard. You can select the link and copy it by hand.',
  },
}

const tr: Strings = {
  langName: 'Türkçe',
  metaTitle: 'BlindNote — Tek seferlik şifreli notlar',
  nav: {
    shareFreely: 'Özgürce Paylaş',
    whatIs: 'BlindNote Nedir?',
    encryption: 'Uçtan Uca Şifreleme',
    howItWorks: 'Nasıl Çalışır?',
    faq: 'Sıkça Sorulan Sorular',
    createSecret: 'Gizli Mesaj Oluştur',
  },
  header: {
    home: 'BlindNote ana sayfa',
    mainMenu: 'Ana menü',
    mobileMenu: 'Mobil menü',
    create: 'Oluştur',
    openMenu: 'Menüyü aç',
    closeMenu: 'Menüyü kapat',
    language: 'Dil',
  },
  footer: { footerMenu: 'Alt menü' },
  theme: { label: 'Tema', system: 'Sistem', light: 'Açık', dark: 'Koyu' },
  compose: {
    title: 'Özgürce Paylaş',
    subtitle: 'Açıldığında kendini yok edecek bağlantı oluşturun.',
  },
  composer: {
    button: 'Gizli Mesaj Oluştur',
    loading: 'Şifreleniyor…',
    label: 'Gizli mesajın',
    placeholder: 'Yalnızca onun gözleri için…',
    empty: 'Önce mesajını yaz.',
  },
  linkReady: {
    title: 'Bağlantı Oluşturuldu!',
    subtitle: 'Bağlantıda yer alan mesaj sadece bir kez açılabilir.',
  },
  linkCard: {
    yourLink: 'Bağlantın: ',
    copy: 'Kopyala',
    copied: 'Kopyalandı',
    copiedAnnounce: 'Bağlantı panoya kopyalandı.',
    keyHint: 'Vurgulu kısım senin anahtarın — sunucumuza asla ulaşmaz.',
  },
  reveal: {
    title: 'Tak Tak',
    subtitle: 'Sana özel gizli bir mesaj var!',
    envelopeExtra: 'Mühür bir kez kırılır. Açtığın anda mesaj sunucudan silinir.',
  },
  revealCard: {
    view: 'Mesajı Görüntüle',
    opening: 'Açılıyor…',
    breakSeal: 'Mührü Kır ve Oku',
    breakingSeal: 'Mühür kırılıyor…',
    onlyOnceShort: 'Sadece bir kez görüntülenebilir.',
    onlyOnceLong: 'Mesajı sadece bir kez görüntüleyebilirsin.',
  },
  opened: {
    title: 'Bu mesajı bir kez görüntüleyebilirsin!',
    subtitle: 'Mesajı açtığın anda bağlantı imha edildi. Bu mesaj tekrar görüntülenemeyecek.',
  },
  noteViewer: {
    copyText: 'Metni kopyala',
    copied: 'Kopyalandı',
    secretMessage: 'Gizli mesaj',
    disappears: 'Sayfadan çıkınca bu not yok olur',
    opened: 'Mesajın açıldı',
  },
  notFound: {
    title: 'Mesajı Bulamadık!',
    subtitle: 'Bağlantı kendini yok etmiş olabilir.',
    poofTitle: 'Puf!',
    poofHeadline: 'Bu mesaj burada değil.',
    poofBody: 'Belki çoktan okundu, belki de hiç var olmadı. Tek kullanımlık mesajlar iz bırakmaz.',
    poofHint: 'Beklediğin bir mesajsa, biri senden önce açmış olabilir. Gönderene haber ver.',
    bannerTitle: 'Yeni bir mesaj mı paylaşacaksın?',
    bannerText: 'Saniyeler içinde tek kullanımlık, şifreli bir bağlantı oluştur.',
    reasonsHeading: 'Ne olmuş olabilir?',
    reasons: [
      {
        title: 'Mesaj zaten açılmış olabilir',
        text: 'Tek kullanımlık bağlantılar açıldığı anda kendini yok eder.',
      },
      {
        title: 'Bağlantı eksik kopyalanmış olabilir',
        text: '# işaretinden sonraki kısım dahil, bağlantının tamamını kontrol et.',
      },
      {
        title: 'Gönderene haber ver',
        text: 'Mesajı yeniden oluşturup sana yeni bir bağlantı gönderebilir.',
      },
    ],
  },
  openedCta: {
    heading: 'Cevap Gönder',
    bannerTitle: 'Cevap vermek ister misin?',
    bannerText: 'Cevabın da aynı şekilde: tek kullanımlık, iz bırakmadan.',
    button: 'Gizli Cevap Oluştur',
  },
  cta: {
    replyHeading: 'Cevap Gönder',
    replyHint: 'Cevabın da tek kullanımlık bir bağlantıyla gider.',
    replyPlaceholder: 'Cevabını yaz…',
    replyButton: 'Gizli Cevap Oluştur',
    bannerTitle: 'Sıra sende',
    bannerText: 'Sen de iz bırakmadan, tek kullanımlık bir mesaj gönder.',
    createButton: 'Gizli Mesaj Oluştur',
    pathsSub: 'Tek kullanımlık, uçtan uca şifreli.',
    pathsLearnTitle: 'BlindNote nasıl çalışır?',
    pathsLearnSub: 'Mesajlar neden kendini yok eder?',
  },
  how: {
    title: 'Tek Kullanımlık Güvenli Notlar',
    description: 'Yalnızca bir kez görüntülenebilen hassas bilgileri güvenle paylaşmanın en doğru yolu.',
    stepLabel: (n) => `${n}. adım`,
    steps: [
      { title: 'Yaz', text: 'Paylaşmak istediğiniz hassas bilgiyi mesaj alanına yazın.' },
      {
        title: 'Bağlantı Oluştur',
        text: 'Sistem, içeriği yalnızca bir kez görüntülenebilen tek kullanımlık güvenli bir link oluşturur.',
      },
      {
        title: 'Paylaş',
        text: 'Alıcı linki bir kez açabilir; açıldığı anda içerik otomatik olarak yok edilir ve bir daha erişilemez.',
      },
    ],
  },
  encryption: {
    title: 'Uçtan Uca Şifreleme',
    description:
      'Şifreleme anahtarı hiçbir zaman sunucularda saklanmaz; yalnızca oluşturulan bağlantının bir parçası olarak çalışır. Bu bağlantı olmadan — biz dahil — hiç kimse paylaştığınız bilginin şifresini çözemez.',
  },
  anatomy: {
    labels: ['Domain', 'Gizli ID', 'Şifreleme Anahtarı'],
    caption:
      'Örnek bağlantı: alan adı, gizli kimlik ve şifreleme anahtarından oluşur. Şifreleme anahtarı sunucuya hiçbir zaman gönderilmez.',
    oneTime: 'Tek kullanımlık',
    boxNumber: 'Kutu numarası · sunucuda',
    keyOnlyYou: 'Anahtar · sadece sende',
    neverServer: 'Bu parça hiçbir zaman sunucuya gitmez.',
    boxNote: 'Anahtar olmadan açılmaz.',
  },
  tagline: {
    aria: 'Güvenlik özellikleri',
    oneTime: 'Tek Seferlik Okuma',
    zeroKnowledge: 'Biz Bile Göremeyiz',
    e2e: 'Uçtan Uca Şifrelenmiş',
  },
  faq: {
    heading: 'Sıkça Sorulan Sorular',
    intro: 'Merak ettiklerinin cevabı burada. Kısaca: mesajın tarayıcında şifrelenir, bir kez açılır ve yok olur.',
    items: [
      {
        question: 'Bu hizmeti neden kullanmalıyım?',
        answer:
          'Bir şifreyi, bir anahtarı ya da özel bir notu, sohbet geçmişinde kalıcı iz bırakmadan paylaşmak için. Bağlantı yalnızca bir kez açılır; açıldığı anda not sunucudan silinir. Hesap açmanız gerekmez, e-posta adresinizi sormayız.',
      },
      {
        question: 'Nasıl çalışır?',
        answer:
          'Notunuz gönderilmeden önce tarayıcınızda şifrelenir; sunucuya yalnızca şifreli hâli gider. Şifreyi çözen anahtar, bağlantının # işaretinden sonraki kısmında durur ve tarayıcılar o kısmı hiçbir isteğe eklemez. Yani anahtar bize hiç ulaşmaz.',
      },
      {
        question: 'Notumu siz (ya da bir başkası) okuyabilir misiniz?',
        answer:
          'Hayır. Anahtar bize hiç ulaşmadığı için elimizde yalnızca açamadığımız şifreli bir metin var. Erişim kaydı (log) tutmuyoruz ve size bağlı hiçbir şey saklamıyoruz — yalnızca anonim toplamlar: kaç not oluşturulup okunduğu, siteye kaç ziyaret geldiği gibi. IP adresiniz yalnızca bellekte, kötüye kullanımı sınırlamak ve ziyaretçi sayısını tahmin etmek için kullanılır, hiçbir yere yazılmaz.',
      },
      {
        question: 'Silinen not geri gelir mi? Yedek tutuyor musunuz?',
        answer:
          'Not okunduğunda ona ait 32 baytlık anahtarı sileriz; anahtar gidince şifreli metin artık kimse için açılamaz. Hiçbir yedek, hiçbir snapshot almayız — silinen geri gelmez. Diskteki baytların fiziksel olarak yok edildiğini iddia etmeyiz; modern bir SSD’de bunu kimse kanıtlayamaz. Güvencemiz matematikseldir: anahtar yok.',
      },
      {
        question: 'Bu hizmet neyi korumaz?',
        answer:
          'Dürüst olalım: notu okuyan kişinin ekranını koruyamayız. Alıcının cihazındaki bir zararlı yazılım, bir ekran görüntüsü ya da alıcının notu okuduktan sonra kopyalaması bizim elimizde değil. Koruma, notun bizdeki yolculuğu içindir — karşı taraftaki ekran için değil.',
      },
      {
        question: 'Bağlantıyı başkası ele geçirirse ne olur?',
        answer:
          'Bağlantının kendisi anahtardır: onu ilk açan notu okur. Bunu engelleyemeyiz, engellediğimizi de söylemeyiz. Ama bir faydası var: beklediği notu “bulunamadı” olarak gören kişi, birinin ondan önce açtığını öğrenmiş olur. Bağlantı karşı tarafa ulaşmadıysa, kimseye ulaşmadığını varsaymayın.',
      },
      {
        question: 'Notu hiç açmazsam ne olur?',
        answer: 'Not ve anahtarı, hiç okunmamış olsa bile 24 saat sonra silinir.',
      },
    ],
  },
  backToTop: 'Sayfanın başına dön',
  errors: {
    tooLarge: 'Not 64 KB’tan büyük olamaz. Biraz kısaltıp tekrar dener misin?',
    offline: 'Sunucuya ulaşılamadı. Bağlantını kontrol edip tekrar dener misin?',
    throttled: 'Çok hızlı gidiyorsun. Biraz bekleyip tekrar dene.',
    rejected: 'Not gönderilemedi. Metni kısaltıp tekrar dener misin?',
    server: 'Bir şeyler ters gitti. Tekrar dener misin?',
    noCrypto: 'Tarayıcın şifreleme yapamıyor. Güncel bir tarayıcı gerekiyor.',
    copyFailed: 'Panoya kopyalanamadı. Bağlantıyı seçip elle kopyalayabilirsin.',
  },
}

/** The dictionaries, keyed by locale. */
export const DICTS: Record<Locale, Strings> = { en, tr }

function isLocale(value: string): value is Locale {
  return (SUPPORTED as readonly string[]).includes(value)
}

/** The remembered choice, if any and still supported. */
function readStoredLocale(): Locale | null {
  try {
    const value = localStorage.getItem(STORAGE_KEY)
    return value && isLocale(value) ? value : null
  } catch {
    return null
  }
}

/** Remembers a choice for next time. Best-effort: a blocked store is ignored. */
function storeLocale(locale: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, locale)
  } catch {
    // Private mode or blocked storage: detection runs again next load.
  }
}

/**
 * The locale to open in: the remembered choice first, otherwise the first of
 * the browser's language preferences we support, otherwise English.
 */
function detectLocale(): Locale {
  const stored = readStoredLocale()
  if (stored) return stored

  const prefs =
    typeof navigator === 'undefined'
      ? []
      : navigator.languages && navigator.languages.length > 0
        ? navigator.languages
        : [navigator.language]

  for (const pref of prefs) {
    const base = pref?.toLowerCase().split('-')[0]
    if (base && isLocale(base)) return base
  }
  return DEFAULT_LOCALE
}

type LocaleState = { locale: Locale; setLocale: (locale: Locale) => void }

const CopyContext = createContext<Strings>(DICTS[DEFAULT_LOCALE])
const LocaleContext = createContext<LocaleState>({ locale: DEFAULT_LOCALE, setLocale: () => {} })

/**
 * Holds the active locale and keeps `<html lang>` and the document title in
 * step with it. The initial value is detected once, on first render.
 */
export function LocaleProvider(props: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(() => detectLocale())

  useEffect(() => {
    document.documentElement.lang = locale
    document.title = DICTS[locale].metaTitle
  }, [locale])

  const setLocale = (next: Locale) => {
    storeLocale(next)
    setLocaleState(next)
  }

  return (
    <LocaleContext.Provider value={{ locale, setLocale }}>
      <CopyContext.Provider value={DICTS[locale]}>{props.children}</CopyContext.Provider>
    </LocaleContext.Provider>
  )
}

/** The active language's copy. */
export function useCopy(): Strings {
  return useContext(CopyContext)
}

/** The active locale and a setter that also remembers the choice. */
export function useLocale(): LocaleState {
  return useContext(LocaleContext)
}
