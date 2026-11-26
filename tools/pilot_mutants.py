import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
from collections.abc import Generator, Mapping, Sequence
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from check_mutants import (
    PLAYED,
    Fault,
    build_executable,
    check_fault,
    passed,
    read_manifest,
    receipt,
    require_unique_anchor,
    run_case,
    source_path,
)

PILOT = (
    "played-tail-hit-not-reacted",
    "appearance-ordinary-body-loses-flicker-phase",
)
RUNTIME = "0.8.0"
FIELD = re.compile(
    r"(?P<indent>\s*)(?P<name>[a-z_][a-z_0-9]*)(?:: (?P<value>.+))?,"
)


def switched_field(fault: Fault, ordinal: int) -> str:
    original = FIELD.fullmatch(fault.before)
    changed = FIELD.fullmatch(fault.after)
    if not original or not changed or original["name"] != changed["name"]:
        raise ValueError(
            f"{fault.name} needs a literal build: expected one struct field"
        )
    name = original["name"]
    before = original["value"] or name
    after = changed["value"] or name
    return f"{original['indent']}{name}: if ::gamma_rt::a({ordinal}) {{ {after} }} else {{ {before} }},"


def instrument(root: Path, faults: Sequence[Fault]) -> None:
    for ordinal, fault in enumerate(faults, 1):
        path = source_path(root, fault.file)
        if not path.is_relative_to(root / "crates/game/src/production"):
            raise ValueError(f"{fault.name} must change a gameplay decision")
        original = path.read_bytes()
        require_unique_anchor(fault, original)
        path.write_bytes(
            original.replace(
                fault.before.encode(),
                switched_field(fault, ordinal).encode(),
                1,
            )
        )
    with (root / "crates/game/Cargo.toml").open(
        "a", encoding="utf-8", newline="\n"
    ) as manifest:
        manifest.write(
            f'\n[dependencies.cargo-gamma-rt]\nversion = "={RUNTIME}"\n'
        )


def build_dependencies(root: Path, *, locked: bool) -> tuple[str, ...]:
    command = [
        os.environ.get("CARGO", "cargo"),
        "tree",
        "-p",
        "tools",
        "--edges",
        "normal,build",
        "--prefix",
        "none",
        "--format",
        "{p} [{f}]",
    ]
    if locked:
        command.append("--locked")
    output = subprocess.check_output(command, cwd=root, text=True)
    return tuple(
        sorted(
            dict.fromkeys(
                line.removesuffix(" (*)") for line in output.splitlines()
            )
        )
    )


@dataclass(frozen=True)
class PilotReceipt:
    clean: Mapping[str, Any]
    faulty: Mapping[str, Any]


def selectable(root: Path, faults: Sequence[Fault]) -> list[PilotReceipt]:
    normal = build_dependencies(root, locked=True)
    if any(package.startswith("cargo-gamma-rt v") for package in normal):
        raise ValueError(
            "the ordinary replay build already depends on cargo-gamma-rt"
        )
    instrument(root, faults)
    expected_packages = tuple(
        sorted((*normal, f"cargo-gamma-rt v{RUNTIME} []"))
    )
    if build_dependencies(root, locked=False) != expected_packages:
        raise ValueError(
            "the pilot changed the replay build dependencies beyond cargo-gamma-rt"
        )
    executable = build_executable(root)
    receipts = []
    for ordinal, fault in enumerate(faults, 1):
        clean = run_case(root, fault.case, executable)
        baseline = receipt(clean[1], fault.case)
        if not passed(clean, fault.case) or baseline is None:
            raise ValueError(f"selectable baseline failed for {fault.case}")
        code, output = run_case(
            root,
            fault.case,
            executable,
            env={**os.environ, "GAMMA_ACTIVE": str(ordinal)},
        )
        faulty = receipt(output, fault.case)
        expected = {
            "status": "difference",
            "cut": fault.cut,
            "field": fault.field,
        }
        if (
            code != (0 if fault.case.startswith(PLAYED) else 1)
            or faulty is None
            or faulty.get("outcome") != expected
        ):
            raise ValueError(
                f"{fault.name}: expected cut{fault.cut} {fault.field}, receipt mismatch (exit {code})"
            )
        restored = run_case(root, fault.case, executable)
        if (
            not passed(restored, fault.case)
            or receipt(restored[1], fault.case) != baseline
        ):
            raise ValueError(f"selectable restoration failed for {fault.name}")
        print(
            f"SELECTABLE {fault.name} cut{fault.cut} {fault.field} restored",
            flush=True,
        )
        receipts.append(PilotReceipt(baseline, faulty))
    return receipts


def literal(
    root: Path, faults: Sequence[Fault], receipts: Sequence[PilotReceipt]
) -> None:
    built = build_executable(root)
    vanilla = root.parent / built.name
    shutil.copy2(built, vanilla)
    for index, (fault, expected) in enumerate(
        zip(faults, receipts, strict=True)
    ):
        clean = run_case(
            root,
            fault.case,
            vanilla,
            env={**os.environ, "GAMMA_ACTIVE": str(index + 1)},
        )
        if (
            not passed(clean, fault.case)
            or receipt(clean[1], fault.case) != expected.clean
        ):
            raise ValueError(
                f"literal and selectable baselines differ for {fault.case}"
            )
        restored_binary = vanilla if index < len(faults) - 1 else None
        if not check_fault(
            root,
            fault,
            expected.clean,
            restored_binary,
            expected_fault=expected.faulty,
        ):
            raise ValueError(
                f"literal and selectable results differ for {fault.name}"
            )


@contextmanager
def worker(
    parent: Path, archive: Path, mode: str
) -> Generator[Path, None, None]:
    started = time.perf_counter()
    print(f"WORKER {mode} start", flush=True)
    with tempfile.TemporaryDirectory(
        dir=parent, prefix=f"{mode}-"
    ) as directory:
        root = Path(directory) / "workspace"
        root.mkdir()
        with tarfile.open(archive) as snapshot:
            snapshot.extractall(root, filter="data")
        os.environ["CARGO_TARGET_DIR"] = str(root / "target")
        yield root
    print(
        f"WORKER {mode} seconds={time.perf_counter() - started:.6f}", flush=True
    )


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    revision = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=root, text=True
    ).strip()
    print(
        f"PILOT revision={revision} workers=1 per method, empty Cargo targets",
        flush=True,
    )
    os.environ.pop("GAMMA_ACTIVE", None)
    os.environ.pop("GAMMA_REACH", None)
    for variable in ("SM_CACHE", "SM_ROM", "SM_AUDIO_NATIVE"):
        if value := os.environ.get(variable):
            os.environ[variable] = str((root / value).resolve())
    os.environ.setdefault("SM_ROM", str(root / "rom/sm.smc"))
    target = root / "target"
    target.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(
        dir=target, prefix="replay-pilot-"
    ) as directory:
        parent = Path(directory)
        archive = parent / "source.tar"
        subprocess.run(
            ["git", "archive", "-o", str(archive), revision],
            cwd=root,
            check=True,
        )
        with worker(parent, archive, "selectable") as tree:
            _, roster, _ = read_manifest(tree / "tools/mutants.toml")
            by_name = {fault.name: fault for fault in roster}
            faults = [by_name[name] for name in PILOT]
            receipts = selectable(tree, faults)
        with worker(parent, archive, "literal") as tree:
            literal(tree, faults, receipts)
    print(
        "PILOT PASS: complete clean, faulty and restored receipts agree",
        flush=True,
    )


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
