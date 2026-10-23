# Engine reduction continuation - 2026-10-22

Read [PLAN.md](PLAN.md) first and use [ITERATION.md](ITERATION.md). Preserve the
complete game and exact behavior while reducing code, extracting content as
validated data and eliminating all unsafe operations. Item 4, the complete
player-projectile lifecycle, remains active. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
subsystem boundaries or ownership.

## Current implementation

The latest stage shares the Ceres escape doorway after treadmill commit
`c0ad5aad`. Existing mechanism_doors gains CeresEscape and shares make_solid
with Blue doors; B9F1 routes through the existing checked adapter. No module,
trait method or registration added. Four cells preserve lower 12 bits and gain
solid type 8. Origin is captured; each wrapped stride observes width after the
preceding write. The original middle pair of consecutive immutable width reads
has no intervening observer and equals the existing doubled-width operation.
RAM remains authoritative; 91/115 setups shared, 10 room/event/boss + 14 residual.

Main 105169 Rust lines/190 files; unsafe 2880 unchanged, casts 6011 (-5).
Maintained implementation 105731; installers/launcher included 106641.
Candidate 114182/294, gap 9013, unadopted. Main delta [4,-42,-102],
composition [6,-2,50], verification [517,7137,24052] lines/tokens/LF bytes;
implementation plus checks [523,7135,24102]. Local production two lines overlap
composition. No dependency/native/installer additions or net reduction claim.

Use `runtime-impact-working/CERES_DOOR_HANDOFF.md`, `ceres-door-price.json`
and `ceres-door-check.log`. All 23 suites pass in 24.272693s (suite 23.9583265s),
explicit process exit 0, source identities 229 runtime/275 local. New result
[8800,10912,9,300]: 8800 successful and 10912 matching checked-failure comparisons,
9 source/reentry groups and 300 mixed observations. The failures are part of
19712 comparisons with the actual frozen original: its checked helper fails
when any reached row cursor is FFFE/FFFF and retains prior writes. No undefined
native execution. Symbolic cases cover captured origin, live strides and every
operation's failure prefix. Retained Blue door comparisons still pass.

Actual DF45 room header selects DF57 by default and DF71 with boss bit 1.
Escape callback C96E spawns BA48 at (15,38), then haze; default C976 only haze.
Room width 16, height 48. Eight boss/PLM/HDMA pool combinations plus one
flag-change/repeated-entry group pass. BA48 uses B9F1 and AAE3 Delete, so the
first PLM tick deletes it without a draw, followed by two empty ticks.
Real player movement and both projectile axes check the four solid cells and
neighboring rows. Whole-RAM collision tails, boss/event/BTS preservation and
retained haze allocation/DMA state agree. The 300 observations count each game:
156 row checks, 48 player movement checks and 96 projectile reactions.

Packed haze arguments are checked for alignment before any native dereference
or write. All eight source groups admitted, zero exclusions. Hardware holders
remain live during hardware access; later full-pool reentry needs no backend.
Three literal ranges totaling 76 bytes independently match primary banks;
ROM-derived fixtures stay local. Static doorway setup is covered, not the
retained CeresDoor actor, haze rendering, full encounter/movie, complete
composition runtime, boot-to-credits or persistent Session ownership.

Production release/frontend/verifier/source 50.033665s; candidate release
41.5264568s; carrier 17.3172993s; harness 16.4909373s; checks 24.272693s,
all first attempts, explicit exit 0. Production hash:
`dc529dccede5c84b8b30699a275fc7ae31f74fb241f26eb0493eb91f8863623f`.
Changed formatting passes; no lib/verify_source changes. Composition has exactly
three changed files matching main and transformed local source. No production
source edits after builds. Review strengthened fixture assertions before builds.

Retained treadmill results: [11120,18,19,4880]. See TREADMILL_HANDOFF.md,
treadmill-price.json and treadmill-check-attempt1.log for row/carry details,
C800 aliases and the native FF8F origin limit. Those 18 checked-only prefixes
are separate from defined-original comparisons; never execute invalid originals.

Retained results:crumbling15670/0/17/4302,glass8604/1460/22/138,
escape16704/5742/10/192,doors570/12/30/3,Tourian21924/8700/10/80,
animal42008/4628/19/13,Shaktool26700/6/4,booster91874/8354/20/21,
sound15300/274/2304,quota6118/66/112/48,eye13320/3440/56/24,
plants1360/1504/8/640,cannon3630/1420/46/8,item3476/1320/14/32,
scroll5816/1296/52,station3240/112/149/6,Chozo592/4/4/16,hands52/163/12,
source342/672/47,closing74/120/7,collision198/545/176/1. Glass/escape/animal/
booster failures are included; Tourian8700/treadmill18 are separate checked-only
cases. Crumbling4302 counts paired cell/frame observations(8604 collision calls).

Preserve exclusions: unfrozen advance_tide always reads odd RAM197D; terrain18
visible restoration reads odd RAM1E67. Neither is defined-reference coverage.
Current guards do not resolve inherited Intro mismatch. Qualified item/cannon/
Chozo and full-contact limits remain; eye acid initialization stays excluded.
Scroll bounded nonreturn observations were not repeated.

`ceres-door-parent/` preserves prior main/carrier/local/composition;
`treadmill-parent/` and earlier snapshots remain intact.
Historical snapshots/caches/failed logs remain, including animal BE30 routing
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
