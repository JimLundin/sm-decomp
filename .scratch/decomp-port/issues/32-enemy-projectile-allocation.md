# Enemy-projectile allocation and PLM room effects
Status: resolved
Blocked by: 31
Triage: ready-for-agent

Translate the bank86 highest-free-slot enemy-projectile allocator and room-effect
initializers, preserving definition fields, original partial resets, parameters,
RNG and coordinate aliases. Connect native PLM escape spawning and supported
effect instructions. Full canonical WRAM/actual pinned C is the correctness
oracle; external spawn globals require explicit context rather than fabricated
WRAM aliases. Verify valid inputs and cleanup runs/worktree before resolution.

## Completion evidence

Integrated 1d0a28d: eighteen-slot highest-free allocator, room/enemy graphics
APIs, enable/disable/clear and nine initializers: escape wall, Torizo statue,
eye projectile/sweat/smoke, Mother Brain glass shard, tube crack/shards/bubbles.
Definition fields, partial resets, RNG and coordinate scratch preserved.
Supported parameter bounds checked before mutation; full pool still writes
init parameter. External spawn-point-dependent initializers remain unsupported.

B927 escape preinstruction now uses native spawning through ROM-aware dispatch.
Eight PLM spawning instructions call native allocation, including four glass
shards and two smoke clouds, preserving audio and partial pool exhaustion.

Actual-C full-tree evidence:
out/port/enemy-projectile-allocation/20261010T230633.151973Z/verdict.json
passes303cases. PLM evidence:
out/port/plm-projectile-effects/20261010T230714.137767Z/verdict.json
passes1,444cases including64new effect instructions and12escape frames.
Four independent allocator tests, full Rust suite, fmt and Clippy pass.
Rejected generator overlap run retained; effect scripts moved away from renderer
data. Fixture ROM now includes pinned effect tables alongside synthetic scripts.
Offline only; enemy-projectile scheduling/movement remains unported.

Cleanup --apply reclaimed0bytes; deduplication reclaimed77,332,480bytes in
enemy-projectile-allocation and235,274,240bytes in plm-projectile-effects.
Removed integrated implementer worktree including289,782,611bytes of target.
Both accepted evidence sets authenticated after cleanup; rejected diagnostic
evidence and small fixture inputs retained. 38GBfree.
