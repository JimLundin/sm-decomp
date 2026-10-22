# Engine reduction handoff - 2026-10-22

Read [PLAN.md](PLAN.md) first, then [ITERATION.md](ITERATION.md). This records
an interruption requested by the user, not a completed refactor. Preserve the
complete game and exact behavior while reducing code and removing all unsafe
operations. Item 4, the complete player-projectile lifecycle, remains active.
Read [the architecture migration](target-architecture/MIGRATION.md) before
changing subsystem boundaries or ownership.

## Current checkout and verified baseline

HEAD before this documentation handoff is `74a27e83` on `main`. That commit
streamlined PLAN and added ITERATION; the guide is now tracked, not an unrelated
untracked file. The last implementation commit remains `de9bf05e`:

- `ff02c9b7`: shared projectile room scans between the RAM path and owned draft.
- `de9bf05e`: shared impact/retirement through `projectile_lifecycle` and its
  checked RAM adapter. Those modules and `ClearProjectile`, `KillProjectile`
  and `start_impact` forbid unsafe code.

At that implementation baseline, main has **101,537 Rust lines in 134 files**,
2,940 counted unsafe declarations and 6,160 cast-containing lines. Maintained
implementation is 102,099 lines; including installers/launcher, 102,930.
The last verified prospective owned composition is **110,602 Rust lines**,
**+9,065 over main**, and remains unadopted. Its headless Rust 2024 release
library builds, but the full composition has no runtime/movie comparison.

The terminal change added 186 main lines and removed 48 draft lines: +138
complete lines for safety/architecture. These are costs, not reduction.
The substantial reduction and removal of all unsafe operations remain unfinished.

## Unfinished flight draft

The following production edits are preserved in the worktree, **uncommitted**:

- New, untracked `sm/src/game/projectile_flight_control.rs`.
- Modified `sm/src/game/projectile_flight.rs`.
- Module/source registrations in `sm/src/lib.rs` and `sm/src/verify_source.rs`.

They were targeted-formatted before handoff, but **have not been compiled or
behavior-tested**. Earlier passing checks do not cover them. No source edits or
compiler runs were made while preparing this documentation handoff.

The draft shares update, advance, collision-axis ordering, extension cleanup/
update and offscreen visibility between production and the owned model.
`projectile_flight_control` forbids unsafe code and defines `Flight`, `AXES`,
`X_ACCELERATION`, `Y_ACCELERATION` and `Context<O>` with generic `Direction`.
Its shared functions are `clear_extensions`, `update`, `advance`, `collide`,
`update_extension` and `visible`.

The RAM adapter uses checked accesses and explicit unsafe effect calls.
`update`, `advance` and `collide` retain unsafe signatures; `update_extension`
and `delete_offscreen` become safe. Launch setup, missile acceleration,
extension creation and linked-extension contact remain dependencies to migrate.
This does not complete the flight responsibility or unsafe elimination.

The handoff progress run reports **101,651 Rust lines (+114)**, 2,937 counted
unsafe declarations (-3) and 6,156 cast lines (-4). Other residue counters are
unchanged. This includes the untracked controller and is an unvalidated draft
cost. Complete replacement accounting, including the owned adapter and deleted
owned bodies, has not been done; do not apply the old composition price to it.

Order and access constraints to verify against the independent old source:

- Trail decrement reads/writes native-endian words; expiry resets through LE
  `checked_word!`. Reread the live selector only after the trail callback.
- Capture direction before acceleration/callbacks; look up axes after missile
  acceleration. Invalid directions must preserve the original failure prefix.
- Diagonal collisions retain the original selector across horizontal callbacks.
  Super Missile extension collision precedes testing horizontal hit/return.
- An empty axes mask still executes vertical collision. Beam hits test bit 0;
  missile hits test any nonzero value.
- Offscreen positions remain native-endian, camera values LE. Read Y only after
  X passes. Extension cleanup visits slots 8, 6, 4, 2, 0.
- Preserve odd/broad selectors, overlapping fields, callback order, live reads,
  canonical admission and fallback behavior. Never retain an authoritative typed
  owner across RAM-authoritative execution; transfer the complete owner.

## Local work and validation

Experiment root, outside Git:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/`.

**Reuse `runtime-impact-working/`.** Do not copy another carrier over it.
Its `flight-parent/` contains untouched pre-draft backups of
`projectile_flight.rs` (main at `de9bf05e`), `harness.rs`, `flight.rs`,
`flight_model.rs`, `encoded_flight.rs`, `runtime_external_flight.rs`,
`native_flight.rs`, `flight_checks.rs` and `runtime_flight_compat_checks.rs`.

Edited owned draft files in the working directory:

- `flight.rs`: reexports the shared `Flight`.
- `flight_model.rs`: shares phase bodies through a context adapter.
- `encoded_flight.rs`: uses shared acceleration/axes data.
- `runtime_external_flight.rs`: removes redundant Flight conversion.

`adopt_flight_control.py` already ran successfully. **Do not rerun it over the
edited files.** Owned admission and broad-selector fallbacks remain. Context
`advance` calls `advance_with` to recheck admission after a trail callback can
change the selector. Native direction providers retain generic `Direction`.

Next steps, in order:

1. Recheck HEAD/worktree; review the existing draft rather than implementing a
   second controller. Keep the independent `flight-parent` reference untouched.
2. Extend preparation with a wrapper around `prepare_impact.py`. Its chain
   rebuilds `runtime-source` from `runtime-baseline` plus enumerated adaptations.
   Compile the new shared module unchanged, exposing it publicly only in the
   carrier. Replace the carrier flight implementation with current production,
   changing only visibility as needed. Handle safe as well as unsafe function
   signatures. Do not accidentally route both comparison sides through new code.
3. Reuse `old_flight.rs` / `flight_checks.rs` for controlled raw-versus-owned
   phases and `runtime_flight_compat_checks.rs` for actual-runtime compatibility.
   Cover the ordering constraints above, timer carry/expiry, invalid directions,
   zero/all axes masks, offscreen boundaries, odd/broad slots, aliases, callback
   failures and actual trail/room consumers. Include native flight fixtures.
4. Run relevant `runtime_flight_checks.rs`, `runtime_collision_checks.rs`,
   `runtime_session_checks.rs` and `runtime_beam_checks.rs`. Extend source checks
   from `check_impact.py` and `verify_main_impact.py` for the new preparation;
   old result files are not passes for changed source.
5. Check formatting and final source correspondence; build normal production
   release library/frontend/verifier and check loaded DLL source identity.
6. Price every replacement helper, both adapters, registrations and deletions.
   Recount the complete prospective composition; disclose any growth. Update
   PLAN's current status and commit the validated change to main.

No relevant movie segment is established for this draft. PLAN permits focused
constructed comparisons of actual implementations with explicit limits;
unrelated green frames add no coverage. Synthetic content and controlled
hardware do not establish scheduler encounters or full-game fidelity.

The inherited carrier is baseline `d03f819d` plus explicit adaptations,
**edition 2021**, not the full current engine. Production is **edition 2024**.
Earlier source correspondence uses audited migration `e88ff8da` and
pre-migration `49846692`; explicitly check final inputs.

## Last verified results

`runtime-impact-stage/` is the frozen source/results snapshot before the flight
draft. Preserve it. Relevant files: `impact-final-results.json`,
`main-source-check-impact.json`, `impact-production-source.json`,
`impact-composition-price.json`, `impact-composition/` and
`impact-composition.rlib`. Earlier room results are in `runtime-room-stage/`.

Recorded lifecycle-stage result vectors (not fresh handoff runs):

| Export | Result |
| --- | --- |
| runtime_retirement_experiment | [1137,603,84,48] |
| transition_experiment | [349,300,70] |
| runtime_animation_experiment | [524,204,30,6] |
| runtime_animation_short_experiment | [55] |
| flight_experiment | [529,848,42] |
| runtime_collision_experiment | [196,535,172,1] |
| runtime_flight_experiment | [92,1594,4,1,247] |
| runtime_session_experiment | [170,1823,53,97] |
| runtime_beam_experiment | [338,817,32,52,3] |

All 141 runtime and 200 draft compiled inputs matched. Normal production
release/DLL source checks passed; the prospective headless library built in
45.49s. No movie ran. Persistent Session authority, remaining consumers,
reset/load/rewind/external-writer checks and broader unsafe elimination remain.

## Tools and continuation rules

No compiler or subagent was running at handoff. The Rust migration agent has
completed; gameplay work was single-agent. Do not start new subagents without
authorization. Follow the iteration guide when parallel work is authorized.

Use PowerShell with `login:false`, `rg`, and one heavy compiler at a time across
the machine (`-j 1`). Recheck HEAD/index before every Git write; never reserve
Git locks. Never commit ROMs, states, ROM-derived assets or local harnesses.
The current documentation commit deliberately leaves the untested source draft
unstaged. Continue from this worktree; a fresh checkout alone lacks that draft
and the owned model edits outside Git.

Edition 2024 is adopted. Stable rustc `1.99.1 (58cf8f9c3 2026-10-13)` works.
Installed `nightly-2026-10-22` is upstream version
`1.101.0-nightly (1a2c4797e 2026-10-21)`, but Defender event 1121 / ASR rule
`01443614-CD74-433A-B99E-2ECDC07BFC25` blocks execution. Its version is not
runtime-verified. Keep stable until an administrator authorizes the compiler;
do not bypass the block or change security settings. Migration commands:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/rust-2024-migration/native-handoff.txt`.

Local tools:

- Python: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe`
- Rust: `C:/Users/Jim.Lundin/.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin`
- sh: `C:/Users/Jim.Lundin/AppData/Local/Programs/Git/bin/sh.exe`
- SDL: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/lib`

Set PATH/RUSTC to the usable stable tools and LIBRARY_PATH to SDL; see
[Windows shell setup](ITERATION.md#windows-shell-setup). Normal build:
`cargo build --release --locked -j 1 -p sm --lib --bin sm --example sm_verify`.
Set `SM_VERIFY_SOURCE_HASH` using `tools/verify_dll.py::source_hash(Path.cwd())`.
Use `load_host`, `check_source` and `check_library` on
`target/release/examples/sm_verify.dll`, with LIBRARY_PATH set in the loader too.
Stream compiler output to local logs.

Carrier commands are `python build.py --library`, then `python build.py`, then
the updated checker with selected export names. Finish flight preparation first.
Targeted formatter: `rustfmt --edition 2024 --config skip_children=true,reorder_modules=false,reorder_imports=false`.
Repository-wide formatting already fails on the baseline. Do not format
`verify_source.rs` wholesale or introduce unrelated formatting changes.

Earlier interruption history is in Git at `74a27e83:docs/HANDOFF.md`; those
room-scan instructions are superseded by the landed room and lifecycle stages.
