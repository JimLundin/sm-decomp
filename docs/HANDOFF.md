# Engine reduction continuation - 2026-10-22

Read [PLAN.md](PLAN.md) first and use [ITERATION.md](ITERATION.md). Preserve the
complete game and exact behavior while reducing code, extracting content as
validated data and eliminating all unsafe operations. Item 4, the complete
player-projectile lifecycle, remains active. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
subsystem boundaries or ownership.

## Current implementation

The latest stage shares the paired Tourian invisible walls after `373b0203`.
Setups B551/B56F (source integers8697169/8697199) retire into a safe core/checked
adapter. Actual headers B763/B767 use those setups and AAE3 Delete. Both modules
forbid unsafe. Preserve captured origin, rounded odd terrain offsets, low twelve
bits, ten writes and ten subsequent live width observations. The final computed
next block is not accessed. Candidate typed setup paths publish/share/reimport;
RAM remains authoritative. 86/115 setups are shared, 29 remain.

Main: **104,882 Rust lines in 184 files**, unsafe declarations2,885 unchanged,
casts6,024 (-2). Maintained implementation105,444; installers/launcher included
106,354. Prospective composition: **113,769 in 285 files**, **+8,887 over main**,
unadopted. Main cost +50 lines/+515 tokens/+1,731 normalized bytes; complete
implementation +78/+785/+2,888. Retained verification +526/+6,879/+22,530;
implementation plus checks604 lines. Local production +27 lines overlaps the
composition. No duplicate owned algorithm existed. No dependency/native/installer/
launcher additions, net reduction, full unsafe retirement, persistent authority
or backend-independence claim.

Use `runtime-impact-working/TOURIAN_WALL_HANDOFF.md`, `tourian-wall-price.json`
and `tourian-wall-check-attempt-1.log`. Selected run passes25.56s with identities
218 runtime/255 local: 21,924 defined-original/main/candidate direct comparisons,
8,700 separate checked-only failure-prefix comparisons, ten actual producer groups
and80 collision samples. Every original sequence is preflighted across its ten
visited terrain words. Reached FFFE/FFFF offsets access outside RAM in the original;
never execute them as fidelity evidence. Checked main/candidate failures retain
valid preceding writes. FFFC/FFFD are valid; an unused final next-block FFFE/FFFF
is also valid. Symbolic tests assert all31 observation/effect failure positions,
changed widths/origin and every write value, including the tenth width read.

Actual main A9EFE6 close and A9F138 open callbacks run with bounded aligned
EnemyLayout40 overlays and native scroll alignment assertions. Camera512/513 and
countdown0/1 cover their trigger boundaries; actual acceleration runs at the
selected stationary target. Both wall sources spawn in order at(48,3),(31,3).
One-free-slot cases assert allocation order and are not scheduled with fake occupied
headers. Actual AAE3 Delete releases normally created slots. Real collision sampling
checks all20 cells in each normally allocated wall pair, Air versus Solid, preserved
low bits, current-block selection and scan state.

Candidate constructors are compared after normalizing explicitly asserted outer
A9 enemy/camera/scroll writes. This is wall/PLM evidence, not an independent owned
encounter callback. No old whole-enemy callback, full enemy sequence, room loading,
startup, rendering/audio, movie or complete composition/gameplay claim. Selected
streams contain only Delete; drawing callbacks are unexercised.

Liquid limitation: `advance_tide` always reads native u16 at odd RAM0x197D, reached
by every unfrozen `lava_surface`. Do not execute that path as defined-reference
evidence. Carrier-only wrappers expose unchanged `advance_motion`; full liquid
engine, tide and rendering remain uncovered.

Historical correction: 18 visible terrain restore/render cases (nine kinds by
two aliases) reach DrawPlm's native u16 dereference at odd RAM0x1E67. They are
not defined-native fidelity evidence. Preserve logs and independent setup results;
do not rerun the undefined cases. Current selected suites exclude them. This does
not diagnose the inherited Intro mismatch in `DRAWING_HANDOFF.md`.

Animal42008/4628/19/13, Shaktool26700/6/4, booster91874/8354/20/21,
sound15300/274/2304, quota6118/66/112/48, eye-door13320/3440/56/24,
plants1360/1504/8/640, cannon3630/1420/46/8, item3476/1320/14/32,
scroll5816/1296/52, station3240/112/149/6, Chozo592/4/4/16, hands52/163/12,
source342/672/47, closing74/120/7 and collision198/545/176/1 pass. Animal and
booster direct totals include their matching bounds failures; Tourian's checked-only
count is separate from original-defined comparisons. Historical qualified graphics/
cannon/Chozo evidence and full-contact limits remain unchanged; eye acid initialization
stays excluded. Unchanged scroll bounded nonreturn observations were not repeated.

Production release/frontend/verifier/source passed59.58s; prospective release56.33s,
carrier21.78s, harness26.05s and selected checks25.56s all passed first attempts.
No source changes followed builds. Production hash:
`6c76eabad1724d1b588697e012ea550247f3d4344b81435e60b283455969db26`.
Changed gameplay/root formatting and diff checks pass; manifest additions conform
with baseline formatting differences unchanged. Compiler lane released; DLL unloaded.

`tourian-wall-parent/` preserves prior main/carrier/local/composition sources.
Earlier snapshots/caches and failed logs remain. Animal rescue's candidate/fixture
BE30 correction and failed/diagnostic runs remain documented in its handoff; main
numeric dispatch was unchanged. Do not rerun historical preparation scripts;
composition lives under `impact-composition/sm/src`. Continue the next coherent
responsibility among15 remaining room/event/boss setups, then14 residual identities.
Runtime promotion needs reset/load/rewind/external-writer checks; boot-to-credits
and independent renderer/audio remain unfinished. The parallel movie/state
investigation is documented in [MOVIE_STATE_TRANSLATION.md](MOVIE_STATE_TRANSLATION.md);
it implements no pipeline.

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
