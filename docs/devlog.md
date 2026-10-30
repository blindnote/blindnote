# Dev log

One dated entry per working session: what was built, what was decided, what was measured.

`CLAUDE.md` is the contract and says *what* the project is. This file is the narrative and says
*how it got there* — including the decisions that were taken while writing code and the reasoning
behind them. When a decision here proves durable it is promoted into `CLAUDE.md`, and the reasoning
stays here.

Entries are append-only and in chronological order, newest last. Nothing in this file is a rule;
when it disagrees with `CLAUDE.md`, `CLAUDE.md` wins.

## 2026-10-29 — step 1: the workspace and the gates

The repository held `CLAUDE.md` and `LICENSE.md` and nothing else. This session built the Cargo
workspace, one `axum` route, and the CI gates that every later commit has to pass.

### Built

- A three-file root: `Cargo.toml` (workspace), `rust-toolchain.toml` (pinned to 1.99.1 exactly),
  `deny.toml`.
- `crates/server` with `config.rs`, `router.rs`, `lib.rs` and `main.rs`: the `blindnote` binary
  serves `GET /healthz` and shuts down gracefully on `SIGINT` or `SIGTERM`.
- `.github/workflows/ci.yml`: `fmt`, `clippy -D warnings`, `cargo test`, `cargo deny check`.
- `.gitattributes`, so that git attributes a diff hunk to the Rust function or Markdown heading it
  belongs to. That is what makes hunk-level commit splitting accurate.

### Verified, not assumed

Run against the built binary rather than only through unit tests:

| Check | Result |
|---|---|
| `GET /healthz` | `200`, `content-type: text/plain; charset=utf-8`, `cache-control: no-store` |
| `GET /does-not-exist` | `404` |
| No `Server` header | Confirmed — axum does not send one, so nothing has to be stripped |
| `SIGTERM` | Logs `draining in-flight requests`, exits `0` |
| `BLINDNOTE_BIND=nonsense` | Logs the variable name and the parser's complaint, exits `1` |

Dependency versions were read from the crates.io API rather than from memory. Two corrections came
out of that, both now in `CLAUDE.md`:

- `tower-http` stays on **0.6.11**, not the newer 0.7.1, because `axum` 0.8.9 requires `^0.6.8` and
  taking 0.7 would compile both into the dependency graph.
- `rust-embed` 8.13.0 has a **`deterministic-timestamps`** feature. That is directly relevant to
  the reproducible build in step 9 and should be switched on when the client is embedded in step 5.

### Decisions taken while writing this

**Three crates, not one.** `crates/server` exists now; `crates/store` and `crates/crypto` follow in
step 2. The split is not for compile times — it is so that somebody auditing the central claim
("burning a note destroys its key") can read two small crates instead of one large one. That is
trust-hierarchy item 2 made cheaper to check.

**Configuration from the environment, with no config file.** The binary's behaviour should be
visible in the unit that starts the container, not in a file on the host that nobody diffs.

**Config parsing is factored into pure functions.** Not stylistic: `std::env::set_var` is `unsafe`
in edition 2024, and these crates `forbid(unsafe_code)`, so a test *cannot* set an environment
variable. Either the parsing is separable from the reading, or it is untestable. A small example of
a hard rule changing a design decision somewhere nobody expected it to.

**A config error never echoes the value it rejected.** Configuration is where secrets live and
these messages are logged. There is a test asserting exactly this, so a later "helpfully include
the value" change fails CI.

**`panic = "unwind"` kept in release, deliberately.** `abort` is tempting for a process holding key
material, but `Drop` does not run on abort — which means `zeroize` would not run either. Unwinding
keeps the zeroizing destructors. A `CatchPanicLayer` goes in with the API in step 4 so a panicking
handler returns 500 instead of killing the connection pool.

**`overflow-checks = true` in release.** Wrapping arithmetic is a silent bug class, and none of this
binary's hot paths are arithmetic: the expensive work is AES in the browser and `fsync` in SQLite.

**`clippy::pedantic` on, with `doc_markdown` allowed.** The one pedantic lint that had to go wanted
"BlindNote" backticked in prose, which would make every doc comment read like code.

**`cargo audit` dropped; `cargo deny check` is the only dependency gate.** Both read the same RustSec
advisory database, and `cargo deny` also covers licences, banned crates and source registries. Now
recorded in `CLAUDE.md` §13 so it does not get "fixed" back.

**Third-party GitHub Actions pinned to commit SHAs, not tags**, and `persist-credentials: false` on
checkout. A tag is a mutable pointer, and this workflow is a step in the chain that is supposed to
prove which bytes were built. Only two actions are used, both first-party: `actions/checkout` and
`actions/cache`. `cargo-deny` is installed with `cargo install --locked` at a pinned version rather
than through a third action.

**Banned the TLS stacks in `deny.toml`** (`openssl`, `openssl-sys`, `native-tls`). TLS is terminated
by Caddy in its own container and never in this process, so one of those crates appearing in the
graph is a signal that something started making outbound connections.

**`notes` carries no creation timestamp.** This closes the design inconsistency that used to be
§12.8: `created_at_coarse` was coarsened to blunt timing correlation while `expires_at` stayed
precise and the TTL menu is public, so `expires_at − TTL` recovered the exact creation time anyway.
No code path reads a creation time — the row is deleted on burn and on sweep — so the honest fix is
to store none rather than to jitter two columns. Decided before the schema exists, which is the
cheapest possible moment.

### A wrinkle worth naming

The commit-granularity rule is one file per commit, and the goal is that each commit builds on its
own. On a greenfield workspace those two pull against each other: a crate manifest with no sources
does not build, and `lib.rs` does not compile without the modules it declares. This session chose
granularity over per-commit buildability for the bootstrap, keeping the workspace manifest, the
member manifest and `Cargo.lock` together as one genuinely inseparable commit. From step 2 onward
every commit builds, because there is something to build on.

Related: a new Rust file arrives with its `#[cfg(test)] mod tests` inside it, and a new file cannot
be hunk-split in the working tree. So a new module lands as one commit including its tests —
reverting it reverts both, which is correct. *Modifications* still split: code hunks and test hunks
get separate commits.

### Next

Step 2, the SQLite layer. It is where the product's central claim lives, so it gets proved before
anything is built on top: the two files, the PRAGMAs, the `user_version` migration runner, the
atomic burn, the expiry sweeper, and the §12.2 test that byte-scans `keys.db` after a burn to check
the 32 bytes are actually gone from the raw file.

## 2026-10-29 — steps 2 and 3: the crypto crate, the store, and two SQLite surprises

This is the session's substantial work: the layer where the product's central claim lives, built
before anything was put on top of it. Two crates, 65 tests, and two places where the design in
`CLAUDE.md` turned out to be wrong when checked against SQLite's documentation.

### Built

- `crates/crypto` — `Id` (128-bit), `NoteKey` (32 bytes), `Nonce` (96-bit), and the at-rest
  `seal`/`open` pair. 22 tests.
- `crates/store` — the two SQLite files, verified PRAGMAs, a `user_version` migration runner, the
  burn, the abuse-report path and `sweep()`. 37 tests across six integration test files.
- `crates/store/examples/fsync.rs` — the §12.1 measurement, as a committed tool rather than a
  one-off, because the interesting half of it has to be run on the VPS.

### Two things `CLAUDE.md` had wrong

**A burn cannot be one transaction.** §7.7 said the `note_keys` row was "deleted in the same
transaction" as the note row. SQLite's WAL page says the opposite, verbatim: "Transactions that
involve changes against multiple ATTACHed databases are atomic for each individual database, but
are not atomic across all databases as a set." So the design as written was not slow or
inelegant — it was impossible.

The replacement is two atomic steps in an order chosen so the window between them is harmless:

| | Step 1 | Step 2 | A crash in between leaves |
|---|---|---|---|
| **Create** | write the key | write the note | a key with nothing to open — useless, expires on its own |
| **Burn** | take the note row | take the key | the same thing |

Both directions converge on "an orphan key", which is the one leftover that costs nothing. The
other orderings both produce a note whose key is missing: a note that exists and cannot be opened,
which for a user who was told their note was created is data loss.

This is also why `note_keys` gained an `expires_at` column that duplicates the note's deadline.
With it, each file expires its own rows and there is no orphan concept at all: a key written by a
crashed create is destroyed on time without anybody consulting the note file, with no cross-file
join and no race against a note row that has not landed yet. The cost is that the key file now
knows when its rows expire. Both files sit on the same volume, so there is no threat model in which
an attacker holds one and not the other, and the independence is worth more than the secrecy of a
timestamp.

**An abandoned `RETURNING` statement may not commit.** The burn SQL is `DELETE ... RETURNING`,
stepped exactly once because it returns at most one row. SQLite's RETURNING page says the database
changes all happen "during the first invocation of sqlite3_step()" — which is reassuring — but also
that "The commit does not occur until the statement finishes." Relying on autocommit around a
statement nobody steps to exhaustion would be relying on something undocumented, so every take and
every delete is wrapped in an explicit `IMMEDIATE` transaction that is committed by hand.

### The measurement that changed a decision

`synchronous = FULL` on local APFS/NVMe, 4 KB payload, 300 iterations:

| | create (2 commits) | burn (2 commits + WAL truncate) |
|---|---|---|
| as first written | p50 **130 µs** | p50 **179 µs** |
| with `fullfsync = ON` | p50 **16.0 ms** | p50 **16.0 ms**, p99 ~25 ms |

A factor of 120. macOS `fsync()` returns once the write has reached the drive, not once the drive
has committed it; only `F_FULLFSYNC` waits for the latter, and SQLite only issues it when
`PRAGMA fullfsync` is on. So the first row is not a durability number at all — it is what
`synchronous = FULL` quietly means on a Mac without that pragma.

`fullfsync` and `checkpoint_fullfsync` are now on. Production is Linux, where `fsync()` is expected
to wait for the device and the pragma should be inert — *expected* and *should*, because that is
not yet measured, and the way to settle it is to run the committed example on the box both ways.
The test suite went from 0.02 s to 0.3 s, which is the right price for measuring the real thing.

Worth keeping in view: if the Hetzner Volume lands anywhere near 16 ms, one writer connection per
file caps creates at roughly 30 per second. That is almost certainly fine for this service, but it
is the number that would change the storage design, which is exactly why §16 puts the measurement
before the HTTP layer.

### A bug the tests caught, which is the point of writing them first

The last-view condition started as `views_left <= 1`. It reads as "one view or fewer, so this is
the final read" and it is wrong: a row with `views_left = 0` matches it, so a note that was already
spent would be served again. It is now `views_left = 1`, `create` refuses a zero-view note outright,
and two tests hold both halves — one that the API refuses it, one that a hand-written zero-view row
is still not served. The second test writes the row with raw SQL, because `create` will not produce
it; that is the only way to test a guard against a future bug.

### Decisions taken while writing this

**The key's bytes live behind a `Box`.** Moving a `[u8; 32]` held inline copies it and leaves the
old copy unscrubbed, because Rust runs no destructor on a moved-from value — so a `Drop` that
zeroizes would scrub one copy and miss every intermediate. Behind a `Box`, a move copies the
pointer, there is exactly one copy to scrub, and the constructors write randomness straight into the
allocation without it ever appearing on the stack. `Drop` is hand-written rather than derived so a
reader auditing the central claim can see the scrub without expanding a macro.

**`Debug` is redacted on `Id`, `NoteKey` and `Nonce`.** A note id is half of the secret in a link, a
key is the whole secret, and a nonce is a per-note correlation handle. All three print a
placeholder, which turns "never log a note id" from a rule every author has to remember into a
property of the type: a later `tracing::debug!(?id)` prints `Id(redacted)`.

**One `StoreError::Unavailable` for three states.** Missing, expired and already-read come out of
one branch of one function. Not three errors that happen to be formatted alike today — the caller
is given no way to tell them apart, so no caller can leak the difference. A test asserts the three
render identically, which is cheap and would catch a well-meaning split.

**The associated data binds a blob to its row.** `AAD = version byte ‖ note_id ‖ key_id`, so a blob
cannot be moved to another note's row and cannot be paired with another row's key: either
substitution stops the tag verifying. Without it, write access to the database would allow swapping
rows around undetectably. Five tests cover the substitutions.

**`getrandom` directly, with no userspace PRNG.** A reseeding generator is state that would have to
be kept, protected and zeroized, and at this call volume it buys nothing over asking the kernel.

**PRAGMAs are read back after being applied.** A `secure_delete` that silently stayed off would make
a documented guarantee false with nothing in the logs to say so, so `Store::open` fails with
`StoreError::Pragma` if `journal_mode`, `synchronous`, `fullfsync`, `secure_delete` or `temp_store`
did not take. `temp_store = MEMORY` is in that list because a temporary b-tree spilled to a file
would put row content somewhere nothing in this project ever deletes.

**`auto_vacuum = FULL` on the key file only**, and it has to be set before the first table exists —
it cannot be changed afterwards, so a regression here would be permanent for every existing
deployment. That is why there is a test asserting the key file auto-vacuums and the note file does
not.

**An older binary refuses a newer schema.** Carrying on would mean writing rows the newer schema
does not describe, into a file holding users' notes. Refusing to start is the lesser failure and it
is loud.

**The reader pool was not built.** §7.4 calls for one writer plus N readers. Creating, reading and
reporting a note are all writes, so there is no read-only code path for readers to serve; building
the pool now would be building something unused. It goes in the moment a read-only path appears.

**A poisoned mutex is recovered from, not propagated.** A poisoned connection mutex means a previous
holder panicked; the connection is still usable, because any transaction it held was rolled back
when the guard dropped. Refusing to serve from then on would turn one panic into an outage, and
these are users' notes.

**A busy checkpoint is logged, not returned.** By the time the `TRUNCATE` checkpoint runs, the key
row is already deleted and committed — failing the request would not bring the key back, and it
would show a user an error for an operation that succeeded. What must not happen is silence: a
checkpoint that gave up leaves key bytes in the log, so it warns, and the next sweep cleans up.

**Reporting an unknown id succeeds.** Every other endpoint is careful not to be an existence oracle;
a report endpoint that answered "no such note" would be the easiest one in the API, being
unauthenticated and destructive only when it hits.

### Not done, deliberately

- The disk threshold (§7.11). It needs a free-space syscall and therefore a new dependency, and it
  is listed separately from step 2. It goes in with the HTTP layer that has to refuse new notes.
- Periodic `VACUUM`. Its cadence belongs with the disk-threshold work, and an unused method is worse
  than a missing one.
- The sweeper's interval timer. The store exposes `sweep(now)`; owning a `tokio` interval is the
  server's job and lands with step 4.

### Next

Step 4, the HTTP API: create, reveal by `POST`, report, one shared error path, ids in the body and
never in a path, `405` on `GET` to the burn endpoint, and the rate limiter with `X-Forwarded-For`
trusted only from the proxy's own address.

## 2026-10-29 — step 4: the HTTP API, and three things the tests found

The contract the client will be wired to. Four routes, six layers, 45 tests over a real TCP socket.
Total across the workspace: 110 tests, `clippy -D warnings` clean, `cargo deny` clean.

### The API

| Route | Method | Answer |
|---|---|---|
| `/api/notes` | `POST` | `{"id": "<22 chars>"}` |
| `/api/reveal` | `POST` | `{"ciphertext": "<base64url>"}`, and the note is gone |
| `/api/report` | `POST` | `204`, whether or not there was a note |
| `/healthz` | `GET` | `ok` |

Anything else on those paths is `405`. Nothing is `GET`-able except the probe.

### Three bugs the tests found, which is the argument for writing them

**Successful responses had no `Cache-Control`.** The error path sets `no-store` itself, and
`/healthz` sets its own, so every test of an error or a probe passed — and the responses carrying a
note id or a note's ciphertext were cacheable. Caught by a test that checked *every* API answer
rather than the interesting ones. Fixed with a `no-store` layer scoped to the `/api` routes, scoped
rather than global because the embedded assets will want the opposite value.

**The body limit was being downgraded to a `400`.** Mapping every `JsonRejection` to one fixed
`400` — which is right, because axum's own message quotes the request body back and the body is a
note — also swallowed the `413` for an oversized body. "Send something else" and "send less" are
different instructions, and a client that cannot tell them apart retries the same oversized body
forever. The status is now kept when the rejection was about size; the *message* is still replaced,
because a status code quotes nothing.

**A test constant was not a valid id.** `"Z" * 22` looks like a 22-character base64url id and is
not one: 22 characters encode 16 bytes plus four spare bits, so the final character's low two bits
must be zero, and `Z` is `011001`. The decoder was right to refuse it and the test was wrong. Worth
recording because the strictness is deliberate — one canonical encoding per id — and the next person
to hand-write a test id will hit the same wall.

### The detail that is easy to get backwards

`X-Forwarded-For` reads `client, proxy1, proxy2`, and each hop appends the address it accepted the
connection from. So the proxy appends the only entry it observed itself: **the last one**.
Everything before it came from further out, and the first entry came from the client.

The obvious reading — "the client is first, so take the first" — is precisely the bypass. A client
that sends `X-Forwarded-For: 1.2.3.4` is then attributed to 1.2.3.4, and a fresh value per request
is a fresh bucket per request, which is no rate limit at all. Taking the last entry is correct
whether the proxy appends to a client's header or replaces it.

`BLINDNOTE_TRUSTED_PROXY` is unset by default, and unset means believe nobody. That direction is
safe against spoofing and has a trap of its own: behind a proxy, with nothing trusted, every request
is attributed to the proxy and the whole internet shares one bucket. The server now warns about this
at startup, because the failure is quiet and looks like the limiter working.

### Decisions taken while writing this

**Lifetimes are named on the wire, not counted.** `{"ttl": "10m" | "1h" | "24h"}` rather than a
number of seconds, so the permitted set is stated once in the server and cannot be widened by a
request. A client asking for a year gets a `400`, not a year.

**`404` for an unavailable note, not `410`.** "Gone" would be an admission that it once existed,
which is exactly the distinction the response exists to withhold. One `ApiError` variant produces
it, so a later "helpful" split between missing and burned would have to change that type — which is
a conversation rather than an accident. A test compares the two answers on status, body and the
headers a client can see.

**Report answers `204` either way.** An endpoint that said "no such note" would be the easiest
oracle in the API: unauthenticated, and destructive only when it hits.

**A malformed id is a `400`, an unavailable note is a `404`.** The shape of an id is public and no
note is consulted to decide it, so the distinction leaks nothing — and a client bug that looked
identical to "already read" would be miserable to debug.

**Two limiters, not one.** A flood and a slow drip are different threats: one would exhaust the
server, the other would fill the disk over a day. The burst limit sits on every API route; the
hourly create quota sits only on create, expressed as one token every `3600/N` seconds with `N`
available at once — so a person can make a handful of notes in a row, as people do.

**The limiters' state is pruned on an interval.** Governor's keyed state grows by one entry per
distinct source address and never shrinks on its own. A limiter meant to bound memory that can
instead be made to exhaust it is worse than no limiter, because it looks like protection.

**`/healthz` is outside the rate limiter and the body limit.** A throttled probe makes an
orchestrator believe the service is down, and restarting a healthy process is a worse outcome than
answering one extra probe.

**Layer order, outermost first:** security headers, caught panics, timeout, rate limit, concurrency
limit, body limit. The headers go outside the panic catcher so even the `500` it produces leaves
with them on it. The rate limit goes outside the concurrency limit so a client being throttled does
not occupy one of the slots it is being throttled for.

**`run` was split into `run` and `serve`.** `serve` takes an already-bound listener, so the caller
can learn the address first. The tests bind port zero and read back what the kernel chose; a future
deployment could be handed a socket by `systemd`. This started as something the tests needed and is
a better shape regardless — which is the only kind of test-driven API change worth making.

**Every configuration default now lives in one `Default` impl** that `from_env` reads and overrides.
Two lists of defaults would eventually disagree, and the one that mattered would be whichever the
reader did not open. It also makes `Config` constructible in a test, which matters because
`std::env::set_var` is `unsafe` in edition 2024 and these crates forbid `unsafe`.

**The permissions policy is written out in full.** There is no wildcard meaning "and anything added
later", so a feature absent from the list is one that stays enabled when the browser gains it. The
list is long on purpose; revisit it when the client changes, not when it looks long.

**`tower_governor`'s default features are off.** They include `tonic`, which would drag a whole gRPC
stack into a binary that serves four routes. `governor` is named directly only to spell out the
configuration type, which is generic over that crate's middleware and has no default for it.

**The HTTP tests go over TCP.** Not thoroughness for its own sake: the rate limiter keys on the peer
address, which only exists when there is a connection, and the layers are wired in an order a direct
router call would not exercise. The client is `hyper` plus `hyper-util`, both already in the graph
through axum, so the tests speak HTTP without adding a client library to the tree.

### A measurement, and a small fix that came out of it

The end-to-end run left `keys.db-wal` at 8 KB after a burn, which contradicted the comment saying
the log is truncated. It turned out the burn *does* truncate it to zero — a test now asserts that
directly — and the 8 KB came from the sweep that ran afterwards with nothing to delete. A statement
that matches no rows still commits, and that commit still writes frames.

Those frames carry no key material, so this was tidiness rather than a leak. But "the key log is
empty" is a claim worth being able to check at any moment rather than only in the instant after a
burn, so the sweeper now truncates unconditionally, and a test covers the empty-sweep case too.

### Not done, deliberately

- The disk threshold (§7.11). It needs a free-space syscall and a new dependency, and refusing new
  notes is a decision that belongs next to the `VACUUM` cadence.
- The CSP. It pins script hashes, so it cannot be written before there is a bundle to hash (step 7).
- Submitting the domain to the HSTS preload list. The header says `preload`; the submission needs a
  domain, and there is not one yet.

### Next

Step 5: import the designer's components, delete `src/demo/`, build with Vite, embed through
`rust-embed` with Brotli. `rust-embed` 8.13.0 has a `deterministic-timestamps` feature that the
reproducible build in step 9 will want switched on.

## 2026-10-29 — steps 5 and 6: the interface, and the measurement that checks rule 1

The designer's interface is in, wired to the API, encrypted in the browser, and baked into the
binary. 121 tests, `clippy -D warnings` clean, `cargo deny` clean, and a read path of **86,232
bytes brotli** against a 90 KB budget that is now a hard CI gate.

The one that matters most is not a test: a HAR recording of the whole flow in a real Chrome,
scanned for the key, the plaintext and the fragment. **None of the three appears in any request
url, header or body across all 17 requests.** That is hard rule 1 checked rather than trusted.

### What the two API calls carried

| Call | Body |
|---|---|
| `POST /api/notes` | `{"ciphertext":"EZpC_dhqMbp0…"}` — 91 bytes |
| `POST /api/reveal` | `{"id":"O3eomCxyF-Y-rWCbTAdUDA"}` — 31 bytes |

No key. No note id in any url. No `GET` anywhere near a note.

### Three bugs the browser found, which no unit test could

**A note link arriving without a reload did nothing.** Opening `#ID/KEY` in a tab already on the
site is a same-document navigation: the fragment changes, nothing reloads, and React's state
initializer never runs again. The address bar showed the note and the screen showed the previous
one. The app now listens for `hashchange` — but acts only on a fragment that parses as a note
link, because the site's own menu uses `#nedir`, `#sifreleme` and `#sss`, and reacting to an
anchor would throw away a note the reader had not opened yet.

**The not-found headline claimed the note never existed.** "Bu mesaj hiç olmadı." The server
cannot distinguish a note that never existed from one already read — deliberately, since that
distinction is the existence oracle every endpoint avoids — and the common case is the second.
The headline is neutral now, and the subtitle carries the thing the project's own notes ask for:
a recipient who finds an expected note already burned has learned that somebody got there first.
That is tamper evidence and a feature, so the interface says it.

**The server's payload ceiling refused a full-size note.** It was 64 KB of plaintext plus a
16-byte tag; the browser also prepends a 12-byte nonce, so the largest note the composer accepts
arrives as 64 KB + 28 bytes. A user would have been told their note was too large when it was not.
The bound is now 64 KB plus 512 bytes of framing headroom, expressed as headroom rather than a
sum: the client's exact format is not the server's business, and the bound should not need
correcting every time it changes.

One thing that looked like a bug and was not: the reveal button sat below the fold in the
automation's short viewport, so the first click landed on nothing. Worth knowing the next time a
browser run looks broken.

### Decisions taken while writing this

**`rust-embed` was dropped for a build script.** The crate embeds bytes and stops there, which
leaves Brotli compression, content types, ETags and the content security policy's script hashes
to be arranged elsewhere. One build script is less code and more reproducible: the table it
generates is sorted, and carries no timestamps and no paths from the building machine. It also
means `rust-embed`'s `deterministic-timestamps` feature is moot — there are no timestamps to make
deterministic.

**Compression is decided per file, not per extension.** A woff2 is already Brotli inside, so a
second pass gains a fraction of a percent for a doubled footprint. The build script keeps the
compressed copy only when it comes out under 95% of the original, which is one fewer list of
extensions to keep correct. Measured: the fonts keep no compressed copy, the JS and CSS do.

**Three caching rules, not one.** Content-hashed `/assets/*` are immutable for a year, because the
bytes behind one of those names never change. `index.html` is `no-store`: it names those files and
the policy that will pin their hashes is served with it, so a stale copy would reference scripts
the current policy does not allow — and it is a few hundred bytes compressed. Fonts sit in
between: stable in practice, but their names carry no hash, so they revalidate.

**The fragment is forgotten after a read and never before.** Before the read it is the only copy
of the link that survives a reload, and an in-app browser inside a chat application reloads more
often than you would think — losing the note there would look exactly like data loss. Afterwards
the key opens nothing, and leaving it in the address bar leaves it in screenshots, in a shared
screen and in whatever the browser syncs.

**A link is validated before it is used.** `parseNoteLink` checks both lengths and that both halves
decode, before any request. Reading destroys, so a truncated link has to fail *before* the burn: a
bad link should cost a trip back to wherever it came from, not the note.

**Lifetimes are named on the wire, not counted.** `{"ttl": "10m" | "1h" | "24h"}`. The permitted
set is stated once, in the server, and cannot be widened by a request.

**No console logging anywhere in the client.** Not even for a decryption failure. A browser
extension can read the console, and this is a client whose whole job is to hold a secret the
server never sees.

**A missing `client/dist` is a warning, not a build failure.** The generated table comes out empty,
`build.rs` prints a cargo warning, the server logs an error at startup, and the tests that cover
serving the interface skip themselves. A Rust change should not need a Node toolchain to prove
itself — but a binary that serves an API and no interface looks healthy from every angle except a
browser's, so it says so twice.

**`run` was split into `run` and `serve`.** `serve` takes an already-bound listener so the caller
can learn the address first; the tests bind port zero and read back what the kernel chose. It
started as something the tests needed and is a better shape regardless, which is the only kind of
test-driven API change worth making.

### Still open

- **The content security policy.** Verified against MDN during this session: a CSP hash matches an
  external script **only if the element repeats it in an `integrity` attribute**. So step 7 is not
  just a header — `build.rs` has to stamp `integrity` onto the `<script>` and `<link>` tags in
  `index.html` as well. The hashes are already computed and emitted; they are unused for now and
  marked as such.
- The interface reaches neither the `10m`/`1h` lifetimes nor the report endpoint. Both exist in
  the API and have no control in the design.
- HSTS preload submission, which needs a domain.

### Next

Step 7: the policy. `script-src 'sha256-…'` and `style-src 'sha256-…'` from the hashes the build
already has, plus the matching `integrity` attributes, plus `connect-src 'self'`,
`object-src 'none'`, `base-uri 'none'`, `form-action 'self'`, `frame-ancestors 'none'` and
`require-trusted-types-for 'script'`. Then a test that loads the page in a real browser with the
policy on and asserts the console is free of policy violations — because a policy that silently
blocks the application's own script is the failure mode here.

## 2026-10-29 — step 7: the policy, and the proof that it is not decorative

The content security policy, built from the bundle's own hashes. 130 tests, all gates clean, and
the whole flow run in a real Chrome under the policy with an **empty console**.

```
default-src 'none';
script-src 'sha256-wXIuffKtwwC5hXlVGkyiQD6aelAkT1TWvRwgOcaRA20=';
style-src  'sha256-RiDVlNQ4TVtzMQO3I9qUzTxjQoGVNd4dnaXaO0m7Cac=';
font-src 'self'; connect-src 'self';
object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none';
require-trusted-types-for 'script'
```

### The half of this that is easy to miss

A CSP hash matches an **inline** script by its text. For an **external** script it matches only
when the element repeats the same hash in an `integrity` attribute — checked against MDN before any
of this was written, because getting it wrong produces a policy that looks strict and silently
blocks the application's own bundle.

So pinning the bundle takes three places carrying one value: the `integrity` attribute in the page,
the hash of the file that is served, and the source list in the header. `build.rs` computes all
three from one set of bytes — it stamps the page, then re-hashes the page because its own bytes just
changed — and a test asserts the three agree. If they ever drift, the browser refuses the script and
the page goes blank with nothing to say why, which is the worst kind of failure to debug.

### Why the browser run is the real test

The page mounting at all is the proof. Had the stamped hash not matched the served bytes, Chrome
would have refused the script and `#root` would have stayed empty. It mounted, the note was created,
the link opened, the note revealed with its Turkish characters intact, and the console stayed empty
through all of it — so `require-trusted-types-for 'script'` does not break React 19 either, which
was the open question.

Checked beforehand rather than discovered: no `style={{…}}` attributes anywhere in the components
(so no `style-src-attr` problem), no `dangerouslySetInnerHTML` (so nothing for Trusted Types to
block), and the twelve `url()` in the CSS are all fonts (so `font-src 'self'` is enough). The nine
`data:` occurrences in the bundle turned out to be React's own object keys, not data URLs — which is
why `img-src` is absent rather than permissive.

### Decisions taken while writing this

**`default-src 'none'` first.** Every directive the policy does not name falls back to it, so a
browser feature nobody has thought of yet is denied rather than permitted. Listing what to deny is
a list that is wrong every time a browser ships something.

**`form-action 'none'`, not the `'self'` originally written down.** Every request this application
makes goes through `fetch`, so no form ever navigates, and `'none'` is the stricter truth. `'self'`
would have permitted something that never happens.

**No `img-src`, no `media-src`, no `worker-src`.** `default-src 'none'` covers them, and naming a
directive with a permissive value to "be safe" is how a policy stops meaning anything.

**One policy, not route-scoped yet.** §8.2 calls it route-scoped because the password route will
add `'wasm-unsafe-eval'` for Argon2-WASM. There is no password route, so there is one policy; the
scoping arrives with the route that needs it.

**An empty bundle pins `'none'` rather than nothing.** A binary built without `client/dist` emits
`script-src 'none'`, which is the correct policy for a binary that serves no scripts. An empty
directive would have been the same thing by accident rather than on purpose.

### Next

Step 8, password mode: Argon2id through `hash-wasm` and the route-scoped `'wasm-unsafe-eval'` that
goes with it. After that what is left needs things this session does not have — Docker for the
reproducible build, and the machine for steps 3, 9 and 10.

## 2026-10-29 — the free-space floor: the other half of "notes are never lost"

The last hard rule with an unwritten half. Rule 6 says notes are never lost and that a disk
threshold refuses *new* notes; the refusal existed only as a sentence until now. 139 tests, all
gates clean.

A full disk is the one ordinary failure that can destroy a note somebody is waiting to read:
SQLite cannot commit, the write-ahead log cannot grow, and the sweeper cannot delete. So creation
stops while there is still room to finish serving what is already stored.

### What is gated, and what is not

| Operation | On a nearly-full disk | Why |
|---|---|---|
| create | **refused**, 503 | the only operation that consumes space |
| reveal | allowed | makes room, and the note is already somebody's |
| report | allowed | makes room |
| sweep | allowed | makes room |

Four of the seven new tests are about the second column rather than the first. A floor that also
blocked reads would turn a nearly-full disk into data loss, which is the thing it exists to
prevent — so that is the half worth asserting.

### Decisions taken while writing this

**An absolute floor, not a percentage.** What matters is whether there is room to finish serving
what is stored, and that is an absolute quantity. Five percent of a 10 GB volume is 500 MB and of
a 1 GB volume is 50 MB: a percentage is a small number exactly when it needs not to be. The
default is 256 MiB, which is roughly four thousand full-size notes of headroom plus room for the
logs and a sweep.

**`f_bavail`, not `f_bfree`.** Most filesystems reserve a few percent for root. Counting that
reserve as available would mean the refusal arrives only after the writes have already started
failing, which is the opposite of the point.

**One `statvfs` per create, uncached.** A few microseconds against the two `fsync`-backed commits
a create already costs — around 16 ms each on the machine measured earlier. There is nothing to
gain from caching it and a stale answer to lose.

**A missing data directory is an error, not zero free bytes.** Zero would read as "the disk is
full" and refuse every note, which is the wrong answer to "this path does not exist". There is a
test for it, because the lazy version of this function returns zero on any failure and looks
reasonable while doing so.

**`rustix` named directly rather than a new crate.** It was already in the dependency graph
through `tempfile`, so naming it adds nothing. One call is used: `statvfs`.

**503, and no `Retry-After`.** 503 rather than 500 because it is neither the client's fault nor a
bug, and because reading an existing note still works — a client told "internal error" has no
reason to believe the second part. No `Retry-After` because how long genuinely depends on when the
stored notes expire or are read, and a guessed value is a worse answer than none.

### Still not done, and why

Periodic `VACUUM`. With `auto_vacuum = FULL` the key file already shrinks on every delete, and the
note file reuses its freed pages with `secure_delete` zeroing them, so the cost of *not* vacuuming
is disk that stays allocated rather than data that lingers. Deciding the cadence needs a measured
answer to "how much does the note file grow under real traffic", which needs real traffic.

## 2026-10-29 — the move to a real host: examining the old box, then provisioning and hardening the new one

This session left the code and went to infrastructure (CLAUDE.md step 10). Server addresses, keys and
the real deploy config stay out of this repo (section 10) and live with the operator; what follows is
the narrative and the decisions, not the coordinates.

### The old box, examined before it is destroyed

The 2020-era machine from section 9 is still live and still serving the 2021 Laravel version — the one
section 1 describes, which stored notes as plaintext. That claim was verified directly from its
database this session: the `notes` table is `id, note (text), created_at` with no encryption anywhere,
169 rows pending (ids run to ~1481, so the one-time-read deletes have been working), and the content is
test strings and bot spam, no real user secret. The export was taken three ways (a `mysqldump`, JSON,
and a readable listing) and handed to the operator before anything is torn down. The box has no
provider backups and keeps its data on the root disk, matching section 9's intent.

Getting in was its own lesson: the key the project SSH config names was still authorized but login
failed, and a web console's keyboard layout had silently corrupted a pasted key (`+`→`=`, `@`→`2`).
The reliable path was the provider's rescue system over the API — and even there, writing to the
mounted root filesystem wedged in uninterruptible I/O while reads ran fine, which is one more reason
the plan is to replace this disk, not nurse it.

### The new box: provisioned, then hardened before it serves anything

A fresh current-generation machine (the section 9 "CX22 class" successor), Ubuntu 24.04 LTS, built
with key-only access. Hardening was applied in stages, testing login between each so a mistake could
never lock the operator out:

- **SSH.** An unprivileged admin user with its key and `NOPASSWD` sudo first, tested, *then* root login
  disabled. `AllowUsers` pinned to that one account, passwords and keyboard-interactive off, X11 and
  all forwarding off, `MaxAuthTries 3`. Crypto narrowed to AEAD ciphers, ETM MACs, and
  curve25519 + `sntrup761x25519` key exchange; host and user keys limited to ed25519. The drop-in is
  named to sort before cloud-init's so its values win (sshd takes the first occurrence of each
  keyword).
- **Core dumps off** (section 9, item 5 — a crash must never write a key to disk): `core_pattern` set
  to `|/bin/false`, `suid_dumpable=0`, a `* hard core 0` limit, systemd coredump storage none, and
  apport disabled. This is the host-side companion to the crypto-shredding guarantee.
- **sysctl**: `kptr_restrict`, `dmesg_restrict`, `yama.ptrace_scope`, the `protected_*` fs settings,
  reverse-path filtering, redirects and source-routing off, SYN cookies on. `ip_forward` is left
  alone on purpose — Docker needs it, and setting it to 0 would break container networking.
- **ufw** default-deny inbound with 22/80/443, SSH allowed before the firewall was enabled.
  **fail2ban** on the sshd jail.

A further section 9 network-hardening step is tracked privately.

### Containerized, and the image proven to run

The server is one binary that embeds the client, so the image is a two-step build (client, then the
binary that bakes it in) landing in a distroless non-root runtime — 45 MB total. It builds from the
pinned toolchain and runs read-only-rootfs with a tmpfs `/tmp` and the data on a mounted volume; a
smoke test confirmed `/healthz` 200, the embedded client served, `GET` on the burn route 405, a clean
log, and the two SQLite files created by uid 65532 under the read-only root. Base images are pinned by
tag now; digest pinning and `SOURCE_DATE_EPOCH` belong to the reproducible-build + cosign work (step 9)
that comes next, alongside the Caddy-fronted compose and the DNS cutover to the new box.

## 2026-10-29 — the trust chain, made real: signed image, pull-model deploy

Step 9 and the rest of step 10. The image now exists as a signed artifact and the
server runs it the way section 3 and section 9 describe, end to end.

### What was built

- **The release workflow** (`.github/workflows/release.yml`): on a `v*` tag it builds
  the image, pushes it to GHCR by digest with SLSA provenance and an SBOM, then signs it
  **keyless with cosign** — the signature carries the workflow's own identity
  (`…/release.yml@refs/tags/vX`), which is the string the server pins. Third-party actions
  are pinned to commit SHAs; supply chain is the product, not an afterthought.
- **`verify.sh`**: the section 3 ask made runnable — check the signature *and its identity*,
  then rebuild from the tagged source and compare digests. A hash nobody checks is theatre.
- **Base images pinned by digest** in the Dockerfile, and `SOURCE_DATE_EPOCH` driven from the
  tag commit so the build is reproducible rather than merely pinned.

### The deploy is pull-only, and verified before it runs

`v0.1.0` was tagged; the workflow produced and signed
`ghcr.io/blindnote/blindnote@sha256:2649a81…`. On the host, cosign verified that digest
against the pinned identity (signature valid, certificate from the trusted CA, presence in
the transparency log confirmed) **before** anything was pulled, and only then did the compose
stack switch from the locally-built image to the signed digest — both app and Caddy now run
by digest, not by tag. A `systemd` timer repeats exactly that on a schedule: resolve the
latest signed digest, verify the signature and that it came from our release workflow at a
version tag, and deploy only if that passes. Nothing authorized ever connects inward — the
box reaches out, verifies, and updates itself.

The private GHCR package is pulled with a read-only token held on the host (the operator's
decision for the pre-launch window); it can pull the image and nothing else. One further section 9 hardening step is tracked privately. What remains before launch is the DNS cutover to the new host, the section 11 docs
(`README`, `SECURITY.md`, `security.txt`, abuse policy), decommissioning the old box, and
the history secret-scan before the repo goes public.

## 2026-10-29 — the favicons, and the two CSP directives they needed

The page shipped without any favicon at all: no `<link rel="icon">`, nothing in
`public/`. A tab showed the browser's blank-document glyph. This closes that,
with the modern icon set rather than a lone `.ico`.

### What was added

- **One mark, generated from the logo.** The brand's see-no-evil monkey is the
  right face for a service called BlindNote, and it already exists as the
  `LOGO_MONKEY` mark the designer kept apart from the illustration set. The
  favicons are generated *from that source* (`client/src/components/Logo.tsx`),
  its CSS custom properties substituted for their literal hex, so the icon can
  never drift from the logo. Rendered with `rsvg-convert`; the `.ico` assembled
  with ImageMagick.
- **The files** (`client/public/`): `icon.svg` (the scalable primary — a cream
  squircle tile with the monkey at 62.5% so it survives 16px), `favicon.ico`
  (16/32/48 in one file), `apple-touch-icon.png` (180, opaque square — iOS masks
  the corners itself), `icon-192.png` and `icon-512.png` (manifest, purpose
  `any`), `icon-maskable-512.png` (the monkey kept inside the central 80% safe
  zone), and `manifest.webmanifest`.
- **Four links in `index.html`** — the `.ico` at 32×32, the SVG, the apple-touch
  icon, the manifest — plus a `theme-color` meta. The evilmartians minimal set,
  not a twenty-tag pile.

### The two directives it needed

The policy is `default-src 'none'`, which also governs images and the manifest
when they are not named — so the icons the browser went to fetch were refused by
our own CSP. Two directives, both `'self'`, fix it and widen nothing to a third
party: `img-src 'self'` (favicons and the apple-touch icon are `img-src`, which
is not obvious) and `manifest-src 'self'`. A same-origin image cannot be an
exfiltration beacon, which is the whole point of the directive. The build also
learned one content type: `.webmanifest` → `application/manifest+json`.

### Verified

`cargo test -p blindnote-server` green. Against the running binary: the policy
carries both new directives, all seven assets return 200 with the right content
type (`image/svg+xml`, `image/vnd.microsoft.icon`, `application/manifest+json`
among them), and a real browser loaded the page with an empty console — no CSP
refusal for any icon or the manifest. Shipped as `v0.1.3`.

## 2026-10-29 — two languages, chosen by the browser, not the IP

The interface shipped Turkish-only. This makes it bilingual — English and Turkish —
and opens in the visitor's own language, with English as the default.

### How the language is chosen

Detection is client-side and private by design. The order is: a remembered choice
in `localStorage` first, then the first of `navigator.languages` we support, then
English. **Not the IP address** — an IP lookup needs a third party or server-side
geolocation, it is wrong behind a VPN (the operator's own case), and this service
logs nothing about who loaded the page. `navigator.languages` is the honest signal
and keeps the static-binary, no-log model intact. A small segmented switcher in the
header and footer lets anyone override; the choice is remembered and `<html lang>`
and the document title follow it.

### How the copy is wired

No i18n library — the dependency list stays `react` + `react-dom` (every package is
a line in the CSP and an entry in the reproducible build). Instead `src/lib/i18n.tsx`
holds one typed `Strings` shape and both dictionaries, a `LocaleProvider`, and a
`useCopy()` hook. Components already used React context for their design variants, so
reading copy from context is idiomatic here rather than a new pattern. Every piece of
interface copy — the five screens, the three marketing sections, the nav, the error
messages, the aria labels — now comes from the dictionary; the old `src/lib/content.ts`
(FAQ and error strings) folded into it and was removed. UI copy stays its own language
on both sides, per the content rule.

### Verified

`tsc -b` (strict) and `cargo test -p blindnote-server` green. Read path is **89,098 B
brotli** against the 92,160 B gate — i18n added ~2.9 KB and there is ~3 KB of headroom.
In a real browser: an `en` browser opened in English (`<html lang="en">`, English h1
and nav); clicking the Turkish toggle flipped every string, set `lang="tr"`, updated
the title, and persisted the choice to `localStorage`; the console stayed empty.
Shipped as `v0.1.4`.

### One thing to decide (content, not code)

The trust badge reads "İsviçre Korumalı" / "Swiss-Protected", but the server runs in
Germany (Hetzner, nbg1). I translated it faithfully rather than silently changing the
claim, but for a product whose whole pitch is "we don't lie to you" (section 1) this is
worth a deliberate decision: correct the wording, or drop the badge. Flagged, not fixed.

## 2026-10-30 — honest trust badges, and a warmer placeholder

Three small composer changes the operator art-directed live, and the resolution of the
badge decision the `v0.1.4` entry flagged.

### The "Swiss-Protected" badge is gone

It was false: the server runs in Germany (Hetzner, nbg1), not Switzerland. And writing
"German-Protected" instead buys nothing — Germany's data protection is strong, but it
carries none of the "Swiss privacy" brand signal, so the swap would trade a lie for an
oddity. The real differentiator here was never geography; it is technical (section 1/3):
the server cannot read the note. So the badge now leads with that truth.

The trust row is now **One-Time Read · Zero-Knowledge · End-to-End Encrypted**. "No Trace"
also went — its meaning is already covered by zero-knowledge (the server sees nothing) and
one-time-read (nothing is left afterwards), and dropping it let the product's actual
identity, self-destruction, take the lead slot. The keys were renamed `noTrace`/`swiss` →
`oneTime`/`zeroKnowledge` across the `Strings` type, both dictionaries, and both `Tagline`
usages (badge list and the plain-text variant) — one inseparable change, since the rename
has to compile as a unit. No new icons: the existing eye-off / shield / lock are reused.

### Placeholder and spacing

The compose placeholder was a list of examples ("Wi-Fi password, private notes…"); it now
reads **"For their eyes only…"** / **"Yalnızca onun gözleri için…"**, a nod to the
see-no-evil mark. And the trust badges sat jammed under the create button, so the desktop
stack gap went from 12 px to 24 px.

### Verified

`tsc -b` (strict) and `cargo test` green. Read path **89,016 B brotli** against the
92,160 B gate (the copy is slightly shorter than before, so headroom grew a touch). In a
real browser, both languages: the new placeholder and the three badges render, the badges
sit on one row with clear space below the button, and the Turkish strings (Tek Seferlik
Okuma · Sıfır Bilgi · Uçtan Uca Şifrelenmiş) fit without overflow. Shipped as `v0.1.5`.

## 2026-10-30 — designer review: UI findings, and a real E2E suite

A design/QA pass (seven findings, desktop + iOS) came back. All were legitimate and none
touched a hard rule; the fixes shipped as `v0.1.6`, and — the larger change — the project
grew an end-to-end test suite, because it had none.

### The findings

- **D-03 (zero-knowledge badge).** The icon was a shield-with-cross that read as a medical
  mark; it is now `server-off`. The Turkish "Sıfır Bilgi" is the term for a zero-knowledge
  *proof* and said nothing to a lay reader, so it became **"Biz Bile Göremeyiz"** — the plain
  consequence: the note is encrypted in the browser, so the operator cannot read it. English
  "Zero-Knowledge" stays as the term its audience knows; the two are localized, not translated.
- **D-04 (scroll-spy).** The nav links scrolled to sections but never marked the one in view.
  A small `useActiveSection` hook (IntersectionObserver, no dependency) now sets `aria-current`
  and a brown underline on the section under the header. The first cut highlighted one section
  behind — a wide trigger band let the previous section's lower edge linger — fixed by taking
  the *bottommost* section touching the band, not the topmost.
- **D-05 (clickable envelope).** The reveal envelope is now a real `<button>` (keyboard for
  free, a light hover/press scale that respects `prefers-reduced-motion`). Reading burns the
  note, so a second trigger must not fire a second burn: the guard is a `useRef` in `App`,
  set before the request and cleared in `finally`, which makes the reveal idempotent no matter
  how many controls call it.
- **D-01 / D-02 / M-01.** The header create button is filled on the home screen and outline
  elsewhere; the desktop nav is centered with a three-column grid; the language switcher moved
  into the mobile header (left of the hamburger) rather than living only inside the menu.
- **M-02 (iOS auto-zoom).** The composer textarea was `text-sm` (14 px), and iOS Safari zooms
  any field below 16 px on focus. It is now `text-base` (16 px) on mobile, `md:text-xl`
  unchanged, plus a `blur()` before the screen changes so the keyboard dismisses. No
  `maximum-scale=1` — that would also kill pinch-zoom (accessibility).

### The E2E suite

The repo had solid Rust tests and *zero* client or end-to-end tests — the worst gap for a
browser-delivered E2EE product, where the whole claim is about what the browser does and does
not send. Playwright (a dev-dependency only; it touches neither the runtime bundle nor the
CSP) now drives Chromium against **the real server binary with the client embedded in it** —
the shipped artifact, started by `e2e/serve.sh` on a throwaway data directory — not a dev
server.

Thirteen tests, the load-bearing ones first:

- **Hard rule 1, automated.** The key and the plaintext appear in no request body, no API URL
  and no `Referer`, across the whole compose → reveal flow. This is the §12.9 HAR scan turned
  into a gate rather than a one-off.
- **Hard rule 3.** `GET /api/reveal` is 405; loading a link fires no reveal — only the click
  does.
- **Hard rule 4.** A missing note and an already-read note return byte-identical 404s.
- **One-time.** A link opens once, then reads as not-found.
- **D-05 single-fire.** Firing the envelope and the button synchronously still sends exactly
  one burn.
- Plus the happy path, both languages' badges and placeholder, the scroll-spy `aria-current`,
  the 16 px textarea, the mobile-header switcher, and no horizontal overflow at 320 px.

CI gained an `e2e` job that builds the client, bakes it into the binary and runs the suite.

### Verified

Full gate green before release: `cargo fmt`, `clippy -D warnings`, 139 Rust tests, 13 E2E
tests, and the read path at **89,684 B brotli** against the 92,160 B budget (2,476 B spare;
the scroll-spy, the envelope button and the extra copy together cost ~670 B). Shipped as
`v0.1.6`.
