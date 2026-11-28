"""Exercise cache reuse, recovery and batch status with synthetic captures."""

from hashlib import sha256
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import time
import unittest


SCRIPT = Path(__file__).with_name("record_caches.py").resolve()

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


def digest(data):
    return sha256(data).hexdigest()


def lean_record():
    # One whole VblankWithoutNmi record, with zeroed registers and no memories or events.
    core = struct.pack("<4sBQIHBI??", b"SMFR", 2, 0, 0, 0, 1, 0, False, False)
    cpu = bytes(16)
    memories = bytes(5 * 4)
    ppu = bytes(153)
    events = bytes(4 + 4)
    frame = core + cpu + memories + ppu + events
    return struct.pack("<4sBBI", b"SMFL", 1, 0, len(frame)) + frame + memories


class RecordCachesTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # The batch reads caches with the tools binary. Build it once here, so no batch times out on a cold build.
        cls.tools = os.environ.get("SM_TOOLS")
        if not cls.tools:
            root = SCRIPT.parents[2]
            artifacts = Path(os.environ.get("CARGO_TARGET_DIR") or root / "target")
            subprocess.run([os.environ.get("CARGO") or "cargo", "build", "--quiet", "--locked", "--profile", "verify",
                            "--manifest-path", str(root / "Cargo.toml"), "--target-dir", str(artifacts), "-p", "tools"],
                           check=True)
            cls.tools = str(artifacts / "verify/tools")

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bundle = self.root / "bundle"
        (self.bundle / "pads").mkdir(parents=True)
        (self.bundle / "seeds" / "capture").mkdir(parents=True)
        self.state = self.bundle / "seeds" / "capture" / "0001.lss"
        self.state.write_bytes(b"synthetic seed")
        self.fixtures = self.root / "fixtures"
        self.fixtures.mkdir()
        self.records = lean_record()
        (self.fixtures / "lean.bin").write_bytes(self.records)
        self.cache = self.root / "cache"
        self.rom = self.root / "rom"
        self.rom.touch()
        self.calls = self.root / "calls"
        self.calls.touch()
        self.stub = self.root / "oracle"
        self.stub.write_text(STUB)
        self.stub.chmod(0o755)
        self.lock = self.root / "lock.toml"
        self.pads = b"synthetic pad"
        self.lsnes = self.root / "lsnes"
        self.lsnes.write_text("synthetic lsnes\n")
        self.lsnes.chmod(0o755)
        self.server = digest(b"")

    def write_inputs(self, rows):
        self.lock.write_text(
            'format = 2\n'
            'oracle = { emulator = "synthetic", version = "test", '
            f'server = "{self.server}", fixes = [] }}\n'
            + "".join(f"""
[[row]]
id = "{row}"
start = "capture:1"
seed_record = "{digest(self.state.read_bytes())}"
pads = "{digest(self.pads)}"
frames = 1
""" for row in rows)
        )
        (self.fixtures / "lean.txt").write_text(
            "# sm-decomp record cache lean 2\nrow fixture\n"
            f"seed_record {digest(self.state.read_bytes())}\nseed_words -\n"
            f"pads {digest(self.pads)}\nframes 1\n"
            f"lsnes {digest(self.lsnes.read_bytes())}\nserver {self.server}\n"
            f"# frame offset length cut\n0 0 {len(self.records)} V\n"
        )

    def add_pad(self, row):
        (self.bundle / "pads" / f"{row}.pads").write_bytes(self.pads)

    def run_batch(self, *rows, watches="", jobs=2, env=None, prepare=True):
        if prepare:
            self.write_inputs(rows)
        return subprocess.run(
            [
                sys.executable, SCRIPT, "-", self.bundle, self.rom,
                self.cache, str(jobs),
            ],
            input="\n".join(rows),
            capture_output=True,
            text=True,
            env={**os.environ, "ORACLE": str(self.stub), "CALLS": str(self.calls),
                 "WATCH_PCS": watches, "SM_LOCK": str(self.lock), "SM_TOOLS": self.tools,
                 "SM_LSNES": str(self.lsnes),
                 "FIXTURES": str(self.fixtures), "FINAL_CACHE": str(self.cache),
                 **(env or {})},
            timeout=10,
        )

    def assert_recorded(self, *rows, watches=""):
        result = self.run_batch(*rows, watches=watches)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_missing_pad_fails_batch_and_other_rows_finish(self):
        self.add_pad("first")
        result = self.run_batch("missing", "first")
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((self.cache / "first" / "lean.txt").is_file())

    def test_failed_last_worker_fails_batch(self):
        self.add_pad("first")
        self.add_pad("failed")
        result = self.run_batch("first", "failed", jobs=8)
        self.assertNotEqual(result.returncode, 0)
        self.assertTrue((self.cache / "first" / "lean.txt").is_file())
        self.assertCountEqual(self.calls.read_text().splitlines(), ["first", "failed"])

    def test_worker_can_start_when_a_later_worker_finishes_first(self):
        for row in ("first", "second", "third"):
            self.add_pad(row)
        (self.fixtures / "await-third").touch()
        self.assert_recorded("first", "second", "third")
        self.assertCountEqual(self.calls.read_text().splitlines(), ["first", "second", "third"])

    def test_warm_reuse_requires_records_and_requested_watches(self):
        self.add_pad("warm")
        directory = self.cache / "warm"
        self.assert_recorded("warm")
        (directory / "lean.bin").unlink()
        self.assert_recorded("warm")
        self.assert_recorded("warm", watches="$00:80AA")
        (directory / "hits.txt").write_text("# watched 0080aa\n")
        self.assert_recorded("warm", watches="828080")
        self.assert_recorded("warm", watches="82:8080,$+00:80AA")
        self.assertEqual(self.calls.read_text().splitlines(), ["warm"] * 4)

    def test_truncated_and_stale_caches_are_regenerated(self):
        self.add_pad("stale")
        directory = self.cache / "stale"
        self.assert_recorded("stale")
        index = directory / "lean.txt"
        index.write_text(index.read_text().rsplit("\n", 2)[0] + "\n")
        self.assert_recorded("stale")
        records = directory / "lean.bin"
        records.write_bytes(self.records[:-1])
        self.assert_recorded("stale")
        records.write_bytes(b"X" + self.records[1:])
        self.assert_recorded("stale")
        self.state.write_bytes(b"new synthetic seed")
        self.assert_recorded("stale")
        self.assert_recorded("stale")
        self.assertEqual(self.calls.read_text().splitlines(), ["stale"] * 5)

    def test_a_cache_from_another_lsnes_or_server_is_regenerated(self):
        self.add_pad("drift")
        self.assert_recorded("drift")
        self.lsnes.write_text("another synthetic lsnes\n")
        self.assert_recorded("drift")
        self.server = digest(b"another server")
        self.assert_recorded("drift")
        self.assertEqual(self.calls.read_text().splitlines(), ["drift"] * 3)

    def test_unusable_checker_or_lock_preserves_warm_row(self):
        self.add_pad("warm")
        self.assert_recorded("warm")
        index = self.cache / "warm" / "lean.txt"
        original = index.read_bytes()
        old_tools = self.root / "old-tools"
        old_tools.write_text(f"#!{sys.executable}\nraise SystemExit(1)\n")
        old_tools.chmod(0o755)
        malformed_lock = self.root / "malformed.toml"
        malformed_lock.write_text("invalid")
        for change in (
            {"SM_TOOLS": str(old_tools)},
            {"SM_TOOLS": "true"},
            {"SM_LOCK": str(malformed_lock)},
            {"SM_LOCK": str(self.root / "missing.toml")},
        ):
            result = self.run_batch("warm", env=change)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(index.read_bytes(), original)
        self.assertEqual(self.calls.read_text().splitlines(), ["warm"])

    def test_parallel_requests_keep_both_watch_sets(self):
        self.add_pad("warm")
        self.write_inputs(["warm"])

        def wait_for(signal):
            for _ in range(250):
                if (self.fixtures / signal).exists():
                    return
                time.sleep(0.02)
            self.fail(f"worker did not reach {signal}")

        def batch(watches, held):
            return subprocess.Popen(
                [sys.executable, SCRIPT, "-", self.bundle, self.rom, self.cache, "2"],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                env={**os.environ, "ORACLE": str(self.stub), "CALLS": str(self.calls), "WATCH_PCS": watches,
                     "SM_LOCK": str(self.lock), "SM_LSNES": str(self.lsnes), "FIXTURES": str(self.fixtures),
                     "FINAL_CACHE": str(self.cache), "HOLD_ORACLE": "1" if held else ""},
            )

        first = batch("0080aa", True)
        second = None
        try:
            first.stdin.write("warm\n")
            first.stdin.close()
            wait_for("held")
            second = batch("828080", False)
            second.stdin.write("warm\n")
            second.stdin.close()
            self.assertEqual(second.stdout.readline(), "warm waits for another recorder\n")
            self.assertEqual(self.calls.read_text().splitlines(), ["warm"])
        finally:
            (self.fixtures / "release").touch()
        for process in (first, second):
            output = process.stdout.read()
            self.assertEqual(process.wait(timeout=10), 0, output)
            process.stdout.close()

        self.assert_recorded("warm", watches="0080aa,828080")
        self.assertEqual(self.calls.read_text().splitlines(), ["warm", "warm"])

    def test_failed_watch_refresh_preserves_previous_cache(self):
        self.add_pad("warm")
        self.assert_recorded("warm")
        row = self.cache / "warm"
        original = {file.name: file.read_bytes() for file in row.iterdir()}
        (self.fixtures / "interrupt").touch()
        result = self.run_batch("warm", watches="828080")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual({file.name: file.read_bytes() for file in row.iterdir()}, original)
        self.assert_recorded("warm")
        self.assertEqual(self.calls.read_text().splitlines(), ["warm", "warm"])

    def test_corrupt_output_fails_even_when_oracle_returns_success(self):
        self.add_pad("corrupt")
        (self.fixtures / "interrupt").write_text("success")
        result = self.run_batch("corrupt")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.cache / "corrupt" / "lean.txt").exists())

    def test_interrupted_publication_is_not_reused(self):
        self.add_pad("interrupted")
        marker = self.fixtures / "interrupt"
        marker.touch()
        result = self.run_batch("interrupted")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.cache / "interrupted" / "lean.txt").exists())
        marker.unlink()
        self.assert_recorded("interrupted")
        self.assert_recorded("interrupted")
        self.assertEqual(self.calls.read_text().splitlines(), ["interrupted"] * 2)


if __name__ == "__main__":
    unittest.main()
