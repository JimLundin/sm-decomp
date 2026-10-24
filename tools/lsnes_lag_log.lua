-- lsnes rr2-beta23: read-only per-frame observations, never change input/state.
-- Flat NMI counters include boot/reset and coroutine waits: this log does not classify lag.
-- input.get observes live controls; movie_subframe and WRAM joypad record playback evidence.
local path = assert(os.getenv("SM_LSNES_LOG"), "set SM_LSNES_LOG to a new output path")
local limit = tonumber(os.getenv("SM_LSNES_FRAMES"))
assert(limit == nil or (limit > 0 and limit == math.floor(limit)), "invalid frame limit")
for _, suffix in ipairs({"", ".polls.csv", ".error.txt"}) do
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
exec("set-speed turbo")
exec("clear-pause-on-end")
