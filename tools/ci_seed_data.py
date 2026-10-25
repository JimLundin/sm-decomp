#!/usr/bin/env python3
"""Pack or restore ROM-encrypted corpus inputs and original-emulator seeds."""
import argparse
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
        r"target/seeds/[A-Za-z0-9-]+/(?:rooms\.csv|lag\.csv(?:\.polls\.csv|\.events\.csv)?|"
        r"[0-9]{4}-[0-9A-F]{4}-(?:wram|sram|vram|cgram|oam|apuram)\.bin|"
        r"[0-9]{4}-[0-9A-F]{4}-registers\.csv)", name))


def selected_rows(args):
    rows = tomllib.loads((args.root / "movies/segments.toml").read_text())["segment"]
    known = {row["id"] for row in rows}
    if set(args.segment) - known:
        raise ValueError("unknown selected row")
    return [row for row in rows if "pending" not in row
            and (not args.segment or row["id"] in args.segment)]


def pack(args):
    rows = selected_rows(args)
    captures = {row["seed"].split(":")[0] for row in rows if "seed" in row}
    files = {}
    for capture in sorted(captures):
        directory = args.root / "target/seeds" / capture
        for path in sorted(directory.iterdir()):
            name = path.relative_to(args.root).as_posix()
            if valid_name(name):
                if not path.is_file() or path.is_symlink():
                    raise ValueError("seed capture contains nonregular file")
                files[name] = path
        for suffix in ("", ".polls.csv", ".events.csv"):
            if f"target/seeds/{capture}/lag.csv{suffix}" not in files:
                raise ValueError("capture is missing original logs")
    for row in rows:
        if row["inputs"].startswith("corpus-"):
            continue
        name = "movies/" + row["inputs"]
        if not valid_name(name):
            raise ValueError("invalid corpus input name")
        path = args.root / name
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
    inputs = {"movies/" + row["inputs"] for row in rows if not row["inputs"].startswith("corpus-")}
    captures = {row["seed"].split(":")[0] for row in rows if "seed" in row}
    needed_indexes = {f"target/seeds/{capture}/rooms.csv" for capture in captures}
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
                if name not in inputs and not (parts[:2] == ("target", "seeds") and parts[2] in captures):
                    continue
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
        if (inputs | needed_indexes) - restored:
            raise ValueError("seed archive lacks selected corpus data")
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
    args = parser.parse_args()
    args.root = args.root.resolve()
    (pack if args.mode == "pack" else restore)(args)


if __name__ == "__main__":
    main()
