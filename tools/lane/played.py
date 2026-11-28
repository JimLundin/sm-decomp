import os
import subprocess
import sys
from pathlib import Path

USAGE = """usage: played.py [<row> | <row>/<start>-<end>]...

Plays the reviewed runs of scenarios/replay/played.toml, every run without names, in one build
and one test process. Each run prints its held cuts and stop beside the file's, and the exit is
nonzero when any differs. Needs SM_CACHE and SM_ROM; SM_LOCK defaults to the lsnes lock."""
TEST = "replay::played::tests::reviewed_runs_hold"


def main(arguments: list[str]) -> int:
    if any(argument.startswith("-") for argument in arguments):
        print(USAGE, file=sys.stderr)
        return 2
    for name in ("SM_CACHE", "SM_ROM"):
        if not os.environ.get(name):
            print(f"played: set {name}", file=sys.stderr)
            return 2
    top = Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True,
                              check=True).stdout.strip())
    environment = os.environ | {
        "SM_LOCK": os.environ.get("SM_LOCK") or str(top / "scenarios/locks/lsnes.toml"),
        "CARGO_PROFILE_TEST_OPT_LEVEL": os.environ.get("CARGO_PROFILE_TEST_OPT_LEVEL") or "3",
        "SM_PLAYED": " ".join(arguments),
    }
    command = ["cargo", "test", "-p", "verify", "--lib", "--", "--ignored", "--exact", TEST, "--nocapture"]
    lines = []
    with subprocess.Popen(command, cwd=top, env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                          text=True, errors="replace") as test:
        assert test.stdout is not None
        for line in test.stdout:
            print(line, end="", flush=True)
            lines.append(line.rstrip("\n"))
    if test.returncode != 0:
        return test.returncode
    if f"test {TEST} ... ok" not in lines or not any(
            line.startswith("test result: ok. 1 passed; 0 failed; 0 ignored") for line in lines):
        print(f"played: {TEST} did not run as the one test", file=sys.stderr)
        return 4
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
