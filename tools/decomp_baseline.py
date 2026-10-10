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
    parser.add_argument("--timeout", type=int, default=120)
    args = parser.parse_args()
    if not 1 <= args.frames <= 1000000 or args.timeout < 1:
        parser.error("frames/timeout outside supported bounds")
    source = ROOT / "deps/snesrev-sm"
    rom = ROOT / "rom/sm.sfc"
    run = ROOT / "out/decomp/baseline" / datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
    run.mkdir(parents=True)
    (run / "saves").mkdir()
    (run / "fixtures").mkdir()
    verdict = {"contract": "decomp-c-baseline-v1", "source_revision": REVISION,
               "requested_frames": args.frames, "rust_routines": 0,
               "original_rom_parity": "not-tested", "input_adaptation": "bit-reversal-only; no lag compensation",
               "status": "error"}
    try:
        dependencies = json.loads((ROOT / "tools/dependencies.json").read_text())
        pin = next(item for item in dependencies if item["name"] == "snesrev-sm")
        if pin["revision"] != REVISION or digest(ROOT / "deps/downloads" / pin["archive"]) != pin["sha256"]:
            raise ValueError("dependency archive does not match the frozen pin")
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
        verdict["compiler"] = subprocess.check_output(["cc", "--version"], text=True).splitlines()[0]
        command = ["cc", "-std=gnu11", "-O2", "-fno-strict-aliasing", "-ffunction-sections",
                   "-fdata-sections", "-iquote", str(source / "src"), *map(str, units), str(host),
                   "-Wl,--gc-sections", "-Wl,--wrap=Call", "-Wl,--wrap=cpu_runOpcode",
                   "-Wl,--wrap=DecompressToMem", "-lm", "-o", str(run / "headless")]
        verdict["build_command"] = command
        with (run / "build.log").open("w") as log:
            subprocess.run(command, stdout=log, stderr=log, check=True, timeout=args.timeout)
        verdict["binary_sha256"] = digest(run / "headless")
        verdict["replay_commands"] = []
        for prefix in ("replay", "repeat"):
            command = [str(run / "headless"), str(rom), str(args.inputs.resolve()), str(args.frames), str(run / f"{prefix}.state.bin")]
            verdict["replay_commands"].append(command)
            with (run / f"{prefix}.log").open("w") as log, (run / f"{prefix}.effects.log").open("w") as errors:
                subprocess.run(command, cwd=run, stdout=log, stderr=errors, check=True, timeout=args.timeout)
            fixture_digests = {p.name: digest(p) for p in sorted((run / "fixtures").glob("*.wram"))}
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
