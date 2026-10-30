#!/usr/bin/env bash
# Don't trust us -- check (CLAUDE.md section 3).
#
# Given a released version, this verifies two independent things:
#   1. the published image was signed by THIS repo's release workflow, at that
#      exact tag (cosign keyless -- the identity, logged in the Rekor
#      transparency log; not just "some signature");
#   2. the client your browser actually runs is byte-for-byte the client built
#      from the source at that tag: every served asset hashes to what the source
#      produces, and the served page pins those same hashes (SRI / CSP), so the
#      browser would refuse anything else.
#
# The whole image is signed and carries SLSA provenance + an SBOM (see the
# release workflow). The server *binary* is not yet bit-for-bit reproducible, so
# this checks what matters most for a browser-delivered app: the code your
# browser executes. A hash nobody checks is theatre, so this script is the ask.
#
# Usage:  ./verify.sh v0.1.7 [host]      # host defaults to blindnote.com
# Needs:  cosign, node + npm, git, curl, openssl. Run inside a checkout of this repo.
set -euo pipefail

IMAGE="${IMAGE:-ghcr.io/blindnote/blindnote}"
ISSUER="https://token.actions.githubusercontent.com"
WORKFLOW="https://github.com/blindnote/blindnote/.github/workflows/release.yml"

TAG="${1:-}"
HOST="${2:-blindnote.com}"
if [ -z "$TAG" ]; then
  echo "usage: $0 <version-tag, e.g. v0.1.7> [host]" >&2
  exit 2
fi

echo "==> 1/3  Verifying the signature and its identity"
cosign verify "${IMAGE}:${TAG#v}" \
  --certificate-identity "${WORKFLOW}@refs/tags/${TAG}" \
  --certificate-oidc-issuer "$ISSUER" > /dev/null
echo "    OK: the published image is signed by ${WORKFLOW} at ${TAG}"

echo "==> 2/3  Building the client from the source at ${TAG}"
if ! git rev-parse "refs/tags/${TAG}" > /dev/null 2>&1; then
  echo "    tag ${TAG} is not in this checkout; run 'git fetch --tags' first" >&2
  exit 1
fi
# A linked worktree so the current branch and working tree are never touched.
work="$(mktemp -d)"
cleanup() { git worktree remove --force "$work" > /dev/null 2>&1 || true; rm -rf "$work"; }
trap cleanup EXIT
git worktree add --quiet --detach "$work" "refs/tags/${TAG}"
( cd "$work/client" && npm ci --no-audit --no-fund > /dev/null 2>&1 && npm run build > /dev/null 2>&1 )
built="$work/client/dist"

echo "==> 3/3  Comparing https://${HOST} with the bytes built from source"
sha() { openssl dgst -sha256 -binary "$1" | xxd -p -c 256; }
b64() { openssl dgst -sha256 -binary "$1" | openssl base64 -A; }

fail=0
served_index="$(curl -fsS "https://${HOST}/")"

# Vite names assets by a hash of their contents, so the served site must carry
# the exact files the source built -- and each must be byte-identical.
for f in "$built"/assets/*; do
  name="$(basename "$f")"
  tmp="$(mktemp)"
  if ! curl -fsS "https://${HOST}/assets/${name}" -o "$tmp"; then
    echo "    MISSING on the server: /assets/${name}" >&2; fail=1; rm -f "$tmp"; continue
  fi
  if [ "$(sha "$f")" = "$(sha "$tmp")" ]; then
    echo "    OK   byte-identical: /assets/${name}"
  else
    echo "    FAIL differs:        /assets/${name}" >&2; fail=1
  fi
  rm -f "$tmp"
  # The page must pin this asset's hash (CSP for scripts, SRI integrity for both),
  # so the browser rejects a substitution.
  case "$name" in
    *.js|*.css)
      h="$(b64 "$f")"
      if printf '%s' "$served_index" | grep -q "sha256-${h}"; then
        echo "    OK   page pins sha256-${h}"
      else
        echo "    FAIL page does not pin sha256-${h} (${name})" >&2; fail=1
      fi
      ;;
  esac
done

if [ "$fail" -eq 0 ]; then
  echo "==> MATCH. The code your browser runs is the code published at ${TAG}."
else
  echo "==> MISMATCH." >&2
  echo "    If the live site is running a different release than ${TAG}, verify that tag." >&2
  echo "    Otherwise the served client is not the published one -- open an issue." >&2
  exit 1
fi
