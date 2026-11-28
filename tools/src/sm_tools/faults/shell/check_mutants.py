import argparse
import contextlib
import fcntl
import functools
import os
import signal
import sys
import time
from collections.abc import Callable, Generator, Mapping, Sequence
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.faults.core.check_mutants import (
    ADAPTERS,
    BASELINE,
    BEHAVIOURS,
    DOMAINS,
    INDEX,
    MANIFEST,
    OWNER_FOLDERS,
    RUST_PANIC,
    SELECTABLE,
    Coverage,
    Fault,
    Logged,
    Manifest,
    Receipt,
    Replay,
    baseline_verdict,
    build_errors,
    catalogue_domains,
    catalogue_rules,
    check_inputs,
    coverage,
    fault_verdict,
    manifest,
    owned_source,
    replay_command,
    reported_executable,
    rule_coverage,
    source_path,
    timing,
    validate_anchors,
    validate_coverage,
)
from sm_tools.faults.core.selectable_faults import (
    check_selector_dependencies,
    dependencies,
    instrumented_files,
    selectable_sources,
)
from sm_tools.faults.shell import rust_syntax
from sm_tools.shell import process

ROOT = Path(__file__).resolve().parents[5]
REPLAY_CRASH_SIGNALS = (
    signal.SIGSEGV,
    signal.SIGABRT,
    signal.SIGILL,
    signal.SIGBUS,
    signal.SIGFPE,
)
CRASHES = FrozenList((
    RUST_PANIC,
    *(-number for number in REPLAY_CRASH_SIGNALS),
))
SELECTOR_VARIABLES = ("GAMMA_ACTIVE", "GAMMA_REACH")
REPLAY_SOURCES = ("SM_ROM", "SM_CACHE", "SM_AUDIO_NATIVE")

type Listing = Callable[[Path], FrozenOrderedSet[Path]]


class Arguments(argparse.Namespace):
    command: str | None = None
    logs: Sequence[Path] = ()


def tracked_inputs(root: Path) -> FrozenOrderedSet[Path]:
    listed = process.checked(
        ("git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"),
        cwd=root,
    )
    return FrozenOrderedSet(
        root / os.fsdecode(name) for name in listed.split(b"\0") if name
    )


def scratch_files(tree: Path) -> FrozenOrderedSet[Path]:
    return FrozenOrderedSet(
        Path(directory, name)
        for directory, _, names in os.walk(tree)
        for name in names
    )


def file_bytes(path: Path) -> bytes | None:
    return path.read_bytes() if path.exists() else None


@dataclass(frozen=True, slots=True)
class FrozenInputs:
    files: FrozenOrderedDict[Path, bytes | None]
    listing: Listing = tracked_inputs

    def check(self, root: Path) -> str:
        started = time.perf_counter()
        current = FrozenOrderedDict(
            (path, file_bytes(path)) for path in self.files
        )
        check_inputs(self.files, self.listing(root), current)
        return timing("input-check", time.perf_counter() - started)


def capture(root: Path, listing: Listing = tracked_inputs) -> FrozenInputs:
    return FrozenInputs(
        FrozenOrderedDict((path, file_bytes(path)) for path in listing(root)),
        listing,
    )


def cargo() -> str:
    return os.environ.get("CARGO", "cargo")


def build_executable(root: Path, *, target: Path) -> Path:
    started = time.perf_counter()
    command = (
        cargo(),
        "build",
        "--locked",
        "--profile",
        "verify",
        "-p",
        "tools",
        "--bin",
        "tools",
        "--message-format=json",
        "--target-dir",
        str(target),
    )
    built = process.run(command, cwd=root)
    elapsed = time.perf_counter() - started
    print(timing("compilation", elapsed), flush=True)
    output = built.stdout.decode("utf-8", errors="replace")
    if built.status:
        for message in build_errors(output):
            print(message, file=sys.stderr)
        raise process.CommandError(command, built.status)
    executable = (root / reported_executable(output)).resolve()
    if not executable.is_file():
        message = "reported tools executable is missing"
        raise ValueError(message)
    return executable


def run_case(
    root: Path, case: str, executable: Path, environment: Mapping[str, str]
) -> Replay:
    command = replay_command(case, str(executable), environment)
    started = time.perf_counter()
    done = process.run(command, cwd=root, environment=environment)
    return Replay(done.status, done.output(), time.perf_counter() - started)


def replay_dependencies(tree: Path, *, locked: bool) -> FrozenOrderedSet[str]:
    command = (
        cargo(),
        "tree",
        "-p",
        "tools",
        "--edges",
        "normal,build",
        "--prefix",
        "none",
        "--format",
        "{p} [{f}]",
        *(("--locked",) if locked else ()),
    )
    listed = process.checked(command, cwd=tree, capture_stderr=False)
    return dependencies(listed.decode())


def coverage_inputs(root: Path) -> Coverage:
    return coverage(
        (root / INDEX).read_text(encoding="utf-8"),
        (root / BASELINE).read_text(encoding="utf-8"),
        catalogue_rules(catalogue_specs(root)),
    )


def catalogue_specs(root: Path) -> FrozenOrderedDict[str, str]:
    domains = catalogue_domains((root / DOMAINS).read_text(encoding="utf-8"))
    specs = (root / BEHAVIOURS / f"{domain}.md" for domain in domains)
    return FrozenOrderedDict(
        (str(spec), spec.read_text(encoding="utf-8")) for spec in specs
    )


def resolved_source(root: Path, source: str) -> Path:
    return owned_source(
        (root / source_path(source)).resolve(),
        FrozenList(
            (root / adapter.parent).resolve() / adapter.name
            for adapter in ADAPTERS
        ),
        FrozenList((root / folder).resolve() for folder in OWNER_FOLDERS),
    )


def sources(root: Path, loaded: Manifest) -> FrozenOrderedDict[str, bytes]:
    for file in loaded.callers:
        _ = resolved_source(root, file)
    return FrozenOrderedDict(
        (entry.file, resolved_source(root, entry.file).read_bytes())
        for entry in loaded.faults
    )


def selectable_files(
    root: Path, inputs: FrozenInputs, loaded: Manifest
) -> FrozenOrderedDict[PurePosixPath, bytes]:
    files = FrozenOrderedDict(
        (PurePosixPath(path.relative_to(root)), data)
        for path, data in inputs.files.items()
        if data is not None
    )
    syntax = FrozenOrderedDict(
        (file, rust_syntax.parse(files[source_path(file)]))
        for file in instrumented_files(loaded)
    )
    return selectable_sources(files, syntax, loaded)


def sync_tree(
    root: Path, tree: Path, files: Mapping[PurePosixPath, bytes]
) -> None:
    for directory, _, names in os.walk(tree, topdown=False):
        folder = Path(directory)
        for name in names:
            if PurePosixPath((folder / name).relative_to(tree)) not in files:
                (folder / name).unlink()
        if folder != tree and not any(folder.iterdir()):
            folder.rmdir()
    for relative, data in files.items():
        copied = tree / relative
        if (
            copied.is_symlink()
            or not copied.is_file()
            or copied.read_bytes() != data
        ):
            copied.unlink(missing_ok=True)
            copied.parent.mkdir(parents=True, exist_ok=True)
            _ = copied.write_bytes(data)
        copied.chmod((root / relative).stat().st_mode & 0o7777)


def check_dependencies(root: Path, tree: Path) -> None:
    check_selector_dependencies(
        str(root),
        str(tree),
        replay_dependencies(root, locked=True),
        replay_dependencies(tree, locked=False),
    )


@dataclass(frozen=True, slots=True)
class Build:
    tree: Path
    executable: Path
    environment: Mapping[str, str]


def in_order[V](
    jobs: Sequence[Callable[[], Logged[V]]], workers: int
) -> Generator[V]:
    with ThreadPoolExecutor(workers) as pool:
        futures = FrozenList(pool.submit(job) for job in jobs)
        try:
            for future in futures:
                done = future.result()
                for line in done.lines:
                    print(line, flush=True)
                yield done.value
        finally:
            pool.shutdown(cancel_futures=True)


def check_baseline(build: Build, case: str) -> Logged[Receipt | None]:
    replay = run_case(build.tree, case, build.executable, build.environment)
    return baseline_verdict(case, replay)


def check_fault(
    build: Build, ordinal: int, entry: Fault, baseline: Receipt
) -> Logged[bool]:
    selected = {**build.environment, "GAMMA_ACTIVE": str(ordinal)}
    faulty = run_case(build.tree, entry.case, build.executable, selected)
    restored = run_case(
        build.tree, entry.case, build.executable, build.environment
    )
    return fault_verdict(entry, faulty, restored, baseline, CRASHES)


def check_phase(tree: Path, frozen: FrozenInputs, phase: str) -> None:
    try:
        print(frozen.check(tree), flush=True)
    except ValueError as error:
        message = f"{phase}: {error}"
        raise ValueError(message) from error


def replay_selectable(
    build: Build, faults: Sequence[Fault], frozen: FrozenInputs, workers: int
) -> bool:
    cases = FrozenList(FrozenOrderedSet(entry.case for entry in faults))
    baselines: dict[str, Receipt] = {}
    baseline_jobs = FrozenList(
        functools.partial(check_baseline, build, case) for case in cases
    )
    with contextlib.closing(in_order(baseline_jobs, workers)) as results:
        for case, baseline in zip(cases, results, strict=True):
            if baseline is None:
                break
            baselines[case] = baseline
    check_phase(build.tree, frozen, "baseline phase")
    if len(baselines) != len(cases):
        return False
    fault_jobs = FrozenList(
        functools.partial(
            check_fault, build, ordinal, entry, baselines[entry.case]
        )
        for ordinal, entry in enumerate(faults, 1)
    )
    verdicts = FrozenList(in_order(fault_jobs, workers))
    check_phase(build.tree, frozen, "fault phase")
    return all(verdicts)


def replay_environment(root: Path) -> FrozenOrderedDict[str, str]:
    inherited = FrozenOrderedDict(
        (name, value)
        for name, value in os.environ.items()
        if name not in SELECTOR_VARIABLES
    )
    resolved = FrozenOrderedDict(
        (name, str((root / value).resolve()))
        for name in REPLAY_SOURCES
        if (value := inherited.get(name))
    )
    return inherited | resolved


def run_selectable(root: Path, inputs: FrozenInputs, loaded: Manifest) -> int:
    started = time.perf_counter()
    environment = replay_environment(root)
    scratch = root / SELECTABLE
    scratch.mkdir(parents=True, exist_ok=True)
    with (scratch / "lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        tree = scratch / "source"
        sync_tree(root, tree, selectable_files(root, inputs, loaded))
        print(inputs.check(root), flush=True)
        check_dependencies(root, tree)
        frozen = capture(tree, scratch_files)
        executable = build_executable(tree, target=scratch / "build")
        print(frozen.check(tree), flush=True)
        success = replay_selectable(
            Build(tree, executable, environment),
            loaded.faults,
            frozen,
            len(os.sched_getaffinity(0)),
        )
        print(inputs.check(root), flush=True)
    elapsed = time.perf_counter() - started
    print(
        timing(f"selectable faults={len(loaded.faults)}", elapsed), flush=True
    )
    return 0 if success else 1


def parser() -> argparse.ArgumentParser:
    made = argparse.ArgumentParser(
        description="Check replay fault detection and restoration.",
        allow_abbrev=False,
    )
    commands = made.add_subparsers(dest="command")
    _ = commands.add_parser(
        "selectable", help="run the complete manifest in one selectable build"
    )
    _ = commands.add_parser(
        "validate",
        help="validate the manifest without building or running replay",
    )
    rules = commands.add_parser(
        "rule-coverage",
        help="report every catalogue rule using logs from reviewed runs of this roster",
    )
    _ = rules.add_argument("logs", nargs="*", type=Path, metavar="LOG")
    return made


def run(arguments: Sequence[str], root: Path) -> int:
    parsed = parser().parse_args(arguments, namespace=Arguments())
    raw = (root / MANIFEST).read_bytes()
    loaded = manifest(raw.decode())
    known = coverage_inputs(root)
    validate_coverage(known, loaded.faults)
    validate_anchors(loaded.faults, sources(root, loaded))
    if parsed.command == "rule-coverage":
        logs = FrozenList(
            log.read_text(encoding="utf-8") for log in parsed.logs
        )
        print("\n".join(rule_coverage(known.rules, loaded.faults, logs)))
        return 0
    if parsed.command == "validate":
        return 0
    inputs = capture(root)
    if inputs.files.get(root / MANIFEST) != raw:
        message = "fault manifest changed during validation"
        raise ValueError(message)
    return run_selectable(root, inputs, loaded)


def main() -> int:
    return run(sys.argv[1:], ROOT)
