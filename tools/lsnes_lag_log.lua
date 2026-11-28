-- lsnes rr2-beta23: read-only per-frame observations, never change input/state.
-- Flat NMI counters include boot/reset and coroutine waits: this log does not classify lag.
local path = assert(os.getenv("SM_LSNES_LOG"), "set SM_LSNES_LOG to a new output path")
local limit = tonumber(os.getenv("SM_LSNES_FRAMES"))
assert(limit == nil or (limit > 0 and limit == math.floor(limit)), "invalid frame limit")
for _, suffix in ipairs({"", ".error.txt"}) do
    local existing = io.open(path .. suffix, "rb")
    if existing then
        existing:close()
        error("refusing to overwrite existing capture: " .. path .. suffix)
    end
end
-- The format crates/oracle/src/cli/lag_log.rs reads: a magic and a version, then per movie
-- frame its number and the words after it, little-endian.
local function word(value) return string.char(value % 256, math.floor(value / 256) % 256) end
local log = assert(io.open(path, "wb"))
log:write("SMLAGLOG", word(1))
local declared_frames = movie.framecount()
local rows = 0
local last_frame = 0
-- Set to an existing, empty target/seeds/<movie> directory. Dumps stay local.
local seed_dir = os.getenv("SM_LSNES_SEEDS")
local seeds = nil
local seed_room = nil
local seed_mb_phases = os.getenv("SM_LSNES_MB_PHASE_SEEDS") == "1"
local seed_mb_form = nil
local seed_pending = false
local seed_count = 0
local seed_origin_frame = nil
local seed_regions = {{"wram", 0x7E0000, 0x20000}, {"sram", 0x10000000, 0x2000},
    {"vram", 0x10000, 0x10000}, {"cgram", 0x21000, 0x200},
    {"oam", 0x20000, 0x220}, {"apuram", 0, 0x10000}}
if seed_dir then
    local existing = io.open(seed_dir .. "/rooms.toml", "rb")
    if existing then existing:close(); error("seed directory already has rooms.toml") end
    seeds = assert(io.open(seed_dir .. "/rooms.toml", "wb"))
end
local function capture_room(frame, boundary)
    if not seeds then return end
    local room = memory.readword(0x7E079B)
    if room ~= seed_room then
        seed_room = room; seed_pending = true; seed_origin_frame = nil
        seed_mb_form = nil
    end
    if seed_mb_phases and room == 0xDD58 then
        -- $7E7800 is Mother Brain's form (bank A9:B507), not her attack cycle.
        local form = memory.readword(0x7E7800)
        if form ~= seed_mb_form then
            seed_mb_form = form; seed_pending = true; seed_origin_frame = nil
        end
    end
    if not seed_pending or memory.readword(0x7E0998) ~= 8 then return end
    if not boundary then
        if not seed_origin_frame then seed_origin_frame = frame end
        return
    end
    if not seed_origin_frame then return end
    frame = seed_origin_frame
    assert(movie.readonly(), "seed capture must remain read-only")
    seed_count = seed_count + 1
    -- This is after the first gameplay frame; frame is the next zero-based input.
    for _, region in ipairs(seed_regions) do
        local name, base, size = region[1], region[2], region[3]
        local vma = assert(memory.find_vma(base), "missing seed memory region: " .. name)
        assert(vma.baseaddr == base and vma.size == size, "unexpected seed region: " .. name)
        local file = string.format("%04d-%04X-%s.bin", seed_count, room, name)
        local existing = io.open(seed_dir .. "/" .. file, "rb")
        if existing then existing:close(); error("refusing to overwrite seed: " .. file) end
        local output = assert(io.open(seed_dir .. "/" .. file, "wb"))
        local bytes = memory.readregion(base, size)
        local encoded = {}
        for index = 0, size - 1 do encoded[index + 1] = string.char(bytes[index]) end
        assert(output:write(table.concat(encoded)))
        assert(output:close())
    end
    seeds:write(string.format("[[seed]]\nroom_index = %d\nroom = \"%04X\"\nmovie_frame = %d\n\n",
        seed_count, room, frame))
    seeds:flush()
    print(string.format("Seed %d: room $%04X after movie frame %d", seed_count, room, frame))
    seed_pending = false
    seed_origin_frame = nil
end
function on_frame_emulated()
    assert(movie.readonly(), "recording movie must remain read-only")
    local frame = movie.currentframe()
    assert(frame == last_frame + 1, "nonsequential movie frame")
    capture_room(frame)
    log:write(word(frame % 65536), word(math.floor(frame / 65536)),
        word(memory.readword(0x7E05B6)), word(memory.readword(0x7E05B8)),
        word(memory.get_lag_flag() and 1 or 0), word(memory.readword(0x7E0998)),
        word(memory.readword(0x7E079B)), word(memory.readword(0x7E0AF6)),
        word(memory.readword(0x7E0AFA)))
    rows = rows + 1
    last_frame = frame
    if rows % 1000 == 0 then log:flush() end
    if frame == declared_frames or frame == limit then
        log:flush()
        log:close()
        log = nil
        exec("quit-emulator")
    end
end
function on_quit()
    if seeds ~= nil then seeds:close(); seeds = nil end
    if log ~= nil then log:flush(); log:close(); log = nil end
end
local function guarded(callback)
    return function(...)
        local ok, message = pcall(callback, ...)
        if not ok then
            local failure = assert(io.open(path .. ".error.txt", "wb"))
            failure:write(tostring(message), "\n")
            failure:close()
            exec("quit-emulator")
        end
    end
end
on_frame_emulated = guarded(on_frame_emulated)
if seed_dir then
    -- The video callback can suspend in the middle of a CPU opcode. Observe
    -- the next normal wait's PLB fetch instead, before its stack effects.
    memory.registerexec(0x1808348, guarded(function()
        capture_room(movie.currentframe(), true)
    end))
end
exec("set-speed turbo")
exec("clear-pause-on-end")
