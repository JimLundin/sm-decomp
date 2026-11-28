lazy from pathlib import Path

from ordered import FrozenOrderedDict, FrozenOrderedSet

from sm_tools.oracle.core.relock import (
    COUNTER_WORD,
    HEADER,
    ROOM_WORD,
    WRAM,
    Cut,
    TableSeed,
    Verdict,
    plan_order,
    rung_for,
)
from sm_tools.oracle.shell.relock import Relock

ROOM = 0xDF45
CUT = Cut("any", 0, "any-boundary", 1, "DF45", 8538)
WITH_COUNTER = TableSeed("DF45", 8538, 7839)
WITHOUT_COUNTER = TableSeed("DF45", 8538, None)


def record(
    room: int = ROOM, counter: int = 7839, *, memories: bool = True
) -> bytes:
    data = bytearray(HEADER + WRAM)
    data[HEADER - 1] = int(memories)
    data[HEADER + ROOM_WORD : HEADER + ROOM_WORD + 2] = room.to_bytes(
        2, "little"
    )
    data[HEADER + COUNTER_WORD : HEADER + COUNTER_WORD + 2] = counter.to_bytes(
        2, "little"
    )
    return bytes(data)


def relock(tmp_path: Path, seed: TableSeed = WITH_COUNTER) -> Relock:
    (tmp_path / "out/any-boundary").mkdir(parents=True, exist_ok=True)
    _ = (tmp_path / "out/any-boundary/0001.lss").write_bytes(b"state")
    return Relock(
        tmp_path / "plan",
        tmp_path / "out",
        tmp_path / "logs",
        FrozenOrderedSet[str](),
        FrozenOrderedDict({"CAPTURES": str(tmp_path / "captures")}),
        FrozenOrderedDict({"any-boundary": FrozenOrderedDict({1: seed})}),
    )


def check(
    tmp_path: Path, data: bytes, seed: TableSeed = WITH_COUNTER
) -> Verdict:
    checker = relock(tmp_path, seed)
    _ = (tmp_path / "out/any-boundary/0001.record").write_bytes(data)
    return checker.check_cut(CUT)


def capture(tmp_path: Path, data: bytes) -> None:
    folder = tmp_path / "captures/target/seeds/any-boundary"
    folder.mkdir(parents=True, exist_ok=True)
    _ = (folder / "0001-DF45-wram.bin").write_bytes(data)


def outcome(verdict: Verdict) -> tuple[bool, bool]:
    return verdict.passed, verdict.compared


def test_a_window_holds_its_captures_together_in_frame_order() -> None:
    lines = [
        "skip 100-x 3 no-lsnes-movie",
        "cut any 9000 any-boundary 10 DF8D 12793",
        "cut any 900 any-boundary 2 DF8D 8950",
        "cut any 9000 any-boundary 4 DF45 9500",
        "derive scenario-a 5 any 5 --word 09C2=0001",
    ]
    assert sorted(lines, key=plan_order) == [
        "cut any 900 any-boundary 2 DF8D 8950",
        "cut any 9000 any-boundary 4 DF45 9500",
        "cut any 9000 any-boundary 10 DF8D 12793",
        "derive scenario-a 5 any 5 --word 09C2=0001",
        "skip 100-x 3 no-lsnes-movie",
    ]


def test_a_capture_starts_from_the_latest_rung_before_its_step() -> None:
    ladder = "# rung\n3000\n9000\n"
    assert rung_for(ladder, 8999) == 3000
    assert rung_for(ladder, 9000) == 9000
    assert rung_for(ladder, 100) == 0
    assert rung_for(None, 9000) == 0


def test_a_seed_needs_its_memories_room_and_frame_counter(
    tmp_path: Path,
) -> None:
    assert outcome(check(tmp_path, record())) == (True, False)
    missing = check(tmp_path, record(memories=False)).line
    assert "without memories" in missing
    assert "room DF8D, table DF45" in check(tmp_path, record(room=0xDF8D)).line
    late = check(tmp_path, record(counter=7838)).line
    assert "frame counter 7838, table 7839" in late


def test_work_ram_differs_from_its_capture_only_outside_the_stack_page(
    tmp_path: Path,
) -> None:
    wram = bytearray(record()[HEADER:])
    wram[0x1F10] ^= 1
    capture(tmp_path, bytes(wram))
    assert outcome(check(tmp_path, record())) == (True, True)
    wram[0x0010] ^= 1
    capture(tmp_path, bytes(wram))
    verdict = check(tmp_path, record())
    assert outcome(verdict) == (False, True)
    assert ": 1 WRAM bytes differ" in verdict.line


def test_a_table_without_a_frame_counter_takes_the_captures(
    tmp_path: Path,
) -> None:
    unknown = check(tmp_path, record(), WITHOUT_COUNTER).line
    assert "neither the table nor a capture" in unknown
    capture(tmp_path, record(counter=7838)[HEADER:])
    late = check(tmp_path, record(), WITHOUT_COUNTER).line
    assert "frame counter 7839, table 7838" in late
    on_time = check(tmp_path, record(counter=7838), WITHOUT_COUNTER)
    assert outcome(on_time) == (True, True)
