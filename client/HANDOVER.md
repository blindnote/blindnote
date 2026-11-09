# BlindNote — interface (React components)

The **interface layer** of BlindNote, the one-time, end-to-end encrypted note-sharing site.
Ported from the design in the Figma "Blindnote" file (node 2469:18109) and developed together
during design review. **Encryption, network calls and routing are not in this repo**; those are left to the engineer.

- React 19 · TypeScript (strict) · Vite 8 · Tailwind CSS v4 (`@theme`)
- No extra packages; fonts are self-hosted (`public/fonts`)
- Read path (JS + CSS) ~86 KB brotli — target ≤ 90 KB

## Running

```bash
npm install
npm run dev      # http://localhost:5173 — demo flow + the "Preview" panel at the bottom
npm run build    # type check + production build (dist/)
```

## Folders

| Path | What's there |
|---|---|
| `src/components/` | **Presentational components only**: data in via props, events out via callbacks. No `fetch`, crypto or storage. |
| `src/styles/app.css` | `@theme` design variables (color, radius, shadow, animation) and font definitions |
| `src/demo/` | **TEMPORARY.** The demo (`DemoApp`) that shows the flow with fake state, plus the preview panel. Deleted in production. |
| `public/fonts/` | Baloo 2 and Raleway woff2 (with their OFL licenses alongside) |

### Screens (`src/components`)

| Component | Figma | When |
|---|---|---|
| `ComposeScreen` | Anasayfa | Writing a message |
| `LinkReadyScreen` | Bağlantı Oluşturuldu | Link generated, copy |
| `RevealScreen` | Mesaj Aç | The recipient opened the link; the content is **not decrypted until the button is pressed** (so chat apps' link previews do not burn the note) |
| `NoteOpenedScreen` | Mesaj Açıldı | The note is shown |
| `NoteNotFoundScreen` | Mesaj Bulunamadı | The link is burned / invalid |
| `PageLayout` | — | The common skeleton: header → screen → 3 info sections → footer → back-to-top button |

## What the engineer must wire

The components hold no state; the following must be done on the calling side (the route/page layer). An example flow is in `src/demo/DemoApp.tsx`.

| Component prop | Expected work |
|---|---|
| `ComposeScreen.onSubmit`, `status`, `error` | Encrypt the text in the browser → send the ciphertext to the server → build the `#ID/KEY` link. While submitting, `status="submitting"`. Error text via `error` (shown in the monkey's speech bubble). |
| `LinkReadyScreen.onCopy`, `copied`, `copyError` | `navigator.clipboard.writeText(url)`. `copied` must stay `true` until the screen is left (design decision). |
| `RevealScreen.onReveal`, `status`, `error` | Fetch the ciphertext by the ID in the link and delete it from the server; decrypt in the browser with the key. |
| `NoteOpenedScreen.text`, `onCopyText`, `onReply` | The decrypted text is printed as plain text (no HTML is interpreted). |
| `PageLayout.onCreateNote`, `ctaAfterScroll`, `ctaOnMobileAlways` | The header's create button ("Gizli Mesaj Oluştur / Oluştur"). On the compose screen `ctaAfterScroll` (visible once the box scrolls off); on other screens `ctaOnMobileAlways`. |

> **Security note:** The encryption key sits after the `#` in the link and must **never be sent to the server**.
> The secret ID is also after the `#`, so it does not go in the first HTTP request; the page must send it separately.
