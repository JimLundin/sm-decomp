-- The lsnes backend's server (crates/oracle/src/lsnes.rs). lsnes rr2-beta23
-- (bsnes v085 core) runs it as a separate process; no emulator code is linked.
--
-- Requests arrive on stdin, one line each. The first is read at start and
-- each later one right after the previous record:
--   S <pad hex> <memories 0|1> [<next pad hex>]
--                                step: this frame's pad, then run to its cut;
--                                the next step's pad, when the client knows it
--   V <path>                     save the state at the frame boundary that
--                                ends the coming step: after its cut, or
--                                inside its NMI, but before the next NMI
--   R                            press reset for the coming frame
--   W <offset hex> <word hex>    write a little-endian word to work RAM
--                                ($7E:0000 + offset); asked for before a
--                                resumed run's state has loaded, right
--                                after it loads
--   Q                            quit
-- One record per video frame, cut at the NMI handler's RTI, or at lsnes's
-- frame boundary (line 241) in a frame without an NMI. It is written to
-- OH_RECORD (overwritten each time) and announced on stdout as "R <bytes>":
-- at the boundary, or at the RTI when the NMI outlasts the boundary.
--
-- lsnes commits a frame's input at its start (on_input, at the boundary), and
-- input.set has no effect anywhere else. An NMI that outlasts the boundary
-- returns early in the next lsnes frame, whose input is already committed,
-- so that frame takes the next pad the client announced with this step; the
-- next step must then bring the same pad. Without an announced pad such a
-- frame is still cut at the boundary, with its NMI running.
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
-- ones (joypad reads, wait loops) and the cut.
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
local BUS = 0x1000000
local NMI = memory.readword(BUS + 0xFFEA)
local WAIT_ENTRIES = {0x808338, 0x858136, 0x8584A3}

-- The NMI handler's own RTI, found once from the ROM: the vector's
-- `REP #$30; JML handler`, then the handler's one epilogue, which pops what its
-- prologue pushed (PLY PLX PLA PLD PLB RTI). A callback that stays registered
-- there takes the cut. Unregistering a callback from inside lsnes's dispatcher
-- made lsnes call the freed entry (docs/oracle/HARNESS.md, "lsnes crashes").
local function find_rti()
    local function byte(address) return memory.readbyte(BUS + address) end
    assert(byte(NMI) == 0xC2 and byte(NMI + 1) == 0x30 and byte(NMI + 2) == 0x5C,
        "NMI vector is not REP #$30; JML")
    local handler = byte(NMI + 3) + byte(NMI + 4) * 256 + byte(NMI + 5) * 65536
    local epilogue = {0x7A, 0xFA, 0x68, 0x2B, 0xAB, 0x40}
    local found = nil
    for at = handler, handler + 0x200 - #epilogue do
        local matches = true
        for offset, value in ipairs(epilogue) do
            if byte(at + offset - 1) ~= value then matches = false; break end
        end
        if matches then
            assert(found == nil, "NMI handler has more than one epilogue")
            found = at + #epilogue - 1
        end
    end
    return assert(found, "NMI handler epilogue not found")
end
local NMI_RTI = find_rti()

local pad, dump = 0, true
-- The next step's pad, as announced; the pad committed ahead for the frame
-- after a boundary inside an NMI; the save that waits for this step's boundary.
local next_pad, committed, save_path = nil, nil, nil
-- A resumed state saved inside an NMI returns from it first, with the counts
-- of a step already recorded: they restart at that RTI.
local dangling = os.getenv("OH_START") ~= nil
-- nmi_frame: the lsnes frame the running NMI began in, the record's frame
-- even when the NMI returns in the next one.
local nmi_s, nmi_pc, nmi_frame, in_nmi = nil, nil, nil, false
local reads, nmi_reads = 0, 0
local wait_entry = nil
local reset_next = false
-- Work-RAM writes waiting for the start state: a load requested at start
-- happens at the coming frame boundary, after the first requests are read.
local loading, pending_writes = os.getenv("OH_START") ~= nil, {}

local function write_word(offset, value)
    memory.writeword(0x7E0000 + offset, value)
end

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

-- Stop the run: an error raised in a memory callback reaches only lsnes.log
-- and lsnes keeps running, desynced.
local function abort(message)
    log("frame " .. tostring(movie.currentframe()) .. ": " .. message)
    exec("quit-emulator")
end

local function read_request()
    while true do
        local line = io.stdin:read("*l")
        if line == nil or line == "Q" then exec("quit-emulator"); return end
        local op, rest = line:match("^(%a) ?(.*)$")
        if op == "S" then
            local hex, flag, ahead = rest:match("^(%x+) (%d) ?(%x*)$")
            if hex == nil then abort("bad request: " .. line); return end
            pad, dump = tonumber(hex, 16), flag == "1"
            next_pad = ahead ~= "" and tonumber(ahead, 16) or nil
            if committed ~= nil and committed ~= pad then
                abort(string.format("step pad %04x, announced %04x", pad, committed))
            end
            committed = nil
            return
        elseif op == "V" then
            save_path = rest
        elseif op == "R" then
            if committed ~= nil then
                abort("reset asked for after its frame's input was committed")
                return
            end
            reset_next = true
        elseif op == "W" then
            local offset, value = rest:match("^(%x+) (%x+)$")
            offset, value = tonumber(offset, 16), tonumber(value, 16)
            assert(offset and offset < 0x20000 - 1 and value < 0x10000, "bad write: " .. line)
            if loading then
                pending_writes[#pending_writes + 1] = {offset, value}
            else
                write_word(offset, value)
            end
        else
            error("bad request: " .. line)
        end
    end
end

-- The record of this step's cut, taken at the RTI and sent at the boundary;
-- owed: the boundary passed inside the NMI, so the RTI sends it.
local captured, owed = nil, false

-- kind: 0 NMI return, 1 vblank without NMI, 2 NMI still running.
-- r: the CPU registers, as memory.getregisters() returns them.
local function capture(kind, pc, r)
    local parts = {
        "OHR1", u8(kind), u32(pc or 0), u8(nmi_reads > 0 and 1 or 0), u32(reads),
        wait_entry and (u8(1) .. u16(wait_entry[1]) .. u16(wait_entry[2])) or (u8(0) .. u16(0) .. u16(0)),
        u8(0), u32(kind == 1 and movie.currentframe() or nmi_frame), u16(pad),
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

-- One record per step; then the next requests.
-- A measurement trace (OH_TRACE: hex addresses, comma-separated; OH_TRACE_LOG;
-- `oracle movie --trace` is the LakeSnes side): the beam at each watched
-- address, and a "cut" line after each record, so the client's steps delimit
-- the trace. lsnes fires exec callbacks on operand fetches too, so watch
-- instruction starts only. A write callback on $420B never fired here.
local trace_path = os.getenv("OH_TRACE_LOG")
local trace = {}

local function trace_cut()
    if trace_path == nil then return end
    trace[#trace + 1] = "cut " .. tostring(movie.currentframe())
    local file = io.open(trace_path, "a")
    if file then file:write(table.concat(trace, "\n"), "\n"); file:close() end
    trace = {}
end

local function send()
    trace_cut()
    local data = captured
    local file = assert(io.open(record_path, "wb"))
    assert(file:write(data))
    assert(file:close())
    io.stdout:write("R " .. #data .. "\n")
    io.stdout:flush()
    captured, reads, wait_entry = nil, 0, nil
    read_request()
end

local function stack_byte(s, offset)
    return memory.readbyte(0x7E0000 + (s + offset) % 0x2000)
end

-- The cut: the handler's RTI with S back at the NMI's S, so the RTI returns to
-- the interrupted instruction. The record holds the registers after the RTI:
-- it pops P, PC and PB, and with P.x set the index registers' high bytes clear.
-- PC is the interrupted instruction's address, the one the RTI returns to (an
-- exec callback at that instruction would report one past it).
memory.registerexec(BUS + NMI_RTI, function()
    if dangling then
        dangling, reads, wait_entry = false, 0, nil
        return
    end
    if not in_nmi then return end
    local r = memory.getregisters()
    if r.s ~= nmi_s then return end
    in_nmi = false
    if captured ~= nil then return end
    local p = stack_byte(nmi_s, 1)
    r.s, r.p = (nmi_s + 4) % 65536, p
    r.pc, r.pb = nmi_pc % 65536, math.floor(nmi_pc / 65536)
    if math.floor(p / 16) % 2 == 1 then r.x, r.y = r.x % 256, r.y % 256 end
    capture(0, nmi_pc, r)
    if owed then
        owed = false
        send()
    end
end)

memory.registerexec(BUS + NMI, function()
    if in_nmi then return end
    local s = memory.getregisters().s
    nmi_s = s
    nmi_pc = stack_byte(s, 2) + stack_byte(s, 3) * 256 + stack_byte(s, 4) * 65536
    in_nmi, nmi_reads, dangling = true, 0, false
    nmi_frame = movie.currentframe()
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
    local frame_pad = committed or pad
    for index = 0, 11 do
        input.set(0, index, math.floor(frame_pad / 2 ^ (15 - index)) % 2)
    end
    if reset_next then input.reset(); reset_next = false end
end)

on_frame_emulated = guarded(function()
    if save_path ~= nil then
        exec("save-state " .. save_path)
        save_path = nil
    end
    if captured == nil and in_nmi and next_pad ~= nil then
        owed, committed = true, next_pad
        return
    end
    if captured == nil then
        local r = memory.getregisters()
        if in_nmi then capture(2, nmi_pc, r) else capture(1, 0, r) end
    end
    send()
end)

-- The start state is in: apply the writes asked for before it loaded. A save
-- requested now would land a frame later (lsnes saves before it loads at a
-- boundary, and a save asked for here waits for the next one), so an edited
-- seed is its base state plus these writes (crates/oracle/src/lsnes.rs).
on_post_load = guarded(function()
    if not loading then return end
    loading = false
    for _, write in ipairs(pending_writes) do write_word(write[1], write[2]) end
    pending_writes = {}
end)

if trace_path ~= nil then
    -- Hits from this lsnes frame on (OH_TRACE_FROM); cut lines always.
    local trace_from = tonumber(os.getenv("OH_TRACE_FROM") or "0")
    for hex in string.gmatch(os.getenv("OH_TRACE") or "", "%x+") do
        local address = tonumber(hex, 16)
        memory.registerexec(BUS + address, function()
            if movie.currentframe() < trace_from then return end
            local r = memory.getregisters()
            trace[#trace + 1] = string.format("pc %06x %d %d", address, r.vcounter, r.hcounter)
        end)
    end
end

exec("set-speed turbo")
-- A resumed run. On the command line a state's core type (snes_ntsc) does not
-- match the ROM's (SNES) and lsnes hangs, so the script loads it: read-write,
-- so on_input still sets the pads, at the coming frame boundary, before any
-- frame is emulated.
local start = os.getenv("OH_START")
if start then exec("load-state " .. start) end
read_request()
