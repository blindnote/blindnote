# client

The browser side of BlindNote: the designer's presentation components, plus the
encryption, network and routing they were written to be wired into.

`HANDOVER.md` is the designer's own handover document, kept verbatim in Turkish
as it was written. It describes the components and the contract this directory
fulfils. Where the two disagree, this file is newer.

## Layout

| Path | What is in it |
|---|---|
| `src/components/` | **Presentation only**, from the designer: data in through props, events out through callbacks. No `fetch`, no crypto, no storage. Imported as handed over |
| `src/styles/app.css` | `@theme` design tokens and the font declarations |
| `src/lib/` | Ours: encryption, the API calls, link parsing, and the interface copy |
| `src/App.tsx` | Ours: the whole of the routing and state |
| `public/fonts/` | Baloo 2 and Raleway as woff2, with their OFL licences |

`src/demo/` was the designer's temporary preview harness and was deleted on
import, as their document said it would be.

## Building

```bash
npm ci
npm run build    # type-check, then produce dist/
npm run dev      # http://localhost:5173, with no server behind it
```

`npm run dev` serves the interface but not the API, so composing a note fails
at the network call. Run the Rust binary for the whole flow: it embeds `dist/`
and serves it together with `/api`.

## What the browser does, and does not do

- **Encryption is here and only here.** A note is encrypted with AES-256-GCM
  before it is sent, using the native WebCrypto implementation. There is no
  crypto library: `crypto.subtle` is in every browser this site supports, and
  every added dependency is another line in the content security policy and
  another entry in the reproducible build.
- **The key never leaves.** It is exported into the URL fragment, which
  browsers do not send in an HTTP request. The server holds ciphertext it
  cannot read.
- **The note id is in the fragment too**, which is why the id travels in a
  request *body* rather than a path: a path id would reach the server's logs
  and `Referer`, and a link-preview bot following it could burn the note before
  the recipient saw it.
- **A link is validated before it is used.** Reading a note destroys it, so a
  truncated or mistyped link is rejected before the request that burns it. A
  bad link costs a trip back to wherever it came from; a bad link that burned
  would cost the note.
- **There are no runtime dependencies beyond React.** No router: which screen
  to show follows from one question, and that is not worth a package.

## Two things the design does not cover

- **No expiry picker.** Every note gets the 24-hour default, because there is
  no control for choosing otherwise. The server accepts `10m` and `1h` already.
- **No way to report a link.** The endpoint exists and destroys a note without
  decrypting anything; nothing in the interface reaches it yet.
