# Original-ROM feasibility for the restart ordinary-shot pilot

Date: 2026-10-16. Read-only research for restart ticket 05 and coordinator
`01a1414e-7fc1-73c3-969e-a7294824368a`. No build, replay, authentication or source
change was performed. Historical metadata below was inspected, not freshly
authenticated. The only new artifact is this note.

## Finding

There is a retained original-ROM capture reaching DF45 and Samus `(128,72)`.
There is **no directly reusable observed original shot lifecycle** in the
identified captures. The first gameplay evidence gap is observation granularity:
the longer capture has ten words at main-dispatch entry, with no projectile
entry/return state, actual input-read ledger or ordered shot effects. Native
`boot02` step 8379 cannot be identified with an original video frame.

## Retained evidence actually inspected

### Original-only attestation through 18,000 frames

Run: `/home/jim/sm-decomp/out/runs/20261010T082358.413496Z-53c5f3809a64/`.

- `verdict.json`: historical original-attestation pass, completed frames 18,000,
  15,184 events, `acceptance_eligible: false`. Its reason explicitly limits the
  result to the original capture.
- `generated/original.csv`: retained observations, claimed SHA-256
  `445d190a5f9c7b0ea62f5dd37291c0497ca7b4c29bcf37e4294e5e1782f270b7`.
- `generated/original.capture.json`, `generated/original.lua`,
  `logs/original-replay.json`, `manifest.json`: actual capture schema, observer,
  command/completion report, frame-label convention and provenance pointers.
- `generated/original-build/source/src/util/lsnes-dumpavi.util` exists locally
  (193,976,224 bytes). This is the attested capture's binary, distinct from the
  193,184,440-byte binary at `deps/lsnes/source/src/util/lsnes-dumpavi.util` used
  by the prefix path. Do not substitute one identity for the other.

Bounded samples from `generated/original.csv`:

| Original frame / event | Observed state |
| --- | --- |
| 8508 / 7742, line 7743 | game state 7, room DF45, X=128, Y=0, health=99 |
| 8539 / 7772 | game state 8, room DF45, X=128, Y=0 |
| 8640 / 7873, line 7874 | game state 8, room DF45, X=128, Y=72 |
| 8641 / 7874 and 8642 / 7875 | same ten-word location/health/item observations |
| 8645 / 7878 | X=130, Y=73: the original movie has already begun moving |

The final checkpoint is frame 17989/event 15184, room 9E9F. Completion through
18,000 comes from metadata, not an invented event at frame 18,000.

The observer records actual opcode fetch at `$82:8948`, **dispatch entry**, with
frame/event plus game state, room, X/Y, health, items and four clock words. It
does not record dispatch completion, held/pressed words, CPU call state, full
WRAM, projectile arrays or hardware effects. Matching this capture with an
instruction-level reference is not a C host-step input adapter. See
`crates/pipeline/src/checkpoints.rs:59`, `crates/pipeline/src/lsnes_observer.lua`
and `.scratch/recomp-readiness/issues/04-original-core-synchronization.md`.

### Causally mapped prefix through original frame 1600

Final historical result:
`/home/jim/sm-decomp/out/port/controller-terminal-seam/20261011T072946.717098Z/verdict.json`.
It explicitly reuses the native capture from
`out/port/controller-1600-frontier/20261011T072145.761041Z/verdict.json`.
Both directories exist. Final result records 1,097 native and candidate
completions and `matching_causal_prefix: 1097`, overall nonaccepting.

`native/events.csv` ends at original frame 1600/event 781083 with game state 30,
room 0. `native/state-781081.wram` exists and is 131,072 bytes. This capture
does not reach DF45. It has useful actual controller read pairs, source event
ordering and full WRAM at controller entry/return, boot clear and completed
dispatches; no projectile call observations exist in the hook contract.

The reusable design is in `crates/pipeline/src/decomp_prefix.rs:39` (hooks),
`:140` (observer construction), `:248` (poll mapping), `:415` (capture), and
`crates/pipeline/src/prefix_observer.lua.in`. Current request scope is hard
limited to 320..2000 frames at `decomp_prefix.rs:755`. It is unsuitable as an
unchanged command for the DF45 pilot.

### Longer legacy and local Ceres diagnostics

- `out/sync/full-any-lsnes.csv` reaches frame 129712/event 93694, room 91F8.
  `full-any-lsnes.capture.json` declares 129,712 frames, but omits the full
  provenance needed by current original reuse. Ticket 04 explicitly marks this
  legacy capture ineligible. Its same ten-word observer adds no shot evidence.
- `out/sync/ceres-input-lsnes.csv` includes duplicate frame 8508 polls and runs
  through 8800. Its Lua performs a **debug read** of `$80:4218` at controller
  entry; it does not retain actual hardware-read callbacks or return snapshots.
  Frame 8640 shows 0000, 8641 shows 8100, 8642 shows C180. These are diagnostic
  observations, not an admitted causal input map.
- `out/sync/ceres-lsnes.txt` is a 43,627,542-byte instruction trace covering
  8491..8520. Its supplied Lua enables trace at 8490. It ends before the
  observed center-platform state and contains no retained WRAM entry/return
  fixture contract. Only the first/last lines and observer were inspected.

## Exact reuse constraints

The two principal retained runs are **JSON-era** artifacts. Their
`verdict.toml` files do not exist. Current Rust `decomp-prefix` reuse reads
`verdict.toml` and validates current observer contract, executable, movie,
frame count and all native hashes (`decomp_prefix.rs:419–444`, `:835`). Current
readiness reuse likewise reads `verdict.toml`, `manifest.toml` and a TOML capture
sidecar (`readiness_sync.rs:641–679`). Do not advertise either retained directory
as a working argument to today's reuse commands without validated migration.
Also retain relocated provenance pointers: the 18,000-frame metadata references
its former `/tmp/sm-readiness-ticket-04` origin and `/home/jim/sm-recomp` assets.

The existing Rust capture orchestration is reusable source:
`crates/pipeline/src/replay.rs:48` launches the local lsnes binary with verified
movie ROM/core identity; `readiness_sync.rs:233` supplies the stronger attested
capture metadata. Both currently use the ten-word observer. A changed observer
and changed input movie are new capture identities, not reuse of an old verdict.

`tools/decomp/original_oracle.c` is not a ready projectile oracle: it hardcodes
the decompressor at `$80:B119`, accepts source/destination pointers, forbids
MMIO and interrupts and supplies a synthetic return frame. Its name does not
imply a general original gameplay runner.

## Smallest useful next experiment, not executed

1. Use the retained DF45 rows to bound an **original-only** capture around
   room entry and the center-platform state, roughly original frame 8640.
   Start from reset with a separately identified original-input scenario that
   preserves the movie's boot prefix, then holds position, fires once and waits
   for removal. Do not replay native step 8379 as a frame index or transplant
   the native WRAM snapshot into the original movie run.
   Use the already-present lsnes executable and generate a new observer into
   the experiment's `native/observe.lua` from Rust, following
   `decomp_prefix.rs:418`. No legacy JSON migration is needed for this fresh
   capture; migration is relevant only if actual old-evidence reuse is attempted.
2. Keep a passive source-order controller/dispatch ledger from reset, but write
   large per-call states only in the bounded shot interval. Reuse actual
   `$4218/$4219` read callbacks from the prefix observer; capture the real
   projectile entry and matching return, CPU call context, canonical WRAM and
   relevant ordered effects. Symbols are `$90:B887` FireUnchargedBeam,
   `$90:AECE` HandleProjectile, `$93:8254` DrawPlayerExplosions2 and `$93:834D`
   DrawBombAndProjectileExplosions (`out/symbols.toml:6554`, `:6500`, `:7639`,
   `:7640`). Return sites and call matching still require explicit validation.
3. Express firing as a named controller button, then use the appropriate movie
   encoding. Restart native `0x200` is in host bit order; original register
   words use another order (`restart-pipeline/runtime-interface.md:103` in the
   restart worktree). Verify what the original actually consumed.
4. The shortest comparison can be a **routine Fixture pair** using observed
   original entry state and call context in an independent C/Rust differential
   subject. It establishes only that routine result and effects. A fresh-reset
   mixed Live replay equivalence claim still needs causal mapping beyond frame
   1600 and equivalent scenario entry conditions. Preserve that distinction.

This needs a small Rust orchestration/reader and a bounded new observer contract,
not a broad readiness run or complete original movie resynchronizer. Cache the
new immutable original fixture/capture separately from the pilot's warm
actual-C loop. No original comparison is presently established for ticket 05.

## Completion evidence and cleanup

Read-only source/metadata inspection and bounded capture samples only. No
verification processes, run directories, extracted fixtures, worktrees or build
outputs were created; no cleanup utility invocation was applicable. Reclaimed
bytes: 0. `df -B1 /home/jim/sm-decomp` reported 107,793,477,632 bytes available.
All existing evidence dependencies were left untouched. No production or
restart code, ticket, map or acceptance record was edited.
