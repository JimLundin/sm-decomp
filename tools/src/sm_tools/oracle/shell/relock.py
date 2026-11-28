import functools
import gzip
import os
import shutil
import subprocess
import sys
import tarfile
import time
import zipfile
from concurrent.futures import FIRST_COMPLETED, Future, ThreadPoolExecutor, wait
from dataclasses import dataclass
from pathlib import Path
lazy from collections.abc import Callable, Mapping, Sequence

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet
lazy from ordered import OrderedSet

from sm_tools.ci.core.replay_data import CIPHER
from sm_tools.ci.shell.replay_data import cipher_env, normalized, sha256
from sm_tools.oracle.core.relock import (
    ALL_PHASES,
    BUNDLE,
    MOVIES,
    USAGE,
    Captured,
    Cut,
    Derivation,
    Player,
    RefusedError,
    Seed,
    TableSeed,
    Verdict,
    Window,
    bundle_name,
    check_cut,
    check_derivation,
    check_summary,
    cuts,
    derivations,
    derive_command,
    derive_done,
    derived,
    expected_counter,
    lock_start,
    movie_command,
    not_decrypted,
    phase_steps,
    pinned,
    plan,
    realign_command,
    realign_counter,
    realign_done,
    record_refusal,
    recut_count,
    recut_lines,
    recut_plan,
    recut_summary,
    recut_targets,
    same_state,
    seed_name,
    seed_toml,
    state_verdict,
    table_seeds,
    window_done,
    window_frames,
    window_listing,
    window_log,
    window_order,
    window_tables,
)

ROOT = Path(__file__).resolve().parents[5]
LOCK = ROOT / "scenarios/locks/lsnes.toml"
MOVIE_TABLES = ROOT / "movies"
TABLE_PREFIX = "room-seeds-"

type Tables = Mapping[str, Mapping[int, TableSeed]]


def seed_path(directory: Path, capture: str, index: int) -> Path:
    return directory / capture / seed_name(index)


def held_seeds(directory: Path) -> FrozenOrderedSet[Seed]:
    return FrozenOrderedSet(
        (state.parent.name, int(state.stem))
        for state in sorted(directory.glob("*/*.lss"))
        if state.stem.isdecimal()
        and state.stem == seed_name(int(state.stem))
        and state.is_file()
    )


def room_tables() -> FrozenOrderedDict[str, FrozenOrderedDict[int, TableSeed]]:
    return FrozenOrderedDict(
        (
            path.stem.removeprefix(TABLE_PREFIX),
            table_seeds(path.read_text(encoding="utf-8")),
        )
        for path in sorted(MOVIE_TABLES.glob(f"{TABLE_PREFIX}*.toml"))
    )


def ladder_texts(ladders: Path) -> FrozenOrderedDict[str, str]:
    return FrozenOrderedDict(
        (movie, (ladders / movie / "ladder.txt").read_text())
        for movie in MOVIES.values()
        if (ladders / movie / "ladder.txt").is_file()
    )


def plan_seeds(native: Path, ladders: Path) -> FrozenList[str]:
    return plan(
        LOCK.read_text(encoding="utf-8"),
        held_seeds(native),
        room_tables(),
        ladder_texts(ladders),
    )


def run_staggered(
    jobs: int, stagger: float, tasks: Sequence[Callable[[], None]]
) -> None:
    started: list[Future[None]] = []
    last: float | None = None
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        for task in tasks:
            running = [job for job in started if not job.done()]
            if len(running) >= jobs:
                _ = wait(running, return_when=FIRST_COMPLETED)
            if last is not None:
                time.sleep(max(0.0, last + stagger - time.monotonic()))
            last = time.monotonic()
            started.append(pool.submit(task))
    for job in started:
        job.result()


def logged(command: Sequence[str], log: Path, *, check: bool = False) -> int:
    with log.open("w", encoding="utf-8") as output:
        return subprocess.run(
            command,
            stdin=subprocess.DEVNULL,
            stdout=output,
            stderr=subprocess.STDOUT,
            check=check,
        ).returncode


@dataclass(frozen=True, slots=True)
class Relock:
    plan: Path
    out: Path
    logs: Path
    windows: OrderedSet[str]
    environment: Mapping[str, str]
    tables: Tables

    def setting(self, name: str) -> str:
        value = self.environment.get(name)
        if not value:
            message = f"set {name}"
            raise RefusedError(message)
        return value

    def staggered(self, tasks: Sequence[Callable[[], None]]) -> None:
        run_staggered(
            int(self.environment.get("JOBS") or 8),
            float(self.environment.get("STAGGER") or 4),
            tasks,
        )

    def cuts(self) -> FrozenList[Cut]:
        return cuts(self.plan.read_text(encoding="utf-8"), self.windows)

    def derivations(self) -> FrozenList[Derivation]:
        return derivations(
            self.plan.read_text(encoding="utf-8"),
            self.windows,
            held_seeds(self.out),
        )

    def player(self, movie: str) -> Player:
        oracle, rom = self.setting("ORACLE"), self.setting("ROM")
        inputs = Path(self.setting("INPUTS")) / f"{movie}.inputs"
        return Player(oracle, rom, str(inputs))

    def window(self, window: Window) -> None:
        arguments: list[str] = []
        for capture, seeds in window_tables(window).items():
            table = (
                self.logs
                / "tables"
                / f"{window.movie}-{window.rung}-{capture}.toml"
            )
            _ = table.write_text(seeds)
            (self.out / capture).mkdir(parents=True, exist_ok=True)
            arguments += ["--native-seeds", str(table), str(self.out / capture)]
        if window.rung > 0:
            ladders = Path(self.setting("LADDERS"))
            state = ladders / window.movie / f"{window.rung:06d}.lss"
            arguments += ["--from", str(state), str(window.rung)]
        command = movie_command(
            self.player(window.movie), window_frames(window), arguments
        )
        log = self.logs / f"{window.movie}-{window.rung}.log"
        begun = time.monotonic()
        status = logged(command, log)
        seconds = round(time.monotonic() - begun)
        with log.open("a", encoding="utf-8") as output:
            _ = output.write(window_log(window, status, seconds))
        print(window_done(window, status, seconds), flush=True)

    def cut_phase(self) -> None:
        order = window_order(self.cuts())
        _ = (self.logs / "windows.txt").write_text(window_listing(order))
        self.staggered(
            FrozenList(
                functools.partial(self.window, window) for window in order
            )
        )

    def derive(self, derivation: Derivation) -> None:
        log = self.logs / f"derive-{derivation.capture}.log"
        base = seed_path(self.out, derivation.base, derivation.base_index)
        seed = seed_path(self.out, derivation.capture, derivation.index)
        command = derive_command(
            self.setting("ORACLE"),
            self.setting("ROM"),
            f"{base}.lss",
            f"{seed}.lss",
            derivation.words,
        )
        status = logged(command, log)
        with log.open("a", encoding="utf-8") as output:
            _ = output.write(derived(derivation, status) + "\n")
        print(derive_done(derivation, status), flush=True)

    def derive_phase(self) -> None:
        self.staggered(
            FrozenList(
                functools.partial(self.derive, derivation)
                for derivation in self.derivations()
            )
        )

    def expected(self, cut: Cut, captured: Captured | None) -> int | None:
        seeds = self.tables.get(
            cut.capture, FrozenOrderedDict[int, TableSeed]()
        )
        wram = captured.wram if captured else None
        return expected_counter(seeds.get(cut.index), wram)

    def realign_phase(self) -> None:
        for cut in self.cuts():
            stem = seed_path(self.out, cut.capture, cut.index)
            record = stem.with_suffix(".record")
            if not record.is_file():
                continue
            data = record.read_bytes()
            captured = captured_wram(self.environment, cut.capture, cut.index)
            expected = self.expected(cut, captured)
            counter = realign_counter(data, expected)
            if counter is None or expected is None:
                continue
            table = (
                self.logs / "tables" / f"realign-{cut.capture}-{cut.index}.toml"
            )
            _ = table.write_text(seed_toml(cut.index, cut.room, cut.frame + 1))
            realigned = self.logs / "realign"
            realigned.mkdir(exist_ok=True)
            command = realign_command(
                self.player(cut.movie),
                f"{stem}.lss",
                cut,
                str(table),
                str(realigned),
            )
            log = self.logs / f"realign-{cut.capture}-{cut.index}.log"
            _ = logged(command, log, check=True)
            for suffix in (".lss", ".record"):
                _ = (realigned / f"{seed_name(cut.index)}{suffix}").replace(
                    stem.with_suffix(suffix)
                )
            print(realign_done(cut, counter, expected), flush=True)

    def check_cut(self, cut: Cut) -> Verdict:
        stem = seed_path(self.out, cut.capture, cut.index)
        state, record = stem.with_suffix(".lss"), stem.with_suffix(".record")
        if not state.is_file() or not record.is_file():
            return check_cut(cut, None, None, None)
        captured = captured_wram(self.environment, cut.capture, cut.index)
        expected = self.expected(cut, captured)
        return check_cut(cut, record.read_bytes(), expected, captured)

    def check_derivation(self, derivation: Derivation) -> Verdict:
        stem = seed_path(self.out, derivation.capture, derivation.index)
        log = self.logs / f"derive-{derivation.capture}.log"
        seeded = (
            stem.with_suffix(".lss").is_file()
            and stem.with_suffix(".words").is_file()
        )
        return check_derivation(
            derivation,
            seeded=seeded,
            log=log.read_text(encoding="utf-8") if log.is_file() else None,
        )

    def check_phase(self) -> bool:
        found, made = self.cuts(), self.derivations()
        verdicts = FrozenList((
            *(self.check_cut(cut) for cut in found),
            *(self.check_derivation(derivation) for derivation in made),
        ))
        failures = sum(not verdict.passed for verdict in verdicts)
        compared = sum(verdict.compared for verdict in verdicts)
        summary = check_summary(len(found) + len(made), compared, failures)
        lines = (*(verdict.line for verdict in verdicts), summary)
        _ = (self.logs / "check.txt").write_text(
            "".join(f"{line}\n" for line in lines)
        )
        print(summary)
        return failures == 0

    def run(self, phase: str) -> bool:
        (self.logs / "tables").mkdir(parents=True, exist_ok=True)
        self.out.mkdir(parents=True, exist_ok=True)
        steps = phase_steps(phase)
        if "cut" in steps:
            self.cut_phase()
        if "realign" in steps:
            self.realign_phase()
        if "derive" in steps:
            self.derive_phase()
        return "check" not in steps or self.check_phase()


def captured_wram(
    environment: Mapping[str, str], capture: str, index: int
) -> Captured | None:
    found = next(
        (
            file
            for directory in environment.get("CAPTURES", "").split()
            for file in sorted(
                (Path(directory) / "target/seeds" / capture).glob(
                    f"{seed_name(index)}-*-wram.bin"
                )
            )
        ),
        None,
    )
    return Captured(str(found), found.read_bytes()) if found else None


def members(path: Path) -> FrozenList[tuple[str, bytes]]:
    with zipfile.ZipFile(path) as archive:
        return FrozenList(
            (info.filename, archive.read(info)) for info in archive.infolist()
        )


def same_native_state(old: Path, new: Path) -> bool:
    return same_state(members(old), members(new))


def pack(tree: Path, rom: Path) -> None:
    env = cipher_env(rom)
    plain = tree.with_name(f"{BUNDLE}.tar.gz")
    with (
        plain.open("wb") as raw,
        gzip.GzipFile("", "wb", 9, raw, mtime=0) as compressed,
        tarfile.open(
            fileobj=compressed, mode="w", format=tarfile.GNU_FORMAT
        ) as archive,
    ):
        archive.add(tree, BUNDLE, filter=normalized)
    name = bundle_name(sha256(plain))
    encrypted = tree.with_name(name)
    _ = subprocess.run(
        [*CIPHER, "-salt", "-in", str(plain), "-out", str(encrypted)],
        env=env,
        check=True,
    )
    decrypted = subprocess.run(
        [*CIPHER, "-d", "-in", str(encrypted)],
        env=env,
        capture_output=True,
        check=True,
    )
    if decrypted.stdout != plain.read_bytes():
        raise RefusedError(not_decrypted(name))
    print(pinned(name, sha256(encrypted), str(encrypted)))


def compared(old: Path, work: Path, capture: str, index: str) -> str:
    was = old / "seeds" / capture / index
    new = work / "seeds" / capture / index
    name = f"{capture}:{index}"
    if (
        not new.with_suffix(".record").is_file()
        or not new.with_suffix(".lss").is_file()
    ):
        return f"FAIL {name} not cut"
    before = was.with_suffix(".record").read_bytes()
    after = new.with_suffix(".record").read_bytes()
    if refusal := record_refusal(name, before, after):
        return refusal
    same = same_native_state(was.with_suffix(".lss"), new.with_suffix(".lss"))
    return state_verdict(name, same=same)


def ppu_recut(
    old: Path,
    plan_path: Path,
    work: Path,
    rows: Sequence[str],
    environment: Mapping[str, str],
) -> bool:
    logs = work / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    records = FrozenList(
        (path.parent.name, path.stem)
        for path in (old / "seeds").rglob("*.record")
    )
    lock = LOCK.read_text(encoding="utf-8")
    wanted = FrozenOrderedSet(lock_start(lock, row, str(LOCK)) for row in rows)
    targets = recut_targets(records, wanted)
    print(recut_count(targets))
    lines = recut_lines(plan_path.read_text(encoding="utf-8"))
    recut = logs / "plan.txt"
    _ = recut.write_text(recut_plan(targets, lines, str(plan_path)))
    begun = time.monotonic()
    relock = Relock(
        recut,
        work / "seeds",
        logs,
        FrozenOrderedSet[str](),
        environment,
        room_tables(),
    )
    _ = relock.run("cut")
    _ = relock.run("realign")
    print(f"cut in {round(time.monotonic() - begun)}s")
    report = FrozenList(
        compared(old, work, capture, index) for capture, index in targets
    )
    failures = FrozenList(line for line in report if line.startswith("FAIL"))
    summary = recut_summary(len(targets), len(failures))
    _ = (logs / "compare.txt").write_text(
        "".join(f"{line}\n" for line in (*report, summary))
    )
    print(summary)
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return False
    tree = work / BUNDLE
    shutil.rmtree(tree, ignore_errors=True)
    _ = shutil.copytree(old, tree)
    for capture, index in targets:
        _ = shutil.copyfile(
            work / "seeds" / capture / f"{index}.record",
            tree / "seeds" / capture / f"{index}.record",
        )
    pack(tree, Path(relock.setting("ROM")))
    return True


def dispatch(arguments: Sequence[str], environment: Mapping[str, str]) -> int:
    match arguments:
        case ["plan", native, ladders]:
            print("\n".join(plan_seeds(Path(native), Path(ladders))))
            return 0
        case ["cut", plan_path, out, logs, *rest]:
            phase = rest[0] if rest else ALL_PHASES
            relock = Relock(
                Path(plan_path),
                Path(out),
                Path(logs),
                FrozenOrderedSet(rest[1:]),
                environment,
                room_tables(),
            )
            return 0 if relock.run(phase) else 1
        case ["ppu-recut", old, plan_path, work, *rows]:
            recut = ppu_recut(
                Path(old), Path(plan_path), Path(work), rows, environment
            )
            return 0 if recut else 1
        case _:
            print(USAGE, file=sys.stderr)
            return 2


def main() -> int:
    try:
        return dispatch(FrozenList(sys.argv[1:]), os.environ)
    except RefusedError as error:
        print(f"relock: {error}", file=sys.stderr)
        return 2
