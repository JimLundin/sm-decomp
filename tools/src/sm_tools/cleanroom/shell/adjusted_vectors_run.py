import os
import subprocess
import sys
import tempfile
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path
lazy from collections.abc import Callable, Mapping, Sequence

from ordered import FrozenList, FrozenOrderedDict

from sm_tools.cleanroom.core.adjusted_vectors_run import (
    CLEAN_CRATES,
    HELP,
    LIBTEST_FAILED,
    NAME,
    OWNERS_FOLDER,
    PROBE_TESTS,
    SPECS,
    USAGE,
    VECTOR_FILE,
    VECTORS,
    CannotProbeError,
    Domain,
    Owners,
    RunProbe,
    Verdict,
    Verdicts,
    artifacts,
    build_command,
    build_groups,
    built_crates,
    check_owners,
    completed_test,
    crate_of,
    domain,
    missing,
    owners_line,
    report,
    step,
    test_command,
    unbuilt,
    without_vector_file,
)
from sm_tools.cleanroom.core.adjusted_vectors_run import (
    test_target as target_of,
)


def prove(
    text: str,
    entries: Sequence[str],
    owners: Owners,
    run: Callable[[str], bool],
) -> Verdicts:
    answers: list[bool] = []
    while isinstance(
        found := step(text, entries, owners, FrozenList(answers)), RunProbe
    ):
        answers.append(run(found.text))
    return found


def test_target(crate: str, root: Path) -> FrozenList[str]:
    tests = root / CLEAN_CRATES / crate / "tests"
    dedicated = (tests / "vectors.rs").is_file() or (
        tests / "vectors/main.rs"
    ).is_file()
    return target_of(dedicated=dedicated)


def build(root: Path, crates: Sequence[str]) -> Mapping[str, Path]:
    groups = build_groups(
        FrozenList((crate, test_target(crate, root)) for crate in crates)
    )
    built: dict[str, Path] = {}
    for target, members in groups.items():
        try:
            result = subprocess.run(
                build_command(members, target),
                cwd=root,
                capture_output=True,
                check=False,
            )
        except OSError as error:
            message = "cargo could not be launched"
            raise CannotProbeError(message) from error
        if result.returncode != 0:
            _ = sys.stderr.buffer.write(result.stderr)
            message = "the vector tests don't build"
            raise CannotProbeError(message)
        built.update(
            (crate, Path(executable))
            for crate, executable in built_crates(
                artifacts(result.stdout)
            ).items()
        )
    if (refusal := unbuilt(crates, built)) is not None:
        raise CannotProbeError(refusal)
    return FrozenOrderedDict((crate, built[crate]) for crate in crates)


def passes(
    executable: Path,
    cwd: Path,
    selection: str | None,
    vector_file: Path | None = None,
) -> bool:
    env = dict(without_vector_file(os.environ))
    if vector_file is not None:
        env[VECTOR_FILE] = str(vector_file)
    try:
        result = subprocess.run(
            test_command(str(executable), selection),
            cwd=cwd,
            env=env,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        message = "the test runner could not be launched"
        raise CannotProbeError(message) from error
    return completed_test(
        result.stdout, result.returncode, LIBTEST_FAILED, selection
    )


def file_runs(
    executable: Path, cwd: Path, selection: str | None, scratch: Path
) -> Callable[[str], bool]:
    vector_file = scratch / "vectors.toml"

    def run(text: str) -> bool:
        _ = vector_file.write_text(text, encoding="utf-8")
        return passes(executable, cwd, selection, vector_file)

    return run


def text_if_file(path: Path) -> str | None:
    return path.read_text(encoding="utf-8") if path.is_file() else None


def rule_owners_text(root: Path, name: str) -> str | None:
    return text_if_file(root / OWNERS_FOLDER / f"{name}.toml")


def probe_tests(root: Path) -> bytes | None:
    path = root / PROBE_TESTS
    return path.read_bytes() if path.is_file() else None


def load(root: Path, name: str) -> Domain:
    spec = (root / SPECS / f"{name}.md").read_text(encoding="utf-8")
    text = (root / VECTORS / f"{name}.toml").read_text(encoding="utf-8")
    return domain(
        name, spec, text, rule_owners_text(root, name), probe_tests(root)
    )


def verdicts_of(
    root: Path, name: str, cwd: Path, executable: Path
) -> FrozenList[tuple[str, Verdict]]:
    loaded = load(root, name)
    with tempfile.TemporaryDirectory() as scratch:
        run = file_runs(executable, cwd, loaded.selected, Path(scratch))
        try:
            found = prove(loaded.text, loaded.entries, loaded.owners, run)
        except CannotProbeError as error:
            message = f"{loaded.name}: {error}"
            raise CannotProbeError(message) from None
    return FrozenList(found.items())


def check_domains(root: Path, names: Sequence[str]) -> int:
    loaded = FrozenList(load(root, name) for name in names)
    cwds = FrozenList(root / CLEAN_CRATES / found.crate for found in loaded)
    executables = build(root, FrozenList(found.crate for found in loaded))
    with ProcessPoolExecutor() as pool:
        baselines = pool.map(
            passes,
            [executables[found.crate] for found in loaded],
            cwds,
            [None] * len(loaded),
        )
        for found, passed in zip(loaded, baselines, strict=True):
            if not passed:
                message = (
                    f"{found.name}: the vector test fails on the committed"
                    " vectors"
                )
                raise CannotProbeError(message)
        for found in loaded:
            check_owners(found.text, found.entries, found.owners)
        proofs = FrozenList(
            pool.submit(
                verdicts_of, root, found.name, cwd, executables[found.crate]
            )
            for found, cwd in zip(loaded, cwds, strict=True)
        )
        unproven = 0
        for found, proof in zip(loaded, proofs, strict=True):
            done = report(found.name, FrozenOrderedDict(proof.result()))
            print("\n".join(done.lines))
            unproven += done.unproven
        return unproven


def domains(root: Path) -> FrozenList[str]:
    return FrozenList(
        sorted(
            spec.stem
            for spec in (root / SPECS).glob("*.md")
            if "." not in spec.stem
            and (root / VECTORS / f"{spec.stem}.toml").is_file()
            and (root / CLEAN_CRATES / crate_of(spec.stem)).is_dir()
        )
    )


def check_domain_owners(root: Path, name: str) -> None:
    found = load(root, name)
    try:
        check_owners(found.text, found.entries, found.owners)
    except CannotProbeError as error:
        message = f"{name}: {error}"
        raise CannotProbeError(message) from None
    print(owners_line(name))


def missing_in(root: Path, name: str) -> int:
    spec = root / SPECS / f"{name}.md"
    if not spec.is_file():
        message = f"no spec {spec}"
        raise CannotProbeError(message)
    text = text_if_file(root / VECTORS / f"{name}.toml") or ""
    found = missing(name, spec.read_text(encoding="utf-8"), text)
    print("\n".join(found.lines))
    return found.unproven


def refuse(error: str) -> int:
    print(f"{NAME}: {error}", file=sys.stderr)
    return 2


def run(arguments: Sequence[str], root: Path) -> int:
    if FrozenList(arguments) in HELP:
        print(USAGE)
        return 0
    first = arguments[0] if arguments else ""
    if first == "--missing":
        if len(arguments) == 1:
            return refuse("--missing needs a DOMAIN")
        try:
            return (
                1
                if sum(missing_in(root, name) for name in arguments[1:])
                else 0
            )
        except CannotProbeError as error:
            return refuse(str(error))
    owners_only = first == "--owners"
    named = arguments[1:] if owners_only else arguments
    unproven = 0
    try:
        if owners_only:
            for name in named or domains(root):
                check_domain_owners(root, name)
        else:
            unproven = check_domains(root, named or domains(root))
    except CannotProbeError as error:
        return refuse(str(error))
    return 1 if unproven else 0


def main() -> int:
    return run(FrozenList(sys.argv[1:]), Path.cwd())
