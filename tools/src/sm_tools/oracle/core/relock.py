import re
from dataclasses import dataclass
lazy from collections.abc import Mapping, Sequence

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet
lazy from ordered import OrderedSet

from sm_tools.ci.core.decoded import field, tables, text, toml
from sm_tools.oracle.core.scenario_seeds import SCENARIOS, planned_words

USAGE = """usage: relock plan NATIVE LADDERS
       relock cut PLAN OUT LOGS [cut|realign|derive|check|all] [MOVIE-RUNG...]
       relock ppu-recut OLD PLAN WORK [ROW...]

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
MOVIES = FrozenOrderedDict({
    "100-": "100",
    "any-": "any",
    "low-": "low",
    "spore-": "spore",
})
HEADER = 43
WRAM = 0x20000
STACK = range(0x1F00, 0x2000)
ROOM_WORD = 0x079B
COUNTER_WORD = 0x05B6
PPU_PART = 4 + 1 + 58 + 88
PPU_PART_START = b"OHS1\x04"
BUNDLE = "relock-v2-lsnes"
CUT_INDEX_FIELDS = 5
LOCK_FORMAT = 2
PHASES = FrozenList(("cut", "realign", "derive", "check"))
ALL_PHASES = "all"
RERECORD_HISTORY = FrozenOrderedSet(("rrdata", "rerecords"))
NO_START = FrozenOrderedSet(("power-on",))
SEED_START = re.compile(r"[^:]+:\d+")

type Seed = tuple[str, int]
type Member = tuple[str, bytes]


class RefusedError(Exception):
    pass


@dataclass(frozen=True, slots=True)
class TableSeed:
    room: str
    movie_frame: int
    nmi: int | None


@dataclass(frozen=True, slots=True)
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


@dataclass(frozen=True, slots=True)
class Derivation:
    capture: str
    index: int
    base: str
    base_index: int
    words: FrozenList[str]


@dataclass(frozen=True, slots=True)
class Window:
    movie: str
    rung: int
    cuts: FrozenList[Cut]


@dataclass(frozen=True, slots=True)
class Player:
    oracle: str
    rom: str
    inputs: str


@dataclass(frozen=True, slots=True)
class Captured:
    path: str
    wram: bytes


@dataclass(frozen=True, slots=True)
class Verdict:
    passed: bool
    compared: bool
    line: str


def seed_name(index: int) -> str:
    return f"{index:04d}"


def integer(value: object) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        message = f"expected an integer, found {type(value).__name__}"
        raise TypeError(message)
    return value


def table_seeds(source: str) -> FrozenOrderedDict[int, TableSeed]:
    return FrozenOrderedDict(
        (
            integer(seed["room_index"]),
            TableSeed(
                text(seed["room"]),
                integer(seed["movie_frame"]),
                None if seed.get("nmi") is None else integer(seed["nmi"]),
            ),
        )
        for seed in tables(toml(source), "seed")
    )


def seed_toml(index: int, room: str, frame: int) -> str:
    return (
        f'[[seed]]\nroom_index = {index}\nroom = "{room}"\n'
        f"movie_frame = {frame}\n\n"
    )


def rung_for(ladder: str | None, step: int) -> int:
    if ladder is None:
        return 0
    rungs = FrozenList(
        int(line.split()[0])
        for line in ladder.splitlines()
        if line.strip() and not line.startswith("#")
    )
    return max((rung for rung in rungs if rung <= step), default=0)


def scenario_derivation(capture: str) -> Derivation | None:
    name = capture.removeprefix("scenario-")
    if not capture.startswith("scenario-") or name not in SCENARIOS:
        return None
    base, index = SCENARIOS[name].base.split(":")
    words = FrozenList(
        word
        for offset, value in sorted(planned_words(SCENARIOS[name]).items())
        for word in ("--word", f"{offset:04X}={value:04X}")
    )
    return Derivation(capture, int(index), base, int(index), words)


def numeric(text_field: str) -> int:
    match = re.match(r"\s*(-?\d+)", text_field)
    return int(match.group(1)) if match else 0


def plan_order(line: str) -> tuple[str, str, int, int, str, int, str]:
    fields = [*line.split(), *[""] * 7]
    return (
        fields[0],
        fields[1],
        numeric(fields[2]),
        numeric(fields[6]),
        fields[3],
        numeric(fields[4]),
        line,
    )


def lock_document(lock: str) -> Mapping[str, object]:
    document = toml(lock)
    if document["format"] != LOCK_FORMAT:
        message = "expected format-2 lock"
        raise RefusedError(message)
    return document


def split_start(start: str) -> tuple[str, str]:
    capture, index = start.rsplit(":", 1)
    return capture, index


def plan_line(
    start: str,
    held: OrderedSet[Seed],
    seeds: Mapping[str, Mapping[int, TableSeed]],
    ladders: Mapping[str, str],
) -> str | None:
    capture, raw_index = split_start(start)
    index = int(raw_index)
    if (capture, index) in held:
        return None
    if capture.startswith("scenario-"):
        derivation = scenario_derivation(capture)
        if derivation is None:
            return f"skip {capture} {raw_index} no-scenario-definition"
        return " ".join((
            "derive",
            capture,
            raw_index,
            derivation.base,
            str(derivation.base_index),
            *derivation.words,
        ))
    movie = next(
        (
            movie
            for prefix, movie in MOVIES.items()
            if capture.startswith(prefix)
        ),
        None,
    )
    if movie is None:
        return f"skip {capture} {raw_index} no-lsnes-movie"
    seed = seeds.get(capture, FrozenOrderedDict[int, TableSeed]()).get(index)
    if seed is None:
        return f"skip {capture} {raw_index} no-table-seed"
    frame = seed.movie_frame
    rung = rung_for(ladders.get(movie), frame - 1)
    return f"cut {movie} {rung} {capture} {raw_index} {seed.room} {frame}"


def plan(
    lock: str,
    held: OrderedSet[Seed],
    seeds: Mapping[str, Mapping[int, TableSeed]],
    ladders: Mapping[str, str],
) -> FrozenList[str]:
    document = lock_document(lock)
    starts = (
        FrozenOrderedSet(
            text(field(row, "start")) for row in tables(document, "row")
        )
        - NO_START
    )
    bases = FrozenOrderedSet(
        f"{derivation.base}:{derivation.base_index}"
        for capture, number in map(split_start, starts)
        if (derivation := scenario_derivation(capture))
        and (capture, int(number)) not in held
    )
    lines = FrozenList(
        line
        for start in starts | bases
        if (line := plan_line(start, held, seeds, ladders)) is not None
    )
    return FrozenList(sorted(lines, key=plan_order))


def plan_fields(plan_text: str, kind: str) -> FrozenList[FrozenList[str]]:
    return FrozenList(
        fields
        for fields in (
            FrozenList(line.split()) for line in plan_text.splitlines()
        )
        if fields and fields[0] == kind
    )


def cuts(plan_text: str, windows: OrderedSet[str]) -> FrozenList[Cut]:
    found = FrozenList(
        Cut(f[1], int(f[2]), f[3], int(f[4]), f[5], int(f[6]))
        for f in plan_fields(plan_text, "cut")
    )
    return FrozenList(
        cut for cut in found if not windows or cut.window in windows
    )


def derivations(
    plan_text: str, windows: OrderedSet[str], out_held: OrderedSet[Seed]
) -> FrozenList[Derivation]:
    cut = FrozenOrderedSet(
        (c.capture, c.index) for c in cuts(plan_text, windows)
    )
    found = FrozenList(
        Derivation(f[1], int(f[2]), f[3], int(f[4]), FrozenList(f[5:]))
        for f in plan_fields(plan_text, "derive")
    )
    return FrozenList(
        d
        for d in found
        if not windows
        or (d.base, d.base_index) in cut
        or (d.base, d.base_index) in out_held
    )


def window_order(found: Sequence[Cut]) -> FrozenList[Window]:
    grouped: dict[tuple[str, int], list[Cut]] = {}
    for cut in found:
        grouped.setdefault((cut.movie, cut.rung), []).append(cut)
    order = sorted(
        grouped,
        key=lambda w: (max(c.frame - c.rung for c in grouped[w]), w),
        reverse=True,
    )
    return FrozenList(
        Window(movie, rung, FrozenList(grouped[movie, rung]))
        for movie, rung in order
    )


def window_listing(windows: Sequence[Window]) -> str:
    return "".join(f"{window.movie} {window.rung}\n" for window in windows)


def window_tables(window: Window) -> FrozenOrderedDict[str, str]:
    return FrozenOrderedDict(
        (
            capture,
            "".join(
                seed_toml(c.index, c.room, c.frame)
                for c in window.cuts
                if c.capture == capture
            ),
        )
        for capture in sorted(
            FrozenOrderedSet(cut.capture for cut in window.cuts)
        )
    )


def movie_command(
    player: Player, frames: int, arguments: Sequence[str]
) -> FrozenList[str]:
    return FrozenList((
        player.oracle,
        "movie",
        player.rom,
        player.inputs,
        "--no-memories",
        "--ppu",
        "--frames",
        str(frames),
        *arguments,
    ))


def window_frames(window: Window) -> int:
    return max(cut.frame for cut in window.cuts) + 1


def window_log(window: Window, status: int, seconds: int) -> str:
    return f"relock {window.movie}-{window.rung} exit {status} in {seconds}s\n"


def window_done(window: Window, status: int, seconds: int) -> str:
    return f"window {window.movie}-{window.rung} exit {status} in {seconds}s"


def derive_command(
    oracle: str, rom: str, base: str, seed: str, words: Sequence[str]
) -> FrozenList[str]:
    return FrozenList((oracle, "derive-seed", rom, base, seed, *words))


def derived(derivation: Derivation, status: int) -> str:
    return f"relock derive {derivation.capture} exit {status}"


def derive_done(derivation: Derivation, status: int) -> str:
    return f"derive {derivation.capture} exit {status}"


def word(record: bytes, offset: int) -> int:
    return int.from_bytes(
        record[HEADER + offset : HEADER + offset + 2], "little"
    )


def expected_counter(
    seed: TableSeed | None, captured: bytes | None
) -> int | None:
    if seed is not None and seed.nmi is not None:
        return seed.nmi
    if captured is None:
        return None
    return int.from_bytes(captured[COUNTER_WORD : COUNTER_WORD + 2], "little")


def realign_counter(record: bytes, expected: int | None) -> int | None:
    if len(record) < HEADER + WRAM:
        return None
    counter = word(record, COUNTER_WORD)
    if expected is None or (counter + 1) % 0x10000 != expected:
        return None
    return counter


def realign_command(
    player: Player, state: str, cut: Cut, table_path: str, realigned: str
) -> FrozenList[str]:
    return FrozenList((
        player.oracle,
        "movie",
        player.rom,
        player.inputs,
        "--no-memories",
        "--ppu",
        "--from",
        state,
        str(cut.frame),
        "--frames",
        str(cut.frame + 2),
        "--native-seeds",
        table_path,
        realigned,
    ))


def realign_done(cut: Cut, counter: int, expected: int) -> str:
    return (
        f"realign {cut.capture}:{cut.index} one step later"
        f" (frame counter {counter}, capture {expected})"
    )


def seed_refusal(
    cut: Cut, name: str, record: bytes, expected: int | None
) -> str | None:
    if len(record) < HEADER + WRAM or record[HEADER - 1] == 0:
        return f"FAIL {name} record without memories ({len(record)} bytes)"
    room = f"{word(record, ROOM_WORD):04X}"
    counter = word(record, COUNTER_WORD)
    if room != cut.room:
        return f"FAIL {name} room {room}, table {cut.room}"
    if expected is None:
        return (
            f"FAIL {name} frame counter {counter}, and neither the table"
            " nor a capture has one"
        )
    if counter != expected:
        return f"FAIL {name} frame counter {counter}, table {expected}"
    return None


def check_cut(
    cut: Cut,
    record: bytes | None,
    expected: int | None,
    captured: Captured | None,
) -> Verdict:
    name = f"{cut.capture}:{cut.index}"
    if record is None:
        return Verdict(
            passed=False,
            compared=False,
            line=f"FAIL {name} missing ({cut.window})",
        )
    if refusal := seed_refusal(cut, name, record, expected):
        return Verdict(passed=False, compared=False, line=refusal)
    at = f"{name} room {cut.room} frame {cut.frame}"
    if captured is None:
        return Verdict(
            passed=True,
            compared=False,
            line=f"ok   {at}; no lsnes capture to compare",
        )
    wram = record[HEADER : HEADER + WRAM]
    differing = sum(
        1
        for offset, (a, b) in enumerate(zip(wram, captured.wram, strict=False))
        if a != b and offset not in STACK
    )
    if differing:
        return Verdict(
            passed=False,
            compared=True,
            line=f"FAIL {at}: {differing} WRAM bytes differ from {captured.path}",
        )
    return Verdict(
        passed=True, compared=True, line=f"ok   {at}: 0 WRAM bytes differ"
    )


def check_derivation(
    derivation: Derivation, *, seeded: bool, log: str | None
) -> Verdict:
    name = f"{derivation.capture}:{derivation.index}"
    kept = log is not None and derived(derivation, 0) in log.splitlines()
    if seeded and kept:
        return Verdict(
            passed=True,
            compared=False,
            line=f"ok   {name} derived, every word kept at load",
        )
    return Verdict(
        passed=False, compared=False, line=f"FAIL {name} not derived"
    )


def check_summary(seeds: int, compared: int, failures: int) -> str:
    return (
        f"relock check: {seeds} seeds, {compared} compared with an lsnes"
        f" capture, {failures} failing"
    )


def lock_start(lock: str, row: str, lock_path: str) -> Seed:
    rows = FrozenList(
        found for found in tables(toml(lock), "row") if found["id"] == row
    )
    start = text(rows[0].get("start", "")) if rows else ""
    if not SEED_START.fullmatch(start):
        message = f"{row}: no seed start in {lock_path}"
        raise RefusedError(message)
    capture, index = split_start(start)
    return capture, int(index)


def recut_targets(
    records: Sequence[tuple[str, str]], wanted: OrderedSet[Seed]
) -> FrozenList[tuple[str, str]]:
    names = FrozenOrderedSet(
        (capture, seed_name(index)) for capture, index in wanted
    )
    return FrozenList(
        record for record in sorted(records) if not names or record in names
    )


def recut_lines(plan_text: str) -> FrozenOrderedDict[tuple[str, str], str]:
    lines: dict[tuple[str, str], str] = {}
    for line in plan_text.splitlines():
        fields = line.split()
        if len(fields) >= CUT_INDEX_FIELDS and fields[0] == "cut":
            _ = lines.setdefault((fields[3], seed_name(int(fields[4]))), line)
    return FrozenOrderedDict(lines)


def recut_plan(
    targets: Sequence[tuple[str, str]],
    lines: Mapping[tuple[str, str], str],
    plan_path: str,
) -> str:
    missing = FrozenList(
        f"{capture} {index}"
        for capture, index in targets
        if (capture, index) not in lines
    )
    if missing:
        message = f"no cut line in {plan_path} for:\n" + "\n".join(missing)
        raise RefusedError(message)
    return "".join(lines[target] + "\n" for target in targets)


def recut_count(targets: Sequence[tuple[str, str]]) -> str:
    return f"{len(targets)} seed records to re-cut"


def record_refusal(name: str, before: bytes, after: bytes) -> str | None:
    if len(after) != len(before) + PPU_PART:
        return (
            f"FAIL {name} record of {len(after)} bytes, old {len(before)}"
            f" + {PPU_PART}"
        )
    if after[: len(before)] != before:
        return (
            f"FAIL {name} record differs from the old within its"
            f" {len(before)} bytes"
        )
    if after[len(before) : len(before) + 5] != PPU_PART_START:
        return f"FAIL {name} no PPU part after the memories"
    return None


def state_verdict(name: str, *, same: bool) -> str:
    if same:
        return (
            f"ok   {name} old record + PPU part; state equal outside rerecord"
            " history"
        )
    return f"FAIL {name} state differs from the old outside rerecord history"


def recut_summary(targets: int, failures: int) -> str:
    return f"ppu re-cut check: {targets} records, {failures} failing"


def same_state(old: Sequence[Member], new: Sequence[Member]) -> bool:
    old_names = FrozenList(name for name, _ in old)
    new_names = FrozenList(name for name, _ in new)
    old_set = FrozenOrderedSet(old_names)
    new_set = FrozenOrderedSet(new_names)
    if len(old_names) != len(old_set) or len(new_names) != len(new_set):
        return False
    if old_set != new_set:
        return False
    before = FrozenOrderedDict(old)
    after = FrozenOrderedDict(new)
    return all(
        before[name] == after[name]
        for name in old_names
        if name not in RERECORD_HISTORY
    )


def bundle_name(digest: str) -> str:
    return f"{BUNDLE}-{digest[:16]}.tar.gz.enc"


def pinned(name: str, digest: str, path: str) -> str:
    return (
        f"name   {name}\nsha256 {digest}\npath   {path}\n"
        f'pin    bundles = [{{ name = "{name}", sha256 = "{digest}" }}]'
    )


def not_decrypted(name: str) -> str:
    return f"{name}: does not decrypt to the tar"


def phase_steps(phase: str) -> FrozenList[str]:
    if phase == ALL_PHASES:
        return PHASES
    if phase in PHASES:
        return FrozenList((phase,))
    message = f"unknown phase {phase}"
    raise RefusedError(message)
