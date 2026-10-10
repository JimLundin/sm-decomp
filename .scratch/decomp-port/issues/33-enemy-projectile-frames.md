# Native enemy-projectile movement and frame execution
Status: resolved
Blocked by: 32
Triage: ready-for-agent

Translate bank86 shared movement, instruction interpreter and bounded frame
scheduling with room-effect preinstructions. Preserve subposition byte writes,
raw movement returns, relative pointers, timer underflow, descending slots,
global index aliases, palette/audio and deletion ordering. Verify actual pinned
C full WRAM and raw pointer returns; malformed inputs never define C oracle.
Record remaining collision/spawning closures and complete cleanup.

## Completion evidence

Integrated6529e15/79aafd3:45 supported bank86 control/audio/palette/room opcodes,
shared x/y/xy movement, bounded descending scheduler and ten preinstructions.
Glass shard sparkle spawning now includes native CE6D initializer. Fractional
movement preserves shared high-byte-only writes versus full hilo room motion;
timer/deletion behavior, global indices and same-frame lower-slot spawn preserved.
Eye projectile collision, further bank-specific AI and unsupported opcodes remain
explicit errors. Offline ownership does not claim live scheduler installation.

Actual pinned full-tree C evidence:
out/port/enemy-projectile-frames/20261010T231226.808149Z/verdict.json
passes748cases across six seams:360instruction,96movement,160preinstruction,
132frame cases. Six independent Rust examples include fractional carry, odd
relative control, room motion, spawn-and-run same frame, disabled frame and
budget rejection. Full suite/fmt/Clippy pass; adapter integrity test passes.
Two rejected C build runs retained (relative include then unguarded header
duplication), corrected by including actual bank86 translation unit once.

Cleanup --apply reclaimed0bytes; --deduplicate --apply reclaimed187,170,816
snapshot bytes. Removed integrated /tmp/sm-eproj-frame worktree and302,894,616
bytes disposable target. Accepted evidence authenticated after cleanup. Small
fixture ROM retained in out/port/eproj-frame-fixtures;37GBfree. Integrity test
uses temporary directories which were automatically removed after completion.
