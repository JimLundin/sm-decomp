-- The lsnes backend's server (crates/oracle/src/lsnes.rs). lsnes rr2-beta23
-- (bsnes v085 core) runs it as a separate process; no emulator code is linked.
--
-- Requests arrive on stdin, one line each. The first is read at start and
-- each later one right after the previous record, at its cut:
--   S <pad hex> <memories 0|1>   step: this frame's pad, then run to its cut
--   V <path>                     save the state at the next frame boundary,
--                                which lies before the coming step's NMI
--   R                            press reset for the coming frame
--   Q                            quit
-- One record per lsnes frame: taken at the frame's cut (the NMI's RTI, or the
-- boundary when no NMI returned), written to OH_RECORD at the frame boundary
-- (overwritten each time) and announced on stdout as "R <bytes>".
local unpack = table.unpack or unpack
local record_path = assert(os.getenv("OH_RECORD"), "set OH_RECORD")
-- Lua errors in callbacks otherwise reach only lsnes.log; keep them here too.
local log_path = os.getenv("OH_LOG")

local function log(text)
    if log_path == nil then return end
    local file = io.open(log_path, "a")
    if file then file:write(text, "\n"); file:close() end
end

-- Run a callback under a handler that logs the error with its traceback and
-- quits, so a failing script ends the run instead of desyncing it. Only for
-- the frame callbacks: wrapping the memory callbacks costs speed in the hot
-- ones (joypad reads, wait loops), and wrapping the self-removing cut callback
-- made lsnes crash within 154 frames (docs/oracle/HARNESS.md, lsnes backend).
local function guarded(callback)
    return function(...)
        local ok, message = xpcall(callback, debug.traceback, ...)
        if not ok then
            log("frame " .. tostring(movie.currentframe()) .. ": " .. tostring(message))
            exec("quit-emulator")
        end
    end
end
-- Native NMI vector target ($00:FFEA), the game's wait sites (game::WAIT_SITES).
local NMI = 0x1000000 + memory.readword(0x100FFEA)
local WAIT_ENTRIES = {0x808338, 0x858136, 0x8584A3}
local BUS = 0x1000000

local pad, dump = 0, true
local nmi_s, nmi_pc, in_nmi = nil, nil, false
local reads, nmi_reads = 0, 0
local wait_entry = nil
local reset_next = false

local function u8(v) return string.char(v % 256) end
local function u16(v) return string.char(v % 256, math.floor(v / 256) % 256) end
local function u32(v) return u16(v % 65536) .. u16(math.floor(v / 65536)) end

local function region(base, size)
    local bytes = memory.readregion(base, size)
    local parts = {}
    for at = 0, size - 1, 4096 do
        parts[#parts + 1] = string.char(unpack(bytes, at, math.min(at + 4095, size - 1)))
    end
    return table.concat(parts)
end

local function read_request()
    while true do
        local line = io.stdin:read("*l")
        if line == nil or line == "Q" then exec("quit-emulator"); return end
        local op, rest = line:match("^(%a) ?(.*)$")
        if op == "S" then
            local hex, flag = rest:match("^(%x+) (%d)$")
            pad, dump = tonumber(hex, 16), flag == "1"
            return
        elseif op == "V" then
            exec("save-state " .. rest)
        elseif op == "R" then
            reset_next = true
        else
            error("bad request: " .. line)
        end
    end
end

-- The record of this lsnes frame's cut, taken at the RTI and sent at the boundary.
local captured = nil

-- kind: 0 NMI return, 1 vblank without NMI, 2 NMI still running.
local function capture(kind, pc)
    local r = memory.getregisters()
    local parts = {
        "OHR1", u8(kind), u32(pc or 0), u8(nmi_reads > 0 and 1 or 0), u32(reads),
        wait_entry and (u8(1) .. u16(wait_entry[1]) .. u16(wait_entry[2])) or (u8(0) .. u16(0) .. u16(0)),
        u8(0), u32(movie.currentframe()), u16(pad),
        u16(r.a), u16(r.x), u16(r.y), u16(r.s), u16(r.pc % 65536), u16(r.d),
        u8(r.pb), u8(r.db), u8(r.p), u8(r.e),
        u8(dump and 1 or 0),
    }
    if dump then
        parts[#parts + 1] = region(0x7E0000, 0x20000)
        parts[#parts + 1] = region(0x10000, 0x10000)
        parts[#parts + 1] = region(0x21000, 0x200)
        parts[#parts + 1] = region(0x20000, 0x220)
        parts[#parts + 1] = region(0x10000000, 0x2000)
    end
    captured = table.concat(parts)
end

-- One record per lsnes frame, at its boundary; then the next request.
local function send()
    local data = captured
    local file = assert(io.open(record_path, "wb"))
    assert(file:write(data))
    assert(file:close())
    io.stdout:write("R " .. #data .. "\n")
    io.stdout:flush()
    captured, reads, wait_entry = nil, 0, nil
    read_request()
end

local function on_cut()
    local s = memory.getregisters().s
    if not in_nmi or s ~= (nmi_s + 4) % 65536 then return end
    memory.unregisterexec(BUS + nmi_pc, on_cut)
    in_nmi = false
    if captured == nil then capture(0, nmi_pc) end
end

memory.registerexec(NMI, function()
    if in_nmi then return end
    local s = memory.getregisters().s
    local byte = function(offset) return memory.readbyte(0x7E0000 + (s + offset) % 0x2000) end
    nmi_s, nmi_pc = s, byte(2) + byte(3) * 256 + byte(4) * 65536
    in_nmi, nmi_reads = true, 0
    memory.registerexec(BUS + nmi_pc, on_cut)
end)
for _, bank in ipairs({0x00, 0x80}) do
    for address = 0x4218, 0x421F do
        memory.registerread(BUS + bank * 0x10000 + address, function()
            reads = reads + 1
            if in_nmi then nmi_reads = nmi_reads + 1 end
        end)
    end
end
for _, entry in ipairs(WAIT_ENTRIES) do
    memory.registerexec(BUS + entry, function()
        if wait_entry == nil then
            local r = memory.getregisters()
            wait_entry = {r.vcounter, r.hcounter}
        end
    end)
end

on_input = guarded(function()
    for index = 0, 11 do
        input.set(0, index, math.floor(pad / 2 ^ (15 - index)) % 2)
    end
    if reset_next then input.reset(); reset_next = false end
end)

on_frame_emulated = guarded(function()
    if captured == nil then
        if in_nmi then capture(2, nmi_pc) else capture(1, 0) end
    end
    send()
end)

exec("set-speed turbo")
-- A resumed run. On the command line a state's core type (snes_ntsc) does not
-- match the ROM's (SNES) and lsnes hangs, so the script loads it: read-write,
-- so on_input still sets the pads, at the coming frame boundary, before any
-- frame is emulated.
local start = os.getenv("OH_START")
if start then exec("load-state " .. start) end
read_request()
