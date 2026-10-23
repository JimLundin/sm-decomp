# Engine reduction continuation - 2026-10-22

Read [PLAN.md](PLAN.md) first and use [ITERATION.md](ITERATION.md). Preserve the
complete game and exact behavior while reducing code, extracting content as
validated data and eliminating all unsafe operations. Item 4, the complete
player-projectile lifecycle, remains active. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
subsystem boundaries or ownership.

## Current implementation

The latest stage shares Mother Brain escape-door setup B5F8 (8697336) after
Tourian walls at `8112c7df`. Existing mechanism_doors/checked adapter preserve
captured origin, exact odd-byte high access, BTS indexing and three live widths:
9001 followed by D0FF three times. The common installer also serves Grey/Colored
without changing their observation order. Both modules forbid unsafe. Candidate
Door routing publishes/shared-runs/reimports; RAM remains authoritative.
87/115 setups are shared;14 room/event/boss and14 residual identities remain.

Main104886 Rust lines/184 files; unsafe2885 unchanged,casts6021(-3). Maintained
implementation105448; installers/launcher included106358. Candidate113775/285,
gap8889, unadopted. Stage main[4,52,23],composition[6,92,187],verification
[601,7628,25381] in lines/tokens/LF bytes; implementation+checks607 lines. Local
production+2 overlaps composition. No dependency/native/installer additions,
net reduction, independent ownership or backend completion claim.

Use `runtime-impact-working/MOTHER_BRAIN_DOOR_HANDOFF.md`,
`mother-brain-door-price.json` and `mother-brain-door-check-attempt-1.log`.
All19 suites pass22.229812s with219 runtime/257 local source identities.
Escape[16704,5742,10,192]: direct total INCLUDES5742 matching checked original
failures. Original B5F8 installer already bounds-checks FFFD..FFFF; preceding
writes survive. This differs from Tourian's separate checked-only UB exclusions.
Symbolic cases assert all16 ordered observations/failures, changed widths and
captured origin. Ordinary doors remain[570,12,30,3].

Eight actual A9B32A producer groups cover zero/one/eight/eighteen free debris
slots and available/full PLM pools. Source spawns at(0,6), after eight native
CB21 attempts and AIvar_A=B33C. Full overlays are bounds/alignment guarded.
Candidate explicitly retains native debris and outer AI; this is not owned
encounter evidence. Real Samus both-axis collision traverses D0FF to Door BTS1,
using DDBC->AA8C->DE4D, setting E17D/door/state9. Projectile effects also checked.

Separate synthetic B677 at(0,32) runs real AC0B draw94B1 and Delete over two ticks;
carrier asserts y32/cameraY0 before every Emit. Four left Door/extension cells
and four right Air cells retain BTS and consumer behavior. No visible native
DrawPlm or actual source-coordinate script/rendering claim. No movie, whole
encounter, complete composition runtime, boot-to-credits or independent backend.

Production release/frontend/verifier/source48.1155174s; prospective release
43.4677616s; carrier16.81s; harness18.21s. All first attempts pass. Production hash:
`03b9d310264a1df0354f9f9e66f95eb9e8235d824ce943b281dbfe1ee80904b6`.
Three main files are formatted and byte-identical in composition; its fourth
changed file is exactly transformed mechanisms_ram. No main registrations
changed. Inputs remain frozen after builds; compiler lane released/DLL unloaded.

Retained selected results: Tourian21924/8700/10/80,animal42008/4628/19/13,
Shaktool26700/6/4,booster91874/8354/20/21,sound15300/274/2304,
quota6118/66/112/48,eye13320/3440/56/24,plants1360/1504/8/640,
cannon3630/1420/46/8,item3476/1320/14/32,scroll5816/1296/52,
station3240/112/149/6,Chozo592/4/4/16,hands52/163/12,source342/672/47,
closing74/120/7,collision198/545/176/1. Animal/booster failures are included;
Tourian8700 are separate checked-only cases, never undefined original execution.

Preserve historical exclusions: unfrozen advance_tide always reads odd RAM197D;
terrain18 visible restoration reads odd RAM1E67. Neither is defined-reference
coverage. Current offscreen guards do not resolve inherited Intro mismatch.
Qualified item/cannon/Chozo and full-contact limits remain; eye acid initialization
stays excluded. Scroll bounded nonreturn observations were not repeated.

`mother-brain-door-parent/` preserves preceding main/carrier/local/composition.
All historical snapshots/caches/failed logs remain, including animal BE30 routing
correction; main numeric dispatch was always correct. Never rerun historical
broad preparation scripts. Composition is `impact-composition/sm/src`.
Continue remaining setups, then native dispatch and authority dependencies.
Ownership promotion requires reset/load/rewind/external-writer checks. The
parallel movie/state investigation in [MOVIE_STATE_TRANSLATION.md](MOVIE_STATE_TRANSLATION.md)
implements no pipeline.

## Paused launch draft

The reduction agent wound down on 2026-10-22 and paused its ongoing goal.
Its Rust-migration child is completed. No compiler, check, helper or tool session
was left running by that task. Resume only after a new implementation request.
The coordinator found one other running project thread; the inspected older
verification, reduction, architecture and cleanup threads were already inactive.

Resolved in the 2026-10-22 restart. The original three-file draft, based on
`e3fadf56`, remains preserved locally:

- `sm/src/game/launch_motion.rs`: shared beam speed data/selection and aim decoder.
- `sm/src/game/launch_motion_ram.rs`: shared decoding after subpixel clearing.
- `sm/src/game/projectile_flight.rs`: checked beam speed selection.

The adopted main change adds 13 lines; synchronized owned `launch.rs` removes
22 lines, giving -9 complete composition lines, -166 lexical tokens and -509
normalized text bytes. Existing local comparison code adds 167 lines, including
the independently frozen old decoder; including checks gives +158 lines.
No dependency or maintained native/launcher code is added.

In `runtime-impact-working/launch-draft/`, the three production drafts,
`owned-launch.rs`, `main.patch` and `source-hashes.json` preserve exact edited
bytes. The restart synchronized main, working `launch.rs`, local launch-motion
modules and `runtime-source`.
`launch-parent/` preserves pre-draft production,
owned launch, harness/preparation and relevant previous composition inputs.
The local `WIND_DOWN.md` records the historical dirty stop point; this section
supersedes its draft locations. Preserve existing caches,
backups and results; `impact-composition/` now includes this launch change.

Use `prepare_flight_control.py` and selected exports `owned_launch_experiment`,
`runtime_launch_experiment`, `flight_experiment`, `runtime_beam_experiment`,
`runtime_flight_compat_experiment` and `runtime_session_experiment`.
The launch checks pass 395 owned cases and 1,070 RAM/owned runtime cases,
including invalid diagnostics, kind bounds, captured selection, odd/broad
indices, fraction/direction aliases and live history rereads. The old decoder
matches pre-change Git source and is separately compiled; production diagnostics
stop at the native assertion boundary, while owned fixtures exercise returning
diagnostics. No movie runs or full-composition runtime comparison are claimed.
Final production and prospective Rust 2024 headless release builds pass.
Results: `launch-final-results.json`, `launch-final-check.log`,
`launch-main-source-check.json`, `launch-production-build.log`,
`launch-composition-price.json` and `launch-composition-build.log`.
Missile acceleration, extension creation and placement are now shared; remaining consumers
and runtime authority transfers are next.

## Local work and validation

Experiment root, outside Git:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/`.

**Reuse `runtime-impact-working/`.** Its name is unchanged to preserve the
working cache. Do not copy an older carrier over it. Current files:

- `prepare_flight_control.py`: extends `prepare_impact.py`, exposes carrier
  entries and registers the shared controller and independent reference.
- `check_flight_control.py`: checks compiled inputs and runs selected exports.
- `runtime_flight_control_checks.rs`: saved production versus current RAM flight
  and owned extension contact using actual trail/room code, constructed content
  and controlled room hooks. Of 1,044 cases, 600 are new extension comparisons,
  divided evenly between the RAM and owned paths against the saved reference.
- `extension-final-results.json`, `extension-final-check.log`:
  final comparison results.
- `main-source-check-extension.json`: exact old macro/wrapper correspondence to
  `ba0d94c1`, current compiled inputs and unchanged launch/acceleration/spawn tails.
- `extension-production-source.json`, `extension-production-build.log`: normal
  release library/frontend/verifier build and loaded DLL source checks.
- `extension-composition-price.json`, `extension-composition-sources.json`,
  `extension-composition-build.log`: complete size/token accounting and prospective
  headless Rust 2024 build. `price_extension.py` reconstructs the changed
  composition files. The working `impact-composition/` now contains this extension
  composition; the older terminal composition is frozen in the stage below.

The independent `flight-parent/` backups include production flight at
`de9bf05e`, `harness.rs`, `flight.rs`, `flight_model.rs`, `encoded_flight.rs`,
`runtime_external_flight.rs`, `native_flight.rs`, `flight_checks.rs` and
`runtime_flight_compat_checks.rs`. Do not overwrite them. The saved production
file was checked against Git; the compiled reference changes only visibility.
The compiled new controller is byte-identical to main, and the production RAM
adapter differs only in visibility needed by the carrier.

`extension-parent/` saves source and results from the preceding `ba0d94c1`
stage, plus the changed prospective composition files. Combined with unchanged
`runtime-impact-stage/` inputs, it supplies the previous full price/reference.
Do not rerun the old `price_flight_control.py`: it expects the deleted axis
wrappers and would overwrite the wrong composition/results.

The current owned changes are in `flight.rs`, `flight_model.rs`,
`encoded_flight.rs` and `runtime_external_flight.rs`. The one-time
`adopt_flight_control.py` has already run; do not rerun it over those edits.
Context `advance` deliberately calls `advance_with` to recheck canonical
admission after a trail callback can change the selector. Broad-selector
fallbacks transfer the complete owner; native providers retain generic Direction.

Latest final result vectors:

| Export | Result |
| --- | --- |
| runtime_flight_control_experiment | [1044,116,96,4] |
| flight_experiment | [529,848,42] |
| runtime_flight_compat_experiment | [446,14,11,2] |
| runtime_flight_experiment | [92,1594,4,1,247] |
| runtime_collision_experiment | [196,535,172,1] |
| runtime_session_experiment | [170,1823,53,97] |
| runtime_beam_experiment | [338,817,32,52,3] |

All 143 runtime and 201 draft compiled inputs match. Production release and
loaded DLL checks pass (60.58s build). The prospective headless Rust 2024
library builds in 42.78s, as recorded in `extension-composition-price.json`;
this supplies no runtime or movie comparison of the full composition.
No movie ran. Synthetic content, controlled hardware/callbacks and these
selected actual-code comparisons do not establish full-game fidelity.

The inherited carrier remains `d03f819d` plus explicit adaptations, edition
2021, not the full current engine. Production is edition 2024. Only five runtime
files differ from the frozen impact carrier: lib/verify registrations, current
flight, the shared controller and independent reference. Earlier source
correspondence uses audited migration `e88ff8da` and pre-migration `49846692`.
Always check the final compiled inputs after changing preparation.

`runtime-impact-stage/` remains the frozen previous terminal stage, including
its results and 110,602-line prospective composition. Earlier room results
remain in `runtime-room-stage/`. Those historical passes do not validate future
source edits. The current continuation and unchanged frozen inputs are needed
to reproduce the local owned work; a fresh Git checkout alone lacks that work.

## Next dependency

Launch, missile acceleration, extension creation, placement, initialization, playback,
trail selection, complete trail emission/side playback/native output, phase ordering
and contact are now shared;
do not implement a second controller or adopt outer wiring alone. Price helpers, adapters, callers,
registrations and deletions together. Then finish remaining consumers and runtime
authority transfers; promotion of the owner still needs reset/load/rewind and
external-writer checks from state migration.

Keep these verified ordering/access constraints through subsequent changes:

- Trail decrement uses native-endian words; expiry resets through LE
  `checked_word!`. Reread the live selector after the trail callback.
- Capture direction before acceleration/callbacks; look up axes after missile
  acceleration. Invalid directions preserve the original failure prefix.
- Diagonal collision retains the original selector across horizontal callbacks.
  Super Missile extension collision precedes horizontal hit/return testing.
- An empty axes mask executes vertical collision. Beams test hit bit 0;
  missiles test any nonzero value.
- Offscreen positions are native-endian, camera values LE. Read Y only after
  X passes. Extension cleanup visits slots 8, 6, 4, 2, 0.
- Preserve odd/broad selectors, overlapping fields, captured versus live reads,
  callback order, canonical admission and complete-owner fallbacks.
- Extension activity reads the unrounded parent selector's following byte;
  low-speed follow and positive-speed placement read LE positions, while the
  negative-speed displacement path alone uses a native word. On collision
  failure, retain the callback's writes and selector. Post-collision impact
  testing reads the restored parent's live kind before possibly releasing the child.

Use the existing controlled `old_flight.rs` / `flight_checks.rs`, native fixtures,
actual-runtime compatibility and room/session/beam consumers. Choose cases for
the changed branches; no relevant movie segment is currently established.
PLAN permits constructed comparisons with explicit limits. Unrelated green
frames add no coverage. Keep the independent reference separate from new code.

## Tools and continuation rules

The Rust migration agent completed. The user authorized coordinated gameplay
implementation and independent read-only review through one integration lane.
No compiler is left running after the completed builds above.
Use PowerShell with `login:false`, `rg`, and one heavy compiler
at a time across the machine (`-j 1`). Recheck HEAD/index before every Git write;
never reserve Git locks. Commit small verified changes to main and run
`sh tools/progress.sh`. Never commit ROMs, states, ROM-derived assets or harnesses.

Edition 2024 is adopted. Stable rustc `1.99.1 (58cf8f9c3 2026-10-13)` works.
Installed `nightly-2026-10-22` is upstream version
`1.101.0-nightly (1a2c4797e 2026-10-21)`, but Defender event 1121 / ASR rule
`01443614-CD74-433A-B99E-2ECDC07BFC25` blocks execution. Keep stable until an
administrator authorizes the compiler; do not bypass the block or change
security settings. Its version is not runtime-verified. Migration commands:
`C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/rust-2024-migration/native-handoff.txt`.

Local tools:

- Python: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/python-3.13.7/python.exe`
- Rust: `C:/Users/Jim.Lundin/.rustup/toolchains/stable-x86_64-pc-windows-gnu/bin`
- sh: `C:/Users/Jim.Lundin/AppData/Local/Programs/Git/bin/sh.exe`
- SDL: `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-tools/SDL2-2.32.10/SDL2-2.32.10/x86_64-w64-mingw32/lib`

Set PATH/RUSTC to stable and LIBRARY_PATH to SDL; see
[Windows shell setup](ITERATION.md#windows-shell-setup). Normal build:
`cargo build --release --locked -j 1 -p sm --lib --bin sm --example sm_verify`.
Set `SM_VERIFY_SOURCE_HASH` using `tools/verify_dll.py::source_hash(Path.cwd())`.
Use `load_host`, `check_source` and `check_library` on
`target/release/examples/sm_verify.dll`, with LIBRARY_PATH set in the loader too.
Stream compiler output to local logs.

Carrier: `python prepare_flight_control.py`, `python build.py --library`,
`python build.py`, then `python check_flight_control.py` with selected export
names. The existing local build.py now reuses content-checked library/harness
outputs and separate incremental caches; a changed fixture does not rebuild
the library. Logs are runtime-build.log and harness-build.log. Compiler options,
dependency bytes, native libraries and output bytes determine reuse; keep the
compiled-input assertions in the selected checks. Launch inputs are synchronized;
keep the separate old decoder in both carrier and harness on subsequent preparation.
For production, use `python tools/verify_dll.py --build-only --frontend-check`
with the installed SDK environment; it now shares normal Cargo flags/artifacts.
Preparation does not compile; reuse unchanged libraries. Formatter:
`rustfmt --edition 2024 --config skip_children=true,reorder_modules=false,reorder_imports=false`.
Repository-wide formatting already fails on the baseline. Do not format
`verify_source.rs` wholesale or introduce unrelated formatting changes.
