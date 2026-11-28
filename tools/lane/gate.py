import datetime
import hashlib
import os
import re
import shutil
import subprocess
import sys
from collections.abc import Callable, Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from changes import changed_since

USAGE = """usage: gate.py

The local source gate: Quality's formatting, lint, dependency, crate-rule and doc-link checks, the
mechanical review findings of tools/lane/prereview.py, and each of these only when the branch,
committed since its base or in the working tree, changes its subject: the lane and clean-room tools' self-tests, verify's and the tools' tests, the planted
faults' manifest, the adjusted vectors' rule owners and each changed domain's adjusted vectors.

A pass with nothing uncommitted records its key: the tree, the base's tree, and what the checks
ran with. A later gate of the same key says whose pass it reuses and runs only the review findings,
which read the commit messages the key leaves out.
SM_GATE_FRESH=1 runs the checks anyway.

Environment: SM_GATE_BASE (default origin/main; a stacked branch names the branch it builds on),
SM_RUST_BIN (the toolchain's bin), SM_GATE_PASSED (where passes are recorded)."""

LANE = "tools/lane"
CLEAN = "tools/cleanroom"
SUITES = {
    f"{LANE}/test_worktree.py": (f"{LANE}/worktree.py", "tools/ci/mtimes.py"),
    f"{LANE}/test_gate.py": (f"{LANE}/gate.py", f"{LANE}/changes.py"),
    f"{CLEAN}/test_adjusted_vectors_run.py": (f"{CLEAN}/adjusted_vectors_run.py", f"{CLEAN}/adjusted_vectors_fixtures"),
}
OWN_TESTS = (
    f"{LANE}/played.py", f"{LANE}/land.py", f"{LANE}/test_land.py", f"{LANE}/prereview.py",
    f"{LANE}/test_prereview.py", f"{LANE}/__init__.py", f"{CLEAN}/__init__.py",
    f"{CLEAN}/adjusted_probe_tests.toml", f"{CLEAN}/adjusted_rule_owners", f"{CLEAN}/box_relay.py",
    f"{CLEAN}/test_box_relay.py", f"{CLEAN}/scan_lexicon", f"{CLEAN}/scan_allow.txt", f"{CLEAN}/allowed_deps.txt",
    f"{CLEAN}/scan_fixtures", f"{CLEAN}/rule_fixtures",
)
VERIFY_LIB = ("crates", "scenarios", "docs/behaviours", "docs/architecture/domains.toml", "README.md", "Cargo.toml",
              "Cargo.lock", "rust-toolchain.toml")
FAULTS = ("tools/mutants.toml", "tools/check_mutants.py", "crates/game/src/production", "crates/game/src/production.rs",
          "crates/verify/src/replay/played.rs", "scenarios/replay/index.toml", "scenarios/replay/baseline.toml")
OWNERS = (f"{CLEAN}/adjusted_rule_owners", f"{CLEAN}/adjusted_probe_tests.toml", f"{CLEAN}/adjusted_vectors_run.py",
          "scenarios/vectors", "docs/behaviours", "crates/clean")
UNINHERITED = ("SM_VECTOR_FILE",)
INCLUDE = re.compile(r"""^\s*(\[\[)?\s*["']?include["']?\s*(=|\]\])""", re.MULTILINE)
DOMAIN = re.compile(r"^docs/behaviours/([^/.]+)\.md$|^scenarios/vectors/([^/]+)\.toml$")
ENVIRONMENT = re.compile(r"^(RUST|CARGO|CLEANROOM_|CLIPPY_CONF_DIR$|SM_RUST_BIN$)")


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=False)


def out(*args: str) -> str:
    return git(*args).stdout.strip()


def self_tests(base: str) -> list[str]:
    known = [path for suite, subjects in SUITES.items() for path in (suite, *subjects)] + list(OWN_TESTS)
    if changed_since(base, [LANE, CLEAN, *(f":(exclude){path}" for path in known)]):
        return list(SUITES)
    return [suite for suite, subjects in SUITES.items() if changed_since(base, [suite, *subjects])]


def changed_domains(base: str) -> list[str]:
    roots = ("docs/behaviours", "scenarios/vectors")
    paths = out("diff", "--name-only", "--no-renames", f"{base}...HEAD", "--", *roots).splitlines()
    status = git("status", "--porcelain", "--no-renames", "--untracked-files=all", "--", *roots).stdout
    paths += [line[3:] for line in status.splitlines()]
    named = {next(name for name in match.groups() if name) for path in paths if (match := DOMAIN.match(path))}
    return sorted(name for name in named
                  if Path(f"docs/behaviours/{name}.md").is_file() and Path(f"scenarios/vectors/{name}.toml").is_file())


def configurations(top: Path) -> list[Path]:
    home = Path(os.environ.get("HOME", "/"))
    cargo_home = Path(os.environ.get("CARGO_HOME") or home / ".cargo")
    xdg = Path(os.environ.get("XDG_CONFIG_HOME") or home / ".config")
    found = [cargo_home / "config", cargo_home / "config.toml", home / "rustfmt.toml", home / ".rustfmt.toml",
             xdg / "rustfmt/rustfmt.toml", xdg / "rustfmt/.rustfmt.toml"]
    for start in (top, Path(os.environ.get("TEMP") or os.environ.get("TMPDIR") or "/tmp")):
        for directory in (start, *start.parents):
            found += [directory / ".cargo/config", directory / ".cargo/config.toml", directory / "rustfmt.toml",
                      directory / ".rustfmt.toml"]
    return found


def includes_another(configs: Sequence[Path]) -> bool:
    for config in configs:
        if config.name in ("config", "config.toml") and config.is_file():
            try:
                if INCLUDE.search(config.read_text(errors="replace")):
                    return True
            except OSError:
                return True
    return False


def digest(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        return str(error)


def ran_with(rust: Path, deny: Path | None, configs: Sequence[Path]) -> str:
    try:
        rustc = subprocess.run([str(rust / "rustc"), "--version"], capture_output=True, text=True, check=False)
        lines = [rustc.stdout + rustc.stderr]
    except OSError as error:
        lines = [str(error)]
    lines += [f"cargo-deny {deny} {digest(deny)}"] if deny else ["no cargo-deny"]
    lines += [sys.executable, sys.version]
    lines += sorted(f"{name}={value}" for name, value in os.environ.items() if ENVIRONMENT.match(name))
    lines += [f"{config} {digest(config)}" for config in configs if config.is_file() or config.is_symlink()]
    return hashlib.sha256("\n".join(lines).encode()).hexdigest()


def pass_key(base: str, rust: Path, deny: Path | None) -> str | None:
    top = Path(out("rev-parse", "--show-toplevel"))
    configs = configurations(top)
    if (out("status", "--porcelain", "--untracked-files=all")
            or re.search(r"(?m)^[a-zS]", out("ls-files", "-v"))
            or git("-C", str(top), "ls-files", "-o", "--exclude-per-directory=.gitignore").stdout.strip()
            or includes_another(configs)):
        return None
    left = git("merge-base", base, "HEAD")
    if left.returncode != 0:
        return None
    return f"{out('rev-parse', 'HEAD^{tree}')}-{out('rev-parse', left.stdout.strip() + '^{tree}')}-" \
           f"{ran_with(rust, deny, configs)}"


@dataclass
class Check:
    name: str
    failure: str
    commands: list[list[str]]
    reason: Callable[[list[str]], list[str]] = lambda log: log
    environment: dict[str, str] = field(default_factory=dict)
    without: tuple[str, ...] = ()
    status: int = 0
    log: list[str] = field(default_factory=list)

    def run(self) -> None:
        environment = {name: value for name, value in (os.environ | self.environment).items()
                       if name not in (*self.without, *UNINHERITED)}
        for command in self.commands:
            done = subprocess.run(command, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                  text=True, errors="replace", check=False)
            self.log += done.stdout.splitlines()
            if done.returncode != 0:
                self.status = done.returncode
                return

    def report(self) -> list[str]:
        shown = [line for line in self.reason(self.log) if line.strip()]
        return [self.failure, *(shown or self.log[-5:])]


def matching(pattern: str, limit: int = 20) -> Callable[[list[str]], list[str]]:
    return lambda log: [line for line in log if re.search(pattern, line)][:limit]


def after_errors(log: list[str]) -> list[str]:
    return [line for index, start in enumerate(log) if start.startswith("error") for line in log[index:index + 7]][:20]


def verify_failures(log: list[str]) -> list[str]:
    failed = [line for line in log if re.match(r"^test .* FAILED$", line)]
    panics = [line for index, start in enumerate(log) if "panicked at" in start for line in log[index:index + 2]]
    return (failed + panics)[:20]


def checks(base: str, rust: Path, deny: Path | None) -> tuple[list[Check], list[str]]:
    cargo = str(rust / "cargo")
    on_path = {"PATH": f"{rust}:{os.environ.get('PATH', '')}"}
    python = sys.executable
    found = [
        Check("fmt", "FMT FAIL", [[cargo, "fmt", "--all", "--check"]], matching(r"^Diff in")),
        Check("clippy", "CLIPPY FAIL",
              [[cargo, "clippy", "--locked", "--workspace", "--all-targets", "--no-deps", "--", "-D", "warnings"]],
              matching(r"^(error|warning)|-->", 10), without=("CLIPPY_CONF_DIR",)),
    ]
    if deny:
        found.append(Check("deny", "DENY FAIL", [[str(deny), "--locked", "check", "bans"]], after_errors,
                           environment=on_path))
    found += [
        Check("crate-rules", "CRATE RULES FAIL", [[cargo, "checks", "crate-rules"]]),
        Check("clean-room", "CLEAN ROOM FAIL", [[cargo, "cleanroom", "check"]]),
        *(Check(suite, f"SELF TEST FAIL {suite}", [[python, suite]], environment={"CARGO": cargo})
          for suite in self_tests(base)),
        Check("doc-links", "DOC LINKS FAIL", [[cargo, "checks", "doc-links"]], lambda log: log[:20]),
    ]
    skipped = []
    if changed_since(base, VERIFY_LIB):
        found.append(Check("verify-lib", "VERIFY LIB FAIL",
                           [[cargo, "test", "-p", "verify", "--lib"], [cargo, "test", "-p", "tools"]],
                           verify_failures, environment=on_path))
    else:
        skipped.append("VERIFY-LIB")
    if changed_since(base, FAULTS):
        found.append(Check("faults", "FAULTS FAIL: tools/check_mutants.py validate",
                           [[python, "-I", "tools/check_mutants.py", "validate"]], lambda log: log[-1:]))
    else:
        skipped.append("FAULTS")
    if changed_since(base, OWNERS):
        found.append(Check("owners", "OWNERS FAIL: tools/cleanroom/adjusted_vectors_run.py --owners",
                           [[python, "-I", f"{CLEAN}/adjusted_vectors_run.py", "--owners"]],
                           matching(r"^adjusted_vectors_run: ")))
    else:
        skipped.append("OWNERS")
    if domains := changed_domains(base):
        found.append(Check("adjusted", "ADJUSTED FAIL: tools/cleanroom/adjusted_vectors_run.py --missing",
                           [[python, "-I", f"{CLEAN}/adjusted_vectors_run.py", "--missing", *domains]],
                           matching(r" has no adjusted vector$")))
    else:
        skipped.append("ADJUSTED")
    return found, skipped


def stamp() -> str:
    return f"{out('rev-parse', '--short', 'HEAD')} {out('branch', '--show-current')}"


def main(arguments: list[str]) -> int:
    if arguments:
        print(USAGE, file=sys.stderr)
        return 2
    base = os.environ.get("SM_GATE_BASE") or "origin/main"
    if git("rev-parse", "--verify", "-q", f"{base}^{{commit}}").returncode != 0:
        print(f"GATE: the base {base} is not a commit here; set SM_GATE_BASE to the one the branch builds on")
        return 2
    cargo = shutil.which("cargo")
    rust = Path(os.environ.get("SM_RUST_BIN") or (Path(cargo).parent if cargo else "."))
    home = Path(os.environ.get("CARGO_HOME") or Path(os.environ.get("HOME", "/")) / ".cargo")
    deny_path = shutil.which("cargo-deny") or str(home / "bin/cargo-deny")
    deny = Path(deny_path) if os.access(deny_path, os.X_OK) else None
    passed = Path(os.environ.get("SM_GATE_PASSED")
                  or Path(os.environ.get("TEMP") or os.environ.get("TMPDIR") or "/tmp") / "sm-gate-passed")
    prereview = Check("prereview", "PREREVIEW FAIL", [[sys.executable, "-I", f"{LANE}/prereview.py", base]])
    prereview.run()
    key = pass_key(base, rust, deny)
    no_deny = "DENY SKIPPED: no cargo-deny here; CI's Quality runs cargo deny check bans"
    if key and os.environ.get("SM_GATE_FRESH") != "1" and (passed / key).is_file() and (passed / key).stat().st_size:
        if prereview.status != 0:
            print("\n".join([*prereview.report(), f"GATE FAILED {out('branch', '--show-current')}"]))
            return 1
        print(f"GATE REUSED: {(passed / key).read_text().strip()} passed this tree over the same base, "
              "with the same tools")
        if not deny:
            print(no_deny)
        print(f"GATE OK {stamp()}")
        return 0
    found, skipped = checks(base, rust, deny)
    with ThreadPoolExecutor(max_workers=len(found)) as pool:
        list(pool.map(Check.run, found))
    after_links = next(index for index, check in enumerate(found) if check.name == "doc-links") + 1
    found.insert(after_links, prereview)
    for check in found:
        if check.name == "crate-rules" and not deny:
            print(no_deny)
        if check.status != 0:
            print("\n".join(check.report()))
        if check.name == "verify-lib" and any(re.search(r"committed_readme_is_current .* FAILED", line)
                                              for line in check.log):
            print("README STALE: run cargo run -p tools --bin tools -- report --write")
    if not any(check.name in SUITES for check in found):
        skipped.insert(0, "SELF-TESTS")
    for name in skipped:
        print(f"{name} SKIPPED: no change to its subject since {base}")
    if any(check.status != 0 for check in found):
        print(f"GATE FAILED {out('branch', '--show-current')}")
        return 1
    if key and not out("status", "--porcelain", "--untracked-files=all") and out("rev-parse", "HEAD^{tree}") == \
            key.split("-")[0]:
        passed.mkdir(parents=True, exist_ok=True)
        now = datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
        (passed / key).write_text(f"{stamp()} {now}\n")
    print(f"GATE OK {stamp()}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
