#!/usr/bin/env bash
# Save a 2.0 to 2.4 backport's docs, with reports made from its published wheels.
#
#     just archive-backport 2.4.3
#
# Such a tag runs the release.yml stored in it, which has no archive job, so main's docs build
# fails until this runs. It does that job's work by hand: downloads the release's wheels from
# PyPI, makes the four wheel reports release CI makes, then runs archive_release.sh, which
# converts the tag's pages with them. Needs macOS, for otool, and Docker, because auditwheel runs
# only on Linux. Tests: scripts/test_archive_release.py.
set -euo pipefail

VERSION=${1:?usage: archive_backport.sh X.Y.Z}
ROOT=$(git rev-parse --show-toplevel)
VENV="$ROOT/.venv-docs/bin/python"
[ -x "$VENV" ] && PYTHON=${PYTHON:-$VENV}
PYTHON=${PYTHON:-python3}
# The message, then any commands that fix it, one to a line.
fail() {
  echo "archive_backport: $1" >&2
  shift
  [ $# -eq 0 ] || printf '    %s\n' "$@" >&2
  exit 1
}

[[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "$VERSION is not X.Y.Z"
git -C "$ROOT" rev-parse --quiet --verify "refs/tags/v$VERSION" >/dev/null ||
  fail "no tag v$VERSION; run git fetch --tags"
if git -C "$ROOT" cat-file -e "v$VERSION:docs-site/src/content/docs" 2>/dev/null; then
  fail "v$VERSION has docs-site/, so its release workflow archives it; nothing to do by hand"
fi
command -v otool >/dev/null || fail "otool not found; the macOS reports need a Mac"
docker info >/dev/null 2>&1 || fail "Docker isn't running; start Docker Desktop and run this again"
"$PYTHON" -c 'import griffe, griffe2md, yaml' 2>/dev/null ||
  fail "$PYTHON lacks the API renderer. Install it once in .venv-docs, then run this again:" \
    "python3 -m venv .venv-docs" \
    ".venv-docs/bin/pip install griffe==1.14.0 griffe2md==1.2.5 pyyaml" \
    "just archive-backport $VERSION"

WHEELS=$(mktemp -d)
trap 'rm -rf "$WHEELS"' EXIT
FILES=$(curl -fsSL "https://pypi.org/pypi/eqty-sdk/$VERSION/json" |
  jq -r '.urls[] | "\(.filename) \(.url)"') || fail "PyPI has no eqty-sdk $VERSION"
# The wheel for each platform, by the part of its file name that names the platform.
wheel() {
  local name
  name=$(awk -v p="$1" '$1 ~ p { print $1; exit }' <<<"$FILES")
  [ -n "$name" ] || fail "PyPI has no $2 wheel for $VERSION"
  curl -fsSL -o "$WHEELS/$name" "$(awk -v n="$name" '$1 == n { print $2 }' <<<"$FILES")"
  echo "$name"
}

for arch in x86_64 aarch64; do
  name=$(wheel "manylinux.*_${arch}[.]whl$" "manylinux $arch")
  report="docs/generated/auditwheel-show-linux-$arch.txt"
  # The image release CI builds the wheels in, with the report script release CI runs.
  docker run --rm -v "$WHEELS:/wheels:ro" -v "$ROOT:/io" -w /io \
    "quay.io/pypa/manylinux2014_$arch" sh -c \
    "py=/opt/python/cp312-cp312/bin/python && \$py -m pip install -q auditwheel 2>/dev/null &&
     \$py scripts/generate_auditwheel_report.py /wheels/$name $report"
done
for arch in arm64 x86_64; do
  name=$(wheel "macosx.*_${arch}[.]whl$" "macosx $arch")
  unzip -q "$WHEELS/$name" -d "$WHEELS/$arch"
  otool -L "$WHEELS/$arch"/eqty_sdk/*.so >"$ROOT/docs/generated/otool-show-macos-$arch.txt"
done

PYTHON=$PYTHON "$ROOT/scripts/archive_release.sh" "$VERSION"
echo "archive_backport: commit docs-site/archive/ and open a PR to main."
