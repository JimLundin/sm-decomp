import subprocess
lazy from pathlib import Path


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
