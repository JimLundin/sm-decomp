import os
import sys
lazy from pathlib import Path

from ordered import FrozenList
lazy import pytest

from sm_tools.ci.shell import check_replay_transition
from sm_tools.ci.shell.replay import run_baseline, run_case

SEPARATOR = "\x1f"
FAKE = f"""#!{sys.executable}
import os
import sys
lazy from pathlib import Path

role = Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ["FAKE_LOG"], "a") as log:
    log.write({SEPARATOR!r}.join([role, *args]) + "\\n")


def status(name):
    return int(os.environ.get(name, "0"))


def after(flag):
    return args[args.index(flag) + 1]


if role == "required-witnesses":
    print("witness log")
    sys.exit(status("WITNESS_STATUS"))
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
TRANSITION = FrozenList(("transition",))
CHECK = FrozenList(("tools", "replay", "check", "scenarios/replay/index.toml"))
INPUTS = FrozenList((
    "--baseline",
    "scenarios/replay/baseline.toml",
    "--cache",
    "recorded-cache",
    "--rom",
    "player-rom",
    "--native",
    "recorded-native",
    "--bundle",
    "",
))
MERGE = FrozenList((
    "tools",
    "replay",
    "merge",
    "scenarios/replay/index.toml",
    "--all",
    "--baseline",
    "scenarios/replay/baseline.toml",
))
CASE = "blocks/missing-requests"


def fakes(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, transition_status: int = 0
) -> Path:
    for role in ("tools", "required-witnesses"):
        fake = tmp_path / role
        _ = fake.write_text(FAKE)
        fake.chmod(0o755)
    log = tmp_path / "calls"
    log.touch()
    monkeypatch.setenv("PATH", str(tmp_path), prepend=os.pathsep)
    for name, value in (
        ("SM_TOOLS", str(tmp_path / "tools")),
        ("FAKE_LOG", str(log)),
        ("SM_CACHE", "recorded-cache"),
        ("SM_ROM", "player-rom"),
        ("SM_AUDIO_NATIVE", "recorded-native"),
    ):
        monkeypatch.setenv(name, value)
    monkeypatch.delenv("SM_BUNDLE", raising=False)

    def transition(_root: Path) -> int:
        with log.open("a") as written:
            _ = written.write(f"{TRANSITION[0]}\n")
        return transition_status

    monkeypatch.setattr(check_replay_transition, "check", transition)
    return log


def calls(log: Path) -> FrozenList[FrozenList[str]]:
    return FrozenList(
        FrozenList(line.split(SEPARATOR))
        for line in log.read_text(encoding="utf-8").splitlines()
    )


def test_checks_one_case_after_the_transition_claims(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    log = fakes(tmp_path, monkeypatch)
    assert run_case(CASE, FrozenList[str]()) == 0
    assert calls(log) == (TRANSITION, (*CHECK, "--case", CASE, *INPUTS))


def test_propagates_the_check_status(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _ = fakes(tmp_path, monkeypatch)
    monkeypatch.setenv("REPLAY_STATUS", "19")
    assert run_case(CASE, FrozenList[str]()) == 19


def test_stops_at_a_failed_transition_check(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    log = fakes(tmp_path, monkeypatch, transition_status=23)
    assert run_case(CASE, FrozenList[str]()) == 23
    assert calls(log) == (TRANSITION,)


def test_merges_every_shard_after_one_transition_check(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    log = fakes(tmp_path, monkeypatch)
    receipt = tmp_path / "receipt"
    assert run_baseline(str(receipt)) == 0
    assert receipt.read_text() == "0/3\n1/3\n2/3\n"
    made = calls(log)
    assert made[0] == TRANSITION
    assert made.count(TRANSITION) == 1
    checks = [call for call in made if call[:3] == CHECK[:3]]
    shards = sorted(call[call.index("--shard") + 1] for call in checks)
    assert shards == ["0/3", "1/3", "2/3"]
    for call in checks:
        assert call[: len(CHECK) + 1 + len(INPUTS)] == (
            *CHECK,
            "--all",
            *INPUTS,
        )
    merges = [call for call in made if call[:3] == MERGE[:3]]
    assert len(merges) == 1
    assert merges[0][: merges[0].index("--receipt")] == MERGE


def test_a_failed_shard_fails_before_merging(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    _ = fakes(tmp_path, monkeypatch)
    monkeypatch.setenv("FAILING_SHARD", "1/3")
    receipt = tmp_path / "receipt"
    assert run_baseline(str(receipt)) != 0
    lines = capsys.readouterr().out.splitlines()
    assert "::error::shard-1 failed with exit 17" in lines
    assert "shard 1/3 log" in lines
    assert not receipt.exists()


def test_a_failed_witness_fails_with_its_log(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    _ = fakes(tmp_path, monkeypatch)
    monkeypatch.setenv("WITNESS_STATUS", "23")
    receipt = tmp_path / "receipt"
    assert run_baseline(str(receipt)) != 0
    lines = capsys.readouterr().out.splitlines()
    assert "::error::witnesses failed with exit 23" in lines
    assert "witness log" in lines
    assert not receipt.exists()


def test_a_failed_transition_check_starts_nothing(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    log = fakes(tmp_path, monkeypatch, transition_status=24)
    receipt = tmp_path / "receipt"
    assert run_baseline(str(receipt)) == 24
    assert not capsys.readouterr().out
    assert calls(log) == (TRANSITION,)
    assert not receipt.exists()
