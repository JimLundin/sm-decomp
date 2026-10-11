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
