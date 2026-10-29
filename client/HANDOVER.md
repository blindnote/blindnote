# BlindNote — arayüz (React bileşenleri)

Tek kullanımlık, uçtan uca şifreli not paylaşım sitesi BlindNote'un **arayüz katmanı**.
Figma "Blindnote" dosyasındaki tasarımdan (node 2469:18109) aktarıldı ve tasarım incelemesinde
birlikte geliştirildi. **Şifreleme, ağ çağrıları ve yönlendirme bu depoda yok**; bunlar yazılımcıya bırakıldı.

- React 19 · TypeScript (strict) · Vite 8 · Tailwind CSS v4 (`@theme`)
- Ek paket yok; fontlar self-host (`public/fonts`)
- Okuma yolu (JS + CSS) brotli ile ~86 KB — hedef ≤ 90 KB

## Çalıştırma

```bash
npm install
npm run dev      # http://localhost:5173 — demo akışı + alttaki "Önizleme" paneli
npm run build    # tip kontrolü + üretim derlemesi (dist/)
```

## Klasörler

| Yol | Ne var |
|---|---|
| `src/components/` | Yalnızca **sunum bileşenleri**: veri props ile girer, olaylar callback ile çıkar. `fetch`, kripto, depolama yok. |
| `src/styles/app.css` | `@theme` tasarım değişkenleri (renk, yarıçap, gölge, animasyon) ve font tanımları |
| `src/demo/` | **GEÇİCİ.** Sahte durumla akışı gösteren demo (`DemoApp`) ve önizleme paneli. Üretimde silinir. |
| `public/fonts/` | Baloo 2 ve Raleway woff2 (OFL lisansları yanında) |

### Ekranlar (`src/components`)

| Bileşen | Figma | Ne zaman |
|---|---|---|
| `ComposeScreen` | Anasayfa | Mesaj yazma |
| `LinkReadyScreen` | Bağlantı Oluşturuldu | Link üretildi, kopyala |
| `RevealScreen` | Mesaj Aç | Alıcı linki açtı; içerik **butona basmadan çözülmez** (sohbet uygulamalarının link önizlemesi notu yakmasın) |
| `NoteOpenedScreen` | Mesaj Açıldı | Not gösteriliyor |
| `NoteNotFoundScreen` | Mesaj Bulunamadı | Link yanmış / geçersiz |
| `PageLayout` | — | Ortak iskelet: header → ekran → 3 bilgi bölümü → footer → yukarı çık butonu |

## Yazılımcının bağlaması gerekenler

Bileşenler durum tutmaz; aşağıdakiler çağıran tarafta (rota/sayfa katmanı) yapılmalı. Örnek akış `src/demo/DemoApp.tsx` içinde.

| Bileşen prop'u | Beklenen iş |
|---|---|
| `ComposeScreen.onSubmit`, `status`, `error` | Metni tarayıcıda şifrele → şifreli hâli sunucuya gönder → `#ID/ANAHTAR` linkini üret. Gönderim sürerken `status="submitting"`. Hata metni `error` ile (maymunun konuşma balonunda gösterilir). |
| `LinkReadyScreen.onCopy`, `copied`, `copyError` | `navigator.clipboard.writeText(url)`. `copied` ekrandan ayrılana kadar `true` kalmalı (tasarım kararı). |
| `RevealScreen.onReveal`, `status`, `error` | Linkteki ID ile şifreli notu çek ve sunucudan sil; anahtarla tarayıcıda çöz. |
| `NoteOpenedScreen.text`, `onCopyText`, `onReply` | Çözülmüş metin düz metin olarak basılır (HTML yorumlanmaz). |
| `PageLayout.onCreateNote`, `ctaAfterScroll`, `ctaOnMobileAlways` | Header'daki "Gizli Mesaj Oluştur / Oluştur" butonu. Yaz ekranında `ctaAfterScroll` (kutu ekrandan çıkınca görünür); diğer ekranlarda `ctaOnMobileAlways`. |

> **Güvenlik notu:** Şifreleme anahtarı linkin `#` sonrasında durur ve sunucuya **hiç gönderilmemeli**.
> Gizli ID de `#` sonrasında olduğu için ilk HTTP isteğinde gitmez; sayfa onu ayrıca göndermelidir.
