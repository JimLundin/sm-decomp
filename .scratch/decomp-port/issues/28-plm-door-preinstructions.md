# Native door shot and room-condition PLM preinstructions
Status: resolved
Blocked by: 27
Triage: ready-for-agent

Translate bank84 shot/missile/power-bomb door wakeups, dud handling, boss/event/
enemy-quota/statue conditions and boot sound-enable preinstruction. Preserve
link/timer writes, super-missile variable119, native queue and system scratch
writes. Use approved public full-WRAM actual-C seam with TDD and independent
examples. Integrate bounded control frame without claiming rendering ownership.
Record verification and cleanup before resolution.

## Completion evidence

Thirteen native preinstructions translated and public dispatcher registered;
bounded control frame now invokes supported door/room conditions before timer
processing, respecting deletion. Original shot masks, super-missile119overlay,
link/timer writes, native dud audio, system/event scratch and expiry preserved.

Actual bank84+bank80 C full-WRAM/raw-return evidence75routines/680cases at
out/port/plm-door-preinstructions/20261010T225127.240913Z/verdict.json.
Fifteen independent PLM examples, full port tests, fmt and Clippy pass.
Rejected overlay mistakes retained in preceding runs; source variables.h confirms
plm_variables0xdf0c differs from plm_variable0x1e17. Offline-only, rendering and
whole PLM coroutine ownership remain excluded.

Cleanup --apply reclaimed0bytes, --deduplicate --apply reclaimed209,846,272bytes
in out/port/plm-door-preinstructions. No worktree created; artifacts/rejections
retained, approximately40GBfree. Final evidence authenticated after cleanup.
