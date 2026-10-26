#!/usr/bin/env python3
"""Build the verifier DLL (Windows) or shared object (Linux), then run a replay or corpus via ctypes."""
import argparse
import ctypes
import hashlib
import json
import os
import runpy
from pathlib import Path
import shutil
import subprocess
import sys
import time

# Windows remains the pinned verifier host; Linux builds the same cdylib as a .so.
WINDOWS = os.name == "nt"
HOST_LIBRARY = "sm_verify.dll" if WINDOWS else "libsm_verify.so"
HOST_TRIPLE = "x86_64-pc-windows-gnu" if WINDOWS else "x86_64-unknown-linux-gnu"


def source_hash(root):
    """Bind cached replays to source contents, including added/deleted modules."""
    paths = set((root / "sm").rglob("*.rs"))
    paths.update((root / "sm" / "src").rglob("*.c"))
    paths.update((root / "sm" / "src").rglob("*.h"))
    paths.add(root / "tools/install_native_music.py")
    paths.add(root / "tools/install_native_config.py")
    paths.add(root / "tools/install_native_desktop.py")
    paths.update(root / name for name in (
        "Cargo.toml", "Cargo.lock", "sm/Cargo.toml", "rust-toolchain.toml",
        "tools/verify_dll.py"))
    paths.update(path for path in (root / ".cargo").glob("*") if path.is_file())
    digest = hashlib.sha256()
    for path in sorted(paths):
        data = path.read_bytes()
        digest.update(path.relative_to(root).as_posix().encode("utf-8") + b"\0")
        digest.update(len(data).to_bytes(8, "little"))
        digest.update(data)
    return digest.hexdigest()


def check_source(host, expected):
    if not hasattr(host, "sm_verify_source_hash"):
        raise RuntimeError("cached DLL has no source stamp; run without --no-build")
    host.sm_verify_source_hash.argtypes = []
    host.sm_verify_source_hash.restype = ctypes.c_void_p
    stamp = host.sm_verify_source_hash()
    if not stamp or ctypes.string_at(stamp, 64) != expected.encode("ascii"):
        raise RuntimeError("cached DLL does not match current source; run without --no-build")


def load_host(dll):
    # Keep SDK search directories alive for as long as this DLL is loaded.
    directories = []
    for path in os.environ.get("LIBRARY_PATH", "").split(os.pathsep) if WINDOWS else ():
        if path:
            for directory in (Path(path), Path(path).parent / "bin"):
                if directory.is_dir():
                    directories.append(os.add_dll_directory(str(directory.resolve())))
    host = ctypes.CDLL(str(dll))
    host._sdk_directories = directories
    return host


def check_library(host, root):
    """Compare the rlib's compiled source bytes, not the host's environment."""
    if not hasattr(host, "sm_verify_source_file"):
        raise RuntimeError("cached DLL has no game-library source data; rebuild")

    class SourceFile(ctypes.Structure):
        _fields_ = [("path", ctypes.c_void_p), ("path_len", ctypes.c_size_t),
                    ("data", ctypes.c_void_p), ("data_len", ctypes.c_size_t)]

    host.sm_verify_source_file.argtypes = [ctypes.c_size_t]
    host.sm_verify_source_file.restype = SourceFile
    expected = {path.relative_to(root).as_posix()
                for path in (root / "sm" / "src").rglob("*")
                if path.suffix in (".rs", ".c", ".h")}
    expected.difference_update(("sm/src/main.rs", "sm/src/verify.rs",
                                "sm/src/verify_dll.rs", "sm/src/verify/checkpoint.rs",
                                "sm/src/verify/seed.rs"))
    for index in range(len(expected) + 1):
        source = host.sm_verify_source_file(index)
        if source.path_len == 0:
            break
        path = ctypes.string_at(source.path, source.path_len).decode("utf-8")
        if path not in expected:
            raise RuntimeError(f"unexpected or repeated game-library source: {path}; rebuild")
        expected.remove(path)
        if ctypes.string_at(source.data, source.data_len) != (root / path).read_bytes():
            raise RuntimeError(f"stale game-library source: {path}; rebuild")
    if expected:
        raise RuntimeError("game-library source list incomplete; update sm/src/verify_source.rs: "
                           + ", ".join(sorted(expected)))
    if not hasattr(host, "sm_verify_backend_source_hash"):
        raise RuntimeError("cached DLL has no backend source stamp; rebuild")
    host.sm_verify_backend_source_hash.restype = ctypes.c_char_p
    bridge = root / "sm/src/snes/backend.c"
    if host.sm_verify_backend_source_hash() != hashlib.sha256(bridge.read_bytes()).hexdigest().encode():
        raise RuntimeError("stale native backend; rerun tools/install_lakesnes_backend.ps1 and rebuild")

    native_hash = runpy.run_path(str(root / "tools/install_native_music.py"))["native_hash"]
    if not hasattr(host, "sm_verify_music_source_hash"):
        raise RuntimeError("cached DLL has no music source stamp; rebuild")
    host.sm_verify_music_source_hash.restype = ctypes.c_char_p
    if host.sm_verify_music_source_hash() != native_hash(root).encode():
        raise RuntimeError("stale native music; rerun tools/install_native_music.py and rebuild")

    if not hasattr(host, "sm_verify_util_source_matches"):
        raise RuntimeError("cached DLL has no utility source stamp; rebuild")
    host.sm_verify_util_source_matches.restype = ctypes.c_bool
    if not host.sm_verify_util_source_matches():
        raise RuntimeError("stale native utilities; rerun tools/install_native_config.py and rebuild")


def cargo_host(root, rustc, output, source, frontend=False, retry=True):
    cargo = shutil.which("cargo") or str(Path(rustc).with_name("cargo.exe"))
    # Backdated files in another checkout can look fresh in a shared target
    # directory. Keep this game's and host's artifacts with their source.
    # The launcher requests the full verification configuration for both the
    # library and example. Ordinary cargo builds keep the non-verifier profile.
    command = [cargo, "build", "-p", "sm", "--release", "--example", "sm_verify",
               "--locked", "-j", "1", "--target-dir", str(root / "target"),
               "--message-format=json"]
    if frontend:
        command.extend(["--lib", "--bin", "sm"])
    environment = dict(os.environ, RUSTC=rustc, SM_VERIFY_SOURCE_HASH=source)
    # CARGO_ENCODED_RUSTFLAGS has Cargo's highest environment precedence. If it
    # is absent, preserve RUSTFLAGS using Cargo's whitespace-separated format.
    flags = environment.get("CARGO_ENCODED_RUSTFLAGS")
    if flags is None:
        flags = "\x1f".join(environment.get("RUSTFLAGS", "").split())
    environment["CARGO_ENCODED_RUSTFLAGS"] = (
        flags + ("\x1f" if flags else "") + "--cfg\x1fsm_verify_dll"
    )
    dll = None
    compiled = []
    log_path = output / "build.log"
    with log_path.open("w", encoding="utf-8") as log:
        process = subprocess.Popen(
            command, cwd=root,
            env=environment,
            text=True, encoding="utf-8", stdout=subprocess.PIPE, stderr=log,
        )
        try:
            for line in process.stdout:
                log.write(line)
                message = json.loads(line)
                if message["reason"] == "compiler-message":
                    if message["message"]["level"] in ("error", "failure-note"):
                        print(message["message"]["rendered"] or "", end="", file=sys.stderr)
                elif message["reason"] == "compiler-artifact":
                    if not message["fresh"]:
                        compiled.append(message["target"]["name"])
                    if message["target"]["name"] == "sm_verify":
                        dll = next((Path(name) for name in message["filenames"]
                                    if name.endswith(Path(HOST_LIBRARY).suffix)), None)
            result = process.wait()
        finally:
            process.stdout.close()
            if process.poll() is None:
                process.terminate()
                process.wait()
    print(f"Cargo compiled: {', '.join(compiled) or 'none (reused)'}; log {log_path}",
          file=sys.stderr, flush=True)
    if result:
        raise subprocess.CalledProcessError(result, command)
    if dll is None:
        raise RuntimeError("Cargo did not report the verifier DLL")
    # Inspect in a separate process so a stale DLL is unloaded before rebuilding.
    checked = subprocess.run(
        [sys.executable, str(root / "tools/verify_dll.py"), "--check-library", str(dll)],
        cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
    )
    if checked.returncode:
        failure = checked.stderr.strip()
        # Native installation/source-list errors cannot be repaired by another
        # Rust build. Retry only Cargo's backdated game-source freshness failure.
        if not retry or "stale game-library source:" not in failure:
            raise RuntimeError(checked.stderr.strip() or "unable to inspect game library")
        print(checked.stderr.strip(), file=sys.stderr)
        print("Rebuilding sm after game-library source check failed", file=sys.stderr)
        # Invalidate Cargo's timestamp check without deleting artifacts or
        # changing source contents. The crate root is always a library input.
        # This also works with pre-existing/seeded targets lacking CACHEDIR.TAG.
        (root / "sm/src/lib.rs").touch()
        return cargo_host(root, rustc, output, source, frontend, retry=False)
    # Preserve --no-build's existing path. An unchanged DLL can still be loaded
    # by another replay; do not overwrite it merely to refresh its timestamp.
    cached = output / HOST_LIBRARY
    if not cached.exists() or cached.read_bytes() != dll.read_bytes():
        shutil.copyfile(dll, cached)
    return cached


def build(root, rustc, use_cargo=None, frontend=False):
    source = source_hash(root)
    output = root / "target" / "verify-dll"
    output.mkdir(parents=True, exist_ok=True)
    library = output / "libsm.rlib"
    dll = output / HOST_LIBRARY
    sdk = [Path(path) for path in os.environ.get("LIBRARY_PATH", "").split(os.pathsep)
           if path]
    compiler = subprocess.check_output([rustc, "-vV"], cwd=root, text=True)
    host = next(line.removeprefix("host: ") for line in compiler.splitlines()
                if line.startswith("host: "))
    if host != HOST_TRIPLE:
        raise RuntimeError(f"verifier host requires the pinned {HOST_TRIPLE} toolchain")
    if use_cargo is None:
        use_cargo = any((path / "libSDL2.dll.a").is_file() for path in sdk)
    if frontend and not use_cargo:
        raise RuntimeError("--frontend-check requires the Cargo/SDL2 build path")
    if use_cargo:
        dll = cargo_host(root, rustc, output, source, frontend)
        if source_hash(root) != source:
            raise RuntimeError("source changed during build; rebuild before replay")
        return dll
    incremental = os.environ.get("CARGO_INCREMENTAL") != "0"
    # Retain 16 codegen units; incremental compilation otherwise defaults to 256.
    common = [rustc, "--edition=2024", "-C", "opt-level=3",
              "-C", "codegen-units=16", "-A", "warnings"]
    common += [arg for path in os.environ.get("LIBRARY_PATH", "").split(os.pathsep)
               if path for arg in ("-L", f"native={path}")]
    sysroot = Path(subprocess.check_output(
        [rustc, "--print", "sysroot"], cwd=root, text=True).strip())
    # The bundled GNU linker cannot read bigobj emitted for the large game;
    # Rust's bundled lld handles those object files without external tools.
    linker = sysroot / "lib" / "rustlib" / host / "bin" / "gcc-ld" / "ld.lld.exe"
    link = ["-C", f"linker={linker}", "-C", "linker-flavor=ld"] if WINDOWS else []
    # Direct-rustc builds have no Cargo freshness check. Bind reuse to source,
    # compiler/options, native archives and output bytes; inspect in a child so
    # the DLL is unloaded before any rebuild.
    stamp = output / "direct-build.json"
    native = sorted({path.resolve() for directory in sdk for path in directory.glob("*.a")})
    def build_identity():
        return hashlib.sha256(json.dumps([
            source, common, compiler, incremental,
            [(str(path), hashlib.sha256(path.read_bytes()).hexdigest()) for path in native],
            [(name, os.environ.get(name, "")) for name in
             ("PATH", "LIBRARY_PATH", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")],
        ]).encode()).hexdigest()

    identity = build_identity()
    if stamp.exists() and dll.exists() and library.exists():
        try:
            saved = json.loads(stamp.read_text())
            if saved == {"inputs": identity, "dll": hashlib.sha256(dll.read_bytes()).hexdigest(),
                         "library": hashlib.sha256(library.read_bytes()).hexdigest()}:
                checked = subprocess.run(
                    [sys.executable, str(root / "tools/verify_dll.py"), "--check-library", str(dll)],
                    cwd=root, capture_output=True, text=True,
                )
                if checked.returncode == 0:
                    print("Direct compiler: none (reused)", file=sys.stderr, flush=True)
                    return dll
        except (OSError, ValueError):
            pass
    stamp.unlink(missing_ok=True)
    subprocess.run(
        common + (["-C", f"incremental={output / 'incremental-sm'}"] if incremental else [])
        + ["--crate-name", "sm", "--crate-type", "rlib", "--cfg", "sm_verify_dll",
                  "sm/src/lib.rs", "-o", str(library)],
        cwd=root, check=True,
    )
    subprocess.run(
        common + (["-C", f"incremental={output / 'incremental-host'}"] if incremental else [])
        + ["--crate-name", "sm_verify", "--crate-type", "cdylib",
                  "--cfg", "sm_verify_dll",
                  *link,
                  "sm/src/verify_dll.rs",
                  "--extern", f"sm={library}", "-o", str(dll)],
        cwd=root, env=dict(os.environ, SM_VERIFY_SOURCE_HASH=source), check=True,
    )
    if source_hash(root) != source:
        raise RuntimeError("source changed during build; rebuild before replay")
    if build_identity() != identity:
        raise RuntimeError("native inputs changed during build; rebuild before replay")
    stamp.write_text(json.dumps({"inputs": identity,
                                "dll": hashlib.sha256(dll.read_bytes()).hexdigest(),
                                "library": hashlib.sha256(library.read_bytes()).hexdigest()}))
    return dll



def corpus_child(command, root):
    """Stream existing verifier output and recognize only pre-replay cache rejection."""
    errors, replayed = [], False
    process = subprocess.Popen(command, cwd=root, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, text=True, encoding="utf-8",
                               errors="replace")
    try:
        for line in process.stdout:
            print(line, end="", flush=True)
            replayed |= line.startswith("Verified ") or line.startswith("Checkpoint segment:")
            if line.startswith("verify: "):
                errors.append(line.strip().removeprefix("verify: "))
        result = process.wait()
    finally:
        process.stdout.close()
        if process.poll() is None:
            process.terminate()
            process.wait()
    rejected = {
        "not a verifier checkpoint", "checkpoint frame overflow",
        "checkpoint runtime changed; regenerate from boot", "checkpoint ROM or seed differs",
        "checkpoint input prefix differs", "checkpoint audio backend differs",
        "checkpoint payload is corrupt", "checkpoint has extra state bytes",
        "checkpoint state did not round-trip", "truncated checkpoint",
        "invalid checkpoint boolean", "checkpoint log length overflow",
        "truncated checkpoint log",
    }
    return result, result == 2 and not replayed and len(errors) == 1 and errors[0] in rejected


def run_corpus(root, args):
    import csv
    import tomllib
    import uuid

    recipe_path = args.corpus.resolve()
    with recipe_path.open("rb") as file:
        recipe = tomllib.load(file)
    if (set(recipe) - {"version", "required", "segment"}
            or type(recipe.get("version")) is not int or recipe["version"] != 1):
        raise ValueError("corpus requires version=1 and only required/segment tables")

    def strings(value, field):
        if (not isinstance(value, list) or any(not isinstance(x, str) or not x for x in value)
                or len(set(value)) != len(value)):
            raise ValueError(f"corpus {field} must be a list of distinct nonempty strings")
        return value

    def integer(value, field, minimum):
        if type(value) is not int or not minimum <= value <= 2**64 - 1:
            raise ValueError(f"corpus {field} must be an integer in {minimum}..u64::MAX")
        return value

    required = strings(recipe.get("required"), "required")
    if not required or not isinstance(recipe.get("segment", []), list):
        raise ValueError("corpus requires nonempty required IDs and segment tables")
    segments = {}
    for segment in recipe.get("segment", []):
        allowed = {"id", "inputs", "input_sha256", "start_frame", "frames", "require_hit",
                   "after_hits", "expect", "after_expectations", "pending", "seed", "seed_sha256"}
        if not isinstance(segment, dict) or set(segment) - allowed:
            raise ValueError("corpus segment has unknown fields")
        name = segment.get("id")
        if not isinstance(name, str) or not name or name in segments:
            raise ValueError("corpus segment IDs must be distinct nonempty strings")
        if "pending" in segment:
            if not isinstance(segment["pending"], str) or not segment["pending"]:
                raise ValueError(f"corpus {name}: pending requires a reason")
        else:
            if not isinstance(segment.get("inputs"), str) or not segment["inputs"]:
                raise ValueError(f"corpus {name}: inputs path required")
            segment["inputs"] = (recipe_path.parent / segment["inputs"]).resolve()
            digest = segment.get("input_sha256", "")
            if (not isinstance(digest, str) or len(digest) != 64
                    or any(c not in "0123456789abcdef" for c in digest)):
                raise ValueError(f"corpus {name}: lowercase input_sha256 required")
            segment["start_frame"] = integer(segment.get("start_frame", 0), name + " start_frame", 0)
            if "seed" in segment or "seed_sha256" in segment:
                seed = segment.get("seed", "")
                digest = segment.get("seed_sha256", "")
                if (not isinstance(seed, str) or seed.count(":") != 1
                        or not seed.split(":")[0].replace("-", "").isalnum()
                        or not seed.split(":")[1].isdigit() or int(seed.split(":")[1]) < 1
                        or not isinstance(digest, str) or len(digest) != 64
                        or any(c not in "0123456789abcdef" for c in digest)):
                    raise ValueError(f"corpus {name}: seed requires capture:index and payload SHA-256")
            integer(segment.get("frames"), name + " frames", 1)
            for key, tail in (("require_hit", "after_hits"), ("expect", "after_expectations")):
                segment[key] = strings(segment.get(key, []), name + " " + key)
                if tail in segment:
                    integer(segment[tail], name + " " + tail, 0)
                    if not segment[key]:
                        raise ValueError(f"corpus {name}: {tail} requires {key}")
            if not segment["require_hit"] and not segment["expect"]:
                raise ValueError(f"corpus {name}: require_hit or expect must demonstrate coverage")
        segments[name] = segment
    selected = strings(args.segment, "selection") if args.segment else list(dict.fromkeys(required + list(segments)))
    for name in selected:
        segments.setdefault(name, {"id": name, "pending": "required or selected recipe is missing"})
    partial = bool(args.segment) or not set(required).issubset(selected)
    print(f"Corpus {'PARTIAL selection' if partial else 'full required scope'}: "
          f"{len(selected)} selected, {len(required)} required; {recipe_path}", flush=True)
    def input_matches(segment):
        if "seed" in segment:
            capture, index = segment["seed"].split(":")
            directory = recipe_path.parent.parent / "target/seeds" / capture
            with (directory / "rooms.csv").open(newline="") as file:
                entries = [row for row in csv.DictReader(file)
                           if row["room_index"] == str(int(index)) and row["region"] == "wram"]
            if len(entries) != 1:
                raise ValueError("seed index requires exactly one captured WRAM entry")
            filename = entries[0]["file"]
            if Path(filename).name != filename or not filename.endswith("-wram.bin"):
                raise ValueError("invalid indexed seed filename")
            seed = directory / filename
            digest = hashlib.sha256()
            for region, size in (("wram", 131072), ("sram", 8192), ("vram", 65536),
                                 ("cgram", 512), ("oam", 544), ("apuram", 65536)):
                data = seed.with_name(filename.replace("-wram.bin", f"-{region}.bin")).read_bytes()
                if len(data) != size:
                    raise ValueError(f"seed {region} requires {size} bytes")
                digest.update(data)
            digest.update(seed.with_name(filename.replace("-wram.bin", "-registers.csv")).read_bytes())
            if digest.hexdigest() != segment["seed_sha256"]:
                raise ValueError("seed payload SHA-256 does not match the recipe")
            segment["_seed_path"] = seed
        return hashlib.sha256(segment["inputs"].read_bytes()).hexdigest() == segment["input_sha256"]

    failures = set()
    runnable = []
    for name in selected:
        segment = segments[name]
        if "pending" in segment:
            print(f"FAIL {name}: pending - {segment['pending']}", flush=True)
            failures.add(name)
        else:
            try:
                if not input_matches(segment):
                    raise ValueError("input_sha256 does not match the recipe")
                runnable.append(segment)
            except (OSError, ValueError) as error:
                print(f"FAIL {name}: {error}", flush=True)
                failures.add(name)
    if not runnable:
        print(f"Corpus: 0/{len(selected)} selected passed; 0/{len(required)} required passed", flush=True)
        return 2
    rom = args.rom.resolve()
    rom_hash = hashlib.sha256(rom.read_bytes()).hexdigest()
    command = [sys.executable, str(root / "tools/verify_dll.py")]
    # Corpus mode chooses direct compilation unless Cargo is explicitly requested.
    build_command = command + ["--build-only", "--rustc", args.rustc,
                               "--cargo" if args.cargo else "--direct"]
    if args.no_build:
        build_command.append("--no-build")
    if args.frontend_check:
        build_command.append("--frontend-check")
    source = source_hash(root)
    result, _ = corpus_child(build_command, root)
    if result:
        for segment in runnable:
            print(f"FAIL {segment['id']}: build/source check failed", flush=True)
        return result
    cache = root / "target" / "verify-corpus"

    def cache_directory(segment):
        # One digest retains the full identity without three long Windows path components.
        identity = "\0".join((source, rom_hash, segment["input_sha256"],
                              segment.get("seed_sha256", "")))
        return cache / hashlib.sha256(identity.encode("ascii")).hexdigest()

    def checkpoint_frame(path):
        with path.open("rb") as file:
            header = file.read(56)
        return int.from_bytes(header[32:40], "little") if len(header) == 56 and header[:8] == b"SMVERIFY" else None

    prefix_inputs = {}
    donors = {(item["input_sha256"], item.get("seed_sha256")): item
              for item in segments.values() if "pending" not in item}

    def shares_prefix(segment, donor, frame):
        # Only the same initial state can share a verified input prefix.
        if segment.get("seed_sha256") != donor.get("seed_sha256"):
            return False
        # Compare recipe-bound canonical words; the DLL still validates the checkpoint.
        for item in (segment, donor):
            digest = item["input_sha256"]
            if digest not in prefix_inputs:
                try:
                    data = item["inputs"].read_bytes()
                except OSError:
                    return False
                if hashlib.sha256(data).hexdigest() != digest:
                    return False
                words = bytearray()
                for line in data.split(b"\n"):
                    word = line.strip()
                    if not word or word.startswith(b"#"):
                        continue
                    if len(word) != 4 or any(c not in b"0123456789abcdefABCDEF" for c in word):
                        return False
                    words.extend(word.lower())
                prefix_inputs[digest] = bytes(words)
        left, right = (prefix_inputs[item["input_sha256"]] for item in (segment, donor))
        end = frame * 4
        return end <= min(len(left), len(right)) and left[:end] == right[:end]

    def prefix(segment, force_boot=False):
        start = segment["start_frame"]
        directory = cache_directory(segment)
        directory.mkdir(parents=True, exist_ok=True)
        candidates = []
        if not force_boot:
            for donor in donors.values():
                for path in cache_directory(donor).glob("*.smvchk"):
                    frame = checkpoint_frame(path)
                    if (frame is not None and 0 < frame <= start
                            and shares_prefix(segment, donor, frame)):
                        candidates.append((frame, path))
            candidates.sort(key=lambda pair: (pair[0], pair[1].stat().st_mtime_ns), reverse=True)
        previous, checkpoint = candidates[0] if candidates else (0, None)
        if previous == start:
            return checkpoint
        # Retain useful branch starts while verifying this prefix, bounded by the recipe.
        stops = sorted({item["start_frame"] for item in segments.values()
                        if "pending" not in item and 0 < item["start_frame"] < start
                        and shares_prefix(segment, item, item["start_frame"])})
        regenerated = False
        while previous < start:
            end = next((frame for frame in stops if frame > previous), start)
            output = directory / f"{end}-{uuid.uuid4().hex}.smvchk"
            setup = command + [str(rom), str(segment["inputs"]), "--no-build",
                               "--frames", str(end - previous), "--checkpoint-out", str(output)]
            if "seed" in segment:
                setup += ["--seed", str(segment["_seed_path"])]
            if checkpoint:
                setup += ["--checkpoint-in", str(checkpoint)]
            print(f"SETUP {segment['id']}: input frames {previous}..{end}", flush=True)
            code, rejected = corpus_child(setup, root)
            if code == 0:
                if checkpoint_frame(output) != end:
                    raise RuntimeError("generated checkpoint has an unexpected input frame")
                previous, checkpoint = end, output
                continue
            if not checkpoint or not rejected or regenerated:
                return None
            print(f"Checkpoint rejected for {segment['id']}; regenerate prefix from its origin", flush=True)
            previous, checkpoint = 0, None
            regenerated = True
        return checkpoint

    # Earlier windows can supply checked prefixes to later branches of the same inputs.
    passed = set()
    for segment in sorted(runnable, key=lambda item: (item["start_frame"], str(item["inputs"]))):
        name = segment["id"]
        try:
            if source_hash(root) != source:
                raise RuntimeError("source changed during corpus; rebuild before continuing")
            if not input_matches(segment):
                raise ValueError("input_sha256 does not match the recipe")
            start = segment["start_frame"]
            checkpoint = prefix(segment) if start else None
            if start and checkpoint is None:
                raise RuntimeError("checkpoint setup failed")
            replay = command + [str(rom), str(segment["inputs"]), "--no-build",
                                "--frames", str(segment["frames"])]
            for key in ("require_hit", "expect"):
                for value in segment[key]:
                    replay += ["--" + key.replace("_", "-"), value]
            for key in ("after_hits", "after_expectations"):
                if key in segment:
                    replay += ["--" + key.replace("_", "-"), str(segment[key])]
            if "seed" in segment:
                replay += ["--seed", str(segment["_seed_path"])]
            if "seed" in segment and not start:
                print(f"SEGMENT {name}: seed {segment['seed']}, budget {segment['frames']} frames", flush=True)
                code, _ = corpus_child(replay, root)
                if not input_matches(segment):
                    raise RuntimeError("inputs changed during seeded corpus segment")
                if code:
                    failures.add(name)
                    print(f"FAIL {name}: verifier exit {code}", flush=True)
                else:
                    passed.add(name)
                    print(f"PASS {name}", flush=True)
                continue
            for attempt in range(2):
                directory = cache_directory(segment)
                directory.mkdir(parents=True, exist_ok=True)
                output = directory / f"segment-{uuid.uuid4().hex}.smvchk"
                print(f"SEGMENT {name}: start {start}, budget {segment['frames']} frames", flush=True)
                code, rejected = corpus_child(
                    replay + ["--checkpoint-out", str(output)]
                    + (["--checkpoint-in", str(checkpoint)] if checkpoint else []), root)
                if code == 0 or not checkpoint or not rejected or attempt:
                    break
                checkpoint = prefix(segment, force_boot=True)
                if checkpoint is None:
                    raise RuntimeError("checkpoint regeneration failed")
            if not input_matches(segment):
                raise RuntimeError("inputs changed during corpus segment")
            if code:
                failures.add(name)
                print(f"FAIL {name}: verifier exit {code}", flush=True)
            else:
                end = checkpoint_frame(output)
                if end is None or not start < end <= start + segment["frames"]:
                    raise RuntimeError("segment checkpoint has an unexpected input frame")
                passed.add(name)
                print(f"PASS {name}", flush=True)
        except (OSError, RuntimeError, ValueError) as error:
            failures.add(name)
            print(f"FAIL {name}: {error}", flush=True)
    print(f"Corpus {'PARTIAL' if partial else 'FULL'}: {len(passed)}/{len(selected)} selected passed; "
          f"{len(passed.intersection(required))}/{len(required)} required passed", flush=True)
    return 1 if failures else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rom", type=Path, nargs="?")
    parser.add_argument("inputs", type=Path, nargs="?")
    parser.add_argument("--build-only", action="store_true",
                        help="build and check compiled sources without starting a replay")
    parser.add_argument("--unit-tests", action="store_true",
                        help="run the existing verifier unit tests through the DLL, without a ROM")
    parser.add_argument("--frontend-check", action="store_true",
                        help="include release library/frontend checks in the same Cargo build")
    parser.add_argument("--frames", type=int)
    parser.add_argument("--seed", type=Path, help="import an original-emulator room seed")
    parser.add_argument("--checkpoint-in", type=Path,
                        help="resume a local verifier checkpoint; --frames counts from that point")
    parser.add_argument("--checkpoint-out", type=Path,
                        help="save a new local verifier checkpoint after the last frame")
    parser.add_argument("--require-hit", action="append", default=[], metavar="PROBE",
                        help="require an explicit routine/branch probe to execute (repeatable)")
    parser.add_argument("--expect", action="append", default=[], metavar="EXPECTATION",
                        help="require an observed gameplay condition (repeatable)")
    parser.add_argument("--after-expectations", type=int, metavar="N",
                        help="observe N additional frames after all expectations are met")
    parser.add_argument("--corpus", type=Path, help="run required replay segments from a TOML recipe")
    parser.add_argument("--segment", action="append", default=[], metavar="ID",
                        help="run selected corpus segments only; reports partial coverage")
    parser.add_argument("--after-hits", type=int, metavar="N",
                        help="stop N additional frames after all required probes first execute; "
                             "fail if the movie or --frames limit ends earlier")
    parser.add_argument("--no-build", action="store_true",
                        help="reuse target/verify-dll/sm_verify.dll if its source still matches")
    build_path = parser.add_mutually_exclusive_group()
    build_path.add_argument("--cargo", action="store_const", dest="cargo", const=True,
                            default=None, help="use Cargo release compilation (automatic with SDL2)")
    build_path.add_argument("--direct", action="store_const", dest="cargo", const=False,
                            help="use standalone rustc without SDL2; separate from Cargo artifacts")
    parser.add_argument("--rustc", default=shutil.which("rustc") or str(
        Path.home() / ".cargo" / "bin" / "rustc.exe"))
    args = parser.parse_args()
    if args.seed is not None and (args.unit_tests or args.build_only or args.corpus is not None):
        parser.error("--seed requires a direct replay")
    if args.unit_tests and (args.build_only or args.frontend_check or args.rom is not None
                           or args.inputs is not None or args.corpus is not None or args.segment
                           or args.frames is not None or args.checkpoint_in is not None
                           or args.checkpoint_out is not None or args.require_hit or args.expect
                           or args.after_hits is not None or args.after_expectations is not None):
        parser.error("--unit-tests cannot combine replay, build-only or frontend checks")
    if not (args.build_only or args.unit_tests) and (
            args.rom is None or (args.inputs is None and args.corpus is None)):
        parser.error("rom and inputs are required unless --build-only or --unit-tests is used")
    if args.unit_tests and args.cargo is None:
        args.cargo = False
    if args.frontend_check and args.no_build:
        parser.error("--frontend-check requires a build")
    if args.build_only and (args.frames is not None or args.checkpoint_in is not None
                           or args.checkpoint_out is not None or args.require_hit
                           or args.after_hits is not None or args.expect
                           or args.after_expectations is not None or args.corpus is not None):
        parser.error("--build-only cannot request replay frames, checkpoints or probes")
    if args.frames is not None and not 1 <= args.frames <= 2**64 - 1:
        parser.error("--frames must be a positive u64")
    if args.after_hits is not None:
        if not 0 <= args.after_hits <= 2**64 - 1:
            parser.error("--after-hits must be a nonnegative u64")
        if not args.require_hit:
            parser.error("--after-hits requires at least one --require-hit")
    if args.after_expectations is not None:
        if not 0 <= args.after_expectations <= 2**64 - 1 or not args.expect:
            parser.error("--after-expectations requires --expect and a nonnegative u64")
    if args.segment and args.corpus is None:
        parser.error("--segment requires --corpus")
    root = Path(__file__).resolve().parents[1]
    if args.corpus is not None:
        if (args.inputs is not None or args.frames is not None or args.checkpoint_in is not None
                or args.checkpoint_out is not None or args.require_hit or args.expect
                or args.after_hits is not None or args.after_expectations is not None):
            parser.error("--corpus takes inputs, frames, checkpoints and expectations from its recipe")
        return run_corpus(root, args)
    started = time.perf_counter()
    source = source_hash(root)
    # Paths are resolved before changing cwd so invocation works from anywhere.
    rom = args.rom.resolve() if args.rom else None
    inputs = args.inputs.resolve() if args.inputs else None
    replay_args = ["--unit-tests"] if args.unit_tests else [str(rom), str(inputs)]
    if args.frames is not None:
        replay_args.extend(["--frames", str(args.frames)])
    for probe in args.require_hit:
        replay_args.extend(["--require-hit", probe])
    if args.after_hits is not None:
        replay_args.extend(["--after-hits", str(args.after_hits)])
    for expectation in args.expect:
        replay_args.extend(["--expect", expectation])
    if args.after_expectations is not None:
        replay_args.extend(["--after-expectations", str(args.after_expectations)])
    for option, path in (("--checkpoint-in", args.checkpoint_in),
                         ("--checkpoint-out", args.checkpoint_out), ("--seed", args.seed)):
        if path is not None:
            replay_args.extend([option, str(path.resolve())])
    dll = root / "target" / "verify-dll" / HOST_LIBRARY
    if not args.no_build:
        dll = build(root, args.rustc, args.cargo, args.frontend_check)
    os.chdir(root)
    host = load_host(dll)
    check_source(host, source)
    check_library(host, root)
    if source_hash(root) != source:
        raise RuntimeError("source changed before replay; rebuild before replay")
    print(f"Verifier build/check: {time.perf_counter() - started:.2f}s; source {source}",
          file=sys.stderr, flush=True)
    if args.build_only:
        return 0
    if (args.unit_tests or args.seed is not None or args.checkpoint_in is not None or args.checkpoint_out is not None
            or args.require_hit or args.expect):
        if not hasattr(host, "sm_verify_args"):
            raise RuntimeError("cached DLL predates segment options; run without --no-build")
        host.sm_verify_args.argtypes = [ctypes.c_uint32, ctypes.POINTER(ctypes.c_char_p)]
        host.sm_verify_args.restype = ctypes.c_int
        argv = (ctypes.c_char_p * len(replay_args))(
            *(arg.encode("utf-8") for arg in replay_args))
        result = host.sm_verify_args(len(replay_args), argv)
    else:
        host.sm_verify.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint64]
        host.sm_verify.restype = ctypes.c_int
        result = host.sm_verify(str(rom).encode("utf-8"), str(inputs).encode("utf-8"),
                                args.frames or 0)
    if source_hash(root) != source:
        raise RuntimeError("source changed during replay; result applies only to the printed source")
    print(f"Verifier total: {time.perf_counter() - started:.2f}s", file=sys.stderr)
    return result


if __name__ == "__main__":
    try:
        if len(sys.argv) == 3 and sys.argv[1] == "--check-library":
            check_library(load_host(sys.argv[2]), Path(__file__).resolve().parents[1])
            sys.exit(0)
        sys.exit(main())
    except (OSError, RuntimeError, ValueError, subprocess.CalledProcessError) as error:
        print(f"verify: {error}", file=sys.stderr)
        sys.exit(2)
