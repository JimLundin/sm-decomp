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
if os.getenv("SM_LSNES_BOUNDARIES") == "1" then
    events = assert(io.open(path .. ".events.csv", "wb"))
    events:write("event,movie_frame,kind,nmi,including_lag,state,room,x,y,joypad,scroll_frame,irq_handler,waiting_nmi,direction\n")
    local count = 0
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
            count = count + 1
            events:write(string.format("%d,%d,%s,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d\n", count,
                movie.currentframe(), kind, memory.readword(0x7E05B6), memory.readword(0x7E05B8),
                memory.readword(0x7E0998), memory.readword(0x7E079B),
                memory.readword(0x7E0AF6), memory.readword(0x7E0AFA), memory.readword(0x7E008B),
                memory.readword(0x7E0925), memory.readword(0x7E00AB), memory.readbyte(0x7E05B4), memory.readword(0x7E0791)))
            if count % 100 == 0 then events:flush() end
        end))
    end
end
exec("set-speed turbo")
exec("clear-pause-on-end")
