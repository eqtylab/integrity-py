"""Checks for archive_release.sh and open_archive_pr.sh, run against a throwaway repository.

    python3 -m unittest scripts/test_archive_release.py

The render script is a stub, so this covers which releases are archived and what changes, not
the rendered pages. The reports are left uncommitted, as release.yml's downloads leave them. `gh`
is a stub that records its calls, and a bare repository stands in for GitHub. Not collected by
`just test-py`, which discovers under tests/ only.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS = Path(__file__).resolve().parent
SCRIPT = SCRIPTS / "archive_release.sh"
# Answers `gh pr list` with $FAKE_PRS, fails where asked, and records every call.
FAKE_GH = """#!/usr/bin/env bash
echo "$*" >> "$FAKE_GH_LOG"
case "$1 $2" in
  "pr list") [ -z "$FAKE_LIST_FAILS" ] || exit 1; echo "${FAKE_PRS:-[]}" ;;
  "workflow run") [ -z "$FAKE_DISPATCH_FAILS" ] || exit 1 ;;
  "pr create") while [ $# -gt 0 ]; do
      [ "$1" = --body-file ] && cp "$2" "$FAKE_GH_LOG.body"; shift; done ;;
esac
"""
REPORTS = [
    "auditwheel-show-linux-x86_64.txt",
    "auditwheel-show-linux-aarch64.txt",
    "otool-show-macos-arm64.txt",
    "otool-show-macos-x86_64.txt",
]
# Writes its tag, its folder and one report it was given, so a test sees how it was called.
# With $FAKE_CONVERTER_FAILS it replaces the folder with a partial page and fails, as the real
# one can; with $FAKE_CONVERTER_PAUSES it writes that page, then waits to be stopped.
FAKE_CONVERTER = """import os
import shutil
import sys
import time
from pathlib import Path

tag, out, flag, reports = sys.argv[1:]
assert flag == "--reports"
if os.environ.get("FAKE_CONVERTER_FAILS") or os.environ.get("FAKE_CONVERTER_PAUSES"):
    shutil.rmtree(out, ignore_errors=True)
    Path(out).mkdir(parents=True)
    (Path(out) / "partial.mdx").write_text("partial")
    if os.environ.get("FAKE_CONVERTER_PAUSES"):
        time.sleep(30)
    sys.exit("archive_version: failed")
Path(out).mkdir(parents=True, exist_ok=True)
report = (Path(reports) / "auditwheel-show-linux-x86_64.txt").read_text()
(Path(out) / "index.mdx").write_text(f"{tag} {out}\\n{report}")
"""
# What 2.4's release.yml publishes with: PyPI. A mention of EQTY Lab's index is not an upload.
PYPI_WORKFLOW = """\
# Before 2.4, releases went to https://pypi.eqtylab.io/ instead.
uses: pypa/gh-action-pypi-publish@release/v1
"""
REDIRECT_TO_LATEST = {"source": "/(2\\.4\\.[0-9]+|latest|dev)/(.*)\\.html", "destination": "/$2/"}


class Repo:
    """A repository shaped like integrity-py's main, with release tags."""

    def __init__(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="archive-release-"))
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "t@t")
        self.git("config", "user.name", "t")
        self.write("docs-site/src/content/docs/index.mdx", "latest\n")
        self.write("docs-site/archive/folders.json", "{}\n")
        self.write("scripts/render_api_docs.py", "")
        self.write("scripts/archive_version.py", FAKE_CONVERTER)
        self.write("vercel.json", json.dumps({"redirects": [REDIRECT_TO_LATEST]}))
        self.write(".github/workflows/release.yml", PYPI_WORKFLOW)
        shutil.copy(SCRIPT, self.root / "scripts/archive_release.sh")
        self.reports("main")
        self.commit("main")

    def git(self, *args: str) -> str:
        return subprocess.run(
            ["git", *args], cwd=self.root, check=True, capture_output=True, text=True
        ).stdout

    def write(self, rel: str, text: str) -> None:
        path = self.root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def reports(self, version: str) -> None:
        for name in REPORTS:
            self.write(f"docs/generated/{name}", f"eqty_sdk-{version}-cp38-abi3-{name}\n")

    def commit(self, message: str) -> None:
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)

    def release(self, version: str, with_docs: bool = True, workflow: str = "") -> None:
        """Tag a release. `with_docs=False` tags a tree without docs-site/, like 2.4.x, with
        `workflow` as its release.yml.
        """
        if with_docs:
            self.git("tag", f"v{version}")
            return
        self.git("switch", "-q", "--detach")
        self.git("rm", "-q", "-r", "docs-site")
        if workflow:
            self.write(".github/workflows/release.yml", workflow)
        self.commit(f"{version} without docs-site")
        self.git("tag", f"v{version}")
        self.git("switch", "-q", "main")

    def folders(self) -> dict:
        return json.loads((self.root / "docs-site/archive/folders.json").read_text())

    def changed(self, path: str) -> str:
        return self.git("status", "--porcelain", path)

    def run(
        self, version: str, python: str = sys.executable, reports: str = ""
    ) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["bash", "scripts/archive_release.sh", version, *([reports] if reports else [])],
            cwd=self.root,
            capture_output=True,
            text=True,
            env={**os.environ, "PYTHON": python},
        )


class ArchiveRelease(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = Repo()
        self.addCleanup(shutil.rmtree, self.repo.root, ignore_errors=True)

    def test_the_newest_release_is_archived_with_its_reports(self) -> None:
        self.repo.release("2.5.0")
        self.repo.reports("2.5.0")
        out = self.repo.run("2.5.0")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.5.0": "archive/v2.5"})
        self.assertTrue((self.repo.root / "docs-site/archive/v2.5/index.mdx").is_file())
        self.assertIn("2.5.0", (self.repo.root / "docs/generated" / REPORTS[0]).read_text())
        self.assertIn("vercel.json sends /2.4.x/", out.stdout)

    def test_a_new_patch_replaces_its_minor_s_folder(self) -> None:
        self.repo.write("docs-site/archive/folders.json", json.dumps({"2.5.0": "archive/v2.5"}))
        self.repo.commit("2.5.0 archived")
        self.repo.release("2.5.1")
        self.repo.reports("2.5.1")
        out = self.repo.run("2.5.1")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.5.1": "archive/v2.5"})

    def test_an_older_patch_of_its_minor_changes_nothing_in_the_site(self) -> None:
        self.repo.release("2.5.0")
        self.repo.release("2.5.1")
        self.repo.reports("2.5.0")
        out = self.repo.run("2.5.0")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.changed("docs-site"), "")

    def test_a_backport_is_archived_and_latest_keeps_its_own_reports(self) -> None:
        self.repo.write("docs-site/archive/folders.json", json.dumps({"2.4.2": "archive/v2.4"}))
        self.repo.commit("2.4.2 archived")
        self.repo.release("2.4.3")
        self.repo.release("2.5.0")
        self.repo.reports("2.4.3")
        out = self.repo.run("2.4.3")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.4.3": "archive/v2.4"})
        self.assertEqual(self.repo.changed("docs/generated"), "")
        self.assertNotIn("vercel.json sends", out.stdout)

    def test_a_backport_leaves_latest_s_reports_even_when_its_own_are_staged(self) -> None:
        self.repo.write("docs-site/archive/folders.json", json.dumps({"2.4.2": "archive/v2.4"}))
        self.repo.commit("2.4.2 archived")
        self.repo.release("2.4.3")
        self.repo.release("2.5.0")
        self.repo.reports("2.4.3")
        self.repo.git("add", "docs/generated")
        out = self.repo.run("2.4.3")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.changed("docs/generated"), "")

    def test_a_backport_whose_tag_has_no_docs_site_is_converted_with_its_reports(self) -> None:
        self.repo.write("docs-site/archive/folders.json", json.dumps({"2.4.2": "archive/v2.4"}))
        self.repo.write("docs-site/archive/v2.4/index.mdx", "2.4.2\n")
        self.repo.commit("2.4.2 archived")
        self.repo.release("2.5.0")
        self.repo.release("2.4.3", with_docs=False)
        self.repo.reports("2.4.3")
        out = self.repo.run("2.4.3")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.4.3": "archive/v2.4"})
        page = (self.repo.root / "docs-site/archive/v2.4/index.mdx").read_text()
        dest = self.repo.root.resolve() / "docs-site/archive/v2.4"
        self.assertTrue(page.startswith(f"v2.4.3 {dest}\n"), page)
        self.assertIn(f"eqty_sdk-2.4.3-cp38-abi3-{REPORTS[0]}", page)
        self.assertEqual(self.repo.changed("docs/generated"), "")

    def test_a_backport_s_reports_from_another_folder_leave_docs_generated_alone(self) -> None:
        self.repo.write("docs-site/archive/folders.json", json.dumps({"2.4.2": "archive/v2.4"}))
        self.repo.commit("2.4.2 archived")
        self.repo.release("2.5.0")
        self.repo.release("2.4.3", with_docs=False)
        reports = Path(tempfile.mkdtemp(prefix="reports-"))
        self.addCleanup(shutil.rmtree, reports, ignore_errors=True)
        for name in REPORTS:
            (reports / name).write_text(f"eqty_sdk-2.4.3-cp38-abi3-{name}\n")
        self.repo.write("docs/generated/mine.md", "work in progress\n")
        out = self.repo.run("2.4.3", reports=str(reports))
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.4.3": "archive/v2.4"})
        page = (self.repo.root / "docs-site/archive/v2.4/index.mdx").read_text()
        self.assertIn(f"eqty_sdk-2.4.3-cp38-abi3-{REPORTS[0]}", page)
        self.assertEqual(self.repo.changed("docs/generated"), "?? docs/generated/mine.md\n")

    def test_the_newest_release_s_reports_from_another_folder_reach_latest(self) -> None:
        self.repo.release("2.5.0")
        reports = Path(tempfile.mkdtemp(prefix="reports-"))
        self.addCleanup(shutil.rmtree, reports, ignore_errors=True)
        for name in REPORTS:
            (reports / name).write_text(f"eqty_sdk-2.5.0-cp38-abi3-{name}\n")
        out = self.repo.run("2.5.0", reports=str(reports))
        self.assertEqual(out.returncode, 0, out.stderr)
        for name in REPORTS:
            self.assertIn("2.5.0", (self.repo.root / "docs/generated" / name).read_text())

    def test_reports_from_another_version_are_refused(self) -> None:
        self.repo.release("2.5.0")
        out = self.repo.run("2.5.0")
        self.assertNotEqual(out.returncode, 0)
        self.assertEqual(self.repo.changed("docs-site"), "")

    def test_a_version_with_no_tag_fails(self) -> None:
        self.repo.release("2.5.0")
        self.repo.reports("2.5.1")
        out = self.repo.run("2.5.1")
        self.assertNotEqual(out.returncode, 0)

    def test_any_redirect_from_the_old_release_into_latest_is_named(self) -> None:
        into_latest = [
            ("/(2\\.4\\.[0-9]+|latest|dev)/generated/list\\.html", "/api/assets/#list"),
            ("/(latest|dev|2\\.4\\.[0-9]+)/(.*)\\.html", "/$2/"),
            ("/2\\.4\\.2/api/assets\\.html", "/api/assets/"),
            ("/(2\\.4\\.[0-9]+)(/?)", "/v2/"),
        ]
        for source, destination in into_latest:
            with self.subTest(source=source, destination=destination):
                repo = Repo()
                self.addCleanup(shutil.rmtree, repo.root, ignore_errors=True)
                redirects = [
                    {"source": source, "destination": destination},
                    {"source": "/(2\\.0\\.[0-9]+)/(.*)\\.html", "destination": "/v2.0/$2/"},
                ]
                repo.write("vercel.json", json.dumps({"redirects": redirects}))
                repo.commit("redirects")
                repo.release("2.5.0")
                repo.reports("2.5.0")
                out = repo.run("2.5.0")
                self.assertEqual(out.returncode, 0, out.stderr)
                # open_archive_pr.sh copies lines with this prefix into the PR.
                self.assertRegex(out.stdout, r"(?m)^archive_release: vercel\.json sends /2\.4\.x/")
                self.assertNotIn("/2.0.x/", out.stdout)

    def test_no_versioned_redirect_left_to_retarget_is_not_a_failure(self) -> None:
        self.repo.write(
            "vercel.json",
            json.dumps(
                {"redirects": [{"source": "/(latest|dev)/(.*)\\.html", "destination": "/$2/"}]}
            ),
        )
        self.repo.commit("redirects retargeted")
        self.repo.release("2.5.0")
        self.repo.reports("2.5.0")
        out = self.repo.run("2.5.0")
        self.assertEqual(out.returncode, 0, out.stderr)

    def test_a_failed_render_fails_the_script(self) -> None:
        self.repo.release("2.5.0")
        self.repo.reports("2.5.0")
        out = self.repo.run("2.5.0", python="false")
        self.assertNotEqual(out.returncode, 0)


PYPI_WHEELS = [
    "cp38-abi3-macosx_10_12_x86_64.whl",
    "cp38-abi3-macosx_11_0_arm64.whl",
    "cp38-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64.whl",
    "cp38-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl",
    "cp38-abi3-win_amd64.whl",
]
# Answers https://HOST/PATH from $FAKE_PYPI/HOST/PATH, a PATH ending in / from its index.html.
# A HOST with a .login file answers 401 unless curl is told to read ~/.netrc and it holds the
# file's line; $FAKE_PAGE_STATUS replaces the status of every page. `-o FILE` saves the answer,
# `-w` prints its status after it, and `-f` fails on anything but 200, as on any wheel when
# $FAKE_DOWNLOAD_FAILS is set. Every URL is logged to $FAKE_CURL_LOG, followed by " netrc" when
# curl was told to read it; with $FAKE_CURL_EXIT every call exits with that code.
FAKE_CURL = """#!/usr/bin/env bash
out=; url=; format=; fail=false; netrc=false
while [ $# -gt 0 ]; do case "$1" in
  -o) out=$2; shift ;;
  -w) format=$2; shift ;;
  --netrc-optional) netrc=true ;;
  -*f*) fail=true ;;
  -*) ;;
  *) url=$1 ;;
esac; shift; done
echo "$url$($netrc && echo ' netrc')" >> "$FAKE_CURL_LOG"
[ -z "$FAKE_CURL_EXIT" ] || exit "$FAKE_CURL_EXIT"
path=${url#https://}; host=${path%%/*}
case $path in */) path+=index.html; page=true ;; *) page=false ;; esac
login="$FAKE_PYPI/$host/.login"
if [ -f "$login" ] && ! { $netrc && grep -qxF "$(cat "$login")" "$HOME/.netrc"; }; then code=401
elif [ ! -f "$FAKE_PYPI/$path" ]; then code=404
elif [ -n "$FAKE_DOWNLOAD_FAILS" ] && [[ $path == *.whl ]]; then code=500
else code=200; fi
! $page || code=${FAKE_PAGE_STATUS:-$code}
if [ $code != 200 ]; then ! $fail || exit 22
elif [ -n "$out" ]; then cp "$FAKE_PYPI/$path" "$out"
else cat "$FAKE_PYPI/$path"; fi
status='%{http_code}'
[ -z "$format" ] || printf "${format//"$status"/$code}"
"""
# What 2.0.0 to 2.3.0's release.yml publishes with: EQTY Lab's index, and no other.
LEGACY_WORKFLOW = """\
python3 -m twine upload --repository-url="https://pypi.eqtylab.io/" "${files[@]}"
"""
# The login EQTY Lab's fake index takes.
LOGIN = "machine pypi.eqtylab.io login me password secret"
# `docker info` fails when $FAKE_DOCKER_DOWN is set. `docker run` fails when $FAKE_RUN_FAILS is
# set, or writes the report its command names, for the wheel its command names, into the folder
# mounted at /work. With $FAKE_RUN_STARTED it creates that file, then finishes its work despite
# Ctrl-C, as real docker does when the container's shell is waiting on pip or auditwheel.
FAKE_DOCKER = """#!/usr/bin/env bash
if [ "$1" = info ]; then [ -z "$FAKE_DOCKER_DOWN" ]; exit; fi
[ -z "$FAKE_RUN_FAILS" ] || exit 1
if [ -n "$FAKE_RUN_STARTED" ]; then trap '' INT; touch "$FAKE_RUN_STARTED"; sleep 2; fi
for arg in "$@"; do case "$arg" in *:/work) work=${arg%:/work} ;; esac; done
wheel=$(grep -oE 'eqty_sdk-[^ ]+[.]whl' <<<"$*" | head -n 1)
report=$(grep -oE '/work/[^ ]+[.]txt' <<<"$*" | head -n 1)
echo "$wheel" > "$work/${report#/work/}"
"""
FAKE_OTOOL = """#!/usr/bin/env bash
printf '%s:\\n\\t/usr/lib/libSystem.B.dylib\\n' "$2"
"""


class ArchiveBackport(unittest.TestCase):
    """archive_backport.sh, with the package indexes, Docker and otool faked."""

    def setUp(self) -> None:
        self.repo = Repo()
        self.addCleanup(shutil.rmtree, self.repo.root, ignore_errors=True)
        shutil.copy(SCRIPTS / "archive_backport.sh", self.repo.root / "scripts")
        self.repo.write("docs-site/archive/folders.json", json.dumps({"2.4.2": "archive/v2.4"}))
        self.repo.write("docs-site/archive/v2.4/index.mdx", "2.4.2\n")
        self.repo.commit("2.4.2 archived")
        self.repo.release("2.5.0")
        self.repo.release("2.4.3", with_docs=False)

        side = Path(tempfile.mkdtemp(prefix="archive-backport-"))
        self.addCleanup(shutil.rmtree, side, ignore_errors=True)
        self.pypi = side / "pypi"
        self.pypi.mkdir()
        for tail in PYPI_WHEELS:
            self.publish(f"eqty_sdk-2.4.3-{tail}")
        # Empty modules, so the script's dependency check passes without them installed.
        modules = side / "modules"
        modules.mkdir()
        for name in ("griffe", "griffe2md", "yaml"):
            (modules / f"{name}.py").write_text("")
        (modules / "mdformat.py").write_text('__version__ = "1.0.0"\n')
        bin_dir = side / "bin"
        bin_dir.mkdir()
        for name, text in (("curl", FAKE_CURL), ("docker", FAKE_DOCKER), ("otool", FAKE_OTOOL)):
            (bin_dir / name).write_text(text)
            (bin_dir / name).chmod(0o755)
        # Its own home, so a real ~/.netrc never answers for a test.
        self.home = side / "home"
        self.home.mkdir()
        self.env = {
            **os.environ,
            "PATH": f"{bin_dir}:{os.environ['PATH']}",
            "HOME": str(self.home),
            "FAKE_PYPI": str(self.pypi),
            "PYTHONPATH": str(modules),
            "PYTHON": sys.executable,
            "FAKE_CURL_LOG": str(side / "curl.log"),
        }
        self.curl_log = side / "curl.log"

    def publish(self, name: str, private: bool = False) -> None:
        """List a wheel on PyPI, whose links are absolute, or on EQTY Lab's index, with a link
        relative to the page, as private indexes often give.
        """
        import zipfile

        host = "pypi.eqtylab.io" if private else "pypi.org"
        wheel = self.pypi / (f"{host}/packages" if private else "files.example") / name
        wheel.parent.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(wheel, "w") as whl:
            whl.writestr("eqty_sdk/_rust.abi3.so", "")
        href = f"../../packages/{name}" if private else f"https://files.example/{name}"
        self.page(host).parent.mkdir(parents=True, exist_ok=True)
        with self.page(host).open("a") as page:
            page.write(f'<a href="{href}#sha256=0">{name}</a><br />\n')

    def page(self, host: str = "pypi.org") -> Path:
        return self.pypi / host / "simple/eqty-sdk/index.html"

    def publish_only_to_eqty_lab_s_index(self) -> None:
        """Re-tag 2.4.3 as 2.0 to 2.3 publish: to EQTY Lab's index alone, which needs a login and
        lists older versions' wheels too.
        """
        self.repo.git("tag", "-d", "v2.4.3")
        self.repo.release("2.4.3", with_docs=False, workflow=LEGACY_WORKFLOW)
        shutil.rmtree(self.pypi)
        (self.pypi / "pypi.eqtylab.io").mkdir(parents=True)
        (self.pypi / "pypi.eqtylab.io/.login").write_text(LOGIN)
        for version in ("2.4.2", "2.4.3"):
            for tail in PYPI_WHEELS:
                self.publish(f"eqty_sdk-{version}-{tail}", private=True)

    def run_script(self, version: str = "2.4.3", **env: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["bash", "scripts/archive_backport.sh", version],
            cwd=self.repo.root,
            capture_output=True,
            text=True,
            env={**self.env, **env},
        )

    def test_a_backport_is_archived_with_reports_made_from_its_published_wheels(self) -> None:
        self.repo.write("docs/generated/mine.md", "work in progress\n")
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.4.3": "archive/v2.4"})
        # The fake converter copies the x86_64 Linux report into the page.
        page = (self.repo.root / "docs-site/archive/v2.4/index.mdx").read_text()
        self.assertIn("eqty_sdk-2.4.3-cp38-abi3-manylinux_2_17_x86_64", page)
        # The reports are made and read outside the checkout, so local work there survives.
        self.assertEqual(self.repo.changed("docs/generated"), "?? docs/generated/mine.md\n")
        # The commands that publish it, ready to paste, named as the release job names its PR.
        self.assertTrue(
            out.stdout.endswith(
                "archive_backport: done. To publish it, run:\n"
                "    git switch -c docs/archive-v2.4.3\n"
                "    git add docs-site/archive/\n"
                '    git commit -m "docs: archive the 2.4.3 docs"\n'
                "    git push -u origin docs/archive-v2.4.3\n"
                "    gh pr create --fill\n"
            ),
            out.stdout,
        )

    def test_docker_not_running_fails_first_and_says_so(self) -> None:
        out = self.run_script(FAKE_DOCKER_DOWN="1")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("Docker", out.stderr)
        self.assertEqual(self.repo.changed("docs-site"), "")

    def test_a_python_without_the_renderer_fails_naming_what_to_install(self) -> None:
        # Shadows any installed griffe2md, so the test holds under whichever Python runs it.
        missing = Path(self.env["PYTHONPATH"]).parent / "missing"
        missing.mkdir()
        (missing / "griffe2md.py").write_text("raise ImportError\n")
        out = self.run_script(PYTHONPATH=f"{missing}:{self.env['PYTHONPATH']}")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("python3.12 -m venv --clear .venv-docs", out.stderr)
        self.assertIn("griffe2md==1.2.5 mdformat==1.0.0", out.stderr)

    def test_an_older_mdformat_fails_naming_a_newer_python(self) -> None:
        # What a .venv-docs made with Python 3.9 has, since mdformat 1.0.0 needs 3.10.
        old = Path(self.env["PYTHONPATH"]).parent / "old"
        old.mkdir()
        (old / "mdformat.py").write_text('__version__ = "0.7.22"\n')
        out = self.run_script(PYTHONPATH=f"{old}:{self.env['PYTHONPATH']}")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("Python 3.10 or newer", out.stderr)
        self.assertIn("python3.12 -m venv --clear .venv-docs", out.stderr)
        self.assertEqual(self.repo.changed("docs-site"), "")

    def test_a_release_with_docs_site_is_left_to_its_release_workflow(self) -> None:
        out = self.run_script("2.5.0")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("release workflow", out.stderr)
        self.assertEqual(self.repo.changed("docs-site"), "")

    def test_a_backport_published_only_to_eqty_lab_s_index_is_archived_with_a_login(self) -> None:
        self.publish_only_to_eqty_lab_s_index()
        (self.home / ".netrc").write_text(f"{LOGIN}\n")
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.repo.folders(), {"2.4.3": "archive/v2.4"})
        # This version's wheels, not 2.4.2's listed beside them.
        page = (self.repo.root / "docs-site/archive/v2.4/index.mdx").read_text()
        self.assertIn("eqty_sdk-2.4.3-cp38-abi3-manylinux_2_17_x86_64", page)
        log = self.curl_log.read_text()
        self.assertNotIn("pypi.org", log)
        # Relative links resolved against the page, each download logged in as well.
        self.assertIn(
            "https://pypi.eqtylab.io/packages/eqty_sdk-2.4.3-cp38-abi3-macosx_11_0_arm64.whl"
            " netrc\n",
            log,
        )
        self.assertNotIn("secret", out.stdout + out.stderr)

    def test_eqty_lab_s_index_without_a_login_says_how_to_add_one(self) -> None:
        # nginx answers 401 to no login; an index that hides itself answers 403.
        for status in ("", "403"):
            with self.subTest(status=status or "401"):
                self.setUp()
                self.publish_only_to_eqty_lab_s_index()
                out = self.run_script(FAKE_PAGE_STATUS=status)
                self.assertNotEqual(out.returncode, 0)
                self.assertIn("pypi.eqtylab.io needs a login", out.stderr)
                # The one indented line is the one to add, not a command with the password in it.
                indented = [line for line in out.stderr.splitlines() if line.startswith("    ")]
                self.assertEqual(
                    indented, ["    machine pypi.eqtylab.io login YOUR_NAME password YOUR_PASSWORD"]
                )
                self.assertNotIn(".whl", self.curl_log.read_text())
                self.assertUntouched()

    def test_a_refused_login_says_to_correct_it_not_to_add_another(self) -> None:
        # curl uses the first entry for a host, so a second one added below would never be read.
        self.publish_only_to_eqty_lab_s_index()
        (self.home / ".netrc").write_text("machine pypi.eqtylab.io login me password old\n")
        out = self.run_script()
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("pypi.eqtylab.io refused the login in ~/.netrc", out.stderr)
        self.assertNotIn("Add this line", out.stderr)
        self.assertNotIn(".whl", self.curl_log.read_text())
        self.assertUntouched()

    def test_pypi_is_never_sent_the_login_in_netrc(self) -> None:
        # A `default` entry would answer for every host, PyPI and its file host included.
        (self.home / ".netrc").write_text("default login me password secret\n")
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertNotIn(" netrc", self.curl_log.read_text())

    def test_an_unexpected_answer_from_the_index_is_named(self) -> None:
        out = self.run_script(FAKE_PAGE_STATUS="500")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("PyPI answered 500", out.stderr)
        self.assertNotIn(".whl", self.curl_log.read_text())
        self.assertUntouched()

    def test_a_tag_without_a_release_workflow_is_refused_before_any_download(self) -> None:
        self.repo.git("tag", "-d", "v2.4.3")
        self.repo.git("switch", "-q", "--detach", "v2.5.0")
        self.repo.git("rm", "-q", "-r", "docs-site", ".github")
        self.repo.commit("2.4.3 without docs-site or a workflow")
        self.repo.git("tag", "v2.4.3")
        self.repo.git("switch", "-q", "main")
        out = self.run_script()
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("where it published is unknown", out.stderr)
        self.assertFalse(self.curl_log.exists())
        self.assertUntouched()

    def test_a_release_not_yet_on_pypi_says_to_run_again_once_it_is(self) -> None:
        # PyPI answers 404 for a project with no files at all, and lists only older versions
        # for one without this release's.
        for older_only in (False, True):
            with self.subTest(older_only=older_only):
                self.setUp()
                shutil.rmtree(self.pypi / "pypi.org")
                if older_only:
                    self.publish(f"eqty_sdk-2.4.2-{PYPI_WHEELS[0]}")
                out = self.run_script()
                self.assertNotEqual(out.returncode, 0)
                self.assertIn("PyPI has no eqty-sdk 2.4.3 yet", out.stderr)
                self.assertUntouched()

    def test_a_wheel_missing_from_pypi_fails_naming_it(self) -> None:
        self.page().unlink()
        for tail in PYPI_WHEELS:
            if "arm64" not in tail:
                self.publish(f"eqty_sdk-2.4.3-{tail}")
        out = self.run_script()
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("macosx", out.stderr)
        self.assertIn("arm64", out.stderr)
        self.assertUntouched()
        # Found from PyPI's file list, before any wheel is downloaded.
        self.assertEqual(self.curl_log.read_text().count("files.example"), 0)

    def assertUntouched(self) -> None:
        self.assertEqual(self.repo.changed("docs/generated"), "")
        self.assertEqual(self.repo.changed("docs-site"), "")

    def test_an_older_patch_is_refused_before_any_download_naming_the_newer(self) -> None:
        self.repo.release("2.4.4", with_docs=False)
        out = self.run_script("2.4.3")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("2.4.4", out.stderr)
        self.assertFalse(self.curl_log.exists())
        self.assertUntouched()

    def test_local_changes_in_the_archive_are_refused_before_any_download_and_kept(self) -> None:
        # Untracked even where git status hides untracked files, modified, and staged.
        changes = {
            "untracked": "docs-site/archive/mine.mdx",
            "modified": "docs-site/archive/v2.4/index.mdx",
            "staged": "docs-site/archive/staged.mdx",
        }
        for kind, rel in changes.items():
            with self.subTest(kind):
                self.setUp()
                self.repo.git("config", "status.showUntrackedFiles", "no")
                self.repo.write(rel, "work in progress\n")
                if kind == "staged":
                    self.repo.git("add", rel)
                out = self.run_script()
                self.assertNotEqual(out.returncode, 0)
                self.assertIn("docs-site/archive", out.stderr)
                self.assertFalse(self.curl_log.exists())
                self.assertEqual((self.repo.root / rel).read_text(), "work in progress\n")

    def test_the_newest_release_is_refused_as_not_a_backport(self) -> None:
        self.repo.git("tag", "-d", "v2.5.0")
        out = self.run_script()
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("newest release", out.stderr)
        self.assertFalse(self.curl_log.exists())
        self.assertUntouched()

    def test_a_newer_patch_its_release_workflow_archives_is_not_suggested(self) -> None:
        self.repo.release("2.4.4")
        out = self.run_script("2.4.3")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("release workflow", out.stderr)
        # The whole message is one line, with no command to paste.
        self.assertNotIn("\n    ", out.stderr)

    def test_pypi_unreachable_is_not_reported_as_a_missing_release(self) -> None:
        out = self.run_script(FAKE_CURL_EXIT="6")
        self.assertNotEqual(out.returncode, 0)
        self.assertIn("could not reach PyPI", out.stderr)
        self.assertNotIn("PyPI has no", out.stderr)

    def test_ctrl_c_while_docker_runs_stops_before_anything_is_written(self) -> None:
        import signal
        import time

        started = self.curl_log.parent / "docker-started"
        run = subprocess.Popen(
            ["bash", "scripts/archive_backport.sh", "2.4.3"],
            cwd=self.repo.root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env={**self.env, "FAKE_RUN_STARTED": str(started)},
            start_new_session=True,
        )
        deadline = time.monotonic() + 20
        while not started.exists() and time.monotonic() < deadline:
            time.sleep(0.1)
        self.assertTrue(started.exists(), "docker never ran")
        os.killpg(run.pid, signal.SIGINT)
        out, _ = run.communicate(timeout=20)
        self.assertNotEqual(run.returncode, 0)
        self.assertNotIn("To publish it", out)
        self.assertUntouched()

    def test_stopping_mid_run_puts_the_archive_back(self) -> None:
        # A closed terminal, Ctrl-C and kill, each while the converter is writing the archive.
        import signal
        import time

        for sig in (signal.SIGHUP, signal.SIGINT, signal.SIGTERM):
            with self.subTest(sig.name):
                self.setUp()
                run = subprocess.Popen(
                    ["bash", "scripts/archive_backport.sh", "2.4.3"],
                    cwd=self.repo.root,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    env={**self.env, "FAKE_CONVERTER_PAUSES": "1"},
                    start_new_session=True,
                )
                partial = self.repo.root / "docs-site/archive/v2.4/partial.mdx"
                deadline = time.monotonic() + 20
                while not partial.exists() and time.monotonic() < deadline:
                    time.sleep(0.1)
                self.assertTrue(partial.exists(), "the converter never started")
                os.killpg(run.pid, sig)
                run.communicate(timeout=20)
                self.assertNotEqual(run.returncode, 0)
                self.assertUntouched()


class OpenArchivePr(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = Repo()
        self.addCleanup(shutil.rmtree, self.repo.root, ignore_errors=True)
        self.remote = Path(tempfile.mkdtemp(prefix="archive-remote-"))
        self.addCleanup(shutil.rmtree, self.remote, ignore_errors=True)
        subprocess.run(["git", "init", "-q", "--bare", str(self.remote)], check=True)
        self.repo.git("remote", "add", "origin", str(self.remote))
        shutil.copy(SCRIPTS / "open_archive_pr.sh", self.repo.root / "open_archive_pr.sh")
        bin_dir = self.repo.root.parent / (self.repo.root.name + "-bin")
        bin_dir.mkdir()
        self.addCleanup(shutil.rmtree, bin_dir, ignore_errors=True)
        (bin_dir / "gh").write_text(FAKE_GH)
        (bin_dir / "gh").chmod(0o755)
        self.gh_log = bin_dir / "calls"
        self.env = {
            **os.environ,
            "PATH": f"{bin_dir}:{os.environ['PATH']}",
            "FAKE_GH_LOG": str(self.gh_log),
            "GITHUB_REPOSITORY_OWNER": "eqtylab",
        }
        self.log = self.repo.root.parent / (self.repo.root.name + ".log")
        self.addCleanup(lambda: self.log.unlink(missing_ok=True))
        self.log.write_text("archive_release: 2.5.0 → docs-site/archive/v2.5\n")

    def archive(self) -> None:
        self.repo.write("docs-site/archive/v2.5/index.mdx", "2.5\n")

    def run_script(self, **env: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            ["bash", "open_archive_pr.sh", "2.5.0", str(self.log)],
            cwd=self.repo.root,
            capture_output=True,
            text=True,
            env={**self.env, **env},
        )

    def calls(self) -> list:
        return self.gh_log.read_text().splitlines() if self.gh_log.exists() else []

    def pushed(self) -> bool:
        heads = subprocess.run(
            ["git", "branch", "--list", "docs/archive-v2.5.0"],
            cwd=self.remote,
            capture_output=True,
            text=True,
        )
        return bool(heads.stdout.strip())

    def test_nothing_archived_opens_nothing(self) -> None:
        self.repo.reports("2.5.0")
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertEqual(self.calls(), [])

    def test_it_pushes_opens_the_pr_then_starts_the_checks(self) -> None:
        self.archive()
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertTrue(self.pushed())
        verbs = [" ".join(c.split()[:2]) for c in self.calls()]
        self.assertEqual(verbs, ["pr list", "pr create", "workflow run", "workflow run"])
        self.assertIn("ci.yml", self.calls()[2])
        self.assertIn("docs-check.yml", self.calls()[3])

    def test_an_open_pr_is_left_alone_and_its_checks_restarted(self) -> None:
        self.archive()
        prs = json.dumps([{"headRepositoryOwner": {"login": "eqtylab"}}])
        out = self.run_script(FAKE_PRS=prs)
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertFalse(self.pushed())
        self.assertFalse(any(c.startswith("pr create") for c in self.calls()))
        self.assertEqual(sum(c.startswith("workflow run") for c in self.calls()), 2)

    def test_an_open_pr_from_a_fork_does_not_count(self) -> None:
        self.archive()
        prs = json.dumps([{"headRepositoryOwner": {"login": "someone-else"}}])
        out = self.run_script(FAKE_PRS=prs)
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertTrue(any(c.startswith("pr create") for c in self.calls()))

    def test_a_failed_dispatch_still_leaves_the_pr_open_and_warns(self) -> None:
        self.archive()
        out = self.run_script(FAKE_DISPATCH_FAILS="1")
        self.assertEqual(out.returncode, 0, out.stderr)
        self.assertTrue(any(c.startswith("pr create") for c in self.calls()))
        self.assertIn("::warning::", out.stdout)

    def test_a_failed_pr_lookup_fails_before_pushing(self) -> None:
        self.archive()
        out = self.run_script(FAKE_LIST_FAILS="1")
        self.assertNotEqual(out.returncode, 0)
        self.assertFalse(self.pushed())

    def test_redirects_to_move_are_a_checklist_to_finish_before_merging(self) -> None:
        self.archive()
        with self.log.open("a") as log:
            log.write("archive_release: vercel.json sends /2.4.x/ addresses to latest.\n")
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        body = Path(f"{self.gh_log}.body").read_text()
        self.assertIn("- [ ] vercel.json sends /2.4.x/ addresses to latest.", body)
        self.assertIn("before merging", body)

    def test_a_backport_pr_says_to_merge_promptly(self) -> None:
        self.archive()
        with self.log.open("a") as log:
            log.write("archive_release: backport; latest's pages and reports are unchanged.\n")
        out = self.run_script()
        self.assertEqual(out.returncode, 0, out.stderr)
        body = Path(f"{self.gh_log}.body").read_text()
        self.assertTrue(body.startswith("A backport"))
        self.assertIn("Merge it promptly", body)


if __name__ == "__main__":
    unittest.main()
