"""Exercise cache reuse, recovery and batch status with synthetic captures."""

from concurrent.futures import ThreadPoolExecutor
from hashlib import sha256
import os
from pathlib import Path
import shutil
import struct
import subprocess
import tempfile
import time
import unittest


SCRIPT = Path(__file__).with_name("record_caches.sh").resolve()

STUB = """#!/usr/bin/env bash
set -eu
[[ $1 == corpus && $3 == "$SM_LOCK" && -d $4/pads ]]
shift 4

while (($#)); do
  case $1 in
    --only) row=$2; shift ;;
    --cache) cache=$2; shift ;;
    --watch-pcs) watch_pcs=$2; shift ;;
    *) exit 2 ;;
  esac
  shift
done
echo "$row" >> "$CALLS"
if [[ ${HOLD_ORACLE:-} == 1 ]]; then
  touch "$FIXTURES/held"
  for ((attempt = 0; attempt < 250; attempt++)); do
    [[ -f "$FIXTURES/release" ]] && break
    sleep 0.02
  done
  [[ -f "$FIXTURES/release" ]]
fi
case $row in
  first)
    for ((attempt = 0; attempt < 100; attempt++)); do
      [[ ! -f "$FIXTURES/await-third" || -f "$FINAL_CACHE/third/lean.txt" ]] && break
      sleep 0.02
    done
    [[ ! -f "$FIXTURES/await-third" || -f "$FINAL_CACHE/third/lean.txt" ]]
    ;;
  failed) exit 1 ;;
esac
mkdir -p "$cache/$row"
cp "$FIXTURES/lean.bin" "$cache/$row/lean.bin"
sed "s/^row fixture$/row $row/" "$FIXTURES/lean.txt" > "$cache/$row/lean.txt"
if [[ -f "$FIXTURES/interrupt" ]]; then
  truncate -s 1 "$cache/$row/lean.bin"
  [[ $(cat "$FIXTURES/interrupt") == success ]] && exit 0
  exit 1
fi
if [[ -n ${watch_pcs:-} ]]; then
  python3 - "$watch_pcs" > "$cache/$row/hits.txt" <<'PYTHON'
import sys
addresses = [int(pc.lstrip("$").replace(":", ""), 16) for pc in sys.argv[1].split(",")]
print("# watched " + ",".join(f"{pc:06x}" for pc in addresses))
print("# frame pc line dot peek")
PYTHON
fi
"""


def digest(data):
    return sha256(data).hexdigest()


def lean_record():
    # One whole VblankWithoutNmi record, with zeroed registers and no memories or events.
    core = struct.pack("<4sBQIHBI??", b"SMFR", 1, 0, 0, 0, 1, 0, False, False)
    cpu = bytes(16)
    memories = bytes(5 * 4)
    ppu = bytes(153)
    events = bytes(4 + 3 + 4 + 4 + 8)
    frame = core + cpu + memories + ppu + events
    return struct.pack("<4sBBI", b"SMFL", 1, 0, len(frame)) + frame + memories


class RecordCachesTest(unittest.TestCase):
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

    def write_inputs(self, rows):
        self.lock.write_text(
            'format = 2\nmark_frames = 1000\n'
            'oracle = { emulator = "synthetic", version = "test", '
            f'server = "{digest(b"")}", fixes = [] }}\n'
            + "".join(f"""
[[row]]
id = "{row}"
start = "capture:1"
seed_record = "{digest(self.state.read_bytes())}"
pads = "{digest(self.pads)}"
frames = 1
ticks = 0
overruns = []
nmi_off = ["0"]
facts = {{ }}
absent = ["wram", "vram", "cgram", "oam", "sram", "ppu", "apu", "video", "timing"]
""" for row in rows)
        )
        (self.fixtures / "lean.txt").write_text(
            "# sm-decomp record cache lean 1\nrow fixture\n"
            f"seed_record {digest(self.state.read_bytes())}\nseed_words -\n"
            f"pads {digest(self.pads)}\nframes 1\npictures no\n"
            f"# frame offset length cut\n0 0 {len(self.records)} V\n"
        )

    def add_pad(self, row):
        (self.bundle / "pads" / f"{row}.pads").write_bytes(self.pads)

    def run_batch(self, *rows, watches="", jobs=2, env=None, prepare=True):
        if prepare:
            self.write_inputs(rows)
        return subprocess.run(
            [
                "bash", SCRIPT, "-", self.bundle, self.rom,
                self.cache, str(jobs),
            ],
            input="\n".join(rows),
            capture_output=True,
            text=True,
            env={**os.environ, "ORACLE": str(self.stub), "CALLS": str(self.calls),
                 "WATCH_PCS": watches, "SM_LOCK": str(self.lock),
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

    def test_unusable_checker_or_lock_preserves_warm_row(self):
        self.add_pad("warm")
        self.assert_recorded("warm")
        index = self.cache / "warm" / "lean.txt"
        original = index.read_bytes()
        old_tools = self.root / "old-tools"
        old_tools.write_text("#!/usr/bin/env bash\nexit 1\n")
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
        flock = self.root / "flock"
        flock.write_text(
            '#!/usr/bin/env bash\n'
            'touch "$FIXTURES/competing"\n'
            'exec "$REAL_FLOCK" "$@"\n'
        )
        flock.chmod(0o755)
        competing_env = {
            "PATH": f"{self.root}:{os.environ['PATH']}",
            "REAL_FLOCK": shutil.which("flock"),
        }

        def wait_for(signal):
            for _ in range(250):
                if (self.fixtures / signal).exists():
                    return
                time.sleep(0.02)
            self.fail(f"worker did not reach {signal}")

        with ThreadPoolExecutor(max_workers=2) as workers:
            first = workers.submit(
                self.run_batch, "warm", watches="0080aa",
                env={"HOLD_ORACLE": "1"}, prepare=False,
            )
            try:
                wait_for("held")
                second = workers.submit(
                    self.run_batch, "warm", watches="828080",
                    env=competing_env, prepare=False,
                )
                wait_for("competing")
                self.assertEqual(self.calls.read_text().splitlines(), ["warm"])
            finally:
                (self.fixtures / "release").touch()
            for result in (first.result(), second.result()):
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

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
