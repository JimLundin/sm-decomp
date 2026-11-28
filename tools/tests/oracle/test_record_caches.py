import io
import struct
import sys
from collections import Counter
from concurrent.futures import Future, ThreadPoolExecutor, wait
from dataclasses import dataclass, replace
from hashlib import sha256
lazy from collections.abc import Mapping, Sequence
lazy from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedDict

from sm_tools.oracle.shell.record_caches import AMBIENT, batch, cache_reader

STUB = f"""#!{sys.executable}
import os, shutil, sys, time
from pathlib import Path

arguments = sys.argv[1:]
if arguments[0] != "corpus" or arguments[2] != os.environ["SM_LOCK"] or not Path(arguments[3], "pads").is_dir():
    sys.exit(1)
options = dict(zip(arguments[4::2], arguments[5::2]))
if set(options) - {{"--only", "--cache", "--watch-pcs"}}:
    sys.exit(2)
row, cache = options["--only"], Path(options["--cache"])
fixtures = Path(os.environ["FIXTURES"])
with open(os.environ["CALLS"], "a") as calls:
    calls.write(row + "\\n")


def reaches(ready, attempts=250):
    for _ in range(attempts):
        if ready():
            return True
        time.sleep(0.02)
    return ready()


if os.environ.get("HOLD_ORACLE") == "1":
    (fixtures / "held").touch()
    if not reaches(lambda: (fixtures / "release").is_file()):
        sys.exit(1)
if row == "first":
    third = Path(os.environ["FINAL_CACHE"], "third", "lean.txt")
    if not reaches(lambda: not (fixtures / "await-third").is_file() or third.is_file(), 100):
        sys.exit(1)
if row == "failed":
    sys.exit(1)
(cache / row).mkdir(parents=True, exist_ok=True)
shutil.copy(fixtures / "lean.bin", cache / row / "lean.bin")
index = (fixtures / "lean.txt").read_text().replace("\\nrow fixture\\n", f"\\nrow {{row}}\\n")
(cache / row / "lean.txt").write_text(index)
interrupt = fixtures / "interrupt"
if interrupt.is_file():
    with open(cache / row / "lean.bin", "r+b") as records:
        records.truncate(1)
    sys.exit(0 if interrupt.read_text() == "success" else 1)
if "--watch-pcs" in options:
    addresses = [int(pc.lstrip("$").replace(":", ""), 16) for pc in options["--watch-pcs"].split(",")]
    (cache / row / "hits.txt").write_text(
        "# watched " + ",".join(f"{{pc:06x}}" for pc in addresses) + "\\n# frame pc line dot peek\\n")
"""
POLLS = 250
POLL_SECONDS = 0.02
WAIT_SECONDS = 10


def digest(data: bytes) -> str:
    return sha256(data).hexdigest()


def lean_record() -> bytes:
    core = struct.pack("<4sBQIHBI??", b"SMFR", 2, 0, 0, 0, 1, 0, 0, 0)
    cpu = bytes(16)
    memories = bytes(5 * 4)
    ppu = bytes(153)
    events = bytes(4 + 4)
    frame = core + cpu + memories + ppu + events
    return struct.pack("<4sBBI", b"SMFL", 1, 0, len(frame)) + frame + memories


@pytest.fixture(scope="module")
def tools() -> str:
    return cache_reader(AMBIENT)


@dataclass(frozen=True, slots=True)
class Rig:
    root: Path
    tools: str
    server: str = digest(b"")
    pads: bytes = b"synthetic pad"
    records: bytes = lean_record()

    @property
    def bundle(self) -> Path:
        return self.root / "bundle"

    @property
    def state(self) -> Path:
        return self.bundle / "seeds" / "capture" / "0001.lss"

    @property
    def fixtures(self) -> Path:
        return self.root / "fixtures"

    @property
    def cache(self) -> Path:
        return self.root / "cache"

    @property
    def rom(self) -> Path:
        return self.root / "rom"

    @property
    def calls(self) -> Path:
        return self.root / "calls"

    @property
    def stub(self) -> Path:
        return self.root / "oracle"

    @property
    def lock(self) -> Path:
        return self.root / "lock.toml"

    @property
    def lsnes(self) -> Path:
        return self.root / "lsnes"

    def called(self) -> FrozenList[str]:
        return FrozenList(self.calls.read_text().splitlines())

    def write_inputs(self, rows: Sequence[str]) -> None:
        state = digest(self.state.read_bytes())
        _ = self.lock.write_text(
            (
                'format = 2\noracle = { emulator = "synthetic", '
                f'version = "test", server = "{self.server}", fixes = [] }}\n'
            )
            + "".join(
                f"""
[[row]]
id = "{row}"
start = "capture:1"
seed_record = "{state}"
pads = "{digest(self.pads)}"
frames = 1
"""
                for row in rows
            )
        )
        index = (
            "# sm-decomp record cache lean 2\nrow fixture\n"
            f"seed_record {state}\nseed_words -\n"
            f"pads {digest(self.pads)}\nframes 1\n"
            f"lsnes {digest(self.lsnes.read_bytes())}\n"
            f"server {self.server}\n"
            f"# frame offset length cut\n0 0 {len(self.records)} V\n"
        )
        _ = (self.fixtures / "lean.txt").write_text(index)

    def add_pad(self, row: str) -> None:
        _ = (self.bundle / "pads" / f"{row}.pads").write_bytes(self.pads)

    def environment(
        self, watches: str, env: Mapping[str, str]
    ) -> FrozenOrderedDict[str, str]:
        return FrozenOrderedDict({
            "ORACLE": str(self.stub),
            "CALLS": str(self.calls),
            "WATCH_PCS": watches,
            "SM_LOCK": str(self.lock),
            "SM_TOOLS": self.tools,
            "SM_LSNES": str(self.lsnes),
            "FIXTURES": str(self.fixtures),
            "FINAL_CACHE": str(self.cache),
            **env,
        })

    def arguments(self, jobs: int) -> FrozenList[str]:
        return FrozenList((
            "-",
            str(self.bundle),
            str(self.rom),
            str(self.cache),
            str(jobs),
        ))

    def run_batch(
        self,
        *rows: str,
        watches: str = "",
        jobs: int = 2,
        env: Mapping[str, str] | None = None,
    ) -> int:
        self.write_inputs(rows)
        return batch(
            self.arguments(jobs),
            io.StringIO("\n".join(rows)),
            self.environment(watches, env or {}),
        )

    def assert_recorded(self, *rows: str, watches: str = "") -> None:
        assert self.run_batch(*rows, watches=watches) == 0


@pytest.fixture
def rig(tmp_path: Path, tools: str) -> Rig:
    made = Rig(tmp_path, tools)
    (made.bundle / "pads").mkdir(parents=True)
    made.state.parent.mkdir(parents=True)
    _ = made.state.write_bytes(b"synthetic seed")
    made.fixtures.mkdir()
    _ = (made.fixtures / "lean.bin").write_bytes(made.records)
    made.rom.touch()
    made.calls.touch()
    _ = made.stub.write_text(STUB)
    made.stub.chmod(0o755)
    _ = made.lsnes.write_text("synthetic lsnes\n")
    made.lsnes.chmod(0o755)
    return made


def test_missing_pad_fails_batch_and_other_rows_finish(rig: Rig) -> None:
    rig.add_pad("first")
    assert rig.run_batch("missing", "first") != 0
    assert (rig.cache / "first" / "lean.txt").is_file()


def test_failed_last_worker_fails_batch(rig: Rig) -> None:
    rig.add_pad("first")
    rig.add_pad("failed")
    assert rig.run_batch("first", "failed", jobs=8) != 0
    assert (rig.cache / "first" / "lean.txt").is_file()
    assert Counter(rig.called()) == Counter(("first", "failed"))


def test_worker_can_start_when_a_later_worker_finishes_first(rig: Rig) -> None:
    for row in ("first", "second", "third"):
        rig.add_pad(row)
    (rig.fixtures / "await-third").touch()
    rig.assert_recorded("first", "second", "third")
    assert Counter(rig.called()) == Counter(("first", "second", "third"))


def test_warm_reuse_requires_records_and_requested_watches(rig: Rig) -> None:
    rig.add_pad("warm")
    directory = rig.cache / "warm"
    rig.assert_recorded("warm")
    (directory / "lean.bin").unlink()
    rig.assert_recorded("warm")
    rig.assert_recorded("warm", watches="$00:80AA")
    _ = (directory / "hits.txt").write_text("# watched 0080aa\n")
    rig.assert_recorded("warm", watches="828080")
    rig.assert_recorded("warm", watches="82:8080,$+00:80AA")
    assert rig.called() == ("warm",) * 4


def test_truncated_and_stale_caches_are_regenerated(rig: Rig) -> None:
    rig.add_pad("stale")
    directory = rig.cache / "stale"
    rig.assert_recorded("stale")
    index = directory / "lean.txt"
    _ = index.write_text(index.read_text().rsplit("\n", 2)[0] + "\n")
    rig.assert_recorded("stale")
    records = directory / "lean.bin"
    _ = records.write_bytes(rig.records[:-1])
    rig.assert_recorded("stale")
    _ = records.write_bytes(b"X" + rig.records[1:])
    rig.assert_recorded("stale")
    _ = rig.state.write_bytes(b"new synthetic seed")
    rig.assert_recorded("stale")
    rig.assert_recorded("stale")
    assert rig.called() == ("stale",) * 5


def test_a_cache_from_another_lsnes_or_server_is_regenerated(rig: Rig) -> None:
    rig.add_pad("drift")
    rig.assert_recorded("drift")
    _ = rig.lsnes.write_text("another synthetic lsnes\n")
    rig.assert_recorded("drift")
    replace(rig, server=digest(b"another server")).assert_recorded("drift")
    assert rig.called() == ("drift",) * 3


def test_unusable_checker_or_lock_preserves_warm_row(rig: Rig) -> None:
    rig.add_pad("warm")
    rig.assert_recorded("warm")
    index = rig.cache / "warm" / "lean.txt"
    original = index.read_bytes()
    old_tools = rig.root / "old-tools"
    _ = old_tools.write_text(f"#!{sys.executable}\nraise SystemExit(1)\n")
    old_tools.chmod(0o755)
    malformed_lock = rig.root / "malformed.toml"
    _ = malformed_lock.write_text("invalid")
    for change in (
        {"SM_TOOLS": str(old_tools)},
        {"SM_TOOLS": "true"},
        {"SM_LOCK": str(malformed_lock)},
        {"SM_LOCK": str(rig.root / "missing.toml")},
    ):
        assert rig.run_batch("warm", env=change) != 0, change
        assert index.read_bytes() == original
    assert rig.called() == ("warm",)


def wait_for(signal: Path, running: Future[int]) -> None:
    for _ in range(POLLS):
        if signal.exists():
            return
        _ = wait((running,), timeout=POLL_SECONDS)
    pytest.fail(f"worker did not reach {signal.name}")


def first_line(running: Future[int], capsys: pytest.CaptureFixture[str]) -> str:
    seen = ""
    for _ in range(POLLS):
        seen += capsys.readouterr().out
        if "\n" in seen:
            break
        _ = wait((running,), timeout=POLL_SECONDS)
    return seen.splitlines(keepends=True)[0] if seen else ""


def test_parallel_requests_keep_both_watch_sets(
    rig: Rig, capsys: pytest.CaptureFixture[str]
) -> None:
    rig.add_pad("warm")
    rig.write_inputs(("warm",))

    def started(
        pool: ThreadPoolExecutor, watches: str, *, held: bool
    ) -> Future[int]:
        hold = {"HOLD_ORACLE": "1" if held else ""}
        return pool.submit(
            batch,
            rig.arguments(2),
            io.StringIO("warm\n"),
            rig.environment(watches, hold),
        )

    with ThreadPoolExecutor(max_workers=2) as pool:
        first = started(pool, "0080aa", held=True)
        try:
            wait_for(rig.fixtures / "held", first)
            second = started(pool, "828080", held=False)
            line = first_line(second, capsys)
            assert line == "warm waits for another recorder\n"
            assert rig.called() == ("warm",)
        finally:
            (rig.fixtures / "release").touch()
        assert first.result(timeout=WAIT_SECONDS) == 0
        assert second.result(timeout=WAIT_SECONDS) == 0

    rig.assert_recorded("warm", watches="0080aa,828080")
    assert rig.called() == ("warm", "warm")


def test_failed_watch_refresh_preserves_previous_cache(rig: Rig) -> None:
    rig.add_pad("warm")
    rig.assert_recorded("warm")
    row = rig.cache / "warm"
    original = {file.name: file.read_bytes() for file in row.iterdir()}
    (rig.fixtures / "interrupt").touch()
    assert rig.run_batch("warm", watches="828080") != 0
    assert {file.name: file.read_bytes() for file in row.iterdir()} == original
    rig.assert_recorded("warm")
    assert rig.called() == ("warm", "warm")


def test_corrupt_output_fails_even_when_oracle_returns_success(
    rig: Rig,
) -> None:
    rig.add_pad("corrupt")
    _ = (rig.fixtures / "interrupt").write_text("success")
    assert rig.run_batch("corrupt") != 0
    assert not (rig.cache / "corrupt" / "lean.txt").exists()


def test_interrupted_publication_is_not_reused(rig: Rig) -> None:
    rig.add_pad("interrupted")
    marker = rig.fixtures / "interrupt"
    marker.touch()
    assert rig.run_batch("interrupted") != 0
    assert not (rig.cache / "interrupted" / "lean.txt").exists()
    marker.unlink()
    rig.assert_recorded("interrupted")
    rig.assert_recorded("interrupted")
    assert rig.called() == ("interrupted",) * 2
