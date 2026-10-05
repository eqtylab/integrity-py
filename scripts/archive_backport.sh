#!/usr/bin/env bash
# Save a 2.0 to 2.4 backport's docs, with reports made from its published wheels.
#
#     just archive-backport 2.4.3
#
# Such a tag runs the release.yml stored in it, which has no archive job, so main's docs build
# fails until this runs. It does that job's work by hand: downloads the release's wheels from
# PyPI, makes the four wheel reports release CI makes, then runs archive_release.sh, which
# converts the tag's pages with them. Needs macOS, for otool, and Docker, because auditwheel runs
# only on Linux. Everything that can be checked is checked before anything is downloaded. The
# reports never enter the checkout, and if the run stops early docs-site/archive is put back.
# Tests: scripts/test_archive_release.py.
set -euo pipefail

VERSION=${1:?usage: archive_backport.sh X.Y.Z}
ROOT=$(git rev-parse --show-toplevel)
VENV="$ROOT/.venv-docs/bin/python"
[ -x "$VENV" ] && PYTHON=${PYTHON:-$VENV}
PYTHON=${PYTHON:-python3}
ARCHIVE=docs-site/archive
# One wheel to a line: the platform in its file name, its architecture, the Docker platform its
# report is made on (none for macOS), and the report.
PLATFORMS="manylinux x86_64 linux/amd64 auditwheel-show-linux-x86_64.txt
manylinux aarch64 linux/arm64 auditwheel-show-linux-aarch64.txt
macosx arm64 - otool-show-macos-arm64.txt
macosx x86_64 - otool-show-macos-x86_64.txt"

# The message, then any commands that fix it, one to a line.
fail() {
  echo "archive_backport: $1" >&2
  shift
  [ $# -eq 0 ] || printf '    %s\n' "$@" >&2
  exit 1
}
has_docs_site() {
  git -C "$ROOT" cat-file -e "v$1:docs-site/src/content/docs" 2>/dev/null
}

[[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "$VERSION is not X.Y.Z"
git -C "$ROOT" rev-parse --quiet --verify "refs/tags/v$VERSION" >/dev/null ||
  fail "no tag v$VERSION; run git fetch --tags"
has_docs_site "$VERSION" &&
  fail "v$VERSION has docs-site/, so its release workflow archives it; nothing to do by hand"
# Release versions, oldest first. Only a group's newest patch is saved, and only one older than
# the newest release is a backport; archive_release.sh draws the same lines.
RELEASES=$(git -C "$ROOT" tag --list 'v*.*.*' | sed -nE 's/^v([0-9]+\.[0-9]+\.[0-9]+)$/\1/p' |
  sort -t. -k1,1n -k2,2n -k3,3n)
[ "$VERSION" != "$(tail -n 1 <<<"$RELEASES")" ] ||
  fail "$VERSION is the newest release, so it is not a backport; this is only for backports"
GROUP_NEWEST=$(awk -F. -v g="${VERSION%.*}" '$1"."$2 == g' <<<"$RELEASES" | tail -n 1)
if [ "$GROUP_NEWEST" != "$VERSION" ]; then
  has_docs_site "$GROUP_NEWEST" &&
    fail "$GROUP_NEWEST is newer than $VERSION, and its release workflow saves the" \
      "${VERSION%.*} docs; nothing to do by hand"
  fail "$GROUP_NEWEST is newer than $VERSION, and only the newest patch is saved." \
    "just archive-backport $GROUP_NEWEST"
fi
[ -z "$(git -C "$ROOT" status --porcelain --untracked-files=all -- "$ARCHIVE")" ] ||
  fail "this rewrites $ARCHIVE, which has uncommitted changes. Commit or stash them first:" \
    "git status --untracked-files=all -- $ARCHIVE"
command -v otool >/dev/null || fail "otool not found; the macOS reports need a Mac"
command -v jq >/dev/null || fail "jq not found; install it, then run this again:" "brew install jq"
docker info >/dev/null 2>&1 || fail "Docker isn't running; start Docker Desktop and run this again"
"$PYTHON" -c 'import griffe, griffe2md, yaml' 2>/dev/null ||
  fail "$PYTHON lacks the API renderer. Install it once in .venv-docs, then run this again:" \
    "python3 -m venv .venv-docs" \
    ".venv-docs/bin/pip install griffe==1.14.0 griffe2md==1.2.5 pyyaml" \
    "just archive-backport $VERSION"

status=0
JSON=$(curl -fsSL "https://pypi.org/pypi/eqty-sdk/$VERSION/json") || status=$?
case $status in
  0) ;;
  22) fail "PyPI has no eqty-sdk $VERSION yet; run this again once its release has published" ;;
  *) fail "could not reach PyPI (curl exit $status); check the network and run this again" ;;
esac
FILES=$(jq -r '.urls[] | "\(.filename) \(.url)"' <<<"$JSON") ||
  fail "PyPI's answer for $VERSION is not the JSON expected"
# Each platform's wheel, found before any is downloaded.
WANTED=()
while read -r platform arch docker_platform report; do
  found=$(awk -v p="$platform.*_${arch}[.]whl$" '$1 ~ p { print $1, $2; exit }' <<<"$FILES")
  [ -n "$found" ] || fail "PyPI has no $platform $arch wheel for $VERSION"
  WANTED+=("$platform $arch $docker_platform $report $found")
done <<<"$PLATFORMS"

# Wheels and reports stay in WORK. Once archive_release.sh starts writing the archive, stopping
# for any reason before it finishes, a closed terminal included, puts the archive back.
WORK=$(mktemp -d)
WRITING=false
DONE=false
cleanup() {
  rm -rf "$WORK"
  if $WRITING && ! $DONE; then
    git -C "$ROOT" checkout HEAD -- "$ARCHIVE"
    git -C "$ROOT" clean -fdq -- "$ARCHIVE"
    echo "archive_backport: stopped; $ARCHIVE is as it was." >&2
  fi
}
trap cleanup EXIT
# Ctrl-C reaches docker, but the container's shell waits for pip or auditwheel, so docker exits
# normally and bash would carry on. Stopping here makes Ctrl-C end the run after that step.
trap 'exit 130' INT

mkdir "$WORK/reports"
for row in "${WANTED[@]}"; do
  read -r platform arch docker_platform report name url <<<"$row"
  curl -fsSL -o "$WORK/$name" "$url" || fail "could not download $name from $url"
done
for row in "${WANTED[@]}"; do
  read -r platform arch docker_platform report name url <<<"$row"
  if [ "$platform" = manylinux ]; then
    # The image release CI builds the wheels in, with the report script release CI runs.
    docker run --rm --platform "$docker_platform" \
      -v "$ROOT/scripts:/scripts:ro" -v "$WORK:/work" "quay.io/pypa/manylinux2014_$arch" sh -c \
      "py=/opt/python/cp312-cp312/bin/python &&
       \$py -m pip install -q --disable-pip-version-check --root-user-action=ignore auditwheel &&
       \$py /scripts/generate_auditwheel_report.py /work/$name /work/reports/$report" </dev/null ||
      fail "making the $arch Linux report in Docker failed; the output above says why"
  else
    unzip -q "$WORK/$name" -d "$WORK/$platform-$arch" || fail "could not unpack $name"
    otool -L "$WORK/$platform-$arch"/eqty_sdk/*.so >"$WORK/reports/$report" ||
      fail "otool could not read $name"
  fi
done

WRITING=true
PYTHON=$PYTHON "$ROOT/scripts/archive_release.sh" "$VERSION" "$WORK/reports"
DONE=true
echo "archive_backport: commit $ARCHIVE/ and open a PR to main."
