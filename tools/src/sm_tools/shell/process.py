import subprocess
from dataclasses import dataclass
lazy from collections.abc import Mapping, Sequence
lazy from pathlib import Path


@dataclass(frozen=True, slots=True)
class Finished:
    status: int
    stdout: bytes
    stderr: bytes

    def output(self) -> str:
        chosen = self.stderr if self.status else self.stdout
        return chosen.decode("utf-8", errors="replace")


class CommandError(Exception):
    def __init__(self, command: Sequence[str], status: int) -> None:
        super().__init__(f"command {list(command)} exited {status}")


def run(
    command: Sequence[str],
    *,
    cwd: Path | None = None,
    environment: Mapping[str, str] | None = None,
    stdin: bytes | None = None,
    capture_stderr: bool = True,
) -> Finished:
    done = subprocess.run(
        command,
        cwd=cwd,
        env=environment,
        input=stdin,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE if capture_stderr else None,
        check=False,
    )
    return Finished(done.returncode, done.stdout, done.stderr or b"")


def checked(
    command: Sequence[str],
    *,
    cwd: Path | None = None,
    stdin: bytes | None = None,
    capture_stderr: bool = True,
) -> bytes:
    done = run(command, cwd=cwd, stdin=stdin, capture_stderr=capture_stderr)
    if done.status:
        raise CommandError(command, done.status)
    return done.stdout


def git(
    *arguments: str, cwd: Path | None = None, stdin: str | None = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ("git", *arguments),
        cwd=cwd,
        input=stdin,
        capture_output=True,
        text=True,
        check=False,
    )


def git_out(
    *arguments: str, cwd: Path | None = None, stdin: str | None = None
) -> str:
    return subprocess.run(
        ("git", *arguments),
        cwd=cwd,
        input=stdin,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
