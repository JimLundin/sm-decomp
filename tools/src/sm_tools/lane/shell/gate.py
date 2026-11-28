import datetime as dt
import hashlib
import os
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
lazy from collections.abc import Sequence

from ordered import FrozenList, FrozenOrderedDict

from sm_tools.lane.core.gate import (
    DOMAIN_ROOTS,
    FAULTS,
    NO_DENY,
    NOT_RUN,
    OWNERS,
    SUITES,
    USAGE,
    VERIFY_LIB,
    Changed,
    Check,
    Homes,
    Ran,
    RanWith,
    Tools,
    check_environment,
    checks,
    configurations,
    domain_files,
    failed,
    in_order,
    includes_another,
    is_cargo_configuration,
    key_tree,
    named_domains,
    pass_key,
    prereview_check,
    ran_with_digest,
    report,
    results,
    reused,
    suite_paths,
    unknown_tool_files,
    unresolved,
    untracked_state,
)
from sm_tools.lane.shell.changes import changed_since
from sm_tools.lane.shell.scratch import scratch
from sm_tools.shell.process import git


def out(*arguments: str) -> str:
    return git(*arguments).stdout.strip()


def self_tests(base: str) -> FrozenList[str]:
    if changed_since(base, unknown_tool_files()):
        return FrozenList(SUITES)
    return FrozenList(
        suite for suite in SUITES if changed_since(base, suite_paths(suite))
    )


def changed_domains(base: str) -> FrozenList[str]:
    committed = out(
        "diff",
        "--name-only",
        "--no-renames",
        f"{base}...HEAD",
        "--",
        *DOMAIN_ROOTS,
    ).splitlines()
    status = git(
        "status",
        "--porcelain",
        "--no-renames",
        "--untracked-files=all",
        "--",
        *DOMAIN_ROOTS,
    ).stdout
    paths = (*committed, *(line[3:] for line in status.splitlines()))
    return FrozenList(
        name
        for name in named_domains(paths)
        if all(Path(path).is_file() for path in domain_files(name))
    )


def homes() -> Homes:
    home = Path(os.environ.get("HOME", "/"))
    return Homes(
        home=home,
        cargo=Path(os.environ.get("CARGO_HOME") or home / ".cargo"),
        xdg=Path(os.environ.get("XDG_CONFIG_HOME") or home / ".config"),
        temp=scratch(),
    )


def any_include(configs: Sequence[Path]) -> bool:
    for config in configs:
        if is_cargo_configuration(config) and config.is_file():
            try:
                text = config.read_text(errors="replace")
            except OSError:
                return True
            if includes_another(text):
                return True
    return False


def digest(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        return str(error)


def rustc_version(rust: Path) -> str:
    try:
        rustc = subprocess.run(
            (str(rust / "rustc"), "--version"),
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        return str(error)
    return rustc.stdout + rustc.stderr


def ran_with(rust: Path, deny: Path | None, configs: Sequence[Path]) -> str:
    return ran_with_digest(
        RanWith(
            rustc=rustc_version(rust),
            deny=f"cargo-deny {deny} {digest(deny)}" if deny else None,
            python=sys.executable,
            version=sys.version,
            environment=FrozenOrderedDict(os.environ),
            configurations=FrozenList(
                f"{config} {digest(config)}"
                for config in configs
                if config.is_file() or config.is_symlink()
            ),
        )
    )


def unrecorded(top: Path, configs: Sequence[Path]) -> bool:
    others = git(
        "-C", str(top), "ls-files", "-o", "--exclude-per-directory=.gitignore"
    ).stdout
    return untracked_state(
        out("status", "--porcelain", "--untracked-files=all"),
        out("ls-files", "-v"),
        others,
    ) or any_include(configs)


def key_of(base: str, rust: Path, deny: Path | None) -> str | None:
    top = Path(out("rev-parse", "--show-toplevel"))
    configs = FrozenList(Path(path) for path in configurations(top, homes()))
    if unrecorded(top, configs):
        return None
    left = git("merge-base", base, "HEAD")
    if left.returncode != 0:
        return None
    return pass_key(
        out("rev-parse", "HEAD^{tree}"),
        out("rev-parse", left.stdout.strip() + "^{tree}"),
        ran_with(rust, deny, configs),
    )


def run_check(check: Check) -> Ran:
    environment = check_environment(check, os.environ)
    log: list[str] = []
    for command in check.commands:
        try:
            done = subprocess.run(
                command,
                env=environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                errors="replace",
                check=False,
            )
        except OSError as error:
            return Ran(check, NOT_RUN, FrozenList((*log, str(error))))
        log.extend(done.stdout.splitlines())
        if done.returncode != 0:
            return Ran(check, done.returncode, FrozenList(log))
    return Ran(check, 0, FrozenList(log))


def branch() -> str:
    return out("branch", "--show-current")


def stamp() -> str:
    return f"{out('rev-parse', '--short', 'HEAD')} {branch()}"


def changed(base: str) -> Changed:
    return Changed(
        suites=self_tests(base),
        verify_lib=changed_since(base, VERIFY_LIB),
        faults=changed_since(base, FAULTS),
        owners=changed_since(base, OWNERS),
        domains=changed_domains(base),
    )


def toolchain() -> tuple[Path, Path | None]:
    cargo = shutil.which("cargo")
    rust = Path(
        os.environ.get("SM_RUST_BIN") or (Path(cargo).parent if cargo else ".")
    )
    home = Path(
        os.environ.get("CARGO_HOME")
        or Path(os.environ.get("HOME", "/")) / ".cargo"
    )
    deny = shutil.which("cargo-deny") or str(home / "bin/cargo-deny")
    return rust, Path(deny) if os.access(deny, os.X_OK) else None


def passes() -> Path:
    return Path(
        os.environ.get("SM_GATE_PASSED") or scratch() / "sm-gate-passed"
    )


def reuse(key: str | None, prereview: Ran, *, deny: bool) -> int | None:
    record = passes() / key if key else None
    if (
        record is None
        or os.environ.get("SM_GATE_FRESH") == "1"
        or not record.is_file()
        or not record.stat().st_size
    ):
        return None
    if prereview.status != 0:
        print("\n".join((*report(prereview), f"GATE FAILED {branch()}")))
        return 1
    print(reused(record.read_text()))
    if not deny:
        print(NO_DENY)
    print(f"GATE OK {stamp()}")
    return 0


def record(key: str | None) -> None:
    if (
        key
        and not out("status", "--porcelain", "--untracked-files=all")
        and out("rev-parse", "HEAD^{tree}") == key_tree(key)
    ):
        passed = passes()
        passed.mkdir(parents=True, exist_ok=True)
        now = dt.datetime.now(dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
        _ = (passed / key).write_text(f"{stamp()} {now}\n")


def gate(base: str) -> int:
    rust, deny = toolchain()
    prereview = run_check(prereview_check(base, sys.executable))
    key = key_of(base, rust, deny)
    reused_status = reuse(key, prereview, deny=deny is not None)
    if reused_status is not None:
        return reused_status
    tools = Tools(
        rust, deny, sys.executable, f"{rust}:{os.environ.get('PATH', '')}"
    )
    found = changed(base)
    planned = checks(tools, found)
    with ThreadPoolExecutor(max_workers=len(planned)) as pool:
        ran = in_order(FrozenList(pool.map(run_check, planned)), prereview)
    lines = results(ran, found, base, deny=deny is not None)
    if lines:
        print("\n".join(lines))
    if failed(ran):
        print(f"GATE FAILED {branch()}")
        return 1
    record(key)
    print(f"GATE OK {stamp()}")
    return 0


def run(arguments: Sequence[str]) -> int:
    if arguments:
        print(USAGE, file=sys.stderr)
        return 2
    base = os.environ.get("SM_GATE_BASE") or "origin/main"
    if git("rev-parse", "--verify", "-q", f"{base}^{{commit}}").returncode != 0:
        print(unresolved(base))
        return 2
    return gate(base)


def main() -> int:
    return run(sys.argv[1:])
