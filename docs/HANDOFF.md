# Engine reduction continuation - 2026-10-23

Read [PLAN.md](PLAN.md) first and use [ITERATION.md](ITERATION.md). Preserve the
complete game and exact behavior while reducing code, extracting content as
validated data and eliminating all unsafe operations. Item 4, the complete
player-projectile lifecycle, remains active. Read
[the architecture migration](target-architecture/MIGRATION.md) before changing
subsystem boundaries or ownership.

## Current implementation

Latest correction: room loading now preserves overlapping layer-two/BTS source
contents. The forward-copy bug came from f116c8a6, before projectile consolidation.
An actual-decoder synthetic regression went red then green; nonoverlap and checked
failure-prefix cases pass. Final source
`2e9cd73487ea202c297ea4b07e622bca04c2208eaf4b6f24d8bd7addbdeaea86` passes the
release/frontend/verifier build and Speed Booster inputs 0..27683 through a fresh
checkpoint, including the previously failing input 20857. Both copy branches,
completed loading, gameplay and message probes execute with 120 follow-up frames.
Current main has 109227 Rust lines; the correction adds 89 (16 runtime/probes, 73 checks).
Local artifacts in `20261023/room-mechanisms` include `copy-regression-red.json`,
`copy-regression-fixed.json`, `copy-fixed-prefix.log`, `copy-fixed-message-tail.log`
and `at-20857-fixed-runtime.smvchk`. All checkpoints/observations remain local.

Production projectile consolidation is complete. game/player_projectiles now
owns ten shots, eighteen trail pairs, weapon/charge controls and launch history
through one private 391-word / 782-byte Transaction. Actual player, cinematic,
door, enemy, scene and message callers use it. Fourteen complete adapters and
superseded bank bodies are deleted. Speed Echo (90D4D2) and suit pickup (90EFD3)
execute their real motions.

Canonical, broad, odd and signed accesses use the same algorithms and byte-lane
resolver. Foreign calls publish and relinquish ownership, then import returned
RAM. Ownership does not persist across native gameplay, frame return, reset/load
or oracle intervals. Drop preserves successful mutation prefixes during unwind,
not before panic hooks or abort. Checked word writes publish the low byte before
high-byte failure; atomic slices retain preflight behavior. Signed arithmetic,
evaluation order and native-versus-LE reads remain significant.

Flare tables read loaded_rom; sequence bytes use captured g_rom, which can
differ. Native content capture/read APIs retain explicit unsafe assumptions.
Persistent ownership, backend independence, all-unsafe elimination, reduction
and boot-to-credits coverage remain unfinished.

Projectile milestone source fingerprint:
24bd3d424dea880173b5ae8ebaa9b25dd4fa9f1d0e57696deaac1621bda3e8b7.

- Release library/frontend/verifier build passes in 50.7995 seconds.
- 106 maintained groups pass. All 104 synthetic old/new scenarios compare all
  131072 RAM bytes directly, with zero differences; two groups exercise ownership
  and publication. Checks plus direct comparison take 0.8045 seconds.
- Three expected caught panics cover bytewise bounds, slice preflight and foreign
  callback unwind. Producer cases cover actual launch/motion, impact, expiry and
  Power Bomb DMA configuration. No-op family cases alone prove no flight.
- Oracle inputs 0..8779: 8780 frames, zero failures, exit 0, 24.70 seconds replay.
  Seven required probes all execute, followed by 120 frames: weapons 1649,
  history 1900, flight 85, playback 42, projectile draw 135, explosion draw 22,
  trails 3604. Last first hit is input 8659. Coverage is 427 gameplay frames,
  one room DF45, enemy E23F, no credits. Draw entries alone do not prove emitted
  OAM tiles. Synthetic cases are not exhaustive.
- Independent implementation/caller/provenance/outcome/accounting review approved.
  Formatting passes for all 19 new module files.

Main source: 108821 Rust lines / 226 files. Net increase is 1389 lines /
11644 lexical tokens / 43988 LF UTF-8 bytes: runtime/probes +596 lines,
maintained verification +793. Unsafe declarations 2767 (-22); cast lines 5892
(-26); no_mangle 67; C ABI metric 7. External retained reference/fixture source
adds 136 lines / 4316 tokens / 11355 bytes; initial ownership experiment adds
478 / 5029 / 20068. Complete incremental authored source is +2003 lines /
+20989 tokens / +75411 bytes. Staging copies count once. Archiving the rejected
experimental Session earns no production deletion credit.

Reproduce with the Windows environment in ITERATION.md:

```text
python tools/verify_dll.py --build-only --frontend-check
python tools/verify_dll.py rom/sm.smc movies/speed-booster.inputs --cargo --no-build --frames 8780 --after-hits 120 --require-hit player_projectiles.prepare.weapons --require-hit player_projectiles.history.finished --require-hit player_projectiles.flight --require-hit player_projectiles.playback.frame --require-hit player_projectiles.draw.projectile --require-hit player_projectiles.draw.explosion --require-hit trails.update
```

Maintained checks use the existing source-checked DLL workflow:

```python
import ctypes, runpy
from pathlib import Path
v = runpy.run_path("tools/verify_dll.py")
root = Path.cwd()
host = v["load_host"](root / "target/verify-dll/sm_verify.dll")
v["check_source"](host, v["source_hash"](root))
v["check_library"](host, root)
host.sm_verify_player_projectiles.restype = ctypes.c_uint32
assert host.sm_verify_player_projectiles() == 106
```

sm_verify_player_projectile_ram(id, output, capacity) copies 131072 bytes for
IDs 0..103, or returns zero on failure. Direct old/new comparisons are recorded
locally in consolidation-main-byte-comparisons.json.

Evidence root:
C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/runtime-impact-working/.

Final artifacts: consolidation-main-final-build-2.log,
consolidation-main-final-checks.json, consolidation-main-final-checks.log,
consolidation-main-byte-comparisons.json, consolidation-main-final-oracle.log.
Old observations are in consolidation-old-observations.json (0..8),
consolidation-matrix-observations.json (9..98) and
consolidation-end-observations.json (99..103), with corresponding raw
consolidation-{old,matrix-old,end-old}-ID.ram files. These stay local.
consolidation-parent-766c6246-sm-src/ preserves old main;
consolidation-before-source/ preserves the old experimental composition.

The prior charge-palette 60-suite/59-vector result and nine fatal pairs remain
historical evidence, not new integration coverage. Its artifacts, 815 verification
freeze entries and 284 runtime/338 harness embedded inputs remain unchanged.
consolidation-palette-reference-sm-src/ reconstructs its exact 341-file
composition; original archives and prior handoffs remain preserved.

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

The complete RoomMechanisms proposal and linear foreign-call ownership contract
are reviewed. Capture the source-reviewed 1009-case `baseline_boundaries.rs` draft
from `20261023/room-mechanisms` before runtime replacement. The first 90 actual-old
cases are frozen in `baseline-90-capture.json` and matching binary outputs/DLL.
Preserve those cases byte for byte. Remaining coverage exclusions and actual
runtime outcomes still require review. Follow PLAN's full caller/deletion scope;
do not restart the archived Session or isolated shared-consumer queue.

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
  callback order and the unified byte-lane resolver.
- Extension activity reads the unrounded parent selector's following byte;
  low-speed follow and positive-speed placement read LE positions, while the
  negative-speed displacement path alone uses a native word. On collision
  failure, retain the callback's writes and selector. Post-collision impact
  testing reads the restored parent's live kind before possibly releasing the child.

Use the existing controlled `old_flight.rs` / `flight_checks.rs`, native fixtures,
actual-runtime compatibility and room/session/beam consumers. Choose cases for
the changed branches. The projectile segment above establishes only its recorded
coverage; another responsibility needs its own exercised checks.
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
