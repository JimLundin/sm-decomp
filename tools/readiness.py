#!/usr/bin/env python3
"""Immutable, identity-bound evidence for recompilation readiness commands.

Run is the shared lifecycle for orchestration. A run without a terminal verdict
is incomplete; a scoped pass alone never certifies readiness.
"""

import datetime
import argparse
import contextlib
import fcntl
import hashlib
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
import uuid
import zipfile
from pathlib import Path


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def digest(path):
    result = hashlib.sha256()
    with Path(path).open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def inventory_digest(files):
    return hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def maintained_paths(root):
    """Tracked maintained files plus relevant untracked/ignored source/config."""
    tracked = subprocess.run(["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True)
    names = set(tracked.stdout.decode().split("\0")) - {""}
    source_suffixes = {".rs", ".py", ".toml", ".json", ".patch", ".lua", ".sh", ".yml", ".yaml"}
    for directory in ("crates", "tools", "config", ".cargo", ".github"):
        for path in (root / directory).rglob("*"):
            if path.is_file() and (path.suffix in source_suffixes or path.name in ("Makefile", "justfile")):
                names.add(path.relative_to(root).as_posix())
    for name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml", "justfile",
                 "movies/manifest.json"):
        if (root / name).is_file():
            names.add(name)
    excluded_roots = {".git", ".scratch", ".agents", ".codex", ".aws", "deps", "rom", "out", "target"}
    for name in sorted(names):
        path = Path(name)
        if path.parts[0] in excluded_roots or any(part in ("target", "generated", "__pycache__") for part in path.parts):
            continue
        if ((path.parts[0] == "movies" and name != "movies/manifest.json")
                or path.name.startswith((".env", "credentials", "secrets"))):
            continue
        if path.suffix.lower() in (".pem", ".key", ".p12", ".sfc", ".smc", ".lsmv", ".inputs") or path.name in ("credentials", "id_rsa", "id_ed25519"):
            continue
        if (root / path).is_file():
            if (root / path).is_symlink():
                raise RuntimeError("Maintained input is a symlink; review before snapshot: " + name)
            yield path


def source_inventory(root):
    return [{"path": path.as_posix(), "sha256": digest(root / path),
             "mode": (root / path).stat().st_mode & 0o777}
            for path in maintained_paths(root)]


def verify_movie_inputs(run, entry):
    """Independently parse the clean-start container and compare all available inputs."""
    rom = run.root / "rom/sm.sfc"
    movie = run.root / "movies" / entry["file"]
    if not rom.is_file():
        raise RuntimeError("Missing ROM: place the unheadered Japan/USA Super Metroid ROM at rom/sm.sfc")
    rom_hash = run.guard_input(rom)
    if rom_hash != entry["rom_sha256"]:
        raise RuntimeError("ROM checksum mismatch: supply the verified unheadered Japan/USA ROM at rom/sm.sfc")
    if not movie.is_file():
        raise RuntimeError("Missing original any% movie: run just fetch-movies")
    movie_hash = run.guard_input(movie)
    if movie_hash != entry["sha256"]:
        raise RuntimeError("Original movie checksum mismatch: run just fetch-movies")
    with zipfile.ZipFile(movie) as archive:
        names = archive.namelist()
        if len(names) != len(set(names)):
            raise RuntimeError("Movie container has duplicate members")
        for name in names:
            if name in ("savestate", "savestate.anchor", "movie.bin") or name.startswith(
                    ("moviesram.", "ramcontent.", "setting.")) or (name.startswith("port") and name[4:].isdigit()):
                raise RuntimeError("Unsupported movie initialization: " + name)
        expected_members = {"gametype": "snes_ntsc", "controlsversion": "0",
                            "coreversion": "bsnes v085 (Compatibility core)", "rom.sha256": rom_hash}
        for name, expected in expected_members.items():
            if archive.read(name).decode().strip() != expected:
                raise RuntimeError("Movie container identity mismatch: " + name)
        raw_inputs = archive.read("input")
    buttons = "BYsSudlrAXLR"
    pads = []
    for number, line in enumerate(raw_inputs.decode().splitlines(), 1):
        header, separator, pad = line.partition("|")
        if not separator or header not in ("F.", "F. 0 0") or len(pad) != 12:
            raise RuntimeError("Unsupported movie reset/subframe/controller input at line " + str(number))
        mask = 0
        for index, value in enumerate(pad):
            if value not in (".", " ", buttons[index]):
                raise RuntimeError("Invalid movie button input at line " + str(number))
            if value == buttons[index]:
                mask |= 0x8000 >> index
        pads.append(mask)
    if not pads or len(pads) != entry["frames"]:
        raise RuntimeError("Movie input frame count differs from maintained manifest")
    canonical = "".join("{:04X}\n".format(mask) for mask in pads)
    converted = movie.with_suffix(".inputs")
    if converted.exists():
        run.guard_input(converted)
        try:
            converted_pads = [int(line.strip(), 16) for line in converted.read_text().splitlines()
                              if line.strip() and not line.startswith("#")]
        except ValueError as error:
            raise RuntimeError("Converted movie input contains invalid masks") from error
        if converted_pads != pads:
            raise RuntimeError("Converted movie input differs from original container: run just import-movies")
    table = run.root / "out/generated/steps.rs"
    table_status = "unavailable; regenerated table must be verified before compiled replay"
    if table.exists():
        # Prior generated output is optional provenance, not an immutable input
        # to this preflight: generation later produces separately bound output.
        table_hash = digest(table)
        match = re.search(r'ROM_SHA256:\s*Option<&str>\s*=\s*Some\("([0-9a-fA-F]{64})"\)', table.read_text())
        if digest(table) != table_hash:
            raise RuntimeError("Compiled table changed while checking its identity")
        if not match or match.group(1).lower() != rom_hash:
            raise RuntimeError("Compiled instruction table ROM identity mismatch: regenerate with just recompile-steps")
        table_status = "verified"
    output = run.path / "generated/movie.inputs"
    if output.exists():
        raise RuntimeError("Verified input evidence already exists")
    output.write_text(canonical)
    identity = {"rom_sha256": rom_hash, "movie_sha256": movie_hash,
                "container_input_sha256": hashlib.sha256(raw_inputs).hexdigest(),
                "input_sha256": digest(output), "input_format": "P1 hexadecimal masks, uppercase, LF",
                "frames": len(pads), "seed": 0, "reset": "clean default power-on; no reset/subframe records",
                "profile": "lsnes-profile", "controller": "P1 SNES gamepad", "region": "NTSC",
                "compiled_table_rom": table_status, "input_evidence": "generated/movie.inputs"}
    if table.exists():
        identity["prior_compiled_table_sha256"] = table_hash
    run.manifest["identities"]["inputs"] = identity
    run.save_manifest()
    return identity


def verify_hardware(run):
    """Verify actual local build sources against pinned archive plus maintained patches."""
    dependencies = json.loads((run.root / "tools/dependencies.json").read_text())
    dependency = next(record for record in dependencies if record["name"] == "rustysnes")
    archive = run.root / "deps/downloads" / dependency["archive"]
    if not archive.is_file() or run.guard_input(archive) != dependency["sha256"]:
        raise RuntimeError("Pinned RustySNES download missing or invalid: run just setup")
    upstream = run.root / "deps/rustysnes"
    expected_patches = {}
    patches = []
    for name in ("rustysnes-hdma", "rustysnes-clocked-apu", "rustysnes-clocked-dma",
                 "rustysnes-clocked-interrupts", "rustysnes-lsnes-reset", "rustysnes-readiness-observation", "rustysnes-readiness-checkpoints"):
        patch = run.root / "tools/patches" / (name + ".patch")
        patches.append({"path": patch.relative_to(run.root).as_posix(), "sha256": run.guard_input(patch)})
        records = patch.with_suffix(".json")
        if records.exists():
            run.guard_input(records)
            for record in json.loads(records.read_text()):
                expected_patches[record["path"]] = record["patched_sha256"]
    build_roots = {"rustysnes-core", "rustysnes-cpu", "rustysnes-ppu", "rustysnes-apu",
                   "rustysnes-cart", "rustysnes-savestate"}
    expected = {}
    with tarfile.open(archive, "r:gz") as source:
        for member in source.getmembers():
            relative = Path(*Path(member.name).parts[1:])
            parts = relative.parts
            is_input = (str(relative) in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml")
                        or len(parts) >= 3 and parts[0] == "crates" and parts[1] in build_roots
                        and (parts[2] == "src" or parts[2] in ("Cargo.toml", "build.rs")))
            if member.isfile() and is_input:
                expected[relative.as_posix()] = hashlib.sha256(source.extractfile(member).read()).hexdigest()
    expected.update(expected_patches)
    actual = []
    for relative, identity in sorted(expected.items()):
        path = upstream / relative
        if not path.is_file() or run.guard_input(path) != identity:
            raise RuntimeError("Hardware source differs from pinned patched profile: " + relative +
                               "; preserve edits and run python3 tools/prepare_rustysnes.py --offline")
        actual.append({"path": relative, "sha256": identity})
    for name in build_roots:
        for path in (upstream / "crates" / name / "src").rglob("*"):
            if path.is_file() and path.relative_to(upstream).as_posix() not in expected:
                raise RuntimeError("Unverified extra hardware build input: " + str(path))
    cargo = (run.root / "Cargo.toml").read_text()
    if dependency["revision"] not in cargo or 'features = ["lsnes-profile"]' not in cargo:
        raise RuntimeError("Workspace must select the pinned dependency and explicit lsnes-profile")
    identity = {"revision": dependency["revision"], "archive_sha256": dependency["sha256"],
                "source_sha256": inventory_digest(actual), "files": actual, "patches": patches,
                "features": ["std", "lsnes-profile", "clocked-smp", "clocked-dma", "clocked-interrupts"]}
    run.manifest["identities"]["hardware"] = identity
    run.save_manifest()
    return identity


def doctor_next_steps(report, run):
    if report["status"] == "pass":
        return []
    output = (run.path / report["stdout"]).read_text(errors="replace")
    missing = [line.partition(": ")[2] for line in output.splitlines() if line.startswith("MISSING:")]
    steps = []
    for label in missing:
        if label.startswith("TAS movie"):
            step = "Run just fetch-movies (missing or invalid " + label + ")"
        elif label == "original lsnes recording core":
            step = "Run just setup-lsnes"
        elif label.startswith("Rust "):
            step = "Run rustup toolchain install " + label[5:] + " --component rustfmt --component clippy"
        elif label == "unheadered Super Metroid ROM (expected SHA-1)":
            step = "Place the verified unheadered Japan/USA ROM at rom/sm.sfc"
        elif label == "snesrev symbols":
            step = "Run just symbols (downloaded snesrev source is available)" if (run.root / "deps/snesrev-sm").is_dir() else "Run just setup (snesrev symbols)"
        elif label in ("RustySNES sources", "65816 corpus (256 native-mode opcode files)", "MesenCE binary") or label.startswith(("hardware profile patch", "RustySNES HDMA")):
            step = "Run just setup (" + label + ")"
        else:
            step = "Install required tool " + label
        if step not in steps:
            steps.append(step)
    return steps or ["Inspect logs/doctor.stderr.log and rerun just doctor"]


def preflight(run):
    """Capture source first, then verify setup; preserve all independent failures."""
    run.capture_source()
    failures = []
    doctor = run.run_stage("doctor", ["just", "doctor"])
    failures.extend(doctor_next_steps(doctor, run))
    try:
        entries = json.loads((run.root / "movies/manifest.json").read_text())
        entry = next(record for record in entries if record["file"] == "super-metroid-any-3653.lsmv")
        verify_movie_inputs(run, entry)
        run.record_check("input-identity", {"status": "pass", "complete": True, "frames": entry["frames"]})
    except (OSError, ValueError, KeyError, StopIteration, RuntimeError, zipfile.BadZipFile) as error:
        failures.append(str(error))
        run.record_check("input-identity", {"status": "error", "complete": False, "reason": str(error)})
    try:
        verify_hardware(run)
        run.record_check("hardware-identity", {"status": "pass", "complete": True})
    except (OSError, ValueError, KeyError, StopIteration, RuntimeError, tarfile.TarError) as error:
        failures.append(str(error))
        run.record_check("hardware-identity", {"status": "error", "complete": False, "reason": str(error)})
    compiler = run.run_stage("compiler", ["rustc", "--version", "--verbose"])
    if compiler["status"] == "pass":
        run.manifest["identities"]["compiler"] = {"version": (run.path / compiler["stdout"]).read_text().strip()}
    else:
        failures.append("Install the pinned toolchain from rust-toolchain.toml; inspect compiler logs")
    for name in ("Cargo.lock", "rust-toolchain.toml"):
        if (run.root / name).is_file():
            run.manifest["identities"][name] = {"sha256": run.guard_input(run.root / name)}
    run.manifest["build_settings"] = {"cargo": ["--locked", "--offline"], "profile": "release",
                                     "seed": 0, "features": ["lsnes-profile"]}
    run.save_manifest()
    run.check_sources()
    run.record_check("preflight", {"status": "error" if failures else "pass", "complete": not failures,
                                  "failures": failures})
    return failures


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["preflight"])
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    arguments = parser.parse_args(argv)
    run = Run.create(arguments.root, "preflight", {"domain": "Japan/USA any%", "frames": 18000})
    print("Run: " + str(run.path), flush=True)
    print("Brief: " + str(run.path / "brief.md"), flush=True)
    try:
        failures = preflight(run)
        return run.finalize("error" if failures else "pass", "preflight",
                            "; ".join(failures) if failures else "Setup verified; preflight scope only, readiness gates remain pending",
                            mandatory_checks=["preflight"])
    except KeyboardInterrupt:
        return run.finalize("incomplete", "preflight", "Preflight interrupted")
    except Exception as error:
        return run.finalize("error", "preflight", str(error))


def write_json(path, value):
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


def read_verdict(path):
    """Read terminal evidence, treating missing/malformed verdicts as incomplete."""
    try:
        verdict = json.loads((Path(path) / "verdict.json").read_text())
        manifest = json.loads((Path(path) / "manifest.json").read_text())
        if (verdict.get("schema_version") == 1 and verdict.get("run_id") == manifest.get("run_id")
                and manifest.get("state") == "complete" and verdict.get("finished_utc")
                and verdict.get("status") in ("pass", "divergence", "uncovered", "error", "incomplete")
                and isinstance(verdict.get("acceptance_eligible"), bool)):
            return verdict
    except (OSError, ValueError, AttributeError):
        pass
    return {"status": "incomplete", "acceptance_eligible": False,
            "reason": "No valid terminal verdict; run did not complete"}


class Run:
    """Own one evidence directory. Callers must allocate before checking setup."""

    @classmethod
    def create(cls, root, run_type, scope, accepted_parent=None):
        root = Path(root).resolve()
        timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
        run_id = timestamp + "-" + uuid.uuid4().hex[:12]
        path = root / "out/runs" / run_id
        path.mkdir(parents=True)
        for name in ("source", "generated", "logs", "checks", "diagnostics"):
            (path / name).mkdir()
        manifest = {"schema_version": 1, "run_id": run_id, "type": run_type,
                    "scope": scope, "accepted_parent": accepted_parent,
                    "started_utc": utc_now(), "finished_utc": None, "state": "running",
                    "commands": [], "identities": {}, "reused_evidence": [],
                    "frame_conventions": {"input_indices": "0..17999",
                                          "original_frame_labels": "272..18000 inclusive",
                                          "candidate_frame_offset": 0}}
        run = cls(root, path, manifest)
        run.save_manifest()
        (path / "brief.md").write_text("# Not ready: incomplete\n\nRun allocated; checks have not completed.\n")
        return run

    def __init__(self, root, path, manifest):
        self.root, self.path, self.manifest = root, path, manifest
        self.checks = {}
        self.source_files = None
        self.guarded_inputs = {}

    def capture_source(self):
        self.ensure_open()
        if self.source_files is not None:
            raise RuntimeError("Source snapshot has already been captured")
        files = source_inventory(self.root)
        for entry in files:
            destination = self.path / "source/files" / entry["path"]
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(self.root / entry["path"], destination)
            if digest(destination) != entry["sha256"]:
                raise RuntimeError("Source changed while capturing: " + entry["path"])
        if source_inventory(self.root) != files:
            raise RuntimeError("Source changed while capturing snapshot")
        self.source_files = files
        identity = inventory_digest(files)
        write_json(self.path / "source/inventory.json", {"schema_version": 1,
                   "sha256": identity, "files": files})
        self.manifest["identities"]["source"] = {"sha256": identity, "inventory": "source/inventory.json"}
        self.save_manifest()

    def restore_source(self, destination):
        """Restore verified snapshot into a new directory, never an existing tree."""
        destination = Path(destination)
        if destination.exists():
            raise RuntimeError("Restore destination must not already exist")
        inventory = json.loads((self.path / "source/inventory.json").read_text())
        if inventory_digest(inventory["files"]) != inventory["sha256"]:
            raise RuntimeError("Source inventory is corrupt")
        for entry in inventory["files"]:
            relative = Path(entry["path"])
            if relative.is_absolute() or ".." in relative.parts:
                raise RuntimeError("Unsafe source inventory path")
            if digest(self.path / "source/files" / relative) != entry["sha256"]:
                raise RuntimeError("Source snapshot is corrupt: " + str(relative))
        destination.mkdir(parents=True)
        for entry in inventory["files"]:
            target = destination / entry["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(self.path / "source/files" / entry["path"], target)
            target.chmod(entry["mode"])

    def guard_input(self, path):
        path = Path(path).resolve()
        self.guarded_inputs[str(path)] = digest(path)
        return self.guarded_inputs[str(path)]

    def check_sources(self):
        if self.source_files is None:
            raise RuntimeError("No source snapshot captured")
        if source_inventory(self.root) != self.source_files:
            raise RuntimeError("Maintained source/config changed during checks; run is contaminated")
        for name, identity in self.guarded_inputs.items():
            if not Path(name).is_file() or digest(name) != identity:
                raise RuntimeError("Input changed during checks; run is contaminated: " + name)

    @contextlib.contextmanager
    def shared_mutation(self):
        """Serialize shared generated/build changes; isolated workspaces may avoid it."""
        self.ensure_open()
        lock = self.root / "out/readiness-build.lock"
        lock.parent.mkdir(parents=True, exist_ok=True)
        with lock.open("a") as handle:
            fcntl.flock(handle, fcntl.LOCK_EX)
            try:
                self.check_sources()
                yield
                self.check_sources()
            finally:
                fcntl.flock(handle, fcntl.LOCK_UN)

    def record_check(self, name, report):
        self.ensure_open()
        if name in self.checks:
            raise RuntimeError("Check evidence must not be overwritten: " + name)
        if not name or any(character not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for character in name):
            raise ValueError("Check name must be a safe artifact name")
        self.checks[name] = dict(report)
        write_json(self.path / "checks" / (name + ".json"), report)
        self.save_manifest()

    def record_artifact(self, name, path, copy_to=None):
        """Bind an artifact only after its successful creation, optionally preserving it."""
        self.ensure_open()
        if name in self.manifest["identities"]:
            raise RuntimeError("Artifact identity must not be overwritten: " + name)
        path = Path(path)
        identity = {"sha256": digest(path), "bytes": path.stat().st_size, "path": str(path)}
        if copy_to is not None:
            relative = Path(copy_to)
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("Artifact must stay inside the run")
            target = self.path / relative
            if target.exists():
                raise RuntimeError("Artifact evidence must not be overwritten")
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
            if digest(target) != identity["sha256"]:
                raise RuntimeError("Artifact changed while copying: " + str(path))
            identity["evidence"] = relative.as_posix()
        self.manifest["identities"][name] = identity
        self.save_manifest()
        return identity

    def run_stage(self, name, command, cwd=None, env=None, timeout=None,
                  failure_status="error", guard=True, exit_statuses=None):
        """Log a real subprocess, retaining underlying code/signal and bounded timeout."""
        self.ensure_open()
        if name in self.checks:
            raise RuntimeError("Stage already recorded: " + name)
        if not name or any(character not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for character in name):
            raise ValueError("Stage name must be safe")
        command = [str(argument) for argument in command]
        started = time.monotonic()
        report = {"command": command, "cwd": str(cwd or self.root), "started_utc": utc_now(),
                  "stdout": "logs/" + name + ".stdout.log", "stderr": "logs/" + name + ".stderr.log",
                  "returncode": None, "signal": None, "complete": False, "status": "incomplete"}
        self.manifest["commands"].append(report)
        self.save_manifest()
        process = None
        with (self.path / report["stdout"]).open("wb") as stdout, (self.path / report["stderr"]).open("wb") as stderr:
            try:
                if guard:
                    self.check_sources()
                process = subprocess.Popen(command, cwd=cwd or self.root, env=env,
                                           stdout=stdout, stderr=stderr, start_new_session=True)
                report["returncode"] = process.wait(timeout=timeout)
                report["signal"] = -process.returncode if process.returncode < 0 else None
                report["complete"] = process.returncode >= 0
                report["status"] = ("incomplete" if process.returncode < 0 else
                                    "pass" if process.returncode == 0 else
                                    (exit_statuses or {}).get(process.returncode, failure_status))
                if process.returncode != 0:
                    report["reason"] = "Command exited with " + str(process.returncode)
                if guard:
                    self.check_sources()
            except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
                if process is not None and process.poll() is None:
                    os.killpg(process.pid, signal.SIGTERM)
                    try:
                        process.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
                report.update(status="incomplete", complete=False,
                              reason="Stage timed out" if isinstance(error, subprocess.TimeoutExpired) else "Stage interrupted")
                if process is not None:
                    report.update(returncode=process.returncode,
                                  signal=-process.returncode if process.returncode < 0 else None)
            except (OSError, RuntimeError) as error:
                report.update(status="error", complete=False, reason=str(error))
                stderr.write((str(error) + "\n").encode())
        report.update(elapsed_seconds=time.monotonic() - started, finished_utc=utc_now())
        write_json(self.path / "logs" / (name + ".json"), report)
        self.record_check(name, report)
        return report

    def ensure_open(self):
        if self.manifest["state"] != "running" or (self.path / "verdict.json").exists():
            raise RuntimeError("Completed evidence must never be overwritten")

    def save_manifest(self):
        self.ensure_open()
        write_json(self.path / "manifest.json", self.manifest)

    def finalize(self, status, stage=None, reason=None, mandatory_checks=(), brief_text=None):
        self.ensure_open()
        if status not in ("pass", "divergence", "uncovered", "error", "incomplete"):
            raise ValueError("Unknown terminal status")
        if status == "pass":
            failed = [(name, report) for name, report in self.checks.items()
                      if report.get("status") != "pass" or report.get("complete") is not True]
            if failed:
                # Infrastructure/incomplete failures outrank validation failures;
                # every individual outcome is still preserved in the verdict.
                order = {"error": 0, "incomplete": 1, "divergence": 2, "uncovered": 3}
                stage, report = min(failed, key=lambda item: order.get(item[1].get("status"), 0))
                status = report.get("status", "incomplete")
                if status == "pass" or status not in order:
                    status = "incomplete"
                reason = report.get("reason") or "Recorded check failed or did not complete: " + stage
        eligible = (status == "pass" and self.manifest["type"] in ("readiness", "iteration")
                    and bool(mandatory_checks)
                    and all(self.checks.get(name, {}).get("status") == "pass"
                            and self.checks[name].get("complete") is True
                            for name in mandatory_checks))
        if status == "pass" and mandatory_checks and not all(
                self.checks.get(name, {}).get("status") == "pass"
                and self.checks[name].get("complete") is True for name in mandatory_checks):
            status, reason = "incomplete", "Mandatory checks are absent or incomplete"
        verdict = {"schema_version": 1, "run_id": self.manifest["run_id"],
                   "status": status, "stage": stage, "reason": reason,
                   "scope": self.manifest["scope"], "checks": self.checks,
                   "mandatory_checks": list(mandatory_checks), "acceptance_eligible": eligible,
                   "finished_utc": utc_now(), "evidence": "manifest.json"}
        self.manifest.update(state="complete", finished_utc=verdict["finished_utc"])
        write_json(self.path / "manifest.json", self.manifest)
        (self.path / "brief.md").write_text(brief_text if brief_text is not None else
            "# {}: {}\n\nScope: `{}`\n\n{}\n\n"
            "[Manifest](manifest.json) · [Verdict](verdict.json) · [Logs](logs/)\n".format(
                "Ready" if eligible else "Not ready", status,
                json.dumps(self.manifest["scope"], sort_keys=True), reason or "Checks complete"))
        write_json(self.path / "verdict.json", verdict)
        return 0 if status == "pass" else 1 if status in ("divergence", "uncovered") else 2


if __name__ == "__main__":
    sys.exit(main())
