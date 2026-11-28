import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SEPARATOR = "\x1f"
FAKE = f"""#!{sys.executable}
import os
import sys
from pathlib import Path

role = Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["FAKE_LOG"], "a") as log:
    log.write({SEPARATOR!r}.join([role, *args]) + "\\n")


def status(name):
    return int(os.environ.get(name, "0"))


def after(flag):
    return args[args.index(flag) + 1]


if role == "python":
    if args == ["tools/check_replay_transition.py"]:
        sys.exit(status("TRANSITION_STATUS"))
    if args == ["tools/ci/required_witnesses.py", ""]:
        print("witness log")
        sys.exit(status("WITNESS_STATUS"))
    sys.exit(85)
if args[1] == "check" and "--shard" in args:
    shard = after("--shard")
    Path(after("--write")).write_text(shard + "\\n")
    print(f"shard {{shard}} log")
    sys.exit(17 if shard == os.environ.get("FAILING_SHARD") else 0)
if args[1] == "check":
    sys.exit(status("REPLAY_STATUS"))
if args[1] == "merge":
    receipts = [Path(args[i + 1]).read_text() for i, arg in enumerate(args) if arg == "--receipt"]
    Path(after("--write")).write_text("".join(receipts))
    sys.exit(0)
sys.exit(87)
"""
TRANSITION = ["python", "tools/check_replay_transition.py"]
CHECK = ["tools", "replay", "check", "scenarios/replay/index.toml"]
INPUTS = ["--baseline", "scenarios/replay/baseline.toml", "--cache", "recorded-cache", "--rom", "player-rom",
          "--native", "recorded-native", "--bundle", ""]


class ReplayEntrypoints(unittest.TestCase):
    def setUp(self) -> None:
        scratch = Path(self.enterContext(tempfile.TemporaryDirectory()))
        for role in ("python", "tools"):
            fake = scratch / role
            fake.write_text(FAKE)
            fake.chmod(0o755)
        self.log = scratch / "calls"
        self.log.touch()
        self.receipt = scratch / "receipt"
        self.env = os.environ | {
            "SM_PY": str(scratch / "python"), "SM_TOOLS": str(scratch / "tools"), "FAKE_LOG": str(self.log),
            "SM_CACHE": "recorded-cache", "SM_ROM": "player-rom", "SM_AUDIO_NATIVE": "recorded-native",
        }

    def run_script(self, *args: str, **statuses: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["bash", *args], cwd=ROOT, env=self.env | statuses, capture_output=True,
                              text=True, check=False)

    def calls(self) -> list[list[str]]:
        return [line.split(SEPARATOR) for line in self.log.read_text().splitlines()]


class SharedEntrypoint(ReplayEntrypoints):
    def test_checks_the_baseline_after_the_transition_claims(self) -> None:
        self.assertEqual(self.run_script("tools/ci/replay.sh").returncode, 0)
        self.assertEqual(self.calls(), [TRANSITION, [*CHECK, "--all", *INPUTS]])

    def test_propagates_the_check_status(self) -> None:
        self.assertEqual(self.run_script("tools/ci/replay.sh", REPLAY_STATUS="17").returncode, 17)

    def test_stops_at_a_failed_transition_check(self) -> None:
        self.assertEqual(self.run_script("tools/ci/replay.sh", TRANSITION_STATUS="23").returncode, 23)
        self.assertEqual(self.calls(), [TRANSITION])

    def test_checks_one_case(self) -> None:
        case = "blocks/missing-requests"
        self.assertEqual(self.run_script("tools/ci/replay.sh", "--case", case).returncode, 0)
        self.assertEqual(self.calls(), [TRANSITION, [*CHECK, "--case", case, *INPUTS]])
        failed = self.run_script("tools/ci/replay.sh", "--case", case, REPLAY_STATUS="19")
        self.assertEqual(failed.returncode, 19)


class ParallelEntrypoint(ReplayEntrypoints):
    def run_parallel(self, **statuses: str) -> subprocess.CompletedProcess[str]:
        return self.run_script("tools/ci/replay_parallel.sh", str(self.receipt), **statuses)

    def test_merges_every_shard_after_one_transition_check(self) -> None:
        self.assertEqual(self.run_parallel().returncode, 0)
        self.assertEqual(self.receipt.read_text(), "0/3\n1/3\n2/3\n")
        calls = self.calls()
        self.assertEqual(calls[0], TRANSITION)
        self.assertEqual(calls.count(TRANSITION), 1)
        shards = sorted(call[call.index("--shard") + 1] for call in calls if call[:3] == CHECK[:3])
        self.assertEqual(shards, ["0/3", "1/3", "2/3"])
        for call in calls:
            if call[:3] == CHECK[:3]:
                self.assertEqual(call[:len(CHECK) + 1 + len(INPUTS)], [*CHECK, "--all", *INPUTS])

    def test_a_failed_shard_fails_before_merging(self) -> None:
        result = self.run_parallel(FAILING_SHARD="1/3")
        self.assertNotEqual(result.returncode, 0)
        lines = result.stdout.splitlines()
        self.assertIn("::error::shard-1 failed with exit 17", lines)
        self.assertIn("shard 1/3 log", lines)
        self.assertFalse(self.receipt.exists())

    def test_a_failed_witness_fails_with_its_log(self) -> None:
        result = self.run_parallel(WITNESS_STATUS="23")
        self.assertNotEqual(result.returncode, 0)
        lines = result.stdout.splitlines()
        self.assertIn("::error::witnesses failed with exit 23", lines)
        self.assertIn("witness log", lines)
        self.assertFalse(self.receipt.exists())

    def test_a_failed_transition_check_starts_nothing(self) -> None:
        result = self.run_parallel(TRANSITION_STATUS="24")
        self.assertEqual(result.returncode, 24)
        self.assertEqual(result.stdout, "")
        self.assertEqual(self.calls(), [TRANSITION])
        self.assertFalse(self.receipt.exists())


if __name__ == "__main__":
    unittest.main()
