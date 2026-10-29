#!/usr/bin/env bash
# Don't trust us -- check (CLAUDE.md section 3).
#
# Given a released version, this verifies two independent things:
#   1. the published image was signed by THIS repo's release workflow, at that
#      exact tag (cosign keyless -- the identity, not just "some signature");
#   2. the image rebuilt from the source at that tag matches what was published.
#
# A hash nobody checks is theatre, so this script is the ask: run it, and the
# JavaScript your browser would run is provably the JavaScript we published.
#
# Usage:  ./verify.sh v0.1.0
# Needs:  cosign, docker (with buildx), git. Run it inside a checkout of this repo.
set -euo pipefail

IMAGE="${IMAGE:-ghcr.io/blindnote/blindnote}"
ISSUER="https://token.actions.githubusercontent.com"
WORKFLOW="https://github.com/blindnote/blindnote/.github/workflows/release.yml"

TAG="${1:-}"
if [ -z "$TAG" ]; then
  echo "usage: $0 <version-tag, e.g. v0.1.0>" >&2
  exit 2
fi
REF="${IMAGE}:${TAG#v}"

echo "==> 1/3  Verifying the signature and its identity"
cosign verify "$REF" \
  --certificate-identity "${WORKFLOW}@refs/tags/${TAG}" \
  --certificate-oidc-issuer "$ISSUER" > /dev/null
echo "    OK: signed by ${WORKFLOW} at ${TAG}"

echo "==> 2/3  Rebuilding from the source at ${TAG}"
if ! git rev-parse "refs/tags/${TAG}" > /dev/null 2>&1; then
  echo "    tag ${TAG} is not in this checkout; run 'git fetch --tags' first" >&2
  exit 1
fi
SOURCE_DATE_EPOCH="$(git log -1 --pretty=%ct "refs/tags/${TAG}")"
export SOURCE_DATE_EPOCH
git -c advice.detachedHead=false checkout --quiet "refs/tags/${TAG}"
docker buildx build \
  --build-arg "SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}" \
  --output "type=image,name=blindnote:verify-${TAG},rewrite-timestamp=true" \
  --metadata-file /tmp/blindnote-verify-meta.json \
  .

echo "==> 3/3  Comparing the rebuilt digest with the published one"
LOCAL="$(python3 -c 'import json;print(json.load(open("/tmp/blindnote-verify-meta.json"))["containerimage.digest"])')"
PUBLISHED="$(cosign triangulate --type digest "$REF" 2>/dev/null || true)"
if [ -z "$PUBLISHED" ]; then
  PUBLISHED="$(docker buildx imagetools inspect "$REF" --format '{{json .Manifest.Digest}}' | tr -d '"')"
fi
echo "    rebuilt:   ${LOCAL}"
echo "    published: ${PUBLISHED}"
if [ "$LOCAL" = "$PUBLISHED" ]; then
  echo "==> MATCH. The served code is the published code."
else
  echo "==> DIGESTS DIFFER." >&2
  echo "    This can be a reproducibility gap (a different BuildKit version)" >&2
  echo "    rather than tampering. Compare the embedded binary and the client" >&2
  echo "    bundle before drawing a conclusion, and open an issue either way." >&2
  exit 1
fi
