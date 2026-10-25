-- lsnes rr2-beta23: read-only per-frame observations, never change input/state.
-- Flat NMI counters include boot/reset and coroutine waits: this log does not classify lag.
-- input.get observes live controls; movie_subframe and WRAM joypad record playback evidence.
local path = assert(os.getenv("SM_LSNES_LOG"), "set SM_LSNES_LOG to a new output path")
local limit = tonumber(os.getenv("SM_LSNES_FRAMES"))
assert(limit == nil or (limit > 0 and limit == math.floor(limit)), "invalid frame limit")
for _, suffix in ipairs({"", ".polls.csv", ".events.csv", ".error.txt"}) do
    local existing = io.open(path .. suffix, "rb")
    if existing then
        existing:close()
        error("refusing to overwrite existing capture: " .. path .. suffix)
    end
end
local log = assert(io.open(path, "wb"))
log:write("input_frame,movie_frame,nmi_before,nmi_after,including_lag_before,including_lag_after,emulator_lag,game_state,room,x,y,input_callbacks,first_live_input,last_live_input,joypad_before,joypad_after\n")
local before = nil
local polls = assert(io.open(path .. ".polls.csv", "wb"))
polls:write("movie_frame,callback,subframe,nmi,including_lag,live_controller_word,joypad_latched,movie_subframe_index,movie_subframe\n")
local rows = 0
local last_frame = 0
local events = nil
local event_count = 0
-- Set to an existing, empty target/seeds/<movie> directory. Dumps stay local.
local seed_dir = os.getenv("SM_LSNES_SEEDS")
local seeds = nil
local seed_room = nil
local seed_pending = false
local seed_count = 0
local seed_origin_frame = nil
local seed_origin_subframe = nil
local seed_io_writes = {}
local seed_input_state = nil
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
    registers.seed_movie_frame = frame
    registers.seed_room = room
    registers.seed_nmi = memory.readword(0x7E05B6)
    registers.seed_boundary_frame = movie.currentframe()
    registers.seed_boundary_pc = 0x808348
    registers.seed_boundary_event = event_count
    assert(seed_input_state, "seed has no observed controller read")
    assert(seed_input_state.seed_input_frame <= registers.seed_boundary_frame,
        "seed controller observation is after the seed video frame")
    assert((seed_input_state.seed_input_nmi + 1) % 65536 == registers.seed_nmi,
        "seed controller observation is from another NMI")
    assert(seed_input_state.seed_input_room == room, "seed controller observation is from another room")
    assert(seed_input_state.seed_input_event <= event_count, "invalid controller event association")
    assert(seed_input_state.seed_input_debug == 0 and memory.readword(0x7E05D1) == 0,
        "debug controller normalization is unsupported")
    for _, address in ipairs({0x87, 0x89}) do
        assert(seed_input_state[string.format("seed_input_%04X", address)] == memory.readword(0x7E0000 + address),
            "controller repeat configuration changed after the read")
    end
    for name, value in pairs(seed_input_state) do registers[name] = value end
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
function on_input(subframe)
    assert(movie.readonly(), "recording movie must remain read-only")
    local frame = movie.currentframe()
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
    local movie_index = movie.current_first_subframe()
    local recorded = movie.get_frame(movie_index):serialize()
    assert(not string.find(recorded, "[,\r\n]"), "unexpected movie serialization")
    polls:write(string.format("%d,%d,%d,%d,%d,%d,%d,%d,%s\n", frame, before[4],
        subframe and 1 or 0, memory.readword(0x7E05B6), memory.readword(0x7E05B8),
        word, memory.readword(0x7E008B), movie_index, recorded))
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
    if rows % 1000 == 0 then log:flush(); polls:flush() end
    if frame == movie.framecount() or frame == limit then
        log:flush()
        log:close()
        polls:close()
        log = nil
        exec("quit-emulator")
    end
end
function on_quit()
    if seeds ~= nil then seeds:close(); seeds = nil end
    if log ~= nil then log:flush(); log:close(); polls:flush(); polls:close(); log = nil end
    if events ~= nil then events:close(); events = nil end
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
    assert(os.getenv("SM_LSNES_BOUNDARIES") == "1", "controller seed metadata requires boundary events")
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
if os.getenv("SM_LSNES_BOUNDARIES") == "1" then
    events = assert(io.open(path .. ".events.csv", "wb"))
    events:write("event,movie_frame,kind,nmi,including_lag,state,room,x,y,joypad,scroll_frame,irq_handler,waiting_nmi,direction\n")
    -- CPU-bus exec addresses in lsnes are 0x1000000 + SNES PC.
    -- ReadJoypadInputs runs inside ROM NMI, but at the next native update's start.
    for _, hook in ipairs({{0x808338, "wait_enter"}, {0x80834A, "wait_exit"},
            {0x858136, "no_update_enter"}, {0x858142, "no_update_exit"}, {0x809459, "joypad_begin"}, {0x80946A, "joypad_read"},
            {0x82E3C0, "place_enter"}, {0x82DF6C, "vblank_wait_enter"},
            {0x82DF70, "vblank_wait_exit"}, {0x82E4A8, "place_exit"},
            {0x82E4A9, "more_enter"}, {0x82E524, "scroll_wait_enter"},
            {0x82E52B, "scroll_wait_exit"}, {0x82E565, "more_exit"},
            {0x80AE4E, "scroll_irq_enter"}, {0x80AE73, "scroll_irq_exit"},
            {0x80987B, "irq_enter"}, {0x809602, "lag_nmi"}, {0x82E039, "door_dma"},
            {0x82E41D, "cre_decode_enter"}, {0x82E424, "cre_decode_exit"},
            {0x82E42E, "tiles_decode_enter"}, {0x82E435, "tiles_decode_exit"},
            {0x82E43F, "palette_decode_enter"}, {0x82E446, "palette_decode_exit"}}) do
        local kind = hook[2]
        memory.registerexec(0x1000000 + hook[1], guarded(function()
            assert(movie.readonly(), "recording movie must remain read-only")
            event_count = event_count + 1
            if seed_dir and kind == "joypad_begin"
                    and (seed_pending or memory.readword(0x7E079B) ~= seed_room) then
                -- This is the last observed read before the pending seed. Keep
                -- its exact producer state, not an inferred previous pad word.
                local registers = assert(memory.getregisters())
                assert(registers.d == 0, "nonzero controller direct page is unsupported")
                seed_input_state = {
                    seed_input_version = 1, seed_input_pc = 0x809459,
                    seed_input_event = event_count, seed_input_dp = registers.d,
                    seed_input_frame = movie.currentframe(),
                    seed_input_nmi = memory.readword(0x7E05B6),
                    seed_input_room = memory.readword(0x7E079B),
                    seed_input_debug = memory.readword(0x7E05D1),
                }
                for _, address in ipairs({0x87, 0x89, 0x8B, 0x8F, 0x93, 0x97, 0xA3}) do
                    seed_input_state[string.format("seed_input_%04X", address)] =
                        memory.readword(0x7E0000 + address)
                end
            end
            events:write(string.format("%d,%d,%s,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d\n", event_count,
                movie.currentframe(), kind, memory.readword(0x7E05B6), memory.readword(0x7E05B8),
                memory.readword(0x7E0998), memory.readword(0x7E079B),
                memory.readword(0x7E0AF6), memory.readword(0x7E0AFA), memory.readword(0x7E008B),
                memory.readword(0x7E0925), memory.readword(0x7E00AB), memory.readbyte(0x7E05B4), memory.readword(0x7E0791)))
            if event_count % 100 == 0 then events:flush() end
        end))
    end
end
exec("set-speed turbo")
exec("clear-pause-on-end")
