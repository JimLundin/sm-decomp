import functools
import gzip
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import threading
import time
import tomllib
from collections.abc import Callable, Iterable
from concurrent.futures import Future, ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / "tools/ci"))
sys.path.insert(0, str(ROOT / "tools/oracle"))
from relock_seed_state import same_state
from replay_data import CIPHER, cipher_env, normalized, sha256
from scenario_seeds import SCENARIOS, planned_words

USAGE = """usage: relock.py plan NATIVE LADDERS
       relock.py cut PLAN OUT LOGS [cut|realign|derive|check|all] [MOVIE-RUNG...]
       relock.py ppu-recut OLD PLAN WORK [ROW...]

The lsnes re-lock's seeds (docs/ORACLE.md section 5).

plan: every seed the lock's rows start from that NATIVE/<capture>/NNNN.lss doesn't hold yet, one
line a seed, from the room-seed tables, the scenario edit recipes and the ladders
LADDERS/<movie>/ladder.txt, each window's captures adjacent and in frame order:
  cut MOVIE RUNG CAPTURE INDEX ROOM FRAME     a movie capture, cut from ladder rung RUNG
  derive CAPTURE INDEX BASE BASE_INDEX WORD...  a scenario seed derived from its base capture
  skip CAPTURE INDEX REASON                    a seed no lsnes movie gives

cut: runs a plan, one `oracle movie` run a window (MOVIE-RUNG), longest first, JOBS (8) at a time
and STAGGER (4) seconds apart, writing OUT/<capture>/NNNN.lss with its .record or .words and one
log a job under LOGS. realign re-cuts one step later each capture that followed step FRAME rather
than FRAME-1, which the game's frame counter shows. check holds every planned seed: a cut seed's
room and frame counter equal its table seed's (the counter read from the lsnes room capture where
the table has none), and its work RAM equals the capture's outside the stack page; a derived seed has its .words and a derivation that kept every word.
Environment: ORACLE, ROM, INPUTS (MOVIE.inputs per movie), LADDERS, CAPTURES (space-separated
folders of <dir>/target/seeds/<capture>/NNNN-ROOM-wram.bin), JOBS, STAGGER.

ppu-recut: re-cuts with the PPU tier the seed records of OLD, a decrypted relock-v2-lsnes/, that
PLAN cuts (only those ROWs start from, given ROWs), checks each is the old record and then the PPU
part alone and that its state equals the old outside rerecord history, then packs and encrypts
WORK/relock-v2-lsnes with the old states and prints the lock's new `bundles` line."""
MOVIES = {"100-": "100", "any-": "any", "low-": "low", "spore-": "spore"}
HEADER = 43
WRAM = 0x20000
STACK = range(0x1F00, 0x2000)
ROOM_WORD = 0x079B
COUNTER_WORD = 0x05B6
PPU_PART = 4 + 1 + 58 + 88
LOCK = ROOT / "scenarios/locks/lsnes.toml"
BUNDLE = "relock-v2-lsnes"


class Refused(Exception):
    pass


@dataclass(frozen=True)
class Cut:
    movie: str
    rung: int
    capture: str
    index: int
    room: str
    frame: int

    @property
    def window(self) -> str:
        return f"{self.movie}-{self.rung}"


@dataclass(frozen=True)
class Derivation:
    capture: str
    index: int
    base: str
    base_index: int
    words: tuple[str, ...]


def seed_path(directory: Path, capture: str, index: int) -> Path:
    return directory / capture / f"{index:04d}"


def table_seeds(capture: str) -> dict[int, dict[str, object]]:
    path = ROOT / "movies" / f"room-seeds-{capture}.toml"
    if not path.is_file():
        return {}
    with path.open("rb") as table:
        return {int(str(seed["room_index"])): seed for seed in tomllib.load(table)["seed"]}


def seed_toml(index: int, room: str, frame: int) -> str:
    return f'[[seed]]\nroom_index = {index}\nroom = "{room}"\nmovie_frame = {frame}\n\n'


def rung_for(ladders: Path, movie: str, step: int) -> int:
    ladder = ladders / movie / "ladder.txt"
    if not ladder.is_file():
        return 0
    rungs = [int(line.split()[0]) for line in ladder.read_text().splitlines()
             if line.strip() and not line.startswith("#")]
    return max((rung for rung in rungs if rung <= step), default=0)


def scenario_derivation(capture: str) -> Derivation | None:
    name = capture.removeprefix("scenario-")
    if not capture.startswith("scenario-") or name not in SCENARIOS:
        return None
    base, index = SCENARIOS[name].base.split(":")
    words = tuple(word for offset, value in sorted(planned_words(SCENARIOS[name]).items())
                  for word in ("--word", f"{offset:04X}={value:04X}"))
    return Derivation(capture, int(index), base, int(index), words)


def numeric(field: str) -> int:
    match = re.match(r"\s*(-?\d+)", field)
    return int(match.group(1)) if match else 0


def plan_order(line: str) -> tuple[str, str, int, int, str, int, str]:
    fields = line.split() + [""] * 7
    return fields[0], fields[1], numeric(fields[2]), numeric(fields[6]), fields[3], numeric(fields[4]), line


def plan(native: Path, ladders: Path) -> list[str]:
    with LOCK.open("rb") as lock:
        document = tomllib.load(lock)
    if document["format"] != 2:
        raise Refused("expected format-2 lock")
    starts = {str(row["start"]) for row in document["row"]} - {"power-on"}

    def held(capture: str, index: int) -> bool:
        return seed_path(native, capture, index).with_suffix(".lss").is_file()

    for start in list(starts):
        capture, number = start.rsplit(":", 1)
        derivation = scenario_derivation(capture)
        if derivation and not held(capture, int(number)):
            starts.add(f"{derivation.base}:{derivation.base_index}")
    lines = []
    for start in starts:
        capture, raw_index = start.rsplit(":", 1)
        index = int(raw_index)
        if held(capture, index):
            continue
        if capture.startswith("scenario-"):
            derivation = scenario_derivation(capture)
            lines.append(f"skip {capture} {raw_index} no-scenario-definition" if derivation is None else
                         " ".join(["derive", capture, raw_index, derivation.base, str(derivation.base_index),
                                   *derivation.words]))
            continue
        movie = next((movie for prefix, movie in MOVIES.items() if capture.startswith(prefix)), None)
        if movie is None:
            lines.append(f"skip {capture} {raw_index} no-lsnes-movie")
            continue
        seed = table_seeds(capture).get(index)
        if seed is None:
            lines.append(f"skip {capture} {raw_index} no-table-seed")
            continue
        frame = int(str(seed["movie_frame"]))
        lines.append(f"cut {movie} {rung_for(ladders, movie, frame - 1)} {capture} {raw_index} {seed['room']} {frame}")
    return sorted(lines, key=plan_order)


class Jobs:
    def __init__(self, jobs: int, stagger: float) -> None:
        self.pool = ThreadPoolExecutor(max_workers=jobs)
        self.stagger = stagger
        self.lock = threading.Lock()
        self.last: float | None = None
        self.started: list[Future[None]] = []

    def start(self, job: Callable[[], None]) -> None:
        def staggered() -> None:
            with self.lock:
                if self.last is not None:
                    time.sleep(max(0.0, self.last + self.stagger - time.monotonic()))
                self.last = time.monotonic()
            job()
        self.started.append(self.pool.submit(staggered))

    def wait(self) -> None:
        self.pool.shutdown(wait=True)
        for job in self.started:
            job.result()


@dataclass(frozen=True)
class Relock:
    plan: Path
    out: Path
    logs: Path
    windows: frozenset[str]

    def environment(self, name: str) -> str:
        value = os.environ.get(name)
        if not value:
            raise Refused(f"set {name}")
        return value

    def jobs(self) -> Jobs:
        return Jobs(int(os.environ.get("JOBS") or 8), float(os.environ.get("STAGGER") or 4))

    def lines(self, kind: str) -> list[list[str]]:
        return [fields for fields in (line.split() for line in self.plan.read_text().splitlines())
                if fields and fields[0] == kind]

    def cuts(self) -> list[Cut]:
        cuts = [Cut(f[1], int(f[2]), f[3], int(f[4]), f[5], int(f[6])) for f in self.lines("cut")]
        return [cut for cut in cuts if not self.windows or cut.window in self.windows]

    def derivations(self) -> list[Derivation]:
        cut = {(c.capture, c.index) for c in self.cuts()}
        found = [Derivation(f[1], int(f[2]), f[3], int(f[4]), tuple(f[5:])) for f in self.lines("derive")]
        return [d for d in found if not self.windows or (d.base, d.base_index) in cut
                or seed_path(self.out, d.base, d.base_index).with_suffix(".lss").is_file()]

    def window(self, movie: str, rung: int, cuts: list[Cut]) -> None:
        log = self.logs / f"{movie}-{rung}.log"
        arguments = []
        for capture in sorted({cut.capture for cut in cuts}):
            table = self.logs / "tables" / f"{movie}-{rung}-{capture}.toml"
            table.write_text("".join(seed_toml(c.index, c.room, c.frame) for c in cuts if c.capture == capture))
            (self.out / capture).mkdir(parents=True, exist_ok=True)
            arguments += ["--native-seeds", str(table), str(self.out / capture)]
        if rung > 0:
            arguments += ["--from", str(Path(self.environment("LADDERS")) / movie / f"{rung:06d}.lss"), str(rung)]
        begun = time.monotonic()
        with log.open("w") as output:
            status = subprocess.run([self.environment("ORACLE"), "movie", self.environment("ROM"),
                                     str(Path(self.environment("INPUTS")) / f"{movie}.inputs"), "--no-memories",
                                     "--ppu", "--frames", str(max(cut.frame for cut in cuts) + 1), *arguments],
                                    stdin=subprocess.DEVNULL, stdout=output, stderr=subprocess.STDOUT,
                                    check=False).returncode
            seconds = round(time.monotonic() - begun)
            output.write(f"relock {movie}-{rung} exit {status} in {seconds}s\n")
        print(f"window {movie}-{rung} exit {status} in {seconds}s", flush=True)

    def cut_phase(self) -> None:
        windows: dict[tuple[str, int], list[Cut]] = {}
        for cut in self.cuts():
            windows.setdefault((cut.movie, cut.rung), []).append(cut)
        order = sorted(windows, key=lambda w: (max(c.frame - c.rung for c in windows[w]), w), reverse=True)
        (self.logs / "windows.txt").write_text("".join(f"{movie} {rung}\n" for movie, rung in order))
        jobs = self.jobs()
        for movie, rung in order:
            jobs.start(functools.partial(self.window, movie, rung, windows[(movie, rung)]))
        jobs.wait()

    def derive(self, derivation: Derivation) -> None:
        log = self.logs / f"derive-{derivation.capture}.log"
        with log.open("w") as output:
            status = subprocess.run([self.environment("ORACLE"), "derive-seed", self.environment("ROM"),
                                     str(seed_path(self.out, derivation.base, derivation.base_index)) + ".lss",
                                     str(seed_path(self.out, derivation.capture, derivation.index)) + ".lss",
                                     *derivation.words], stdin=subprocess.DEVNULL, stdout=output,
                                    stderr=subprocess.STDOUT, check=False).returncode
            output.write(f"relock derive {derivation.capture} exit {status}\n")
        print(f"derive {derivation.capture} exit {status}", flush=True)

    def derive_phase(self) -> None:
        jobs = self.jobs()
        for derivation in self.derivations():
            jobs.start(functools.partial(self.derive, derivation))
        jobs.wait()

    def realign_phase(self) -> None:
        for cut in self.cuts():
            stem = seed_path(self.out, cut.capture, cut.index)
            record = stem.with_suffix(".record")
            if not record.is_file() or record.stat().st_size < HEADER + WRAM:
                continue
            counter = word(record.read_bytes(), COUNTER_WORD)
            expected = expected_counter(cut.capture, cut.index)
            if expected is None or (counter + 1) % 0x10000 != expected:
                continue
            table = self.logs / "tables" / f"realign-{cut.capture}-{cut.index}.toml"
            table.write_text(seed_toml(cut.index, cut.room, cut.frame + 1))
            realigned = self.logs / "realign"
            realigned.mkdir(exist_ok=True)
            with (self.logs / f"realign-{cut.capture}-{cut.index}.log").open("w") as output:
                subprocess.run([self.environment("ORACLE"), "movie", self.environment("ROM"),
                                str(Path(self.environment("INPUTS")) / f"{cut.movie}.inputs"), "--no-memories",
                                "--ppu", "--from", str(stem) + ".lss", str(cut.frame), "--frames", str(cut.frame + 2),
                                "--native-seeds", str(table), str(realigned)], stdin=subprocess.DEVNULL,
                               stdout=output, stderr=subprocess.STDOUT, check=True)
            for suffix in (".lss", ".record"):
                (realigned / f"{cut.index:04d}{suffix}").replace(stem.with_suffix(suffix))
            print(f"realign {cut.capture}:{cut.index} one step later (frame counter {counter}, capture {expected})",
                  flush=True)

    def check_cut(self, cut: Cut) -> tuple[bool, bool, str]:
        stem = seed_path(self.out, cut.capture, cut.index)
        name = f"{cut.capture}:{cut.index}"
        if not stem.with_suffix(".lss").is_file() or not stem.with_suffix(".record").is_file():
            return False, False, f"FAIL {name} missing ({cut.window})"
        record = stem.with_suffix(".record").read_bytes()
        if len(record) < HEADER + WRAM or record[HEADER - 1] == 0:
            return False, False, f"FAIL {name} record without memories ({len(record)} bytes)"
        room = f"{word(record, ROOM_WORD):04X}"
        counter = word(record, COUNTER_WORD)
        expected = expected_counter(cut.capture, cut.index)
        if room != cut.room:
            return False, False, f"FAIL {name} room {room}, table {cut.room}"
        if expected is None:
            return False, False, f"FAIL {name} frame counter {counter}, and neither the table nor a capture has one"
        if counter != expected:
            return False, False, f"FAIL {name} frame counter {counter}, table {expected}"
        captured = captured_wram(cut.capture, cut.index)
        if captured is None:
            return True, False, f"ok   {name} room {cut.room} frame {cut.frame}; no lsnes capture to compare"
        wram = record[HEADER:HEADER + WRAM]
        differing = sum(1 for offset, (a, b) in enumerate(zip(wram, captured.read_bytes()))
                        if a != b and offset not in STACK)
        if differing:
            return False, True, (f"FAIL {name} room {cut.room} frame {cut.frame}: {differing} WRAM bytes differ "
                                 f"from {captured}")
        return True, True, f"ok   {name} room {cut.room} frame {cut.frame}: 0 WRAM bytes differ"

    def check_derivation(self, derivation: Derivation) -> tuple[bool, str]:
        stem = seed_path(self.out, derivation.capture, derivation.index)
        log = self.logs / f"derive-{derivation.capture}.log"
        name = f"{derivation.capture}:{derivation.index}"
        derived = log.is_file() and f"relock derive {derivation.capture} exit 0" in log.read_text().splitlines()
        if stem.with_suffix(".lss").is_file() and stem.with_suffix(".words").is_file() and derived:
            return True, f"ok   {name} derived, every word kept at load"
        return False, f"FAIL {name} not derived"

    def check_phase(self) -> bool:
        lines, failures, compared = [], 0, 0
        cuts, derivations = self.cuts(), self.derivations()
        for cut in cuts:
            passed, was_compared, line = self.check_cut(cut)
            failures += not passed
            compared += was_compared
            lines.append(line)
        for derivation in derivations:
            passed, line = self.check_derivation(derivation)
            failures += not passed
            lines.append(line)
        summary = (f"relock check: {len(cuts) + len(derivations)} seeds, {compared} compared with an lsnes capture, "
                   f"{failures} failing")
        (self.logs / "check.txt").write_text("".join(f"{line}\n" for line in [*lines, summary]))
        print(summary)
        return failures == 0

    def run(self, phase: str) -> bool:
        (self.logs / "tables").mkdir(parents=True, exist_ok=True)
        self.out.mkdir(parents=True, exist_ok=True)
        phases: dict[str, Callable[[], object]] = {"cut": self.cut_phase, "realign": self.realign_phase,
                                                     "derive": self.derive_phase, "check": self.check_phase}
        if phase == "all":
            for name in ("cut", "realign", "derive"):
                phases[name]()
            return self.check_phase()
        if phase not in phases:
            raise Refused(f"unknown phase {phase}")
        return phases[phase]() is not False


def captured_wram(capture: str, index: int) -> Path | None:
    return next((file for directory in os.environ.get("CAPTURES", "").split()
                 for file in sorted((Path(directory) / "target/seeds" / capture).glob(f"{index:04d}-*-wram.bin"))),
                None)


def expected_counter(capture: str, index: int) -> int | None:
    nmi = table_seeds(capture).get(index, {}).get("nmi")
    if nmi is not None:
        return int(str(nmi))
    captured = captured_wram(capture, index)
    return int.from_bytes(captured.read_bytes()[COUNTER_WORD:COUNTER_WORD + 2], "little") if captured else None


def word(record: bytes, offset: int) -> int:
    return int.from_bytes(record[HEADER + offset:HEADER + offset + 2], "little")


def lock_start(row: str) -> tuple[str, int]:
    with LOCK.open("rb") as lock:
        rows = [found for found in tomllib.load(lock)["row"] if found["id"] == row]
    start = str(rows[0].get("start", "")) if rows else ""
    if not re.fullmatch(r"[^:]+:\d+", start):
        raise Refused(f"{row}: no seed start in {LOCK}")
    capture, index = start.rsplit(":", 1)
    return capture, int(index)


def pack(tree: Path, rom: Path) -> None:
    env = cipher_env(rom)
    plain = tree.with_name(f"{BUNDLE}.tar.gz")
    with (plain.open("wb") as raw, gzip.GzipFile("", "wb", 9, raw, mtime=0) as compressed,
          tarfile.open(fileobj=compressed, mode="w", format=tarfile.GNU_FORMAT) as archive):
        archive.add(tree, BUNDLE, filter=normalized)
    name = f"{BUNDLE}-{sha256(plain)[:16]}.tar.gz.enc"
    encrypted = tree.with_name(name)
    subprocess.run([*CIPHER, "-salt", "-in", str(plain), "-out", str(encrypted)], env=env, check=True)
    decrypted = subprocess.run([*CIPHER, "-d", "-in", str(encrypted)], env=env, capture_output=True, check=True)
    if decrypted.stdout != plain.read_bytes():
        raise Refused(f"{name}: does not decrypt to the tar")
    digest = sha256(encrypted)
    print(f"name   {name}\nsha256 {digest}\npath   {encrypted}")
    print(f'pin    bundles = [{{ name = "{name}", sha256 = "{digest}" }}]')


def ppu_recut(old: Path, plan_path: Path, work: Path, rows: Iterable[str]) -> bool:
    logs = work / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    records = sorted((path.parent.name, path.stem) for path in (old / "seeds").rglob("*.record"))
    wanted = {(capture, f"{index:04d}") for capture, index in map(lock_start, rows)}
    targets = [record for record in records if not wanted or record in wanted]
    print(f"{len(targets)} seed records to re-cut")
    lines: dict[tuple[str, str], str] = {}
    for line in plan_path.read_text().splitlines():
        fields = line.split()
        if len(fields) >= 5 and fields[0] == "cut":
            lines.setdefault((fields[3], f"{int(fields[4]):04d}"), line)
    missing = [f"{capture} {index}" for capture, index in targets if (capture, index) not in lines]
    if missing:
        raise Refused("no cut line in " + str(plan_path) + " for:\n" + "\n".join(missing))
    recut_plan = logs / "plan.txt"
    recut_plan.write_text("".join(lines[target] + "\n" for target in targets))
    begun = time.monotonic()
    relock = Relock(recut_plan, work / "seeds", logs, frozenset())
    relock.run("cut")
    relock.run("realign")
    print(f"cut in {round(time.monotonic() - begun)}s")
    report = []
    for capture, index in targets:
        was, new = old / "seeds" / capture / index, work / "seeds" / capture / index
        name = f"{capture}:{index}"
        if not new.with_suffix(".record").is_file() or not new.with_suffix(".lss").is_file():
            report.append(f"FAIL {name} not cut")
            continue
        before, after = was.with_suffix(".record").read_bytes(), new.with_suffix(".record").read_bytes()
        if len(after) != len(before) + PPU_PART:
            report.append(f"FAIL {name} record of {len(after)} bytes, old {len(before)} + {PPU_PART}")
        elif after[:len(before)] != before:
            report.append(f"FAIL {name} record differs from the old within its {len(before)} bytes")
        elif after[len(before):len(before) + 5] != b"OHS1\x04":
            report.append(f"FAIL {name} no PPU part after the memories")
        elif not same_state(was.with_suffix(".lss"), new.with_suffix(".lss")):
            report.append(f"FAIL {name} state differs from the old outside rerecord history")
        else:
            report.append(f"ok   {name} old record + PPU part; state equal outside rerecord history")
    failures = [line for line in report if line.startswith("FAIL")]
    summary = f"ppu re-cut check: {len(targets)} records, {len(failures)} failing"
    (logs / "compare.txt").write_text("".join(f"{line}\n" for line in [*report, summary]))
    print(summary)
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return False
    tree = work / BUNDLE
    shutil.rmtree(tree, ignore_errors=True)
    shutil.copytree(old, tree)
    for capture, index in targets:
        shutil.copyfile(work / "seeds" / capture / f"{index}.record", tree / "seeds" / capture / f"{index}.record")
    pack(tree, Path(relock.environment("ROM")))
    return True


def main(arguments: list[str]) -> int:
    try:
        match arguments:
            case ["plan", native, ladders]:
                print("\n".join(plan(Path(native), Path(ladders))))
                return 0
            case ["cut", plan_path, out, logs, *rest]:
                phase = rest[0] if rest else "all"
                return 0 if Relock(Path(plan_path), Path(out), Path(logs), frozenset(rest[1:])).run(phase) else 1
            case ["ppu-recut", old, plan_path, work, *rows]:
                return 0 if ppu_recut(Path(old), Path(plan_path), Path(work), rows) else 1
            case _:
                print(USAGE, file=sys.stderr)
                return 2
    except Refused as error:
        print(f"relock: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
