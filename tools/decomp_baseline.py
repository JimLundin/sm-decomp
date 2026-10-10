#!/usr/bin/env python3
"""Build and replay an independent pinned C subject; never accepts Rust ownership."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import shutil

ROOT = Path(__file__).resolve().parents[1]
REVISION = "578f90b3cc49557bb70060ad033bb90b8cf8ac50"
ROM_SHA256 = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--frames", type=int, default=9000)
    parser.add_argument("--inputs", type=Path, default=ROOT / "movies/super-metroid-any-3653.inputs")
    parser.add_argument("--rom", type=Path, default=ROOT / "rom/sm.sfc")
    parser.add_argument("--output", type=Path, default=ROOT / "out/decomp/baseline")
    parser.add_argument("--timeout", type=int, default=120)
    args = parser.parse_args()
    if not 1 <= args.frames <= 1000000 or args.timeout < 1:
        parser.error("frames/timeout outside supported bounds")
    source = ROOT / "deps/snesrev-sm"
    rom = args.rom.resolve()
    run = args.output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    run.mkdir(parents=True)

    verdict = {"contract": "decomp-c-baseline-v1", "source_revision": REVISION,
               "requested_frames": args.frames, "rust_routines": 0,
               "original_rom_parity": "not-tested", "input_adaptation": "bit-reversal-only; no lag compensation",
               "status": "error"}
    try:
        dependencies = json.loads((ROOT / "tools/dependencies.json").read_text())
        pin = next(item for item in dependencies if item["name"] == "snesrev-sm")
        if pin["revision"] != REVISION or digest(ROOT / "deps/downloads" / pin["archive"]) != pin["sha256"]:
            raise ValueError("dependency archive does not match the frozen pin")
        verdict["source_archive_sha256"] = pin["sha256"]
        # Verify extracted files against the pinned archive, including headers and notices.
        with tarfile.open(ROOT / "deps/downloads" / pin["archive"]) as archive:
            for member in archive.getmembers():
                if member.isfile():
                    relative = Path(*Path(member.name).parts[1:])
                    expected = archive.extractfile(member).read()
                    if (source / relative).read_bytes() != expected:
                        raise ValueError(f"source differs from frozen archive: {relative}")
        if digest(rom) != ROM_SHA256:
            raise ValueError("unsupported ROM identity")
        verdict["rom_sha256"] = digest(rom)
        verdict["inputs_sha256"] = digest(args.inputs)
        units = sorted((source / "src").glob("sm_*.c"))
        units += [source / "src" / name for name in ("spc_player.c", "tracing.c", "util.c")]
        units += sorted((source / "src/snes").glob("*.c"))
        host = ROOT / "tools/decomp/headless.c"
        shutil.copyfile(host, run / "host.c")
        shutil.copyfile(Path(__file__), run / "driver.py")
        shutil.copyfile(source / "LICENSE.txt", run / "LICENSE.txt")
        verdict["sources"] = {str(p.relative_to(ROOT)): digest(p) for p in [*units, host]}
        verdict["license_sha256"] = digest(source / "LICENSE.txt")
        verdict["compiler"] = subprocess.check_output(["cc", "--version"], text=True, timeout=args.timeout).splitlines()[0]
        command = ["cc", "-std=gnu11", "-O2", "-fno-strict-aliasing", "-ffunction-sections",
                   "-fdata-sections", "-iquote", str(source / "src"), *map(str, units), str(host),
                   "-Wl,--gc-sections", "-Wl,--wrap=Call", "-Wl,--wrap=cpu_runOpcode",
                   "-Wl,--wrap=DecompressToMem", "-lm", "-o", str(run / "headless")]
        verdict["build_command"] = command
        with (run / "build.log").open("w") as log:
            subprocess.run(command, stdout=log, stderr=log, check=True, timeout=args.timeout)
        verdict["binary_sha256"] = digest(run / "headless")
        control = run / "observation-control"
        (control / "fixtures").mkdir(parents=True)
        (control / "saves").mkdir()
        with (control / "stdout.log").open("w") as log, (control / "effects.log").open("w") as errors:
            subprocess.run([str(run / "headless"), "--observation-control", str(rom), "unused"],
                           cwd=control, stdout=log, stderr=errors, check=True, timeout=args.timeout)
        control_records = [json.loads(line) for line in (control / "stdout.log").read_text().splitlines() if line.startswith('{')]
        if not control_records:
            raise ValueError("missing observation control report")
        measured = control_records[-1]
        if measured["fallback_calls"] != 1 or measured["fallback_instructions"] < 1 or measured["decompression_calls"] != 1:
            raise ValueError("observation control did not exercise fallback and decompression wrappers")
        verdict["observation_controls"] = measured
        verdict["replay_commands"] = []
        for prefix in ("replay", "repeat"):
            subject = run / prefix
            (subject / "fixtures").mkdir(parents=True)
            (subject / "saves").mkdir()
            command = [str(run / "headless"), str(rom), str(args.inputs.resolve()), str(args.frames), str(run / f"{prefix}.state.bin")]
            verdict["replay_commands"].append(command)
            with (run / f"{prefix}.log").open("w") as log, (run / f"{prefix}.effects.log").open("w") as errors:
                subprocess.run(command, cwd=subject, stdout=log, stderr=errors, check=True, timeout=args.timeout)
            fixture_dir = subject / "fixtures"
            fixture_digests = {p.name: digest(p) for p in sorted(fixture_dir.glob("*.wram"))}
            rows = [json.loads(line) for line in (fixture_dir / "calls.jsonl").read_text().splitlines()]
            calls = []
            for sequence, row in enumerate(rows, 1):
                if row["sequence"] != sequence or not 0 <= row["destination_offset"] < 0x20000:
                    raise ValueError("invalid captured routine arguments")
                for phase in ("before", "after"):
                    name = row.pop(phase + "_file")
                    if (fixture_dir / name).stat().st_size != 0x20000:
                        raise ValueError("incomplete routine state")
                    row[phase] = {"file": name, "sha256": fixture_digests[name], "size": 0x20000}
                row["fixture_id"] = f"decompress-{sequence:04d}"
                rom_capture = fixture_dir / f"decompress-{sequence:04d}.rom"
                rom_digest = digest(rom_capture)
                rom_name = f"rom-{rom_digest}.bin"
                retained_rom = fixture_dir / rom_name
                if retained_rom.exists():
                    rom_capture.unlink()
                else:
                    rom_capture.rename(retained_rom)
                row["rom_view_sha256"] = rom_digest
                row["rom_view_file"] = rom_name
                calls.append(row)
            manifest = {"contract": "decomp-c-captures-v1", "routine": "DecompressToMem",
                        "routine_address": 0x80b119, "source_revision": REVISION,
                        "original_rom_sha256": ROM_SHA256, "rom_view_file": "../rom-view.bin",
                        "rom_view_sha256": digest(subject / "rom-view.bin"),
                        "rom_view_size": (subject / "rom-view.bin").stat().st_size,
                        "rom_expansion": "3 MiB to 4 MiB: mirror final MiB before patches", "calls": calls}
            (fixture_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
            if prefix == "replay":
                original, patched = rom.read_bytes(), (subject / "rom-view.bin").read_bytes()
                # Upstream expands this 3 MiB LoROM by mirroring its final MiB.
                expanded = original + original[0x200000:0x300000]
                if len(expanded) != len(patched):
                    raise ValueError("unexpected in-memory ROM size")
                changed = [i for i, (a, b) in enumerate(zip(expanded, patched)) if a != b]
                verdict["in_memory_rom_patches"] = {"changed_bytes": len(changed),
                    "file_offsets": changed, "rom_view_sha256": manifest["rom_view_sha256"],
                    "original_size": len(original), "rom_view_size": len(patched),
                    "expansion": "3 MiB to 4 MiB: mirror final MiB before patches"}
                first_manifest = manifest
            elif manifest != first_manifest:
                raise ValueError("routine manifests differ on fresh-process replay")
            if prefix == "replay":
                verdict["fixtures"] = fixture_digests
            elif fixture_digests != verdict["fixtures"]:
                raise ValueError("routine fixtures differ on fresh-process replay")
        for suffix in (".log", ".effects.log", ".state.bin"):
            if digest(run / f"replay{suffix}") != digest(run / f"repeat{suffix}"):
                raise ValueError(f"fresh-process replay is nondeterministic: {suffix}")
        records = [json.loads(line) for line in (run / "replay.log").read_text().splitlines() if line.startswith('{')]
        if not records or records[-1]["frame"] != args.frames:
            raise ValueError("missing terminal replay record")
        verdict["terminal"] = records[-1]
        if len(verdict["fixtures"]) != 2 * records[-1]["decompression_calls"]:
            raise ValueError("routine fixture coverage incomplete")
        verdict["state_sha256"] = digest(run / "replay.state.bin")
        verdict["determinism"] = "two fresh processes; progress, fallback/decompression log, WRAM and SRAM match"
        verdict["status"] = "replay-completed"
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        verdict["error"] = str(error)
    (run / "verdict.json").write_text(json.dumps(verdict, indent=2) + "\n")
    print(json.dumps({"status": verdict["status"], "evidence": str(run), "terminal": verdict.get("terminal"), "error": verdict.get("error")}))
    return 0 if verdict["status"] == "replay-completed" else 1


if __name__ == "__main__":
    sys.exit(main())
