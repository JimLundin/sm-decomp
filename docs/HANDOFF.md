# Engine reduction continuation - 2026-10-22

Read [PLAN.md](PLAN.md) first and use [ITERATION.md](ITERATION.md). Preserve the
complete game and exact behavior while reducing code, extracting content as
validated data and eliminating all unsafe operations. Item 4, the complete
player-projectile lifecycle, remains active. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
subsystem boundaries or ownership.

## Current implementation

Latest verified stage shares complete morph bomb, power-bomb and spread-bomb
creation from `4e0b2dcf`. Safe owner-generic rules now own admission, reservation,
ammo/flag order, initialization, cooldown, charge cancellation and all five
spread placements. Main movement-HUD/Draygon callers and the owned model use
these rules. Four bank producers and duplicate owned algorithms are retired;
only the real owned fire_bombs delegate and lifetime consumers remain.
Generated direct-test helper overlays are counted as verification, not production.
Persistent Session authority and candidate adoption remain unfinished.

Preserve reservation without ammo rollback, count-zero cooldown bypass, slot9
fallback, captured power-bomb branch/slot/native byte, and spread's live physical
byte0xC23 after all five initializers. Spread reads charge and position live at
each original boundary. Shared rules retain exact palette/sound/flare order,
complete-owner failure prefixes and byte-compatible RAM; adapters encode kinds
and cooldown keys without hiding transaction sequencing.

All **54 suites pass** in 35.512303s with checker subprocess exit0 and all
**51 retained vectors unchanged**. New [1838,1347,88,12,1206,8,655] means 1,838
three-way RAM/APU/g_fail comparisons, 1,347 ordered postoperation observations,
88 failure scenarios, 12 natural chains, 1,206 ticks, eight independent typed
sequences and 655 authored scenarios. Observations are not actual effect counts.
Failures comprise 74 triplets, two direct dirty-owner cooldown lookups and twelve
checked-only real short-record initializer failures. The latter cover ordinary
creation and each spread slot with missing damage/script data; dirty Session
state and pending type/direction/motion writes are verified in the published failure prefix.
The fourteen checked-only failures and typed sequences are excluded from triplets.
Source identities: 274 carrier and 327 local entries.

Nine ordinary chains enter actual movement-HUD creation (eight modes plus a
static wall), reach natural fuse/explosion/retirement in 62 ticks each, and run
next preparation. Spread earns charge through real input, then a constructed
morph-mode transition; release/timeout chains take 185/374 ticks. Every spread
slot flashes, explodes and clears type/damage. Per-slot terrain masks prove
terrain processing; separate static-wall checks prove aggregate actual mechanism
creation. Synthetic playback content and constructed schedules are explicit.
Actual power-bomb HUD creation consumes one fuse tick (59), then real ROM HDMA
growth reaches the Noob Tube trigger at tick 87, radius 0x2B60; 89 counted steps
include creation and 88 subsequent HDMA/update steps. This is the tube signal/wait endpoint,
not full rupture, explosion or Crystal Flash completion. Typed PB startup is
separately authored. No movie frames, full-game or rendering coverage is claimed.

Frozen reference producers remain independent: all 212 parent main Rust files
match committed `4e0b2dcf`; reference bodies match after removing observation-only
inserts. Independent review approved exact final fixture hashes, all-five slot
checks and genuine initializer failures. Five native fatal pairs remain setup-only
stdout/stderr/status77 checks. Native-access exclusions, inherited Intro mismatch
and little-endian Windows limits remain; no new failing movie frames were added.

Production release/frontend/verifier/source and candidate release builds passed
first attempts in 54.87s/51.06s. Carrier retry passed in 19.90s after missing fixture
imports; harness retries corrected typed fixture arguments. A source-change guard
rejected a midbuild fixture edit; the final frozen harness passed in 18.38s. Logs retain
all attempts, including unchanged reuse of 0.13s and prior focused pass in 1.46s. Production
source stayed frozen throughout fixture corrections. Formatting and progress pass.

Main is **106,997 Rust lines in 214 files**; candidate **115,494 (+8,497)** in 323
files remains unadopted. Deltas in lines/lexical tokens/LF UTF-8 bytes: main
[182,2119,4755], local production [-31,50,-793], composition [151,2169,3959],
carrier-only verification [207,2412,11206], complete verification [2181,22507,86676],
implementation plus checks [2332,24676,90635]. Local production overlaps composition;
carrier verification is included in complete verification. Root independently
recounted explicit UTF-8 bytes and unions including deleted paths. Main/composition
growth is below the estimated ranges due to narrower adapters and full retirement,
not trimmed scope; verification and combined cost fit estimates.
Reference/observation/overlay code totals 414 lines, exceeding its component
upper estimate of 392 by 22; aggregate estimates still hold. Unsafe declarations
fall by four to 2,810; cast-containing lines fall by 14 to 5,935. No size-reduction,
new dependency, backend-independence or persistent-ownership claim.

Next: share complete HUD switching, cooldown and weapon coordination, including
actual player-frame callers and SamusCode04/06 deselection. Trace live selectors,
front-pose/freeze update behavior and grapple/Xray effects before implementation.
Persistent authority still needs external-writer/reset/load/rewind closure.

Final production source hash:
`6ab5422ecbd533726ee1d23fc924a7ce34e05fb812179dd2caa6fc53f09e8ac0`.
Use `runtime-impact-working/MORPH_BOMB_TRANSACTION_HANDOFF.md`,
`morph-bomb-transaction-results.json`, `morph-bomb-transaction-fatal.json`,
`morph-bomb-transaction-price.json` and `morph-complete-check-1.log` for local
evidence. Set PYTHONUTF8=1 for Python source comparisons/pricing. Prior missile,
charge/special/bomb handoffs and frozen snapshots remain available.

Retained setup dispatch [115,199,62,5] covers all 115 semantic identities;
these are routing/source coverage, not 115 independently executed native bodies.
Gate [1348,20,5,4] now runs in the full 54-suite invocation. Its four checked-only
cases retain native undefined-access exclusions. Prior stage details remain in
SETUP_DISPATCH_HANDOFF.md.

Retained Noob Tube [15308,9172,7716,1840] remains in NOOB_TUBE_HANDOFF.md.
Projectile classes use the real beam-terrain producer, not power-bomb radius
simulation. Unused siblings stop before absent callbacks D409/D476/D489/D4BE.
An already-waiting duplicate survives event B; a fresh tube deletes. No visible
renderer, full room decompression or full-game claim.

Retained Torizo [3696,1155,64,1382] remains in TORIZO_STATUE_HANDOFF.md.
Actual Bomb room selection and guarded full statue script are covered; Golden
death uses area2 direct setup. Full death pool excludes pickup initialization.
No visible renderer, DMA execution, debris motion, full encounter or movie claim.

Retained block-reaction [3740,1020,224,974] remains in
BLOCK_REACTION_HANDOFF.md. D127 parent deletes5; D064 starts alongside it,
restores409/deletes410, checked through411. Unused headers and constructed
offscreen placement do not establish vanilla room or visible rendering.

Retained Chozo results [3570,126,126,340,185] include separate native watcher
prefixes and qualified completions; never claim native true-arm completion.
Its constructed offscreen footstep/renderer limits remain documented in
CHOZO_TERRAIN_HANDOFF.md. All prior snapshots and failed logs remain.

Retained periodic results [9784,260,52,144] include 260 separate negative
crash-entry comparisons only; no abort/nonreturn-tail claim. See its handoff
and logs. All 24 retained suites are unchanged.

Retained Ceres results: [8800,10912,9,300], all eight source groups admitted.
Only four free-HDMA groups reach native alignment guards; four full pools
return before argument reads. Ceres setup's 10912 checked-failure comparisons
are separate from 8800 successes. See CERES_DOOR_HANDOFF.md and its logs.

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

`noob-tube-parent/` preserves prior main/carrier/local/composition;
`torizo-statue-parent/`,
`block-reaction-parent/`,
`chozo-terrain-parent/`,
`periodic-damage-parent/`,
`ceres-door-parent/`,
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
