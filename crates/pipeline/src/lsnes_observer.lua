local event = 0
local function checkpoint()
    event = event + 1
    local parts = {tostring(movie.currentframe()), tostring(event)}
    for _,address in ipairs(fields) do
        parts[#parts+1] = string.format('%04X', memory.readword('WRAM', address))
    end
    out:write(table.concat(parts, ','), '\n')
    out:flush()
end
memory.registerexec('BUS', 0x828948, checkpoint)
