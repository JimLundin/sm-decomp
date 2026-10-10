#!/usr/bin/env python3
"""Replay an original LSMV on its recording core and capture game-loop state."""
import argparse
import hashlib
import json
import os
import subprocess
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIELDS = (0x998, 0x79b, 0xaf6, 0xafa, 0x9c2, 0x9a4, 0x9da, 0x9dc, 0x9de, 0x9e0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("movie", type=Path)
    parser.add_argument("--frames", type=int, default=12000)
    parser.add_argument("--rom", type=Path, default=ROOT / "rom/sm.sfc")
    parser.add_argument("--out", type=Path, default=ROOT / "out/sync/lsnes.csv")
    args = parser.parse_args()
    if args.frames <= 0:
        parser.error("--frames must be positive")
    binary = ROOT / "deps/lsnes/source/src/util/lsnes-dumpavi.util"
    if not binary.is_file():
        parser.error("recording emulator missing: run just setup-lsnes")
    movie, rom, out = args.movie.resolve(), args.rom.resolve(), args.out.resolve()
    with zipfile.ZipFile(movie) as archive:
        if archive.read("rom.sha256").decode().strip() != hashlib.sha256(rom.read_bytes()).hexdigest():
            parser.error("movie ROM checksum mismatch")
        if archive.read("coreversion").decode().strip() != "bsnes v085 (Compatibility core)":
            parser.error("movie requires a different recording core")
    out.parent.mkdir(parents=True, exist_ok=True)
    metadata = out.with_suffix(".capture.json")
    metadata.unlink(missing_ok=True)
    script = out.with_suffix(".lua")
    # Use a Lua long string to preserve paths without shell interpolation.
    if "]]" in str(out):
        parser.error("unsupported output path")
    script.write_text("local out = assert(io.open([[" + str(out) + "]], 'w'))\n" +
        "out:write('frame,event,game_state,room,x,y,health,items,time_frames,time_seconds,time_minutes,time_hours\\n')\n" +
        "local fields = {" + ",".join(str(offset) for offset in FIELDS) + "}\n" + """
local event = 0
local function checkpoint()
    event = event + 1
    local parts = {tostring(movie.currentframe()), tostring(event)}
    for _,address in ipairs(fields) do
        parts[#parts+1] = string.format('%04X', memory.readword('WRAM', address))
    end
    out:write(table.concat(parts, ','), '\\n')
    out:flush()
end
memory.registerexec('BUS', 0x828948, checkpoint)
""")
    env = os.environ.copy()
    env["XDG_CONFIG_HOME"] = str(out.parent / "lsnes-config")
    command = [str(binary), "--dumper=INTERNAL-NULL", "--rom-a=" + str(rom),
               "--length=" + str(args.frames), "--lua=" + str(script), str(movie)]
    with out.with_suffix(".log").open("w") as log:
        subprocess.run(command, env=env, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
    metadata.write_text(json.dumps({"completed_frames": args.frames,
        "checkpoint_sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
        "movie_sha256": hashlib.sha256(movie.read_bytes()).hexdigest(),
        "rom_sha256": hashlib.sha256(rom.read_bytes()).hexdigest(),
        "core": "lsnes rr2-beta23 / bsnes v085 Compatibility"}, indent=2) + "\n")
    print("lsnes: captured {} frames -> {}".format(args.frames, out))


if __name__ == "__main__":
    main()
