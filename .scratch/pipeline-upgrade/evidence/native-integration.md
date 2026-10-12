# Current native integration evidence

Status: implementation and verification complete; ticket 01 remains claimed
until integration-owned implementer worktree removal.

Source: migration `c89db9bcf824a3c4f973550d390a95c3f5d05f97` delivered and
already merged at the original integration basis `3ad1bf3`; current upgrade
source freeze `99d799c`. Fixes `47fefbf`, `3da347f`, and `d37c98f` close native
manifest freshness, input rebinding/downgrade, and missing observed-projectile
root imports. Retained historical evidence and v2 formats are preserved.

Current release driver:
`/tmp/sm-pipeline-upgrade-integration/out/ticket-01/retained-tools/sm-pipeline`,
SHA-256 `785ec470d5955b16560bf51c50b40c404e1cc48655c14338b074886b7025ce93`.
It was built with `cargo build -p pipeline --release --locked --offline`;
`native-family-release.log` records the optimized build. A retained release
`sm-evidence` alongside it has SHA-256
`3ba222fe9ac39c0713257fb43eddf733c4bc2cc3564cf8c9a61b359d9f264d05`.
The live run retains its separately sealed reader, SHA-256
`fbcce2ee4cf7228dabde37af47e4e57c4ff5920c253d2400ca9df53c668b2307`.
The explicit worker
root is `/tmp/sm-pipeline-upgrade-workers-2`, capacity two: the developer's
existing root has capacity four and cannot be reused for this configuration.

All new admission sources and output are under the stable integration checkout
`/tmp/sm-pipeline-upgrade-integration`; no sealed evidence points to disposable
implementer source paths. Original ROM and pinned dependencies are shared via
symlinks to the developer checkout; movie bytes were copied unchanged.

## Public controls

Logs under `/tmp/sm-pipeline-upgrade-integration/out/ticket-01/logs/`:

- `manifest-red.log`: current manifest drift erroneously authenticated before
  the fix. `manifest-downgrade-red.log` and `manifest-rebind-red.log` reproduce
  missing seal and retained-path rebound bypasses.
- `native-family-red.log`: actual-C native initializer compiled its generated
  Rust wrapper unsuccessfully because its aliased root module was absent.
- `native-family-green.log`: ten family controls pass, including the actual
  native initializer's full WRAM/eight-word host comparison and current
  manifest drift, missing seal and retained-path rebinding refusals.
- `native-family-clippy.log`, `native-family-fmt.log`: strict pipeline Clippy
  across all targets and workspace formatting pass.
- `port.log`: complete current Rust port suites pass. Actual callback and live
  ABI controls passed in `integration-controls.log`; the initial observations
  invocation lacked its nondefault reader path and was superseded by
  `evidence-process-controls.log`, which passes observations four, packed one
  and process/cache four controls. Native Python/JSON guards also pass.

The typed Rust verification utility retained at
`out/ticket-01/control-tool/src/main.rs` exercises two actual native
allocation/frame cases via the public family CLI. It changes the inherited
WRAM and eighth host word, rebounds inventories, and verifies semantic refusal
before restoring and authenticating the exact original evidence. Controls:
`out/ticket-01/continuation-controls/{host-word-eight,inherited-wram}.stderr`;
restored authentication: `logs/continuation-auth-after-cleanup.toml`.
The original run is
`out/ticket-01/continuation-controls/evidence/1791772687234117-532-0`.

## Current-source predecessors and supplemental admission

- Baseline `out/ticket-01/baseline/1791771350848263293-2`: two fresh actual C
  subjects, 9,000 frames and 23 full-state decompressor captures.
- Decompressor `out/ticket-01/decomp/1791771616417747396-2`: accepted complete
  native offline contract, 40 synthetic fixtures plus 23 captures, three
  original-ROM controls and eleven rejecting fault controls. Pointer:
  `out/ticket-01/decomp/decomp-acceptance.toml`. Post-cleanup authentication:
  `logs/decomp-auth-after-cleanup.toml`.
- RNG `out/ticket-01/rng/1791771805940068260-2`: all 65,536 seeds, 16 captures
  and 8,783 ordered effects in four independent 9,000-frame subjects,
  zero CPU fallback. Post-cleanup authentication:
  `logs/rng-auth-after-cleanup.toml`.
- Supplemental Ceres pad
  `out/ticket-01/final-ceres-pad/1791772581029178-2-0`: 1,296 actual-C cases.
- Supplemental Ceres platform
  `out/ticket-01/final-ceres-platform/1791772582137423-2-0`: 378 actual-C cases.
  Both supplemental contracts retain their exact declared cases and use the
  existing complete-bank dispatcher mode to resolve native supporting calls;
  small manifests remain at `out/ticket-01/supplemental-manifests/`. The initial
  pad attempt without that mode rejected at C link and is retained separately.
  Post-cleanup authentication: `logs/final-ceres-{pad,platform}-auth.toml`.

The six-family current source batch is
`out/ticket-01/final-observed/batch.toml`, status complete, with 13,624 actual-C
cases: 5,409 initializers, 1,840 native continuation-chain cases, 577 frame
boundaries and 5,798 rendering/contact cases. All six authenticated reuse checks
pass after cleanup in `logs/final-batch-reuse-auth.toml`; full results are in
`logs/final-batch.toml`.
The earlier rejected native batch remains at `out/ticket-01/observed/`,
retaining the compile failures that motivated the module import fix.

## Full native text admission

`out/ticket-01/full-native-text/1791773005140544286-2/verdict.toml` records four
fresh C/mixed processes through all 129,712 inputs. The verdict SHA-256 is
`66c126b8b6f58397fbbba2d07aaa503fbf90495dc605959ed1582e7b5249ab61`;
`logs/full-native-text.toml` and `.stderr` retain the command output. This is
native text capture into `sm-evidence-v2`, with the synchronous eight-word
WithGfx projectile ABI. Exact hardware/game state, full terminal WRAM/SRAM,
ordered PPU/APU effects and captures agree; remaining CPU fallback is zero.

Both mixed subjects exercise projectile entries 16 / 95,325 / 8 times,
exclusively in Rust, with zero refusal. Scheduler counts for all nine admitted
IDs are 617 / 50 / 50 / 201 / 75 / 152 / 177 / 528 / 524. Allocations are
16 Ridley, four Ceres pad and four Ceres platform, exclusively in Rust. The
public verifier also admits the material full-WRAM/all-eight-host captures.
The predecessor work runs in Rust: 419 decompressor and 125,529 RNG calls per
mixed subject. Rendering/contact have the separate offline evidence above;
original-ROM parity is not asserted by this live route.

The full command passed before cleanup, and `live-authenticate` passed after
cleanup with four subjects, 1,345 artifacts per subject, retained binary true
and `sm-evidence-v2` storage. Receipt:
`logs/full-native-text-auth-after-cleanup.toml` and `.stderr`.
Text capture measured 1,058.047099159 seconds overall. The existing packed
store mutex serialized captures despite worker capacity two; ticket 02 owns
process/publication measurement and the next scheduling changes.

## Completed scoped cleanup

For each finished run or batch, the retained `sm-pipeline` applied
`cleanup --root <directory> --apply` followed by
`cleanup --root <directory> --deduplicate --apply`. Full batch deduplication
was automatic and the explicit final batch cleanup/dedup each reclaimed zero.
Receipts are under `out/ticket-01/logs/`.

| Scope | Bytes reclaimed | Receipt |
| --- | ---: | --- |
| Baseline, decompressor, failed observed batch, initial failed pad, continuation controls | 0 | Corresponding `*-cleanup.toml` and `*-dedup.toml` |
| RNG deduplication | 176,081,290 | `rng-dedup.toml` |
| Current six-family batch automatic deduplication | 2,722,496,512 | `final-batch.toml` cleanup table |
| Current supplemental Ceres pad / platform deduplication | 233,570,304 / 96,731,136 | `final-ceres-{pad,platform}-dedup.toml` |
| Superseded Ceres pad / platform full-bank deduplication | 233,570,304 / 96,731,136 | `ceres-{pad,platform}-full-bank-dedup.toml` |
| Full native live remaining build archives | 47,642,562 | `full-native-text-cleanup.toml` |
| Full native live deduplication | 0 | `full-native-text-dedup.toml` |

The scoped run utility reclaimed 3,606,823,244 bytes. Disposable Cargo outputs
were removed with `cargo clean --manifest-path out/ticket-01/control-tool/Cargo.toml
--target-dir out/ticket-01/control-tool-target` and
`cargo clean --target-dir out/ticket-01/target`: 301 files / 42.0 MiB and
5,210 files / 1.9 GiB respectively (`control-tool-clean.log`,
`target-clean.log`). The retained verification-tool source and Cargo files
remain; the original utility ran the now-cleaned release path, whose exact
binary is retained under `retained-tools/`.

After these operations, `df -B1 /tmp` reports 225,024,139,264 available bytes.
The stable source checkout, manifests, predecessors, sealed artifacts,
diagnostic failures, full live subject `evidence.toml` pointers, sealed reader
and shared `out/ticket-01/evidence-store` remain. Current finished evidence
authenticates after cleanup; superseded full-bank runs retain diagnostics and
their old seals, and do not supply current-source admission.
Removing `/tmp/sm-pipeline-upgrade-01` is the integration owner's remaining
completion action. It contains no admission source dependencies; all fresh
source seals refer to the retained integration checkout.
