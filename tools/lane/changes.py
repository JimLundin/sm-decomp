import subprocess
import sys
from collections.abc import Sequence


def changed_since(base: str, paths: Sequence[str]) -> bool:
    committed = subprocess.run(["git", "diff", "--quiet", "--no-renames", f"{base}...HEAD", "--", *paths],
                               capture_output=True, check=False)
    if committed.returncode != 0:
        return True
    status = subprocess.run(["git", "status", "--porcelain", "--no-renames", "--untracked-files=all", "--", *paths],
                            capture_output=True, text=True, check=False)
    return status.returncode != 0 or bool(status.stdout.strip())


def main(arguments: list[str]) -> int:
    if len(arguments) < 2:
        print("usage: changes.py BASE PATH..., prints unchanged=true when no PATH changed since BASE",
              file=sys.stderr)
        return 2
    if not changed_since(arguments[0], arguments[1:]):
        print("unchanged=true")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
