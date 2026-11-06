-- MesenCE --testRunner: play an `.inputs` file (one JOY1 word per line) from
-- power-on and write a per-frame log in the column layout of
-- tools/lsnes_lag_log.lua, so `oracle movie --lag-log` can compare it.
-- Needs --debug.scriptWindow.allowIoOsAccess=true for io/os, and
-- --snes.port1.type=SnesController (an empty settings.json has no pad).
-- Delete the portable folder's Saves/ first: Mesen keeps SRAM between runs,
-- and a kept save changes the boot. See docs/ORACLE.md, MEASUREMENTS (g).
-- SM_MESEN_INPUTS: the pad file; SM_MESEN_LOG: a new output path;
-- SM_MESEN_FRAMES: how many frames to log. Rows are cut at Mesen's
-- startFrame (line 0); counter_line is the line of the frame's last $05B8
-- write, wait_line the line the main loop entered its NMI wait.
local inputs_path = assert(os.getenv("SM_MESEN_INPUTS"), "set SM_MESEN_INPUTS")
local log_path = assert(os.getenv("SM_MESEN_LOG"), "set SM_MESEN_LOG")
local limit = assert(tonumber(os.getenv("SM_MESEN_FRAMES")), "set SM_MESEN_FRAMES")

local pads = {}
for line in io.lines(inputs_path) do
    local trimmed = line:match("^%s*(%x+)%s*$")
    if trimmed then pads[#pads + 1] = tonumber(trimmed, 16) end
end

-- JOY1 bit 15 down to bit 4, as the auto-read latches them.
local buttons = {"b", "y", "select", "start", "up", "down", "left", "right", "a", "x", "l", "r"}
local function pad_table(word)
    local input = {}
    for index, name in ipairs(buttons) do
        input[name] = (word & (1 << (16 - index))) ~= 0
    end
    return input
end

local wram = emu.memType.snesWorkRam
local function word(address) return emu.read16(address, wram, false) end

local log = assert(io.open(log_path, "wb"))
log:write("input_frame,movie_frame,nmi_before,nmi_after,including_lag_before,including_lag_after,"
    .. "emulator_lag,game_state,room,x,y,apu_ack_0641,joypad_reads,inputs_polled,pc_at_start,joypad_008b,counter_line,wait_line\n")

local polls = 0      -- inputPolled events so far; the next pad's index
local rows = 0
local joypad_reads = 0
local before_frame, before_nmi = word(0x05B6), word(0x05B8)
local counter_line, wait_line = -1, -1

-- Where in the frame the NMI handler bumps $05B8, and where the main loop
-- enters its wait for NMI ($80:8338): the margins before the next cut.
emu.addMemoryCallback(function() counter_line = emu.getState()["ppu.scanline"] end,
    emu.callbackType.write, 0x05B8, 0x05B8, emu.cpuType.snes, emu.memType.snesWorkRam)
emu.addMemoryCallback(function() wait_line = emu.getState()["ppu.scanline"] end,
    emu.callbackType.exec, 0x808338, 0x808338, emu.cpuType.snes, emu.memType.snesMemory)

emu.addEventCallback(function()
    local word_value = pads[polls + 1] or 0
    emu.setInput(pad_table(word_value), 0, 0)
    polls = polls + 1
end, emu.eventType.inputPolled)

-- ReadJoypadInputs ($80:9459), the hook lsnes_lag_log.lua calls joypad_begin.
emu.addMemoryCallback(function() joypad_reads = joypad_reads + 1 end,
    emu.callbackType.exec, 0x809459, 0x809459, emu.cpuType.snes, emu.memType.snesMemory)

emu.addEventCallback(function()
    local state = emu.getState()
    local pc = (state["cpu.k"] or 0) * 65536 + (state["cpu.pc"] or 0)
    local frame, nmi = word(0x05B6), word(0x05B8)
    log:write(string.format("%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d\n",
        rows, rows + 1, before_frame, frame, before_nmi, nmi,
        joypad_reads == 0 and 1 or 0, word(0x0998), word(0x079B), word(0x0AF6), word(0x0AFA),
        word(0x0641), joypad_reads, polls, pc, word(0x008B), counter_line, wait_line))
    before_frame, before_nmi = frame, nmi
    joypad_reads, counter_line, wait_line = 0, -1, -1
    rows = rows + 1
    if rows >= limit then
        log:close()
        emu.stop(0)
    end
end, emu.eventType.startFrame)
