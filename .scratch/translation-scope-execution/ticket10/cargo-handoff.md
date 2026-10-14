# Ticket 10 Cargo adapter handoff

Status: implementing

Owner: `/root/cargo_acceptance`; integration owner `/root`.
Worktree: `/tmp/sm-ticket10-cargo-20261014`; branch `ticket10-cargo`.
Baseline: `808efe2ee473e633a42bbf330f3872f5a0499df9`.
Owned source: `crates/pipeline/src/translation_cargo.rs` and optional
`crates/pipeline/src/translation_cargo_tests.rs`. Permanent CLI registration
belongs to the integration owner. No production authority, claim, original
submission or gameplay source is writable by this task.

The requested module runs original Cargo acceptance commands against candidate
and integrated frozen source with exact test identities and a hard 60-second
whole-command deadline. It uses durable `translation_execution::operation`
and confinement, sealed offline inputs, bounded evidence, and authenticated
unchanged replay. Cargo success establishes only Rust control acceptance.

Agreed test seams: the public prepared-contract/run/authenticate module boundary
and installed supervisor command. TDD skill read before implementation.

Initial API proposed to root: `CargoKind`, `CargoContract`, `PreparedCargo`,
`StrictRuntime`, `CargoReceipt`, `CargoVerdict`; `prepare`, `run`, `authenticate`,
`references`, `producer_cli`. Legacy readiness contracts remain untouched.

Next: finish the shared-host and compact proofs after the measured supervisor
read-only-open performance correction. Use private target/run roots with
`SM_PIPELINE_WORKERS=4` and `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`.

Preparation red/green: the first public-seam control-inventory test failed on
the unimplemented adapter, then passed after preparation validation, including
`preparation-green-04.stdout`. It requires
the exact original host controls and rejects a renamed control even with an
updated source seal. The real original Cargo proof has reached its intended red
(`original Cargo run is not implemented`) in
`/tmp/sm-ticket10-cargo-proof/real-red-02.stdout`; an earlier fixture-inventory
failure is retained under `real-red.*`.

Current implementation includes original argv/identity tables (31 host, 8+2
compact, 10 pause), frozen embedded compact test-module hashes, public typed
contracts/receipts/authentication, a durable operation per original command,
whole-command deadlines, quota reservations, complete source snapshots and the
installed `translation-cargo-producer --request /inputs/cargo-request.json`
entrypoint. Typechecking and normal producer/unit/public-prefix compilation
passed with temporary local CLI/module registration. The registration is never
part of this worker's commits.

Checkpoint: complete Cargo and environment APIs typecheck. Root may integrate
the source checkpoint while real proofs continue. `prepare_environment` takes
the prepared contract, source checkout, output, runtime, context, timeout seconds
(at most 600), and `build_dependencies`; `authenticate_environment` returns
`Result<()>`, `environment_references` returns retained references. Its separate
producer is `translation-cargo-environment-producer`. Runtime-only preparation
credits no original test gate. Dependency preparation exports vendor libraries
only and excludes source-tree project artifacts.

The first real original31 attempt at `real-01/candidate/cargo-receipt.toml`
retains a 60-second input-discovery exhaustion before any Cargo process started.
It is a bounded failure, not a passing substitute. The new explicit environment
preparation imports the same compiler identity into the same operation root
before the unchanged 60-second original command. Preparation helper typecheck
passed; full proof with the helper is next.

Follow-up hardening freezes environment replay mode/deadline, honors the outer
deadline, authenticates the complete durable intent/reference graph, and bounds
cache decompression/finalization. The copied target cache is writable inside the
quota directory while the published cache remains immutable. The installed
Cargo environment supplies an explicit `/toolchain` sysroot and the integration
owner's narrowly checked `SM_PIPELINE_CONFINED_CARGO=1` executable-identity
fallback. These changes do not extend the original command's 60-second budget.

Real failed attempts preserve separate identities and exact diagnoses:

- `real-02/environment/cargo-environment-receipt.toml`: normal main tried to
  discover its executable through absent `/proc` before producer dispatch.
  The integration owner fixed early producer dispatch.
- `real-03/environment/cargo-environment-receipt.toml`: the initial guard treated
  bubblewrap's empty helper bind target as an exposed helper image. The adapter
  now permits only an absent or empty regular target.
- `real-04/environment/cargo-environment-receipt.toml`: `EACCES` before compiler
  version output. The separate deliberately failing Rust diagnostic at
  `diagnostic-01.stdout` proves request/source reads succeed and the same
  `rustc --version` succeeds with inherited stdin but fails with `Stdio::null()`.
  The confinement owner is correcting the exact two reviewed device bindings;
  no ambient device tree or proc filesystem is requested.

No original Cargo acceptance is credited by these failures or by environment
preparation. Changed producers use fresh private proof roots; stopped operations
are never overwritten or relaunched.

The ignored public-seam proofs now cover all original command inventories,
candidate and integrated identities, authenticated unchanged replay, runtime
denial of an undeclared input, and compile-time `include_str!` refusal. The
compact proof overlays the exact five original controller/test/observer/doc
files from `223f8a1b0c7382817b1290d13c7a41f4e722d5e1` into a private snapshot and
uses the bounded external-only dependency preparation. A read-only ignored
authentication command supports retained receipts after cleanup.

Follow-up checkpoint validation: `producer-build-05.stderr` and
`producer-build-06.stderr` show successful normal producer and unit/public-prefix
compilation. The preparation control-inventory test passed again. The new
read-only authentication test passed on both `real-04` and `diagnostic-01`,
including the latest full durable-intent/reference checks. Trusted authoring
builds used `--config 'profile.dev.package.sha2.opt-level=3'` to reduce input
hashing cost; original producer argv and command deadlines are unchanged.

The sealed original compact claim confirms ownership is exactly those five
overlaid files; it does not own `main.rs`. The installed startup fallback is
therefore preserved without changing Submitted source. Shared-host ownership
also includes `tools/port/game-host-projectile.toml` and
`tools/port/game-host-room-scroll.toml`; pause ownership includes
`tools/port/pause-equipment-render.toml`. The production builder must retain the
complete OriginalClaim ownership even for files Cargo does not read.

Cargo source closure: tracked `crates/**`, root `Cargo.toml`/`Cargo.lock`, and
`deps/rustysnes/Cargo.toml` plus `deps/rustysnes/crates/**` for path resolution.
Compact's `cfg(test)` includes additionally require the six pinned C files
`deps/snesrev-sm/src/{sm_a0.c,sm_a6.c,sm_86.c,sm_80.c,sm_90.c,snes/snes.c}` and
`tools/decomp/headless.c`. Copy actual files into the verifier snapshot, never
mount the authoring dependency symlink.

An immutable stripped copy of the current installed producer is retained as
`/tmp/sm-ticket10-cargo-proof/bin/sm-pipeline-e6acc1e` (60,236,912 bytes). Stripping
changes only the reviewed runtime artifact identity and reduces repeated input
hashing; it does not prebuild candidate code or alter the original commands.

The next real run, `real-05`, passed and identically replayed the installed
compiler prerequisite, then genuinely launched the original host Cargo argv.
Cargo's compiler launch stopped with `EPERM` before executing rustc. The isolated
Rust 1.97 comparison in `spawn-diagnostic-01.stdout` proves an ordinary spawn
works and adding only a no-op `pre_exec` hook fails. The corresponding host
kernel trace `spawn-diagnostic-host-02.strace` identifies precisely
`socketpair(AF_UNIX, SOCK_SEQPACKET|SOCK_CLOEXEC, 0)` as the spawn handshake.
The confinement owner has a passing focused regression for that exact call;
other socket domains/types/protocols remain denied. The failed original Cargo
receipt and its unchanged `run` replay both authenticate.

Actual installed Cargo 1.97 uses `debug/deps`, `.fingerprint` and
`build/<package>-<hash>` paths; the authoring nightly uses
`debug/build/<package>/<hash>`. The cache adapter now supports both and excludes
all source package names and explicit library/binary aliases after Cargo name
normalization. The generator admits only vendor packages/aliases. Its public
preparation test reached red on the original layout rejection, then green;
`cache-layout-red.stdout` and `cache-layout-green-02.stdout` retain the boundary
proof, including refusal of project artifacts in both layouts.

Offline `cargo vendor --locked --offline --versioned-dirs` succeeded at
`/tmp/sm-ticket10-cargo-proof/vendor`: 189,236,115 logical bytes. Its exact file
hash map is persisted in each `prepared.json`. Compiler/proof target:
`/tmp/sm-ticket10-cargo-target`. No production original gate has been run.

Cleanup: pending completed proof runs and both utility passes; root owns final
worktree retirement after preserving referenced source snapshots. The local
untracked ignored `deps` symlink points to the original read-only dependencies;
it is for authoring builds and is never mounted in a Cargo producer namespace.

The completed `real-01` through `real-04` and `diagnostic-01` failures have now
had both utility passes; all returned zero reclaimed bytes. Exact commands use
the retained producer above with `cleanup --root <proof-root> --apply`, then
`cleanup --root <proof-root> --deduplicate --apply`. Receipts are
`<name>-cleanup.stdout.toml` and `<name>-deduplicate.stdout.toml` in the proof
directory. Read-only authentication passed on all five retained roots after
cleanup; `<name>-authenticate-after-cleanup.stdout` records each check. At this
boundary the disposable authoring target used 3,144,560,640 allocated bytes and
remaining filesystem space was 173,531,971,584 bytes. The target is still needed
for remaining real proofs; final target/worktree cleanup remains pending.

The exact installed Cargo and rustc are paired from the same 1.97.1 tree:
`/home/jim/.rustup/toolchains/1.97.1-x86_64-unknown-linux-gnu/bin/{cargo,rustc}`.
Cargo SHA-256 is
`828980723df339d62434390e9fb8ef8831036583343ae2316b7ab5646b5c1953`;
rustc SHA-256 is
`d3a664c970a9fd8361b64194861bebc1ae37b9054e5ee3400dc1c9e691797eea`.
The Cargo cache must use this pair; native recipes may separately seal their
nightly compiler. The root owner added a distinct `cargo_rustc` tool reference.

`real-06` uses immutable producer
`/tmp/sm-ticket10-cargo-proof/bin/sm-pipeline-309d8e1`
(SHA-256 `78db8768c0852677c89298281b9692ceb6fc098c3e7d9a7442c4d00351abf2b3`)
and optimized v2 supervisor
`/tmp/sm-ticket10-native-proofs/confinement-v2/tools/sm-translation-sandbox`
(SHA-256 `d334899b7f5cc7f64120636a803224d2f83e54c8d049e71e57a9d198a5fdf892`).
Its compiler prerequisite passed and identically replayed. The actual original
host Cargo command timed out after its remaining 47.729-second sandbox budget;
the enclosing original command budget stayed 60 seconds. The failed receipt and
unchanged replay both authenticated. It remains a failed original gate.

`pause-01`, using that same pair of immutable artifacts, passes all ten original
pause controls on both candidate and integrated source identities, including
authenticated unchanged replay. Retained compiler output reports 27.92 seconds
for the candidate and 26.59 seconds for the integrated Cargo command. The complete
public-seam proof finished successfully in 105.84 seconds, including prerequisite,
both distinct commands, replay and authentication. This credits only original
Rust controls, not Actual-C or original-ROM evidence.

The v2 supervisor's ptrace observer was then measured spending 3m56s CPU during a
5m59s compact prerequisite. It was scanning the full writable inode tree and every
owned file descriptor before and after read-only linker opens. The native owner
is implementing a focused performance correction while retaining mutation and
quota observation. `compact-01` reached its original 600-second prerequisite
timeout; its failure authenticated and identically replayed before the public
proof assertion. No original compact Cargo command was launched or credited.
The proof completed in 609.67 seconds and is never restarted or rewritten.

The adapter follow-up seals `StrictRuntime.jobs` and the corresponding producer
request counts, validates `1 <= jobs <= workers`, and uses the integration
owner's all-or-nothing shared lease reservation. Cargo receives that exact job
count. Runtime-only prerequisites reserve one worker. Default single-job fields
are omitted during serialization, so earlier sealed runtime and producer request
hashes remain authenticatable. The capacity-refusal public-boundary test reached
red in `jobs-capacity-red.stdout`: invalid job counts previously proceeded to
source/runtime preparation. The test now passes in `jobs-capacity-green.stdout`.
The original argv, control identities and 60-second deadlines are unchanged.

The completed `real-05`, `spawn-diagnostic-01`, `real-06`, `pause-01` and `compact-01` now have
both cleanup utility passes, each reclaiming zero bytes. Post-cleanup read-only
authentication has passed for the first four. The new reader successfully
authenticates the original single-job host timeout and both successful pause
commands after the jobs field was introduced. Compact post-cleanup
authentication is finishing. Final target cleanup follows the remaining proofs;
the integration owner handles worktree retirement.
