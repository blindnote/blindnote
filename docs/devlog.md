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
