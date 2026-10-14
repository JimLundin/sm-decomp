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

## Read-only directory correction: immutable v4

The Rust directory probe retains 256 compiler-output placeholders, opens and
enumerates an existing empty directory 4,096 times through `read_dir`, and opens
it another 4,096 times through `open(O_RDONLY | O_DIRECTORY | O_CLOEXEC)`. With
immutable v3 it timed out at the unchanged five-second bound (5.01 seconds).
The complete `O_TMPFILE` bit combination includes `O_DIRECTORY`; the v3 mask
therefore incorrectly requested full extent scans for ordinary directory reads.
V4 tests that combination separately, while preserving access/create/truncate
checks and the conservative `openat2` path.

Retained root: `/tmp/sm-ticket10-native-proofs/confinement-v4`.
The optimized helper is `tools/sm-translation-sandbox`, SHA-256
`1ec74e0bcc1024ae460d74b47148114526bf6743e2cc4a87302f9ecf551ab6d9`.
All seven runtime tests pass in 4.72 seconds; all eight original confinement
tests pass in 42.69 seconds, including the live-private-descriptor refusal,
unlinked quota controls and six actual Rust linker-descendant compilations.
Exact source, helper, readers, build log and red/green logs remain retained.
V4 changes only helper observer flags, so v1/v2/v3 host clients and the wire
format remain compatible. Existing helpers are unchanged.

No producer from these tests remains active. The test sandboxes and temporary
fixture directories were removed by their existing teardown. Remaining proof
files are retained evidence; final utility cleanup, build-tree and worktree
retirement remain with the integration owner. Free space: 165,076,750,336 bytes.

The separate transient descriptor EACCES observation from Cargo `compact-02`
is still under diagnosis; v4 does not claim to fix a live exec transition.

## Per-producer native reservations and complete captured-Ridley inputs

New native intents record reconciliation mode. Every producer still reserves
its full worst case before publishing an invocation or launching. A completed
producer then releases only the unused part of that reservation; the charge
retains its authenticated operation references, parent output/log copies,
invocation/timing framing and any future callback snapshot expansion. Large
Program/argv declarations receive a serialization-derived framing allowance.
Successful and failed producers follow the same reconciliation. Incomplete
producers keep their reservation. Authentication checks each original peak and
the exact contiguous retained reservation chain. Historical intents and their
unchanged replay keep cumulative mode.

Real proof `/tmp/sm-ticket10-native-proofs/reservation-v2` passed in 0.54 seconds:
two successful confined printf producers and one deliberate invalid-format
failure. Worst-case producer reservation: 3,351,086 bytes; whole proof gate:
3,449,390 bytes; completed retained charge: 32,989 bytes. Expired replay left
producer results unchanged, an insufficient gate created no invocation, both
cleanup passes reclaimed zero bytes, and post-cleanup replay authenticated.
V1's failed test setup remains retained: GNU printf accepted the intended
invalid-option text, so the assertion failed after three successful producers.
Its finished output received both zero-byte cleanup passes; no failure was
converted into acceptance.

Captured Ridley's original declaration contains 5,371 dependency paths. The
Ridley recipe now has a fixed 8,192-path ceiling at contract construction,
preparation and candidate authentication; ordinary recipes retain 4,096.
Its original `ridley-subsystem-events.jsonl` is 71,595,494 bytes, exposing the
separate generic 64 MiB hash/snapshot bound. Only that exact original path and
SHA-256 (`f1f286cae8090e5f23f8e1e9b0a20fcc4de41d91766f349c9a0be7518eec1f35`)
select a fixed 128 MiB streaming profile. The original file is preserved whole
as `dependency-ridley-subsystem-events.jsonl`; all other declared inputs keep
their normal limits. Family artifact inventory and source authentication, native
candidate checks, and native retention references recognize only this exact
reviewed asset. Streaming copies authenticate identity, account reads/writes,
recover their pending publication, and preserve immutable completed bytes.

Core tests retained in `/tmp/sm-ticket10-native-proofs/core-tests-v1` passed:
eight native tests, ten bounded-input tests, five dependency tests and three
reservation tests. The full captured contract test preserves every coverage row
and rejects the same large asset for plain/callback recipes. Generic tests keep
the 64 MiB read/hash default and prove explicit streamed-copy budget, identity,
pending-publication and unchanged-replay controls.

Complete catalog preparation, whole pause 247/360 matrices, specialized native
proofs and historical receipt reauthentication remain next. Exact core reader
and pre-commit source overlay are retained with reservation-v2. The temporary
translation_migration.rs compile-only edit remains unowned and uncommitted.

## Complete catalog audit and whole pause matrices

Frozen source `a817f07`, with the separately retained compile-only migration
overlay, prepared every dynamically discovered catalog entry in
`/tmp/sm-ticket10-native-proofs/catalog-v4`: 177 families and 150,204 declared
Fixture pairs. The separate direct-animation declaration adds 360 pairs,
for 150,564 when that Original constituent is included. The preparation-only
audit finished in 264.95 seconds under the unchanged 600-second bound.
No compiler or Fixture ran in this audit. All original captured-Ridley inputs
and generated PLM inputs were retained whole.

Catalog summary SHA-256:
`c441fbc043d0002b0a9963813bab87c3f0de0d67453c407a4177dc61a4014124`.
Auxiliary summary SHA-256:
`c78f5b4b5f46e7ac73bf2e1716e64efdaaa20bcc3a9546896941c457c012b74a`.
Both cleanup passes completed with zero bytes reclaimed. The external manifest
`catalog-v4-retained.before.sha256` covers every pre-cleanup file; its complete
post-cleanup SHA-256 check passed. Frozen source and preparation declarations
remain dependencies, not disposable build output.

Proof `/tmp/sm-ticket10-native-proofs/pause-v1` passed both complete pause
constituents with immutable helper v4 in 305.69 seconds total:

| Constituent | Fixture pairs | Confined producers | Evidence bytes | Retained reservation |
| --- | ---: | ---: | ---: | ---: |
| pause-equipment-render | 247 | 496 | 182,311,149 | 203,935,843 |
| pause-direct-animation | 360 | 722 | 258,712,081 | 283,053,081 |

Both unchanged replays preserved producer results. Both ordinary cleanup
passes reclaimed zero bytes. Equipment-render deduplication reclaimed
44,564,480 bytes across 741 snapshots; direct-animation deduplication reclaimed
97,517,568 bytes across 1,080 snapshots. Both native receipts authenticated
after cleanup. Receipt SHA-256 values are, respectively,
`006655528c4d766fef701077ef8849337d7d0a942da11ec1d9b0f9f37233489b`
and `33c9c60b69fae32f0dd37c88d56112ed10fb6a7f269243b8e323e82a5394edc2`.

The retained optimized reader is
`/tmp/sm-ticket10-native-proofs/readers/native-tests-a817f07`, SHA-256
`e53642a52a14f15a57e0999cdb26ad481bd61ff413d5d7192c3b1c7b54c89b1d`.
It also authenticated historical cumulative-reservation receipts without
executing producers: host-v4 room-scroll (8 producers, 581 references) and
host-v3 projectile (16 producers, 545 references). Expired replay and retired
compiler/helper paths were exercised, with results unchanged. Their
`a817f07-post-cleanup-authentication.log` files retain the exact results.

Measured post-dedup allocation for the two pause proof directories is
358,662,144 bytes. A deliberately rough projection of this 607-pair sample to
the 150,204 catalog pairs is 109,132,542,404 logical evidence bytes,
120,506,893,477 retained reservation bytes, and 88,752,040,655 allocated bytes.
This is a scale estimate, not an upper bound: callback expansion and fixed
source/compiler overhead differ by family. It does not establish that one
aggregate 64 GiB gate can hold the catalog. The integration owner retains the
original production limits and owns the full installed catalog run.

## Captured-Ridley finalization failure retained

Proof `/tmp/sm-ticket10-native-proofs/ridley-v1` stopped after 417.83 seconds
with `trusted input discovery path budget exhausted`. The captured family
verdict reports all 670 original Fixture pairs verified, but no native receipt
was published, so this is not native acceptance. All 1,343 original confined
producer results remain. The 6,457-pair subsystem recipe never started.

The family tree contains 26,212 files and 670 case directories. Finalization
inventories it, measures its evidence extent, then repeats that measurement
inside nested native authentication before the later family inventory check.
Those repeated walks cumulatively exceed the unchanged 65,536 discovery
counter. The correction must bound each finite declared discovery workload
without dropping any cases or relaxing compiler/Fixture limits. Large complete
matrices also require review of receipt-document size: the measured direct
animation receipt is 4,634,483 bytes for 360 pairs, implying about 83 MiB for
6,457 pairs at that format, beyond the generic document bound.

The original family verdict and intent are separately preserved under
`initial-finalization/`. Ordinary cleanup reclaimed zero bytes; deduplication
reclaimed 292,814,848 bytes across 3,350 snapshots. Every file in the external
`ridley-v1-retained.before.sha256` manifest passed its post-cleanup check. Native
receipt authentication remains incomplete because finalization failed. The
integration owner requested no further
broad native runs here; the priority is the concrete confinement exit-race
correction and finite finalization fixes.

Current free space at this checkpoint: 163,850,432,512 bytes. Final integrated
worktree and disposable build-output retirement remains with the integration
owner after exact source retention and post-retirement authentication.

## Finite discovery passes authenticate all retained captured-Ridley producers

Native inventory, extent and authentication tree passes now each use the
unchanged 65,536-path discovery ceiling. They continue sharing every logical
byte charge and the earliest enclosing deadline. Ordinary nested preparation
still shares its original counter. This isolates repeated discovery of the
same completed tree without extending any compiler/Fixture or whole-gate limit.
It does not yet support one tree larger than the ceiling; complete 6,457-case
family artifact partitioning and compact operation indexes remain separate work.

Reader `/tmp/sm-ticket10-native-proofs/readers/native-tests-discovery-v1`
and the exact source overlay under `discovery-v1/source/` passed all 11 bounded
input tests, eight native tests and two callback tests. One earlier filter
matched zero tests and is retained as such; `bounded-tests-complete.log` records
the actual eleven-test run. The new regression checks that repeated discovery
passes each keep their limit while all passes share byte and deadline failures.

The read-only captured-Ridley finalization audit passed in 27.15 seconds after
cleanup. It authenticated all 670 original Fixture pairs and all 1,343 existing
producer results, including the exact candidate and pinned C source, artifact
inventory and repeated evidence extent. Producer-result modification times
were unchanged. It deliberately published no native receipt for the expired
failed run. Its audit record and log remain beside that proof; the original
failure and original family verdict remain unchanged and separately retained.

All four earlier failed host-v1/host-v2 receipts also authenticated with reader
`a817f07` after their cleanup, preserving their failure status. Host-v1 projectile
retains two producers/212 references, host-v2 projectile two/213, and both
room-scroll failures one/113. These checks did not launch compilers or Fixtures.

The pure callback and PLM command builders now own their argument mapping and
are shared by producer construction and the separate authenticator. The new
reader reauthenticated unchanged room-scroll and projectile material receipts
(8 producers/581 references and 16 producers/545 references respectively).
No broader native matrix was started for this refactor.
