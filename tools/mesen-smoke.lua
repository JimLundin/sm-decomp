-- Headless setup check. Run with tools/mesen --testRunner tools/mesen-smoke.lua rom/sm.sfc --timeout=30
local frames = 0
emu.addEventCallback(function()
    frames = frames + 1
    if frames == 120 then
        print("mesen: ran 120 frames; WRAM[0]=" .. emu.read(0, emu.memType.snesWorkRam))
        emu.stop(0)
    end
end, emu.eventType.endFrame)
