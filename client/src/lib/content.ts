/**
 * Interface copy that is content rather than design: the questions and answers
 * the page carries, and the error messages a user reads.
 *
 * Turkish, because this is what a person reads. Everything else in the
 * repository is English.
 *
 * The answers are written against what the service actually does, not against
 * the strongest claim it could make. The 2021 version of BlindNote called
 * itself end-to-end encrypted while storing plaintext; being careful here is
 * the whole reason this one exists.
 */

import type { FaqEntry } from '../components/Faq'

export const FAQ_ITEMS: ReadonlyArray<FaqEntry> = [
  {
    question: 'Bu hizmeti neden kullanmalıyım?',
    answer:
      'Bir şifreyi, bir anahtarı ya da özel bir notu sohbet geçmişinde iz bırakmadan paylaşmak için. ' +
      'Bağlantı bir kez açılır; açıldığı anda not sunucudan silinir.',
  },
  {
    question: 'Servis kimin için?',
    answer:
      'Hassas bir bilgiyi tek seferlik paylaşması gereken herkes için. Hesap açmanız gerekmiyor, ' +
      'e-posta adresinizi sormuyoruz.',
  },
  {
    question: 'Servis nasıl çalışır?',
    answer:
      'Notunuz gönderilmeden önce tarayıcınızda şifrelenir; sunucuya yalnızca şifreli hâli gider. ' +
      'Şifreyi çözen anahtar, bağlantının # işaretinden sonraki kısmında durur ve tarayıcılar o kısmı ' +
      'hiçbir isteğe eklemez. Yani anahtar bize hiç ulaşmaz.',
  },
  {
    question: 'Bu hizmet ne kadar güvenli?',
    answer:
      'Anahtar sunucuya hiç gönderilmediği için notun içeriğini biz de göremiyoruz. Not okunduğunda ' +
      'ona ait 32 baytlık anahtarı siliyoruz; anahtar gidince şifreli metin artık kimse için ' +
      'açılamaz. Diskteki baytların fiziksel olarak yok edildiğini iddia etmiyoruz — modern bir ' +
      'SSD’de bunu kimse kanıtlayamaz. Güvencemiz matematiksel: anahtar yok.',
  },
  {
    question: 'Bağlantıyı başkası ele geçirirse ne olur?',
    answer:
      'Bağlantının kendisi anahtardır: onu ilk açan notu okur. Bunu engelleyemeyiz ve ' +
      'engellediğimizi söylemeyeceğiz. Ama bir faydası var: beklediği notu “bulunamadı” olarak gören ' +
      'kişi, birinin ondan önce açtığını öğrenmiş olur. Bağlantı karşı tarafa ulaşmadıysa, kimseye ' +
      'ulaşmadığını varsaymayın.',
  },
  {
    question: 'Notu hiç açmazsam ne olur?',
    answer: 'Not ve anahtarı 24 saat sonra, hiç okunmamış olsa bile silinir.',
  },
]

/**
 * Error messages, keyed by what went wrong.
 *
 * Every one tells the reader what to do next, and none of them repeats
 * anything the reader typed.
 */
export const MESSAGES = {
  tooLarge: 'Not 64 KB’tan büyük olamaz. Biraz kısaltıp tekrar dener misin?',
  offline: 'Sunucuya ulaşılamadı. Bağlantını kontrol edip tekrar dener misin?',
  throttled: 'Çok hızlı gidiyorsun. Biraz bekleyip tekrar dene.',
  rejected: 'Not gönderilemedi. Metni kısaltıp tekrar dener misin?',
  server: 'Bir şeyler ters gitti. Tekrar dener misin?',
  noCrypto: 'Tarayıcın şifreleme yapamıyor. Güncel bir tarayıcı gerekiyor.',
  copyFailed: 'Panoya kopyalanamadı. Bağlantıyı seçip elle kopyalayabilirsin.',
} as const
