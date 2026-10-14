# Ticket 10 confined native recipe handoff

Owner: `/root/native_recipes`, delegated by `/root`.
Worktree: `/tmp/sm-ticket10-native-20261014`, branch `ticket10-native`, baseline `808efe2`.
No production claim, submission, ledger or installed policy mutations are authorized here.

## Initial scope

Read domain/cleanup/session rules and original ticket. No producers were running
at the initial transport recovery checkpoint.

The existing native adapter confines each compiler and fixture separately, but
only supports plain families. Implementation will preserve that historical
contract and add explicit optional recipes for dispatcher/projectile, callbacks,
PLM, Ridley, generated PLM fixtures and pause direct animation. New reviewed
catalog provenance will bind catalog, requirements and source inventory without
pretending a prior failed family passed.

Callback artifacts have a dynamic count. The confined recipe will produce a
bounded stream of all before/after full-WRAM snapshots plus the complete ordered
callback log, then the trusted parent will reconstruct the historical artifact
layout and use the original callback checker. No verifier directory is writable
by either candidate or oracle.

Direct animation must preserve the original plan at
`.scratch/decomp-coverage/renderer/direct-animation.toml`: 360 fixture pairs,
canonical ROM sentinel-derived last frames, full WRAM, exact C three-word call.
It will use `pause-direct-animation` and a separate declaration recipe; the
primary family remains `pause-equipment-render` (247 pairs). Root owns composite
admission and all eight-routine union.

## Initial next action

Add a failing public native recipe test for both material game-host matrices;
extend strict compiler/fixture capability declarations, generated remapping and
exact authentication; then run bounded real controls and full conservative
catalog. Retain exact source and perform both cleanup passes before handback.

## First implementation checkpoint

`cargo check -p pipeline --locked --offline --bin sm-pipeline` passed.
The public frozen shared-host contract test failed at the original plain-only
rejection, then passed with all ten original fixture pairs (no changed cases).

Added optional NativeRecipe, reviewed catalog provenance, named strict compiler
inputs/outputs, typed fixture commands and original specialized checkers. The
PLM generator now has a pure reconstruction function; preparation verifies all
1,444 generated fixture rows and the generated ROM. Direct animation reconstructs
the exact 360-case original matrix with the existing gameplay dispatch packing.

First real run: `/tmp/sm-ticket10-native-proofs/host-v1/summary.toml`.
Both rejected receipts authenticated, replayed without modifying a producer,
and authenticated after both cleanup passes. Projectile C compilation exposed
missing `headless_frame.h` in the protected closure; corrected. Room-scroll Rust
compilation retained `observe candidate descriptor directory for pid 21:
Permission denied (os error 13)` from the installed sandbox. This is a retained
failure, not a passing original acceptance. Exact source remains under
`/tmp/sm-ticket10-native-proofs/host-v1/source`.

Fresh bounded run `host-v2` is active with the corrected header closure. Shared
root `/home/jim/sm-decomp/out/pipeline`, workers 4; private target
`/tmp/sm-ticket10-native-target`. Parent/root was notified of the observation
failure and owns any confinement-runtime changes.

Temporary compile-only migration match in `translation_migration.rs` is not
owned, will not be committed; root already owns the permanent exhaustive match.

## Catalog preparation API checkpoint

The source/inventory authentication can now be reused within one reviewed
catalog through opaque `NativeCatalogInputs`, constructed by
`prepare_catalog_inputs_until` and consumed by
`prepare_scope_in_catalog_until`. Each resulting PreparedNative still seals the
exact authenticated Source and inventory; run authentication still checks the
actual source. Reviewed catalog provenance checks the registered family entry
against its sealed catalog. Root separately checks complete current catalog
equality and every conservative family key before using this context.

Reviewed contracts also freeze `tools/port/batch.toml`, captured before/after
fixture assets, and original absolute dependency files. Full-C recipes allow the
original callback/PLM declarations to omit a bank name. No original declaration,
claim, submission or installed policy was changed.

`catalog-v1` is the bounded complete-current-catalog preparation audit, retained
at `/tmp/sm-ticket10-native-proofs/catalog-v1/summary.toml`. Its 600-second
preparation deadline covers all 176 families and no compiler or fixture process
is launched. At this checkpoint, 19 families through `ceres-platform` prepared
with their original fixture counts and no structural failure. The audit is
still running in exec session 83663; do not start a duplicate. Repeated full
manifest parsing remains visible as preparation overhead and will be addressed
if the bounded audit fails.

## Actual-C proof and cleanup checkpoint

`host-v2` retained both 60-second compiler quota failures with unchanged replay,
both cleanup passes, and post-cleanup authentication. A fresh `host-v3` used a
180-second compiler producer limit; the separate original 60-second Rust-control
requirement was not changed.

At `/tmp/sm-ticket10-native-proofs/host-v3`, `game-host-projectile` passed all
seven original fixture pairs, including both ordered HDMA controls and the
material PLM control. Its receipt replay left producers unchanged. Ordinary
cleanup reclaimed 0 bytes; deduplication reclaimed 2,883,584 bytes across 41
snapshots; post-cleanup authentication passed. `game-host-room-scroll` retained
the sandbox stop `observe candidate descriptor directory for pid 35: Permission
denied (os error 13)` during Rust compilation. Its failed receipt replay and both
cleanup passes authenticated, with no acceptance recorded. Exact proof source,
operation receipts and helper identities remain retained.

Root subsequently assigned confinement-only diagnosis/fix work to this owner,
including a narrowly opt-in synthetic `/proc/self/exe` for exact historical
readers. The installed helper must remain unchanged; a new helper will be built
at a distinct retained path after focused tests. Root owns checkout/retention
modules, production integration and full catalog execution. Remaining work:
finish this preparation audit; diagnose the EACCES observation; verify all
three original room-scroll pairs with 17 callbacks each, then specialized
controls; replay and clean every new proof.

Remaining filesystem space at this checkpoint: 175,798,161,408 bytes.

## Preparation audit result and parser follow-up

`catalog-v1` finished after 655.77 seconds. Twenty original declarations through
`collision` prepared. The remaining 156 entries are explicitly recorded as
`trusted input whole-stage deadline exhausted`, beginning at `control`; these
entries have no preparation acceptance. No compiler or fixture ran.

The repeated discovery/registration/provenance path decoded a discovered
declaration up to 13 times. The follow-up parses once during contract construction
and once during preparation, then carries its authenticated registration through
provenance. The expected manifest path, exact asset digest, registered recipe,
and complete fixture coverage remain checked. The audit no longer performs an
extra manifest decode just to obtain a candidate path. A fresh audit will use the
same 600-second budget.

All six active native unit tests pass (five real-proof tests remain explicitly
ignored), including the ten original shared-host fixture pairs and a focused
test rejecting an aliased discovery path. The real proof now accepts an optional
`SM_NATIVE_RECIPE_HELPER` path for a separately retained new helper; its default
still names the original installed helper.

## Confined runtime diagnosis and fix

The Rust runtime probe reproduced the exact native descriptor error on three
consecutive runs in 0.08–0.11 seconds. Minimization reduced it to one worker
thread writing an unlinked file while another thread continued writing. Removing
the concurrent writes made all 16 trial repetitions pass on the old helper.

Ranked hypotheses were pending task exit, a live exec transition, and genuine
live descriptor inaccessibility. Targeted diagnostics recorded PID 7 as zombie,
Tgid 3, PPid/TracerPid 2, UID/GID 0. A non-consuming
`waitid(P_PID, 7, WEXITED | WNOHANG | WNOWAIT | __WALL)` returned that exact PID,
`CLD_EXITED`, status 0. Raw red and instrumented output are retained under
`/tmp/sm-ticket10-native-proofs/confinement-diagnosis`.

The fix permits an EACCES descriptor-directory observation to end only when the
kernel confirms termination of that still-owned, unreaped PID. The main observer
retains and reaps its exact exit result. Live private descriptor denial and its
unlinked-file quota bypass test still pass. Temporary instrumentation is removed.

The Cargo worker's separate differential established that inherited stdin worked
and null stdin failed despite readable request/source files and UID 0. The Rust
regression isolated EACCES at `/dev/null` open. Bubblewrap's normal binds are
nodev; only the two reviewed device nodes, `/dev/null` and `/dev/urandom`, now use
device binds. The regression opens both nodes and starts a child with null stdin,
stdout and stderr; it also asserts that no other device entries exist.

The explicit exact-reader marker is a ReadOnlyBind with source lexically equal to
the canonical Program.executable and destination exactly `/proc/self/exe`.
Launcher-generated empty `/proc/self` directories contain only a symlink to the
already staged `/program`. There is no procfs or host proc bind. Tests verify
the staged device/inode identity and reject path aliases, differing sources,
PID/status/fd paths, and sibling data. Ordinary producers still have no proc path.

All four focused runtime tests pass (including 16 minimized interleavings and
the original 32-thread scenario). The eight existing confinement tests are also
running; their standalone live-private-descriptor regression already passed.
New immutable helper:
`/tmp/sm-ticket10-native-proofs/confinement-v1/tools/sm-translation-sandbox`.
The original installed helpers were not modified. Host clients must rebuild
with the new confinement library as device/self-exe setup happens in the host
launcher; changing the helper path alone is insufficient.

The second same-budget preparation audit finished after 617.57 seconds: 56
families prepared, 119 stopped at the deadline, and `instructions` exposed one
original registered catalog alias that still needs explicit native support.
Neither audit is original actual-C acceptance.

## Runtime and catalog alias follow-up

All eight existing confinement tests passed in 59.34 seconds, including six
real Rust linker-descendant compilations. The v1 runtime checkpoint has 12
passing focused/existing tests.

Current catalog metadata contains exactly one catalog-key/declaration mismatch:
`instructions` selects `tools/port/instructions.toml`, whose original declared
Offline family is `enemy-instructions`. Native registration now finds that
explicit entry by its exact manifest path and authenticates provenance under
the catalog key. NativeContract.family and every original coverage row retain
the declared family identity. Unknown discovered path aliases still reject;
configured keys cannot silently fall back to a different discovered declaration.
All seven active native tests pass, including exact family/coverage preservation
for this alias. Root must keep catalog leaf keys separate from declared native
family names in its composite inventory.

## Optimized runtime v2

The Cargo worker isolated a subsequent EPERM to Rust's `Command::pre_exec`
handshake. Ordinary rustc/null-stdio spawn passed; adding a no-op pre_exec hook
failed. Host strace of the same Rust 1.97 program identified exactly
`socketpair(AF_UNIX, SOCK_SEQPACKET | SOCK_CLOEXEC, 0)`. The Rust confinement
regression reproduced that failure in 0.14 seconds.

The helper now permits only that connected local Unix pair, with exact complete
domain/type/protocol arguments. General socket creation, network families,
stream pairs, pairs without close-on-exec, and other protocols still reject.
The real pre_exec call-site test and its five negative controls pass.

`cargo build --release -p translation-execution --locked --offline --bin
sm-translation-sandbox` produced the optimized helper in 83 seconds. Immutable
path: `/tmp/sm-ticket10-native-proofs/confinement-v2/tools/sm-translation-sandbox`.
SHA-256: `d334899b7f5cc7f64120636a803224d2f83e54c8d049e71e57a9d198a5fdf892`.
Size: 5,667,640 bytes. The eight existing confinement tests pass with this exact
helper (56.80 seconds), and all five focused runtime tests pass (1.96 seconds).
The test harness supports `SM_CONFINEMENT_TEST_HELPER` to validate a retained
helper without rebuilding or replacing it. V1 and original installed helpers
remain intact; the v1 host launcher/wire protocol remains compatible with v2.

Exact tracked input snapshots for both earlier preparation audits are now
retained in `catalog-v1/source` (811b2f0) and `catalog-v2/source` (dc91352) under
`/tmp/sm-ticket10-native-proofs`. Original absolute private inputs, pinned C and
the original requirements path remain dependencies. The preparation audits have
no compiler/fixture receipts and do not establish actual-C acceptance.

## Resumed optimized native proof checkpoint

Commit `ac2b343` adds an optional explicitly frozen source to the complete-catalog
preparation audit and separately checks the original 360-pair direct-animation
declaration. The primary summary still enumerates the dynamically discovered
catalog without a fixed family count. The original 600-second preparation bound
is unchanged.

The optimized pipeline test reader build remains active in exec session `38412`,
using `/tmp/sm-ticket10-native-target`. Resume that session; do not duplicate the
build. Fresh `catalog-v3/source` is archived from `ac2b343` with the private ROM
copied in. Its exact temporary compile-only migration overlay is retained as
`catalog-v3/compile-only-migration-overlay.patch` and present in the source
snapshot; it is not committed to this worker branch.

The Cargo worker's reported possible ptrace wait was corrected by exact ancestry:
the process was compact dependency preparation and continued progressing. No new
confinement-runtime defect is established. Pause candidate and integrated Rust
controls passed on v2; the Cargo worker owns those receipts and the separate
host-control timeout diagnosis.

Next commands use the retained optimized test reader for `catalog-v3`, then whole
native recipe families with immutable v2. Original case coverage and gate budgets
remain fixed. Replay, both cleanup passes, source retention and post-cleanup
authentication remain required for every passing or failed new proof.

## Read-only-open performance correction and optimized catalog result

Commit `ec09afe` preserves all quota-error tracing and extent checks around
mutations, while skipping full tree/descriptor walks for provably read-only
`open` and `openat` calls. `openat2` remains conservative because its flags are
in candidate memory. Cargo's linker had used only seconds of CPU while the
helper used minutes scanning an existing output tree.

The new Rust probe creates 256 empty compiler-output files and performs 8,192
read-only input opens. The first setup trial exhausted its deliberately small
inode quota; that diagnosis is retained separately. With adequate inode capacity,
immutable v2 hit the unchanged five-second timeout. Immutable v3 passes the same
probe with all six focused tests in 2.22 seconds, and all eight original
confinement tests pass in 62.84 seconds, including six real Rust linker controls.
Evidence, source, test readers and build log are retained under
`/tmp/sm-ticket10-native-proofs/confinement-v3`. The helper is
`tools/sm-translation-sandbox`, SHA-256
`21f1bdb8d4bb10cbad2c5c23a24e7e146b2ed9a17fc4695b41d7813c3f2b66c5`.
No client/wire-protocol change is required. Earlier helpers remain untouched.

The optimized native test reader finished in 7m22s and is retained at
`/tmp/sm-ticket10-native-proofs/readers/native-tests-ac2b343`, SHA-256
`319b6e4be120b07f3fc46728598529a16e29ab4fa955089a39e87d3d54bd19e9`.
The complete `catalog-v3` audit finished in 253.61 seconds under its unchanged
600-second bound. The current discovered catalog contains 177 families: 176
prepared and `ridley-subsystem-captured` rejected its known original callback
metadata. The auxiliary direct-animation declaration prepared all 360 pairs.
No compiler or fixture ran in this preparation-only audit. The captured-Ridley
registration correction and original-fixture preservation test are authored but
not yet compiled at this checkpoint.

## Passing original room-scroll matrix

Fresh proof `/tmp/sm-ticket10-native-proofs/host-v4` used immutable v3 and passed
all three original room-scroll fixture pairs, each with all 17 ordered callbacks.
The proof completed in 122.10 seconds. Unchanged replay preserved every producer,
ordinary cleanup reclaimed zero bytes, and deduplication reclaimed 13,762,560
bytes across 213 checked snapshots. Post-cleanup native authentication passed.
Its exact input source, eight producer results, native receipt, family verdict,
cleanup records and summary remain retained.

Room-scroll's final lease is closed and no native producer remains active.
Further compiler/fixture launches are deferred while the Cargo worker runs its
coordinated jobs4 host/compact proofs. The standalone reader still uses cumulative
worst-case reservations; the parent's integrated streaming reservation path is
required for complete large PLM/Ridley actual-C runs within the frozen byte cap.
No family coverage or original time bound was reduced to obtain a pass.

Remaining filesystem space at this checkpoint: 167,771,295,744 bytes. Runtime
test sandboxes were torn down and descendants reaped by the confinement tests;
their retained logs/source/readers have no unsealed run intermediates. Final
build-tree and integrated-worktree retirement remains the integration owner's
cleanup responsibility after the remaining tests and source retention.
