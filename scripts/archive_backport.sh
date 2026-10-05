#!/usr/bin/env bash
# Save a 2.0 to 2.4 backport's docs, with reports made from its published wheels.
#
#     just archive-backport 2.4.3
#
# Such a tag runs the release.yml stored in it, which has no archive job, so main's docs build
# fails until this runs. It does that job's work by hand: downloads the release's wheels from
# PyPI, makes the four wheel reports release CI makes, then runs archive_release.sh, which
# converts the tag's pages with them. Needs macOS, for otool, and Docker, because auditwheel runs
# only on Linux. Everything that can be checked is checked before anything is downloaded, and a
# failure leaves the checkout as it was. Tests: scripts/test_archive_release.py.
set -euo pipefail

VERSION=${1:?usage: archive_backport.sh X.Y.Z}
ROOT=$(git rev-parse --show-toplevel)
VENV="$ROOT/.venv-docs/bin/python"
[ -x "$VENV" ] && PYTHON=${PYTHON:-$VENV}
PYTHON=${PYTHON:-python3}
# The folders archive_release.sh rewrites, and on a backport resets.
OWNED=(docs/generated docs-site/archive)
# One wheel to a line: the platform in its file name, its architecture, the report it makes.
PLATFORMS="manylinux x86_64 auditwheel-show-linux-x86_64.txt
manylinux aarch64 auditwheel-show-linux-aarch64.txt
macosx arm64 otool-show-macos-arm64.txt
macosx x86_64 otool-show-macos-x86_64.txt"

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
# A group's folder holds its newest patch, so an older one has nothing to save.
NEWEST=$(git -C "$ROOT" tag --list "v${VERSION%.*}.*" |
  sed -nE 's/^v([0-9]+\.[0-9]+\.[0-9]+)$/\1/p' | sort -t. -k3,3n | tail -n 1)
[ "$NEWEST" = "$VERSION" ] ||
  fail "$NEWEST is newer than $VERSION, and only the newest patch is saved. Run this instead:" \
    "just archive-backport $NEWEST"
[ -z "$(git -C "$ROOT" status --porcelain -- "${OWNED[@]}")" ] ||
  fail "this rewrites docs/generated and docs-site/archive, which have uncommitted changes." \
    "Commit or stash them first; git status lists them."
command -v otool >/dev/null || fail "otool not found; the macOS reports need a Mac"
docker info >/dev/null 2>&1 || fail "Docker isn't running; start Docker Desktop and run this again"
"$PYTHON" -c 'import griffe, griffe2md, yaml' 2>/dev/null ||
  fail "$PYTHON lacks the API renderer. Install it once in .venv-docs, then run this again:" \
    "python3 -m venv .venv-docs" \
    ".venv-docs/bin/pip install griffe==1.14.0 griffe2md==1.2.5 pyyaml" \
    "just archive-backport $VERSION"

FILES=$(curl -fsSL "https://pypi.org/pypi/eqty-sdk/$VERSION/json" |
  jq -r '.urls[] | "\(.filename) \(.url)"') || fail "PyPI has no eqty-sdk $VERSION"
# Each platform's wheel, found before any is downloaded: platform, arch, report, name, URL.
WANTED=
while read -r platform arch report; do
  found=$(awk -v p="$platform.*_${arch}[.]whl$" '$1 ~ p { print $1, $2; exit }' <<<"$FILES")
  [ -n "$found" ] || fail "PyPI has no $platform $arch wheel for $VERSION"
  WANTED+="$platform $arch $report $found"$'\n'
done <<<"$PLATFORMS"

# Wheels and reports go in WORK, so the checkout changes only once all four reports exist. From
# then on, a failure puts the folders archive_release.sh writes back as they were.
WORK=$(mktemp -d)
WRITING=false
cleanup() {
  status=$?
  rm -rf "$WORK"
  if [ "$status" -ne 0 ] && $WRITING; then
    git -C "$ROOT" checkout HEAD -- "${OWNED[@]}"
    git -C "$ROOT" clean -fdq -- "${OWNED[@]}"
    echo "archive_backport: failed; docs/generated and docs-site/archive are as they were." >&2
  fi
}
trap cleanup EXIT

while read -r platform arch report name url; do
  [ -n "$platform" ] || continue
  curl -fsSL -o "$WORK/$name" "$url" || fail "could not download $name from $url"
done <<<"$WANTED"
while read -r platform arch report name url; do
  if [ "$platform" = manylinux ]; then
    # The image release CI builds the wheels in, with the report script release CI runs. Naming
    # the platform keeps Docker from warning that an x86_64 image runs emulated on Apple silicon.
    case $arch in x86_64) docker_platform=linux/amd64 ;; aarch64) docker_platform=linux/arm64 ;; esac
    docker run --rm --platform "$docker_platform" -v "$ROOT/scripts:/scripts:ro" -v "$WORK:/work" \
      "quay.io/pypa/manylinux2014_$arch" sh -c \
      "py=/opt/python/cp312-cp312/bin/python &&
       \$py -m pip install -q --disable-pip-version-check --root-user-action=ignore auditwheel &&
       \$py /scripts/generate_auditwheel_report.py /work/$name /work/$report" </dev/null ||
      fail "making the $arch Linux report in Docker failed; the output above says why"
  elif [ "$platform" = macosx ]; then
    mkdir "$WORK/$arch" && unzip -q "$WORK/$name" -d "$WORK/$arch" ||
      fail "could not unpack $name"
    otool -L "$WORK/$arch"/eqty_sdk/*.so >"$WORK/$report" || fail "otool could not read $name"
  fi
done <<<"$WANTED"

WRITING=true
while read -r platform arch report rest; do
  cp "$WORK/$report" "$ROOT/docs/generated/$report"
done <<<"$PLATFORMS"
PYTHON=$PYTHON "$ROOT/scripts/archive_release.sh" "$VERSION"
echo "archive_backport: commit docs-site/archive/ and open a PR to main."
