# Equipment interaction: exact independent admission and source contract

Status: awaiting coordinator scaffold review
Observed main seed: `097a7a6eded8b0a9a72fa4efbe040d1868ae7f72`

The user has authorized concurrent decompilation. Start one complete equipment
interaction implementation in a private Git repository. Its `IsolatedProof`
Installation generates and verifies new Rust source without altering main's
production registry, authority, ledgers, held source or original attempts.
Main integration is pending and will need a separately reviewed managed admission.

## Supported route

`translation_launch_prepare::configure` explicitly accepts `IsolatedProof`
templates (`crates/pipeline/src/translation_launch_prepare.rs:224`).
`Adapter::check_admission` consults production migration only for Managed
installations (`translation_execution.rs:751`), and only Managed publication
calls the original queue/progress publisher (`translation_execution.rs:2256`).
The native `ReviewedCatalog` provenance checks a sealed requirements artifact
and equality with the actual registered catalog entry; it does not require a
historical passing family (`translation_native.rs:536`).

This is a private scope identity, not a new admitted effort in the main queue.
No production/authority amendment is needed for the private source execution.
Using main as either source or integration authority would violate this packet.
Keep all primary and integration checkouts inside the private boundary below.

| Resource | Exact proposed path |
| --- | --- |
| Private root | `/home/jim/sm-decomp-equipment-interaction-20261014` |
| Source and private integration repository | `/home/jim/sm-decomp-equipment-interaction-20261014/source` |
| Installed tools | `/home/jim/sm-decomp-equipment-interaction-20261014/tools` |
| Reviewed template | `/home/jim/sm-decomp-equipment-interaction-20261014/review/approved-template.json` |
| Launcher output | `/home/jim/sm-decomp-equipment-interaction-20261014/execution` |
| Shared process-worker budget | `/home/jim/sm-decomp/out/pipeline`, workers `4` |

Use a separate Git repository, not a linked worktree that shares main's Git
metadata. Copy the selected pinned sources and harness inputs into its baseline;
the finite closed catalog is [catalog.toml](catalog.toml). It registers only
`pause-equipment`. The production 177-family catalog is unchanged. This new
scope's private proof does not satisfy any original host/controller/pause gate.

After the actual scaffold, fixtures, source commit and tool References are reviewed
and the Installation template is sealed, the supported commands are:

```sh
/home/jim/sm-decomp-equipment-interaction-20261014/tools/sm-pipeline translation-launch --request /home/jim/sm-decomp/.scratch/equipment-interaction-concurrent-20261014/launch-request.json prepare
/home/jim/sm-decomp-equipment-interaction-20261014/execution/installed/controller translation-launch --plan /home/jim/sm-decomp-equipment-interaction-20261014/execution/plan.json run
```

The request/template do not exist yet: there are no invented hashes or pretend
launchable placeholders. The actual request uses `translation-launch-request-v1`,
`id = equipment-interaction-offline-20261014`, one scope with `after = []`, and
real helper/template References. Reuse the already reviewed Foundry provider
settings and frozen helper identity; keep credentials outside all artifacts.

## Exact source ownership

The implementer's responsibility is limited to these three paths in the private
repository:

- `crates/port/src/pause_equipment.rs`
- `crates/port/tests/pause_equipment.rs`
- `tools/port/pause-equipment.toml`

The coordinator owns the private Cargo scaffold, copied dependency/harness files,
private `lib.rs`, closed catalog and installation seals. The worker is not alone
in the workspace and must preserve all other owners' changes.

Only `crates/port/src/pause_equipment.rs` is model-writable after freezing.
The supervisor accepts direct gameplay `.rs` ownership and requires every
approved file in a compiled Native family closure; test and manifest files
cannot be model-writable (`translation_native.rs:615`,
`translation_execution.rs:1378,1681`). Freeze those declarations before rewriting.

The minimal private Cargo scaffold may contain only the `port` package and
its copied `memory`, `audio`, `pause_equipment_render` modules plus the new
`pause_equipment` module. Copy the upstream license and required harness/pin
inputs. Private `lib.rs` reexports `memory::{Wram, WRAM_SIZE}` and exposes those
modules. This does not add a module to main's held `lib.rs`.

Seed a tracked candidate file with the exact public ABI before installing:

```rust
pub fn equipment_screen_main(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str>;
pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str>;
```

These are signature declarations for review, not an implementation. A scaffold
must fail or explicitly refuse unimplemented behavior; it cannot report a pass.
The full private baseline commit is created after scaffolding. The observed
main seed in scope.json is provenance and must not be substituted for it.

## Complete routine boundary and dependencies

The exact 29 routines, original addresses where available, definition hashes
and call edges are in [scope.json](scope.json). All 29 currently have no stage
observation or Fixture declaration in the shared inventory. Own
`EquipmentScreenMain`, all four categories, their navigation/button handling,
reserve-mode/transfer behavior, arrow/HUD palette behavior and `ChangePaletteValues`.
Do not split this into unconnected leaf implementations.

The root must select its category then call the existing renderer in this order:
`EquipmentScreenDrawItemSelector`, `EquipmentScreenDisplayReserveTankAmount`,
`WriteSamusWireframeTilemapAndQueue`. The three renderer entries and all eight
renderer definitions remain a pinned dependency; their held claim is unchanged.
Reuse `audio::queue_sfx` for `QueueSfx1_Max6` and `QueueSfx3_Max6` with canonical
queue state. This boundary queues sounds; it does not call an APU hardware service.

Fixed Rust dependencies are `memory.rs`, `audio.rs` and
`pause_equipment_render.rs`, with exact current hashes in scope.json.
Pinned C is `sm_82.c`, support `sm_80.c`, `sm_81.c`, `sm_rtl.c` and their
authenticated headers. Use the existing ROM with SHA-256
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
Dynamic closure includes four category functions, two tank functions and the
glowing-arrow function table; lexical call edges alone do not prove that closure.

`ChangePaletteValues` is a static pointer/count helper. Verify it through
button-response and plasma/spazer callers with exact target footprints; do not
invent an unsupported three-argument direct fixture. The remaining 28 routines
have zero, one or two scalar arguments. scope.json records their exact arities
and return types for ordinary `family-verify` declarations.

## Fixed verification requirements before rewriting

Use a plain Native family, ROM-backed `dispatch_rom`,
`rust_imports = ["audio", "pause_equipment_render"]`, and no callback adapter.
Use complete 128-KiB WRAM and raw-return comparisons with actual compiled C.
Serialized SFX queue order, OAM output and seven-byte VRAM queue entries are
compared as canonical state. This proves neither hardware store order nor Live
APU/PPU behavior; those remain outside this Offline domain.

The coordinator and fixture author must freeze a finite manifest before the model
sees a Rewrite capsule. Require direct fixtures for the 28 callable entries
and complete-root cases/continuations that exercise each category and each
static-helper branch. The exact case count and input matrix become contract
coverage; do not install an empty manifest or a representative subset as whole
scope acceptance. The bounded declared domain must include:

- Category/item selection and navigation: tanks 0..1, weapons 0..4, suits 0..5,
  boots 0..2; each applicable direction, collection gaps, hyper-beam refusal of
  beam selection, saved-selection restoration and raw `0xffff` failure returns.
- Equip/unequip operations for beam/suit/boot categories, exact palette and
  tilemap footprint, and both plasma/spazer exclusion directions.
- AUTO/manual toggles, held versus new A input, empty/full reserves, health
  saturation, transfer delay/carry and chained completion. Canonical state
  uses category/item `0755`, delay `0757`, equipment `09A2..09A8`, health
  `09C2/09C4` and reserves `09D4/09D6`.
- Arrow animation across the 32-frame palette cycle, solid on/off footprints,
  and both actual garbage-ROM HUD inputs `82998B` and `829997` (12 bytes each).
- Complete-root renderer effects and valid OAM/animation/VRAM queue boundaries,
  plus SFX queue wrap/full/disabled states supported by the pinned audio module.
- Independent semantic controls for mutually exclusive beams, reserve conservation
  where the C contract guarantees it, exact unchanged-byte footprints and source
  order. Invalid selectors, ROM bounds and queue/output bounds must refuse
  without changing WRAM; do not call unsafe C on those negative inputs.

Reuse existing source and declarations as dependencies, not prior verdicts as
new evidence. Fixtures must cover source-described C quirks instead of repairing
them. The equipment invocation is not the full pause state machine or controller
poll scheduler. Root preflight must cover dependencies before material changes.

Run the independent Rust controls with `cargo test -p port --locked --offline
--test pause_equipment` under the original 60-second initial-test bound.
The maintained Native Installation has focused and Offline stages, both using
fresh evidence for the installed candidate domain; it does not automatically run
this separate Cargo test. Record the Rust controls separately and do not claim
the Native stage ran them. Candidate/integration gates use the same fixed
declarations; each Native family is at most 3600 seconds, compiler 180 seconds
and fixture 5 seconds. Freeze measured finite case/output/context/checkout
allowances rather than importing ticket 10's whole-catalog allowances.

Construct `NativeProvenance::ReviewedCatalog` from this requirements artifact
and the reviewed closed catalog, with their real hashes and a nonempty rationale.
Prepare the full compiled dependency closure and derive dependencies, verification,
pinned-C and adapter hashes from the actual baseline. Every potentially affected
family in this private catalog must have a contract. Historical passing evidence
is unnecessary for this reviewed-catalog mode.

## Review boundary and retained ownership

Next action: coordinator reviews the three-path source ownership, one-file model
rewrite boundary, fixed tests/manifest, private setup files and the complete
declared equipment domain above. Then create the private scaffold and finite
fixtures, freeze their baseline and prepare the launcher. No additional
orchestration subsystem is needed. A tiny Rust packet writer may call the existing
native preparation functions to compute typed seals; no Python tool or duplicate
verification engine is warranted.

Keep `shared-game-host` / `48656a2-host-v1` / attempt 1 and
`integrate-shared-game-host` / `48656a2-host-v1-1` / attempt 1 under
`ceres-20261013`; keep `compact-controller` / `48656a2-compact-v1` / attempt 1
and its submitted worktree; keep `pause-equipment-render-offline` /
`53e916e7fc7edf90ffde5e953a18bcc43fb0ee931256ebd01e446757818f27f1` / attempt 1
under `decomp-run-20261013`. This packet grants none of those paths or identities.

Observed owners on 2026-10-14: current logical authority
`01a136a0-49bb-7690-a360-48def3c2c09b` is notLoaded with its latest turn completed;
host worker `01a133ec-68e2-7033-9ee7-dba26d4db746` and compact worker
`01a133ec-4ef9-78b2-a8d7-248726023ce5` are notLoaded with completed submissions;
renderer worker `01a13623-3b41-7822-b0b3-a99f73df84ac` is idle with a completed
submission. These observations preserve ownership and do not release claims.
The process scan found active ticket 10 Cargo/reader/retention writers; no
replacement or cleanup of those processes was attempted.

## Cleanup

This packet preparation performed reads and authored only these small review
files. It ran no producer, test, fixture, model exchange or inventory update.
No source repository, executable, build tree or run directory was created.
Reclaimed bytes: 0. Last observed free space: 164840325120 bytes.
After future runs, complete both scoped cleanup passes, retain exact source and
receipts, retire disposable private checkouts and authenticate retained evidence.
Private source translation/verification and production integration stay distinct.
