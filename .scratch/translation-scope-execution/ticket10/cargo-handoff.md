# Ticket 10 Cargo adapter handoff

Status: source implemented; host and pause controls passed; compact prerequisite
unresolved; cleaned and ready for integration review.

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

Next: the integration owner completes Standards/Spec review and worktree
retirement, then reauthenticates the retained receipts. A new compact proof
requires a concrete validated supervisor correction and a fresh proof identity.
There is no successful compact dependency cache to publish. Continue using
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
authentication has passed for all five. The new reader successfully
authenticates the original single-job host timeout and both successful pause
commands after the jobs field was introduced. Final target cleanup follows the remaining proofs;
the integration owner handles worktree retirement.

The sealed multi-job producer is
`/tmp/sm-ticket10-cargo-proof/bin/sm-pipeline-e071c3c`, SHA-256
`4a4c6b1abdada505d14753821d56e65455fe70940a57a560f1e110d54f54664f`.
Normal producer compilation passed in `producer-build-10.stderr`.
The retained independent reader is
`/tmp/sm-ticket10-cargo-proof/bin/cargo-proof-reader-e071c3c`, SHA-256
`49e98003decadb3125ca376650eb45e90af0d46986536f8c3d34f19e96f3aac6`.
Both are immutable stripped images and survive disposable target retirement.

Optimized v3 supervisor is
`/tmp/sm-ticket10-native-proofs/confinement-v3/tools/sm-translation-sandbox`,
SHA-256 `21f1bdb8d4bb10cbad2c5c23a24e7e146b2ed9a17fc4695b41d7813c3f2b66c5`.
Its read-only-open regression and all original confinement controls passed.
The v3 host proof `real-07` overlapped an already-running native fixture proof,
waited for all four shared slots, then timed out with its original command
deadline unchanged. After the native owner confirmed every producer was closed,
`real-08` received a quiet worker window and still timed out, with 56.260 seconds
left for the sandbox after preparation. Both failures authenticated, identically
replayed and remain preserved. The quiet result shows queue overlap was not the
sole cause of the host timeout.

`compact-02` used that same v3/multi-job producer pair. Its prerequisite failed
in 40.86 seconds because observing candidate descriptor directory PID 70 returned
`Permission denied (os error 13)`. The native owner is reproducing the concurrent
exec/exit observation race; no original compact controls were launched or
credited. This failure also authenticated and replayed unchanged.

`real-07`, `real-08` and `compact-02` have both cleanup passes (zero bytes reclaimed)
and successful post-cleanup authentication with the retained reader. The
proof-only `SM_CARGO_JOBS` selector permits a fresh single-job versus multi-job
comparison while retaining the same sealed compiler, supervisor, original argv,
original port source and deadlines. Its compiled reader is
`/tmp/sm-ticket10-cargo-proof/bin/cargo-proof-reader-jobs-select-01`, SHA-256
`33a35b1dff9cd0ada176125af7e6c289eeacfdc0b85e0a79647344b93a758e9e`;
`reader-build-11.stderr` records successful compilation.

`real-09` is the successful complete host proof, using the v3 supervisor,
`sm-pipeline-e071c3c`, the paired Cargo/rustc 1.97.1 toolchain, and one Cargo
compiler job within shared capacity four. It passes the exact 31 original
controls on candidate source (Cargo reports 41.80 seconds) and integrated source
(48.10 seconds). The latter inserts a runtime denial assertion into
`Bridge::request` for an undeclared host file. A distinct third identity attempts
`include_str!` of that file and retains the required compiler refusal with no
test acceptance. Every success and refusal authenticates and identically replays.
The complete proof passed in 131.26 seconds. Both cleanup passes reclaimed zero
bytes, and the retained independent reader authenticated all receipts afterward
in 2.50 seconds. Logs and snapshots remain under
`/tmp/sm-ticket10-cargo-proof/real-09` and the corresponding sibling `.stdout`
and cleanup/authentication logs.

The single-job result and the quiet four-job timeout show that additional Cargo
compiler concurrency increases the current observer cost for this workload.
The integration owner will use one Cargo job with shared capacity four. No
original argv, test selection, compiler profile, project-artifact cache or
deadline was changed to obtain the passing host proof.

`compact-03` attempted the separately bounded v3/single-job dependency
prerequisite. It reached the unchanged 600-second timeout, with 605.22 seconds
total proof time. Its failure authenticated and identically replayed; no cache
was published and no original compact Cargo command was launched. Both cleanup
passes reclaimed zero bytes and post-cleanup authentication passed. The
native owner measured progression at 2m18s: 159 writable files, 1m34s supervisor
CPU and 2 seconds Cargo CPU. The supervisor's conservative `O_TMPFILE` mask also
classifies ordinary read-only `O_DIRECTORY` opens as mutating; a focused
regression/fix is in progress independently. A later sample at 7m57s producer
time showed 79.1% supervisor CPU and 0.5% Cargo CPU, while linking ring's build
script. A future helper uses a fresh proof root; this stopped identity remains
unchanged. After the validated next helper, the compact diagnostic owner takes
its original experiment window first, followed by the next Cargo prerequisite.

`compact-04` started at approximately 2026-10-14 12:34:29 UTC after the compact
and native owners confirmed their producer windows were closed. The retained
selector reader ran in unified command session `82885`, host PID `1251183`
(observed live at 12:34:49 UTC). It used one Cargo job, capacity four, the paired
1.97.1 toolchain and `sm-pipeline-e071c3c`. Its helper is
`/tmp/sm-ticket10-native-proofs/confinement-v4/tools/sm-translation-sandbox`,
SHA-256 `1ec74e0bcc1024ae460d74b47148114526bf6743e2cc4a87302f9ecf551ab6d9`.
The native owner's v4 fixes `O_DIRECTORY` versus `O_TMPFILE` classification;
seven runtime controls and eight original confinement controls pass. No compiler
profile, original argv, control identity or deadline changes accompany this run.
The proof root is `/tmp/sm-ticket10-cargo-proof/compact-04`, with sibling
`compact-04.stdout` and `compact-04.stderr` logs. Dependency preparation remains
bounded to 600 seconds and each original acceptance command to 60 seconds.
The exact session was awaited to exit 101 at approximately 12:44:42 UTC. It
reached the unchanged prerequisite timeout; the complete proof took 613.62
seconds. Failure authentication and identical `prepare_environment` replay
passed before the proof assertion. No cache or original compact acceptance
command was produced. Never restart or overwrite this proof identity.

## Completion evidence

The v4 failure remains at
`/tmp/sm-ticket10-cargo-proof/compact-04/environment/cargo-environment-receipt.toml`,
SHA-256 `ad45d57dbb16d9e79888d1a93569797716af58323be6acfe7b36a569e1e26ee4`.
The full proof log `compact-04.stdout` has SHA-256
`4c22e602aae4d08f1233fccc90c46125b397f45099a75cf3f2dfa3686200aa7d`.
At producer 440 seconds the helper consumed 78.5% CPU and Cargo 0.7%, while
compilation had reached `tinystr`; at 530 seconds it was building ICU data.
`compact-04-process-observation.txt` retains a process identity and CPU snapshot,
SHA-256 `3aec14cf6cde038b5ff9146b0fb4576665da0678a59c32653c63cf5e3990a3f2`.
These measurements show remaining observer cost, but do not isolate another
validated optimization. The integration owner requested no unchanged retry.

After all producer processes stopped, both required utility commands completed:

```sh
env SM_PIPELINE_WORKERS=4 SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline /tmp/sm-ticket10-cargo-proof/bin/sm-pipeline-e071c3c cleanup --root /tmp/sm-ticket10-cargo-proof/compact-04 --apply
env SM_PIPELINE_WORKERS=4 SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline /tmp/sm-ticket10-cargo-proof/bin/sm-pipeline-e071c3c cleanup --root /tmp/sm-ticket10-cargo-proof/compact-04 --deduplicate --apply
```

Their retained sibling logs are `compact-04-cleanup.stdout.toml` and
`compact-04-deduplicate.stdout.toml`; both report zero reclaimed bytes.
Post-cleanup read-only authentication passed in 1.09 seconds, recorded in
`compact-04-authenticate-after-cleanup.stdout`.

Temporary `main.rs` and `confinement.rs` edits were restored from backups whose
Git blob identities exactly matched this worktree's HEAD. The local ignored
`deps` symlink was removed. After confirming no writer remained, the disposable
authoring target was measured at 3,150,139,392 allocated bytes and removed with
`rm -r -- /tmp/sm-ticket10-cargo-target`. Remaining filesystem space measured
166,715,719,680 bytes after cleanup and authentication. No owned process remains.
The integration owner still owns final Git worktree retirement.

All sixteen retained roots (`real-01` through `real-09`, `pause-01`, `compact-01`
through `compact-04`, `diagnostic-01`, and `spawn-diagnostic-01`) authenticated
again after target removal, using the retained independent reader from outside
the worktree. Each `<name>-authenticate-after-target-removal.stdout` records its
passing result. Preserve these roots, their exact source snapshots, the vendor
closure, immutable readers/producers, each referenced supervisor version, and
sealed toolchain/runtime inputs. The latest compact authentication log has
SHA-256 `9bf3771e6660ae9f32411c9e367f6efd68ef499112f2bb6976ce1e8e6bb22366`.
No retained TOML/JSON evidence names the implementer worktree. Older runtime
records retain the historical authoring-target executable name, but their
sealed identity records are outside that target; all affected receipts passed
authentication with the target already absent. Exact source dependencies are
copied under the retained proof roots, outside the implementer worktree.

The exact next read-only command, also suitable after worktree retirement, is:

```sh
cd /tmp/sm-ticket10-cargo-proof
env SM_PIPELINE_WORKERS=4 SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_CARGO_PROOF_ROOT=/tmp/sm-ticket10-cargo-proof/compact-04 /tmp/sm-ticket10-cargo-proof/bin/cargo-proof-reader-e071c3c --exact translation_cargo::tests::retained_original_cargo_proofs_authenticate_after_cleanup --ignored
```

A later production retry needs a fresh reviewed source snapshot and validated
changed helper. The retained proof selector's real-run fixture names this
authoring worktree at compile time, so rebuild that fixture against the new
verification checkout for any later real-run proof. Its read-only
authentication command does not depend on the authoring worktree or target.
