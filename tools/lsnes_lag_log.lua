-- lsnes rr2-beta23: read-only per-frame observations, never change input/state.
-- Flat NMI counters include boot/reset and coroutine waits: this log does not classify lag.
-- input.get observes live controls; the WRAM joypad word records playback evidence.
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
local log = assert(io.open(path, "wb"))
log:write("input_frame,movie_frame,nmi_before,nmi_after,including_lag_before,including_lag_after,emulator_lag,game_state,room,x,y,input_callbacks,first_live_input,last_live_input,joypad_before,joypad_after\n")
local declared_frames = movie.framecount()
local before = nil
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
local seed_origin_subframe = nil
local seed_io_writes = {}
local seed_regions = {{"wram", 0x7E0000, 0x20000}, {"sram", 0x10000000, 0x2000},
    {"vram", 0x10000, 0x10000}, {"cgram", 0x21000, 0x200},
    {"oam", 0x20000, 0x220}, {"apuram", 0, 0x10000}}
if seed_dir then
    local existing = io.open(seed_dir .. "/rooms.csv", "rb")
    if existing then existing:close(); error("seed directory already has rooms.csv") end
    seeds = assert(io.open(seed_dir .. "/rooms.csv", "wb"))
    seeds:write("room_index,room,movie_frame,next_input_index,movie_subframe_index,region,size,sha256,file\n")
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
        if not seed_origin_frame then
            seed_origin_frame = frame
            seed_origin_subframe = movie.current_first_subframe()
        end
        return
    end
    if not seed_origin_frame then return end
    frame = seed_origin_frame
    assert(movie.readonly(), "seed capture must remain read-only")
    seed_count = seed_count + 1
    local register_file = string.format("%04d-%04X-registers.csv", seed_count, room)
    local existing = io.open(seed_dir .. "/" .. register_file, "rb")
    if existing then existing:close(); error("refusing to overwrite seed: " .. register_file) end
    local registers = assert(memory.getregisters(), "core has no register observations")
    registers.seed_boundary_pc = 0x808348
    -- DMAP..NTRL are passive readable registers in the pinned bsnes core.
    for channel = 0, 7 do
        for offset = 0, 10 do
            registers[string.format("dma_%d_%d", channel, offset)] =
                memory.readbyte(0x1004300 + channel * 16 + offset)
        end
    end
    for name, value in pairs(seed_io_writes) do registers[name] = value end
    local names = {}
    for name, _ in pairs(registers) do names[#names + 1] = name end
    table.sort(names)
    local output = assert(io.open(seed_dir .. "/" .. register_file, "wb"))
    output:write("name,value\n")
    for _, name in ipairs(names) do
        local value = registers[name]
        if type(value) == "boolean" then value = value and 1 or 0 end
        output:write(string.format("%s,%d\n", name, value))
    end
    assert(output:close())
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
        seeds:write(string.format("%d,%04X,%d,%d,%d,%s,%d,%s,%s\n", seed_count,
            room, frame, frame, seed_origin_subframe, name, size,
            memory.hash_region(base, size), file))
    end
    seeds:flush()
    print(string.format("Seed %d: room $%04X after movie frame %d", seed_count, room, frame))
    seed_pending = false
    seed_origin_frame = nil
end
function on_input()
    assert(movie.readonly(), "recording movie must remain read-only")
    local frame = movie.currentframe()
    assert(frame <= declared_frames, "unexpected input after declared movie end")
    local word = 0
    for index = 0, 11 do
        if input.get(0, index) ~= 0 then word = word + 2 ^ (15 - index) end
    end
    if before == nil then
        assert(frame == last_frame + 1, "nonsequential movie frame")
        before = {frame, memory.readword(0x7E05B6), memory.readword(0x7E05B8),
            0, word, word, memory.readword(0x7E008B)}
    end
    assert(frame == before[1], "new movie frame before prior output")
    before[4] = before[4] + 1
    before[6] = word
end
function on_frame_emulated()
    assert(before ~= nil, "frame output without corresponding input boundary")
    local frame = movie.currentframe()
    assert(frame == before[1], "movie frame changed during emulation")
    capture_room(frame)
    log:write(string.format("%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d\n",
        frame - 1, frame, before[2], memory.readword(0x7E05B6),
        before[3], memory.readword(0x7E05B8), memory.get_lag_flag() and 1 or 0,
        memory.readword(0x7E0998), memory.readword(0x7E079B),
        memory.readword(0x7E0AF6), memory.readword(0x7E0AFA),
        before[4], before[5], before[6], before[7], memory.readword(0x7E008B)))
    rows = rows + 1
    last_frame = frame
    before = nil
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
on_input = guarded(on_input)
on_frame_emulated = guarded(on_frame_emulated)
if seed_dir then
    -- The video callback can suspend in the middle of a CPU opcode. Observe
    -- the next normal wait's PLB fetch instead, before its stack effects.
    -- bsnes has incremented PC for this fetch; retain both observed PC and
    -- the explicit hook address so the importer need not guess a PC shift.
    memory.registerexec(0x1808348, guarded(function()
        capture_room(movie.currentframe(), true)
    end))
    -- These write-only registers cannot be reconstructed by reading the bus.
    -- Observe every CPU-bank mirror, without changing the original write.
    for _, address in ipairs({0x4200, 0x4201, 0x4207, 0x4208, 0x4209, 0x420A, 0x420D}) do
        local name = string.format("io_%04X", address)
        for bank = 0, 0xBF do
            if bank < 0x40 or bank >= 0x80 then
                memory.registerwrite(0x1000000 + bank * 0x10000 + address,
                    guarded(function(_, value)
                        assert(movie.readonly(), "register observation must remain read-only")
                        seed_io_writes[name] = value
                    end))
            end
        end
    end
end
exec("set-speed turbo")
exec("clear-pause-on-end")
