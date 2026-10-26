#!/usr/bin/env python3
"""Pack or restore ROM-encrypted corpus inputs and original-emulator seeds."""
import argparse
import csv
import hashlib
import io
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import tomllib


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def cipher(args, decrypt=False):
    rom = args.rom.read_bytes()
    if hashlib.sha256(rom).hexdigest() != "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72":
        raise ValueError("wrong ROM")
    env = dict(os.environ)
    env["SM_SEED_PASSPHRASE"] = hashlib.sha256(b"sm-decomp-seeds-v1" + rom).hexdigest()
    command = [args.openssl, "enc", "-aes-256-cbc", "-pbkdf2", "-iter", "200000",
               "-md", "sha256", "-pass", "env:SM_SEED_PASSPHRASE"]
    if decrypt:
        command += ["-d", "-in", str(args.bundle)]
    return command, env


def valid_name(name):
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name or ":" in name:
        return False
    return bool(re.fullmatch(r"movies/[A-Za-z0-9_-]+\.inputs", name) or re.fullmatch(
        r"target/checkpoints/[A-Za-z0-9_-]+\.smvchk", name) or re.fullmatch(
        r"target/seeds/[A-Za-z0-9-]+/(?:rooms\.csv|lag\.csv(?:\.(?:polls|events|eof-polls|eof-hardware|phases|milestones)\.csv|\.eof\.json)?|"
        r"snes9x\.(?:json|frames\.csv|events\.csv|state|live\.json|history\.json)|"
        r"[0-9]{4}-[0-9A-F]{4}-(?:wram|sram|vram|cgram|oam|apuram)\.bin|"
        r"[0-9]{4}-[0-9A-F]{4}-registers\.csv)", name))


def selected_rows(args):
    rows = tomllib.loads((args.root / "movies/segments.toml").read_text())["segment"]
    known = {row["id"] for row in rows}
    if set(args.segment) - known:
        raise ValueError("unknown selected row")
    selected = [row for row in rows if "pending" not in row
                and (not args.segment or row["id"] in args.segment)]
    for row in selected:
        if "checkpoint" not in row and "checkpoint_sha256" not in row:
            continue
        if (not isinstance(row.get("checkpoint"), str)
                or not re.fullmatch(r"[A-Za-z0-9_-]+\.smvchk", row["checkpoint"])
                or not isinstance(row.get("checkpoint_sha256"), str)
                or not re.fullmatch(r"[0-9a-f]{64}", row["checkpoint_sha256"])
                or "seed" in row or "seed_sha256" in row):
            raise ValueError("invalid checkpoint name/hash pair or combined seed: " + row["id"])
    return selected


def regular_path(root, name, required=True):
    path = root / name
    if not path.resolve().is_relative_to(root):
        raise ValueError("corpus path escapes checkout")
    if any(part.is_symlink() for part in (path, *path.parents) if part != root):
        raise ValueError("corpus path contains symlink")
    if (required or path.exists()) and not path.is_file():
        raise ValueError("corpus path is not a regular file")
    return path


def runtime_seed_indexes(rows):
    indexes = {}
    for row in rows:
        if "seed" not in row:
            continue
        match = re.fullmatch(r"([A-Za-z0-9-]+):([0-9]+)", row["seed"])
        if not match or not 1 <= int(match[2]) <= 9999:
            raise ValueError("unsupported runtime seed index: " + row["id"])
        indexes.setdefault(match[1], set()).add(int(match[2]))
    return indexes


def runtime_seed_files(root, seed_indexes):
    for capture, indexes in seed_indexes.items():
        prefix = f"target/seeds/{capture}/"
        with (root / prefix / "rooms.csv").open(newline="") as stream:
            entries = list(csv.DictReader(stream))
        for index in indexes:
            matches = [entry for entry in entries
                       if entry["room_index"] == str(index) and entry["region"] == "wram"]
            if len(matches) != 1:
                raise ValueError("seed index requires exactly one captured WRAM entry")
            filename = matches[0]["file"]
            if not re.fullmatch(rf"{index:04d}-[0-9A-F]{{4}}-wram\.bin", filename):
                raise ValueError("unsupported indexed runtime seed filename")
            yield {prefix + filename.replace("-wram.bin", suffix) for suffix in (
                "-wram.bin", "-sram.bin", "-vram.bin", "-cgram.bin",
                "-oam.bin", "-apuram.bin", "-registers.csv")}


def pack(args):
    rows = selected_rows(args)
    files = {}
    runtime_only = getattr(args, "runtime_only", False)
    if runtime_only:
        seed_indexes = runtime_seed_indexes(rows)
        for capture in seed_indexes:
            name = f"target/seeds/{capture}/rooms.csv"
            files[name] = regular_path(args.root, name)
        for expected in runtime_seed_files(args.root, seed_indexes):
            for name in sorted(expected):
                files[name] = regular_path(args.root, name)
        for row in rows:
            if "seed" not in row:
                continue
            capture, index = row["seed"].split(":")
            prefix = f"target/seeds/{capture}/{int(index):04d}-"
            seed = next(path for name, path in files.items()
                        if name.startswith(prefix) and name.endswith("-wram.bin"))
            sha = hashlib.sha256()
            for region, size in (("wram", 131072), ("sram", 8192), ("vram", 65536),
                                 ("cgram", 512), ("oam", 544), ("apuram", 65536)):
                data = seed.with_name(seed.name.replace("-wram.bin", f"-{region}.bin")).read_bytes()
                if len(data) != size:
                    raise ValueError(f"seed {region} requires {size} bytes")
                sha.update(data)
            sha.update(seed.with_name(seed.name.replace("-wram.bin", "-registers.csv")).read_bytes())
            if sha.hexdigest() != row.get("seed_sha256"):
                raise ValueError("seed payload hash mismatch: " + row["id"])
    else:
        captures = {row["seed"].split(":")[0] for row in rows if "seed" in row}
        for capture in sorted(captures):
            directory = args.root / "target/seeds" / capture
            for path in sorted(directory.iterdir()):
                name = path.relative_to(args.root).as_posix()
                if valid_name(name):
                    if not path.is_file() or path.is_symlink():
                        raise ValueError("seed capture contains nonregular file")
                    files[name] = path
            logs = (("snes9x.json", "snes9x.frames.csv", "snes9x.events.csv")
                    if f"target/seeds/{capture}/snes9x.json" in files
                    else ("lag.csv", "lag.csv.polls.csv", "lag.csv.events.csv"))
            for name in logs:
                if f"target/seeds/{capture}/{name}" not in files:
                    raise ValueError("capture is missing original logs")
    for row in rows:
        if "checkpoint" in row:
            name = "target/checkpoints/" + row["checkpoint"]
            path = regular_path(args.root, name)
            if digest(path) != row["checkpoint_sha256"]:
                raise ValueError("checkpoint hash mismatch: " + row["id"])
            files[name] = path
        if not runtime_only and row["inputs"].startswith("corpus-") and "checkpoint" not in row:
            continue
        name = "movies/" + row["inputs"]
        if not valid_name(name):
            raise ValueError("invalid corpus input name")
        path = args.root / name
        if "checkpoint" in row or runtime_only:
            path = regular_path(args.root, name, required="checkpoint" in row)
        # The original options prefix is exactly the first 8,250 supported words.
        if not path.exists() and row["inputs"] == "any-wait-prefix.inputs":
            data = b"".join((args.root / "movies/any-door-prefix.inputs").read_bytes().splitlines(True)[:8250])
        else:
            data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != row["input_sha256"]:
            raise ValueError("corpus input hash mismatch: " + row["id"])
        files[name] = data
    command, env = cipher(args)
    with args.bundle.open("xb") as output:
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=output, env=env)
        try:
            with tarfile.open(fileobj=process.stdin, mode="w|gz") as archive:
                for name, source in sorted(files.items()):
                    info = tarfile.TarInfo(name)
                    info.mode = 0o600
                    info.size = len(source) if isinstance(source, bytes) else source.stat().st_size
                    stream = io.BytesIO(source) if isinstance(source, bytes) else source.open("rb")
                    with stream:
                        archive.addfile(info, stream)
            process.stdin.close()
            if process.wait() != 0:
                raise RuntimeError("seed encryption failed")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
    print(f"Encrypted {len(files)} files; ciphertext SHA256 {digest(args.bundle)}")


def restore(args):
    if not args.sha256 or digest(args.bundle) != args.sha256:
        raise ValueError("encrypted seed bundle SHA256 mismatch")
    rows = selected_rows(args)
    runtime_only = getattr(args, "runtime_only", False)
    inputs = {"movies/" + row["inputs"] for row in rows
              if runtime_only or not row["inputs"].startswith("corpus-") or "checkpoint" in row}
    checkpoints = {"target/checkpoints/" + row["checkpoint"]
                   for row in rows if "checkpoint" in row}
    if any(not valid_name(name) for name in inputs):
        raise ValueError("invalid corpus input name")
    captures = {row["seed"].split(":")[0] for row in rows if "seed" in row}
    needed_indexes = {f"target/seeds/{capture}/rooms.csv" for capture in captures}
    seed_indexes = runtime_seed_indexes(rows) if runtime_only else {}
    command, env = cipher(args, decrypt=True)
    process = subprocess.Popen(command, stdout=subprocess.PIPE, env=env)
    seen, restored = set(), set()
    try:
        with tarfile.open(fileobj=process.stdout, mode="r|gz") as archive:
            for member in archive:
                name = member.name
                if not valid_name(name) or not member.isfile() or name in seen:
                    raise ValueError("invalid or duplicate seed archive member")
                seen.add(name)
                parts = PurePosixPath(name).parts
                if name not in inputs | checkpoints and not (
                        parts[:2] == ("target", "seeds") and parts[2] in captures):
                    continue
                if runtime_only and parts[:2] == ("target", "seeds"):
                    filename = parts[3]
                    if filename != "rooms.csv" and not (
                            re.match(r"[0-9]{4}-", filename)
                            and int(filename[:4]) in seed_indexes[parts[2]]):
                        continue
                regular_path(args.root, name, required=False)
                destination = (args.root / name).resolve()
                if not destination.is_relative_to(args.root):
                    raise ValueError("seed extraction escapes checkout")
                destination.parent.mkdir(parents=True, exist_ok=True)
                with archive.extractfile(member) as source:
                    if destination.exists():
                        if digest(destination) != hashlib.file_digest(source, "sha256").hexdigest():
                            raise ValueError("refusing to replace different existing seed data")
                    else:
                        with destination.open("xb") as output:
                            shutil.copyfileobj(source, output, 1024 * 1024)
                restored.add(name)
        # Consume the cipher trailer before checking OpenSSL's exit status.
        while process.stdout.read(1024 * 1024):
            pass
        if process.wait() != 0:
            raise RuntimeError("seed decryption failed")
        if (inputs | checkpoints | needed_indexes) - restored:
            raise ValueError("seed archive lacks selected corpus data")
        if runtime_only:
            for expected in runtime_seed_files(args.root, seed_indexes):
                if expected - restored:
                    raise ValueError("seed archive lacks selected runtime companions")
        for row in rows:
            name = "movies/" + row["inputs"]
            if name in inputs and digest(args.root / name) != row["input_sha256"]:
                raise ValueError("restored corpus input hash mismatch: " + row["id"])
            if "checkpoint" in row:
                path = args.root / "target/checkpoints" / row["checkpoint"]
                if digest(path) != row["checkpoint_sha256"]:
                    raise ValueError("restored checkpoint hash mismatch: " + row["id"])
    finally:
        process.stdout.close()
        if process.poll() is None:
            process.kill()
            process.wait()
    print(f"Restored {len(restored)} private corpus files; replay retains recipe hash checks")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("pack", "restore"))
    parser.add_argument("bundle", type=Path)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--rom", type=Path, default=Path("rom/sm.smc"))
    parser.add_argument("--openssl", default=shutil.which("openssl") or (
        str(Path(os.environ.get("ProgramFiles", "C:/Program Files")) / "Git/mingw64/bin/openssl.exe")
        if os.name == "nt" else "openssl"))
    parser.add_argument("--sha256", help="required pinned ciphertext digest for restoration")
    parser.add_argument("--segment", action="append", default=[])
    parser.add_argument("--runtime-only", action="store_true",
                        help="pack or restore only replay inputs, checkpoints and selected seed companions")
    args = parser.parse_args()
    args.root = args.root.resolve()
    (pack if args.mode == "pack" else restore)(args)


if __name__ == "__main__":
    main()
