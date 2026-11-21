"""Validate a pinned native fixture archive and publish its directory."""

import argparse
from filecmp import cmp
from pathlib import Path, PurePosixPath
from stat import FILE_ATTRIBUTE_REPARSE_POINT, S_ISDIR, S_ISREG
import subprocess
import sys
import tarfile
import tempfile


def matches_installation(staged, installed):
    info = installed.lstat()
    if getattr(info, "st_file_attributes", 0) & FILE_ATTRIBUTE_REPARSE_POINT:
        return False
    if staged.is_file():
        return S_ISREG(info.st_mode) and info.st_nlink == 1 and cmp(
            staged, installed, shallow=False
        )
    if not S_ISDIR(info.st_mode):
        return False

    staged_entries = {entry.name: entry for entry in staged.iterdir()}
    installed_entries = {entry.name: entry for entry in installed.iterdir()}
    return staged_entries.keys() == installed_entries.keys() and all(
        matches_installation(entry, installed_entries[name])
        for name, entry in staged_entries.items()
    )


def publish(archive_path, destination, folder, count, qualification=()):
    target = destination / folder
    with tempfile.TemporaryDirectory(dir=destination) as staging:
        with tarfile.open(archive_path, "r:gz") as archive:
            entries = archive.getmembers()
            paths = [PurePosixPath(entry.name) for entry in entries]
            if len(entries) != count or len(set(paths)) != len(entries):
                raise ValueError("unexpected native fixture member count")
            for entry, path in zip(entries, paths, strict=True):
                if (
                    not entry.isfile()
                    or path.parts[:1] != (folder,)
                    or len(path.parts) < 2
                    or ".." in path.parts
                    or "\\" in entry.name
                ):
                    raise ValueError("unexpected native fixture member")
            archive.extractall(staging, filter="data")

        packet = Path(staging) / folder
        if qualification:
            subprocess.run(
                [
                    sys.executable, "-B", str(packet / "check_packet.py"),
                    "--root", str(packet), *qualification,
                ],
                check=True,
            )

        # A retry can reuse an earlier packet if its tree agrees with the pinned archive.
        # Always execute the checker from staging, where its code comes from that archive.
        if target.is_symlink() or target.exists():
            if not qualification or not matches_installation(packet, target):
                raise FileExistsError(target)
        else:
            packet.rename(target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("folder")
    parser.add_argument("count", type=int)
    prerequisites = parser.add_mutually_exclusive_group()
    prerequisites.add_argument("--rom")
    prerequisites.add_argument("--parent")
    args = parser.parse_args()
    qualification = ()
    if args.rom is not None:
        qualification = ("--rom", args.rom)
    if args.parent is not None:
        qualification = ("--parent", args.parent)
    try:
        publish(args.archive, args.destination, args.folder, args.count, qualification)
    except (OSError, ValueError, tarfile.TarError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
