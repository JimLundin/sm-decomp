"""Compare native seed members, excluding the emulator's rerecord history."""

import sys
import zipfile
from os import PathLike


RERECORD_HISTORY = frozenset({"rrdata", "rerecords"})


def same_state(old: str | PathLike[str], new: str | PathLike[str]) -> bool:
    """Keep the member set, project identity and all non-history bytes exact."""
    with zipfile.ZipFile(old) as before, zipfile.ZipFile(new) as after:
        old_members = before.namelist()
        new_members = after.namelist()
        if len(old_members) != len(set(old_members)):
            return False
        if len(new_members) != len(set(new_members)):
            return False
        if set(old_members) != set(new_members):
            return False
        return all(
            before.read(name) == after.read(name)
            for name in old_members
            if name not in RERECORD_HISTORY
        )


def main() -> int:
    if len(sys.argv) != 3:
        return 2
    try:
        return 0 if same_state(sys.argv[1], sys.argv[2]) else 1
    except (OSError, zipfile.BadZipFile):
        return 1


if __name__ == "__main__":
    sys.exit(main())
