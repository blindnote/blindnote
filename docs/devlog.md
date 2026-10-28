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
