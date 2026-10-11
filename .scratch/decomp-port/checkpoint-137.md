# Ticket 137 resume checkpoint

Branch `decomp-port-integration`, refreshed on 2026-10-11. Ticket 137 remains
claimed and the broader decomp goal remains active. Earlier checkpoints:
`91c513c` (native RoomGfx/scheduler implementation), `de166b0` (allocation gap
and handoff). Current sources implement that missing native WithGfx bridge.

## Resume immediately

1. Inspect the existing fresh four-process full-route replay; do not duplicate it:
   - Run: `out/port/observed-projectile-with-gfx-live-full/20261011T142927.603436Z`.
   - Root process 878713 (`tools/port_live.py`), tool session 8416, gzip storage,
     129,712 inputs. Sources are frozen.
   - C0 finished 129,712 inputs with zero CPU fallback. C1 has started;
     mixed subjects follow. Read each subject's `replay.stdout` for progress.
2. Require the finished `mixed-C-Rust-integration` verdict with four matching
   inventories/progress, zero CPU fallback, all three native entries exclusively
   Rust with zero support refusals, and all nine scheduler IDs exclusively Rust.
   Require native per-ID allocations: 16 Ridley, four Ceres pads, four platforms,
   with first-two full-WRAM/eight-host-word allocation captures. Internal impact
   and split births are owned by the complete Rust scheduler.
3. Complete read-only material inspection:
   `observed-projectile-with-gfx-live-inspection.json`. Ownership-support agent
   owns it; compare all four subjects and their final verdict member hashes.
   Initial C0 captures prove highest-free allocation and Ceres/Ridley state.
4. After all writing processes finish, clean and authenticate the run:

   ```sh
   python3 tools/port_cleanup.py --root out/port/observed-projectile-with-gfx-live-full/20261011T142927.603436Z --apply
   python3 tools/port_cleanup.py --root out/port/observed-projectile-with-gfx-live-full/20261011T142927.603436Z --deduplicate --apply
   python3 tools/port_authenticate_live.py out/port/observed-projectile-with-gfx-live-full/20261011T142927.603436Z/verdict.json --output .scratch/decomp-port/observed-projectile-with-gfx-live-authentication.json
   ```

5. Record final admission/inspection/cleanup/authentication in ticket 137 and
   `out/port/observed-projectile-with-gfx-final/batch.json`. Resolve ticket 137
   only after all requirements pass. Commit final status/evidence separately.
   Continue the broader decomp effort; whole-game Rust ownership and original
   input/movie parity remain outstanding.

## Verified current implementation

- Native Ridley 93CA, Ceres A2EE/A31B/common A301 and no-op 950C implemented.
- `SpawnEprojWithRoomGfx`, native three-word `SpawnEprojWithGfx`, and `EprojRunAll`
  share immutable bounded support selection and transactional eight-word host
  publication. Unknown support selects C before mutation; accepted errors
  terminate without C retry. Canonical source validation and saturation retain
  native behavior. Same-bank calls route through the bridge.
- `WithGfx` has a genuine three-word actual-C prototype. The two-argument Rust
  fixture seam uses `packed_args: "eproj-with-gfx-v1"`, with parameter in a,
  enemy slot in b low16 and definition in b high16. Bit31 is preserved.
- 298 port tests and strict all-target Clippy pass. Changed Rust files pass
  formatting; earlier projectile/PLM/room formatting discrepancies remain.
  Seven compiled live ABI controls, six native oracle/host controls and 22
  remaining live/batch controls pass (one opt-in skip). Record:
  `observed-projectile-with-gfx-checks.json`.
- Fresh final offline batch:
  `out/port/observed-projectile-with-gfx-final/batch.json`.
  Six families / 13,624 actual-C cases authenticate after cleanup:
  5,409 initializer/allocation, 1,840 chained steps, 577 boundary cases and
  5,798 rendering/contact cases.
  Record: `observed-projectile-with-gfx-offline-authentication.json`.
- WithGfx read-only offline inspection independently verifies 1,530 native
  calls: 1,296 allocations across all 18 highest-free slots, 72 saturations
  changing only parameter, 162 actual-ROM B4B1 graphics OR controls. Full WRAM,
  returns and all eight host words match C/Rust; 9,182 sealed members checked.
  Record: `observed-projectile-with-gfx-offline-inspection.json`.

## Retained earlier live evidence

RoomGfx-only full replay:
`out/port/observed-projectile-corrected-live-full/20261011T135146.556016Z/verdict.json`.
Four subjects pass and authenticate 1,305 artifacts. Each mixed process executes
16 RoomGfx allocations, 95,325 Rust scheduler calls, all nine IDs exclusively
Rust, 866 nonempty/213 fireball scheduler frames, zero support refusals/CPU
fallback, 419 Rust decompressions and 125,529 Rust RNG calls. Sampled later
965E/9688/9696 captures match full WRAM/all eight host words across all four
subjects, including (22,120) impact point and vertical splits (22,106)/(22,134).
This retained frozen run predates the WithGfx bridge and cannot establish Ceres
live allocation ownership. Records: `observed-projectile-live-full-inspection.json`
and `observed-projectile-live-full-authentication.json`.

The earlier 45,000-input replay matches 607 artifacts and six scheduler IDs,
with zero refusals/fallback. Historical stopped/rejected diagnostic baselines
remain retained. `observed-projectile-full-capture-preflight.json` preserves the
wrapped collision-envelope diagnosis and correction.

## Cleanup and workspace

Current offline batch automatic dedup reclaimed 2,722,496,512 bytes; explicit
ordinary/dedup passes reclaimed zero. Test cleanup removed 517,703,381 disposable
Cargo bytes and 151,971 applied draft bytes; temporary test directories cleaned
automatically. Prior full live cleanup reclaimed 960,208,458 snapshot bytes;
post-cleanup 1,305-artifact authentication passed. Cleanup records are
`observed-projectile-cleanup.json`, `observed-projectile-with-gfx-checks.json`
and earlier build-cleanup records. Current live cleanup is pending because
processes still write the run; commands above complete it after termination.

Preserve shared `out/evidence`, subject pointers/readers, exact sealed sources,
binaries, ROM/native scripts, WRAM/host captures and diagnostic verdicts.
No implementer worktrees were created. Preserve unrelated
`.scratch/workspace-cleanup/`, `/tmp/sm-decomp-pipeline` and existing review/trace
worktrees. All agents share the workspace; keep frozen sources unchanged until
this replay completes.
