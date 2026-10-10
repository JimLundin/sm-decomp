# Native PLM room and trigger preinstructions
Status: resolved
Blocked by: 30
Triage: ready-for-agent

Translate remaining room lava, speed booster, Shaktool, escape, triggered block,
Samus proximity, door-bit, Metroid quota, keypress and position preinstructions.
Preserve memory aliases and execution through the frame engine. Verify valid
inputs against actual C and independent Rust tests; spawning-dependent escape
closure requires native enemy-projectile allocation and remains explicit work.
Complete cleanup and record evidence before resolution.

## Completion evidence

Integrated 02425a6: 21 room/gameplay preinstructions covering native lava stages,
booster escape, Shaktool scroll/event changes, escape liquid motion, triggers,
Samus position/proximity, door bits, Metroid quota, keypress and distinct room
argument missile counts. Shared native coordinate divider, valid lava stages
0/6/12/18, and door-bit validation before scratch mutation. Stored BBDC/D779
preinstructions remain unsupported as actual C dispatcher lacks those cases.
Spawning-dependent B927 remains explicitly unported for next allocation closure.

Native single-block drawing instruction 8B17 now builds original RAM draw data,
updates level data and queues rendering using global active slot; proximity
instruction 8D41 now uses actual byte-width SNES division, including zero width.

Actual-C setup/handler evidence:
out/port/plm-room-conditions/20261010T230206.177568Z/verdict.json
passes 96 routines/1,184 full-WRAM/raw-return cases (504 new).
Frame/instruction evidence:
out/port/plm-room-frames/20261010T230207.358573Z/verdict.json
passes six seams/1,368 cases (252 new frame cases plus16 block-draw cases).
Seven new independent condition tests and literal block-draw test pass;
full Rust suite, fmt and Clippy pass. Offline only.

Finished-run cleanup --apply reclaimed 0 bytes for each scope; deduplication
reclaimed 358,875,136 bytes in plm-room-conditions and 217,972,736 bytes in
plm-room-frames. Removed integrated /tmp/sm-plm-conditions worktree including
277,194,695 bytes disposable target. Both evidence sets authenticated afterward;
small fixture assets retained, 38GB free.
