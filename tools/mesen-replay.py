#!/usr/bin/env python3
"""Build a headless Mesen replay script from converted frame inputs.

Launch with --snes.port1.type=SnesController --snes.port2.type=None.
Mesen polls after its first endFrame event; input indices follow polls.
"""
import argparse
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("inputs", type=Path)
parser.add_argument("--frames", type=int, default=3600)
parser.add_argument("--out", type=Path, default=Path("out/mesen-replay.lua"))
args = parser.parse_args()
if args.frames <= 0:
    parser.error("--frames must be positive")
pads = [int(line, 16) for line in args.inputs.read_text().splitlines() if line.strip() and not line.startswith("#")]
if any(pad < 0 or pad > 0xfff0 or pad & 0xf for pad in pads):
    parser.error("inputs must contain SNES gamepad masks (bits 4 through 15)")
pads = pads[:args.frames]
if not pads:
    parser.error("no frames selected")
args.out.parent.mkdir(parents=True, exist_ok=True)
script = "local pads = {" + ",".join(str(pad) for pad in pads) + "}\n"
script += """
local frames = 0
local polls = 0
local function input()
    polls = polls + 1
    local pad = pads[polls] or 0
    local function bit(mask) return (pad & mask) ~= 0 end
    emu.setInput({b=bit(0x8000),y=bit(0x4000),select=bit(0x2000),start=bit(0x1000),
                  up=bit(0x0800),down=bit(0x0400),left=bit(0x0200),right=bit(0x0100),
                  a=bit(0x0080),x=bit(0x0040),l=bit(0x0020),r=bit(0x0010)}, 0, 0)
    local got = emu.getInput(0, 0)
    if got.start ~= bit(0x1000) or got.a ~= bit(0x0080) or got.b ~= bit(0x8000) then
        print("Mesen input injection failed: require a SNES controller on port 1")
        emu.stop(1)
    end
end
local function word(addr)
    return emu.read(addr, emu.memType.snesWorkRam) + 256 * emu.read(addr+1, emu.memType.snesWorkRam)
end
local function frame()
    frames = frames + 1
    if frames == 1 or polls % 600 == 0 or polls == #pads then
        print(string.format("mesen: frame=%d input_polls=%d game_state=$%04X room=$%04X", frames, polls, word(0x998), word(0x79b)))
    end
    if polls == #pads then emu.stop(0) end
end
emu.addEventCallback(input, emu.eventType.inputPolled)
emu.addEventCallback(frame, emu.eventType.endFrame)
"""
args.out.write_text(script)
print("Mesen replay: {} frames -> {}".format(len(pads), args.out))
