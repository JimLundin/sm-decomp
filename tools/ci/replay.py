import os
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path

USAGE = """usage: replay.py baseline RECEIPT | replay.py case ID [ARG...]

baseline: checks the required witnesses and the whole reviewed baseline at once, the witnesses
beside three indexed shards, and merges the shard receipts into RECEIPT, byte for byte the receipt
one unsharded check writes. Every part runs to its end and prints its log under its own label;
any failing part fails the check. Needs SM_CACHE, SM_ROM and SM_AUDIO_NATIVE.

case: checks one indexed diagnostic case, building the verifier from the checkout unless SM_TOOLS
names one; ARGs go to `tools replay check`."""
INDEX = "scenarios/replay/index.toml"
BASELINE = "scenarios/replay/baseline.toml"
SHARDS = 3


def python() -> str:
    return os.environ.get("SM_PY") or sys.executable


def tools() -> str:
    return os.environ.get("SM_TOOLS") or "target/verify/tools"


def transition() -> int:
    return subprocess.run([python(), "tools/check_replay_transition.py"], check=False).returncode


def check(checker: list[str], selection: list[str], extra: list[str]) -> list[str]:
    return [*checker, "replay", "check", INDEX, *selection, "--baseline", BASELINE,
            "--cache", os.environ.get("SM_CACHE", ""), "--rom", os.environ.get("SM_ROM") or "rom/sm.smc",
            "--native", os.environ.get("SM_AUDIO_NATIVE", ""), "--bundle", os.environ.get("SM_BUNDLE", ""), *extra]


def case(name: str, extra: list[str]) -> int:
    if status := transition():
        return status
    head = subprocess.run(["git", "rev-parse", "HEAD"], capture_output=True, text=True, check=False).stdout.strip()
    print(f"Reproduce on {head} with python3 tools/ci/replay.py case {shlex.quote(name)}", flush=True)
    checker = [tools()] if os.environ.get("SM_TOOLS") else [
        os.environ.get("CARGO") or "cargo", "run", "--quiet", "--locked", "--profile", "verify", "-p", "tools",
        "--bin", "tools", "--"]
    return subprocess.run(check(checker, ["--case", name], extra), check=False).returncode


def baseline(receipt: str) -> int:
    for name in ("SM_CACHE", "SM_ROM", "SM_AUDIO_NATIVE"):
        if not os.environ.get(name):
            print(f"replay: {name} is required", file=sys.stderr)
            return 2
    if status := transition():
        return status
    with tempfile.TemporaryDirectory() as directory:
        logs = Path(directory)
        parts = {"witnesses": [python(), "tools/ci/required_witnesses.py", ""]}
        for shard in range(SHARDS):
            parts[f"shard-{shard}"] = check([tools()], ["--all"],
                                            ["--shard", f"{shard}/{SHARDS}", "--write", str(logs / f"shard-{shard}.toml")])
        running = {}
        for label, command in parts.items():
            with (logs / f"{label}.log").open("w") as log:
                running[label] = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT)
        failed = False
        for label, process in running.items():
            status = process.wait()
            log_lines = (logs / f"{label}.log").read_text(errors="replace").splitlines()
            if status == 0:
                print("\n".join([f"::group::{label} passed", *log_lines, "::endgroup::"]), flush=True)
            else:
                failed = True
                print("\n".join([f"::error::{label} failed with exit {status}", *log_lines]), flush=True)
        if failed:
            return 1
        receipts = [argument for shard in range(SHARDS) for argument in ("--receipt", str(logs / f"shard-{shard}.toml"))]
        return subprocess.run([tools(), "replay", "merge", INDEX, "--all", "--baseline", BASELINE, *receipts,
                               "--write", receipt], check=False).returncode


def main(arguments: list[str]) -> int:
    match arguments:
        case ["baseline", receipt]:
            return baseline(receipt)
        case ["case", name, *extra]:
            return case(name, extra)
        case _:
            print(USAGE, file=sys.stderr)
            return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
