# Pause equipment renderer source checkpoint

Worker: `/root/coverage_renderer`; actual thread
`01a13623-3b41-7822-b0b3-a99f73df84ac`.

Exact claim: `pause-equipment-render-offline`, revision
`53e916e7fc7edf90ffde5e953a18bcc43fb0ee931256ebd01e446757818f27f1`,
attempt 1, coordinator `decomp-run-20261013`.

Baseline: `3a8b9ea28b716d90df55f798941bba68e1df6531`.
Branch: `decomp-parallel-pause-equipment-render-offline-1`.
Worktree: `/tmp/sm-decomp-parallel-pause-equipment-render-offline-1`.
Source commit: `a026c883cbd651663c0945155fe409886306e384`.

## Achieved stage

All eight pinned-C routines are translated in readable Rust. Ten meaningful
Rust tests passed through `sm-pipeline exec` with the final source, exit 0.
The final matrix declares 247 full-WRAM/raw-return actual-C fixture pairs for
seven ABI-supported roots. Selector calls additionally exercise the three-word
animation root. The coordinator's exact-three-argument C gate is required to
admit that root directly. No actual-C verified or live/integrated stage is
claimed by this checkpoint.

Owned source remains frozen at these exact paths:

- `crates/port/src/pause_equipment_render.rs`
- `crates/port/tests/pause_equipment_render.rs`
- `tools/port/pause-equipment-render.toml`

No shared lib, catalog, harness or pipeline source was modified.

## Tests and fixtures

[Test request](test-exec.toml), [exit receipt](tests.stdout.toml),
[test output](renderer-tests.stdout), [compiler output](renderer-tests.stderr)
and [timing receipt](renderer-tests.timing.toml) are retained here. Source
hashes and package/run ingestion belong to existing pipeline/evidence tools;
this worker used no manual hash bookkeeping.

247 fixture pairs: menu 54, selector 87, public reserves 48, reserve helper 8,
palette 24, wireframe 8, wireframe+queue 18. Original ROM identity is inherited
from the established manifest seal. Actual providers are `sm_82.c`,
`sm_81.c` and `sm_rtl.c`; no routine C stubs. Continuations chain selector,
reserve animation and wireframe queue state.

The supported input domain is explicit: OAM cursor 0..511 (including odd
aliases), used animation kinds 2/3/6..9 and legal variants/frames, all 16
equipment category rows, reserve amount <= capacity <= 500, and VRAM queue
entries ending before WRAM 0x330. Whole-call preflight rejects bad selected
assets/state unchanged; live pause caller, NMI and hardware acceptance remain
pending. Animation variants intentionally read the low WRAM byte exactly as C.

## Next command and cleanup

The parent owns verification writers, merged-source registration, queue/result
admission and progress publication. It will run the generic family gate with
`--checkout /tmp/sm-decomp-parallel-pause-equipment-render-offline-1 --source
/home/jim/sm-decomp/deps/snesrev-sm`, then the exact direct animation gate.
The worker has no running verification process. Keep this worktree and source
until those gates finish and both cleanup passes preserve/authenticate evidence.

The scratch asset/fixture executables were removed after their processes
finished. Test stdout/stderr/timing/metrics were moved here; the source
worktree is clean. The worktree build target remains disposable. The authored
Rust asset/fixture helper sources in this directory are small source records.
The integration owner
removes disposable builds and retires the worktree after retaining any sealed
source dependencies and authenticating finished evidence. Final cleanup remains
the parent gate; no cleanup completion or gameplay acceptance is claimed here.
