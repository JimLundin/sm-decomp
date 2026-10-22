# Engine reduction handoff — 2026-10-22

Read [PLAN.md](PLAN.md) first; it remains authoritative. This handoff records
the interruption point at the user's request, not a completed refactor.

## Latest continuation point: terminal projectile behavior

This update supersedes the interruption state below. Shared room scans are
adopted. Clearing, impact and retirement now use `projectile_lifecycle` and its
checked RAM adapter; the three public terminal functions and both new modules
forbid unsafe code. The owned draft shares the impact/retirement rules and
preserves generic projectile kinds/cursors and complete owner transfers.
Block reactions and the broader game's unsafe operations remain unfinished.

Main is 101,537 Rust lines (+186 for this change), with 2,940 counted unsafe
declarations (-3) and 6,160 cast lines (-5). The owned draft removes 48 lines;
the complete prospective composition grows by 138 to 110,602. Its remaining
+9,065 overhead is unadopted. This is safety/architecture work, not volume savings.
The full prospective Rust 2024 headless library builds (45.49s).

Final checks: 1,137 actual clearing/retirement comparisons, 349 owned transitions,
524 actual animation comparisons and 55 short-image checks; relevant room,
flight, session and beam suites also pass. All 141 runtime and 200 draft compiled
inputs match. Normal production release library/frontend/verifier builds and
DLL source checks pass. No movie ran; PLAN records the coverage and limits.

Frozen source/results are at the experiment root below under
`runtime-impact-stage/`; working copies remain in `runtime-impact-working/`.
Use `impact-final-results.json`, `main-source-check-impact.json`,
`impact-production-source.json` and `impact-composition-price.json`. The
prospective source is `impact-composition/`, with its built library saved as
`impact-composition.rlib`. Earlier room checks remain in `runtime-room-stage/`.
Do not overwrite these sources from the older encounter snapshot.

Next replace the remaining duplicated flight algorithms, migrate consumers
and delete retained implementations. Preserve the full game, exact behavior,
owned-state destination and substantial reduction objective. The nightly
constraint and tools below are unchanged. No compiler or subagent is left running.

## Historical interruption point

## Objective and current state

Preserve the complete game and exact behavior, including quirks, while reducing
implementation volume, extracting content as data and eliminating all unsafe
operations. The accepted destination is an owned deterministic Session; checked
RAM access is a transitional adapter. PLAN item 4, retiring bank ownership by
gameplay responsibility, is the earliest unfinished item. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
ownership or backend interfaces.

Implementation HEAD at handoff: `af93d627` on `main`. Recent commits:

- `af93d627`: priced the prospective projectile integration; Rust 2024 headless
  library builds, but the integration is unadopted.
- `07f77021`: clarified encounter transfers; 127 comparisons pass.
- `9b81b794`: checked beam firing RAM and bound launch tails; 338 comparisons
  and normal release/source checks pass.

The only source worktree change is **untracked**
`sm/src/game/projectile_room.rs`. It contains a first draft of a safe shared
scan algorithm. It is **unformatted, uncompiled, unverified and unregistered**
in `lib.rs`; production does not call it. Preserve it for review, not adoption
without checks. No production adapter or owned adapter has been implemented.

The pending tool cell `g2:495` completed successfully during handoff. It copied
`runtime-encounter-stage` to `runtime-room-working`, saved current `sm_94.rs` as
`room_before.rs`, and saved draft `room_collision.rs`, `flight.rs` and `harness.rs`
under `room-parent/`. Do not repeat that copy over subsequent work.

No subagent is working. The Rust migration agent completed; gameplay work was
single-agent. No compiler was left running by this work. Do not start additional
agents unless the user or applicable instructions authorize that work.

## Volume and safety accounting

Committed baseline: 101,274 Rust lines / 131 files; 101,836 maintained
implementation lines; 102,667 including installers and added verifier launcher.
Counters: 2,952 unsafe declarations, 6,165 cast-containing lines, 21 raw offsets,
7 extern C blocks, 6 C ABI exports, 65 no_mangle occurrences. These are residue
metrics, not a count of every unsafe operation.

The edition migration added **4,909 lines** for explicit unsafe scopes and
formatting. It exposed existing operations; it did not eliminate them.
The unadopted owned-state integration adds **9,211 lines** because it retains
legacy algorithms while adding owned models, transfers and adapters. It prices
101,274 -> 110,485 Rust lines: 83 added files / 9,367 lines, minus 156 net
existing lines, with six unchanged modules reused. Do not adopt its outer
coordinators alone. The substantial reduction requested has not been achieved.
The handoff progress check reports 101,411 Rust lines: committed 101,274 plus
the 137-line untracked scan draft. Other counters are unchanged. This temporary
increase has no verified replacement saving yet.

## Next focused change: share projectile room scans

Replace both main's `ShotAxis`, `beam_block_collision` and
`missile_block_collision` scan logic in `sm_94.rs`, and the owned draft's
`Firing::collide_room_with`, with one algorithm plus storage/effect adapters.
Remove the superseded bodies; price the shared kernel and both adapters.

The new `projectile_room.rs` declares `Axis::{X,Y}`, `Field`, `Kind`, `Scan`,
`State`, `Context<O>`, `span` and `collide`. `Context` consumes and returns the
owner for synchronous reactions and retirement. It uses `#![forbid(unsafe_code)]`.
Audit the draft against actual retained code before continuing.

For the RAM adapter, use scoped checked byte slices and native-endian words:

| Field | X base | Y base |
| --- | --- | --- |
| Position | 0xb64 | 0xb78 |
| Fraction | 0xb8c | 0xba0 |
| Radius | 0xbb4 | 0xbc8 |
| Speed | 0xbdc | 0xbf0 |

Index displacement is `2 * usize::from(slot >> 1)`. `ScreenHigh` reads the
physical byte at position base + displacement + 1, independently of native word
order. Room width and scroll fields remain little-endian.

Match `rtl::AddToHiLo`: read low and high before either write, add the unsigned
representation of signed displacement with wrapping arithmetic, then write low
before high. Preserve broad and odd selectors, overlapping fields, live rereads,
native versus LE access, callback order and failure prefixes.

A possible adapter is `RamShot(&mut Game)` implementing `State`, with a separate
context converting all `Scan` fields to/from stack `CollInfo`, calling actual
`BlockShotReactHoriz/Vert` and `KillProjectile`. Keep remaining unsafe callbacks
explicit. Legacy reactions are synchronous. Audited shot reactions call
`SpawnPLM`, not the movement path's `SpawnPLMWithCollision`; no observed scan
pointer escape was found. Still preserve every field a callback can modify.

Do not delete `ShotAxis` support indiscriminately: `pair`, `radius` and
`shot_word` also serve `shot_square_slope`. The latter can use checked native
reads. Reusing shared `Axis as ShotAxis` requires changing Horizontal/Vertical
variants to X/Y and retaining needed offset methods. Previous reference search
found no callers of `BlockGetSomePos1/2`; repeat before removal.

In the owned draft, reexport the shared Axis and Scan, implement shared `State`
for `Firing<C,S,T,U,K>`, and adapt existing `Context<Self>` and `retire_with`.
Keep canonical-slot admission and the broad-slot `External::Scan` fallback,
which relinquishes the complete owner. Never retain typed authoritative state
through RAM-authoritative legacy execution. Keep the existing impact/retirement
model; this change shares scans.

Order-sensitive cases include horizontal row calculation before movement,
fraction before position writes, missile X's otherwise unused fraction read,
non-wave Y column-before-width versus wave width-before-column, span rejection
after movement, physical wave high-byte checks, live width after each horizontal
reaction, callback mutation of counters before decrement, beam testing
`unblocked` versus missile testing the reaction return, and retirement ownership.

## Local work and validation

Experiment root (outside Git):
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/`.

- `runtime-room-working/`: mutable continuation directory and saved parents.
- `runtime-encounter-stage/`: frozen encounter inputs/results. Reported results
  are `runtime_encounter_experiment [127,801,8,117,100]` and
  `runtime_session_experiment [170,1823,53,97]`. Actual drain/release/HUD/X-ray,
  synthetic content and controlled hardware; no scheduler encounter or HDMA
  playback/movie. 138 runtime and 194 draft inputs checked; 26 retained bodies
  linked to main through the audited edition migration.
- `runtime-beam-stage/`: last adopted beam checks, 338 comparisons.
- `integration-cost-stage/`: full prospective source, `prepare.py`,
  `composition-price.json`, `source-hashes.json`, logs, `libsm.rlib`, `HANDOFF.txt`.
  Optimized Rust 2024 headless library build passed in 47.80s; no runtime
  comparison, frontend/verifier build or movie for that prospective composition.

The inherited comparison carrier is baseline `d03f819d` plus explicit adapters,
edition 2021. It is not the complete current engine, whose production edition is
2024. Relevant retained main scans correspond through audited migration
`e88ff8da`. Expose the shared module publicly only in the experiment carrier if
needed. Recheck source correspondence on final inputs.

Useful fixtures: `flight_checks.rs`, `old_room_collision.rs`,
`runtime_collision_checks.rs`, `runtime_flight_checks.rs`,
`runtime_session_checks.rs`, `runtime_beam_checks.rs`. Compare captured old code
to both shared RAM and owned paths. Cover broad/odd slots (including 0xfffe),
aliases into room fields, fraction carry/wrap, signed speeds, zero/large radii,
width truncation, every axis/kind, wave bounds, callback mutations/failures,
live width changes, captured versus live selectors and actual room/PLM effects.
Re-run relevant flight/session checks after integration.

No movie segment is established for this scan change. Follow PLAN's Validation
strategy: source-checked constructed real-code comparisons may cover it with
explicit limits; unrelated green frames add no coverage. Adopted behavior changes
also need final formatting, normal release library/frontend/verifier builds and
loaded DLL source checks. No scan checks have run yet.

## Tools and continuation rules

Use PowerShell with `login:false`, `rg` for searches and one heavy compiler at a
time (`-j 1`). Recheck HEAD and index before every Git write; never reserve Git
locks. Commit small validated changes to main with verification in the message,
run `sh tools/progress.sh`, and update PLAN when an item is done or changes.
Do not commit ROMs, save states, ROM-derived assets or experimental harnesses.

Edition 2024 is adopted. Usable stable rustc is
`1.99.1 (58cf8f9c3 2026-10-13)` and `rust-toolchain.toml` selects stable.
Installed `nightly-2026-10-22`, version
`1.101.0-nightly (1a2c4797e 2026-10-21)`, cannot execute because of Defender
event 1121 / ASR rule `01443614-CD74-433A-B99E-2ECDC07BFC25`. Do not bypass it;
the nightly switch remains unfinished. Migration details/native refresh commands:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/rust-2024-migration/native-handoff.txt`.

Local tool paths:

- Python: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe`
- Rust tools: `C:/Users/Jim.Lundin/.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin`
- sh: `C:/Users/Jim.Lundin/AppData/Local/Programs/Git/bin/sh.exe`
- SDL library: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/lib`

Normal build: `cargo build --release --locked -j 1 -p sm --lib --bin sm --example sm_verify`.
Set `LIBRARY_PATH` and `SM_VERIFY_SOURCE_HASH` using `tools/verify_dll.py`'s
`source_hash(Path.cwd())`. DLL checks use `load_host`, `check_source` and
`check_library`, with `LIBRARY_PATH` also set in the Python loader process.
Targeted formatter: `rustfmt --edition 2024 --config skip_children=true,reorder_modules=false,reorder_imports=false`.
Repository-wide formatting already fails on the baseline; do not mix unrelated
formatting changes into this work.
