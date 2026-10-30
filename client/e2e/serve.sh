#!/usr/bin/env bash
# Build the client, bake it into the server binary, and run that binary against a
# throwaway data directory. The E2E tests then exercise the embedded client --
# the artifact that actually ships -- rather than a dev server (CLAUDE.md s3).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../.." && pwd)"
port="${BLINDNOTE_E2E_PORT:-8099}"

( cd "$root/client" && npm run build )
cargo build --quiet -p blindnote-server --manifest-path "$root/Cargo.toml"

bin="$root/target/debug/blindnote"
[ -x "$bin" ] || bin="$root/target/debug/blindnote-server"

data="$(mktemp -d)"
trap 'rm -rf "$data"' EXIT

export BLINDNOTE_BIND="127.0.0.1:${port}"
export BLINDNOTE_DATA_DIR="$data"
# The suite fires bursts the production limits would throttle; raise them so a
# test measures behaviour, not the rate limiter (which has its own Rust tests).
export BLINDNOTE_REQUESTS_PER_SECOND=100000
export BLINDNOTE_REQUEST_BURST=100000
export BLINDNOTE_CREATES_PER_HOUR=1000000

exec "$bin"
