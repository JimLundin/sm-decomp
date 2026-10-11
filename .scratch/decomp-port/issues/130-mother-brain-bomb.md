# Mother Brain bomb lifecycle
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate pinned C482 bomb allocation, C4C8 update, C564 bomb-overlap
destruction and C5C2 wall/floor movement. Preserve wrapping counters, byte
scratch aliases, signed velocity damping, all ten acceleration stages,
explosion and enemy-drop allocation with full external projectile host state.
The native routine collides with Samus's bombs; direct Samus-body contact is
handled by the common scheduler and must not be invented in this callback.
Use even projectile slots 0..34, acceleration index F>>1 below ten, and
unmodified ROM. Verify actual-C allocation, direct lifecycle and scheduled
frames with full WRAM/raw returns/eight host words. Retain sealed evidence
and finish scoped cleanup/authentication. Live ownership remains separate.

## Comments

Implementation assigned to mother_brain_bomb subagent; shared allocator,
dispatcher and module registration are integration-owner changes.

## Completion evidence

Added `crates/port/src/mother_brain_bomb.rs` with native C482 initializer,
C1B8 rectangle/bomb overlap as part of C564 destruction, C4C8 lifecycle and
C5C2 wall/floor movement. Allocator, direct native entry points and host-aware
descending scheduler hooks are integrated. Preserves low-byte scratch writes,
counter underflow/overflow, signed-minimum velocity damping, all ten native
acceleration entries including odd F values, unsigned rectangle overlap,
wall-before-floor reversal, slot reuse, final explosions, audio quotas and
all eight external host words. Missing host rejects before scheduler writes.
There is no direct Samus-body collision in this pinned callback. Real ROM
EC3F enemy-drop weights are `[0,0,0,255,0,0]`; the translated callback calls
the existing native drop allocator and publishes the header, preserving its
actual no-pickup outcome rather than inventing a reward.

All 7,130 frozen-current-source actual-C full-WRAM/raw-return/eight-host-word
cases pass:

- Allocation plus direct byte-alias initializers, 556:
  `out/port/mother-brain-bomb-allocation/20261011T120705.497296Z/verdict.json`.
- Movement boundaries and wrapping acceleration, 721:
  `out/port/mother-brain-bomb-movement/20261011T121418.267478Z/verdict.json`.
- Bomb overlap, source filtering and destruction/drop pool pressure, 2,341:
  `out/port/mother-brain-bomb-collision/20261011T121418.253867Z/verdict.json`.
- All stages, damping boundaries, native updates and final explosions, 3,187:
  `out/port/mother-brain-bomb-update/20261011T121418.255570Z/verdict.json`.
- Host-aware scheduled execution, 325:
  `out/port/mother-brain-bomb-frames/20261011T120705.311214Z/verdict.json`.

Reproduce with `python3 tools/port_bomb_fixtures.py`, then run each generated
manifest with `python3 tools/port_plm_instructions.py --manifest
tools/port/<family>.json --source deps/snesrev-sm --output out/port/<family>`.
The split retains every originally declared lifecycle case; each new family
has its own dispatcher control, so the aggregate contains two additional
control cases.

Seven independent bomb tests pass, including high-byte alias preservation,
signed-minimum damping, wall/floor order and host-aware scheduler allocation
into a reused slot and a lower slot in the same frame. The integration owner
verified all 263 port tests, strict all-target Clippy and 29 Python harness
tests (one opt-in test skipped). New module and test rustfmt checks pass.
The initial synthetic scheduler regression used nonexistent no-op 8003;
its fixture was corrected to native 8170 before final verification and passes.

The initial 6,247-case combined lifecycle process terminated externally with
exit 143 before sealing a verdict. Authoritative process inspection proved
the process absent before restarting as three bounded families. Preserve
`out/port/mother-brain-bomb-lifecycle/20261011T120705.377973Z/interrupted-diagnostic.json`
and its 68,994 hashed artifacts/5,746 intermediate case directories as
nonaccepting interruption diagnostics. All diagnostic hashes authenticate;
no gameplay mismatch was reported, and these partial states are not proof
of accepted coverage.

For all five finished output roots and the interrupted lifecycle root ran
`python3 tools/port_cleanup.py --root <root> --apply` followed by
`--deduplicate --apply`. Ordinary cleanup reclaimed zero bytes throughout.
Finished-family dedup reclaimed 102,760,448 allocation bytes, 94,502,912
movement bytes, 482,607,104 collision bytes, 506,200,064 update bytes and
43,384,832 frame bytes: 1,229,455,360 bytes total. The unsupported interrupted
format contains no identified unsealed build tree, so zero-byte cleanup
retains its binaries, snapshots and case states for diagnostics. No separate
implementer worktree was created. Retained evidence dependencies are source
snapshots, sealed binaries, native ROM and all full-state/host sidecars.
All five final runs and the interruption inventory authenticate after cleanup;
results are in `.scratch/decomp-port/bomb-authentication.json`. Free space
at final authentication: 278,433,026,048 bytes.

This is offline equivalence. Live bomb ownership and original-ROM parity
remain unproved; broad decomp translation remains active.
