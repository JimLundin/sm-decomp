# Ownership preparation handoff

Status: implementation submitted; scoped cleanup complete; integration/drill pending
Owner: `/root/ownership_engine`
Session/thread: `01a13453-b599-79c1-84d5-713e6d1e3f57`
Branch: `decomp-preparation-ownership-20261013`
Worktree: `/tmp/sm-decomp-preparation-ownership-20261013`
Baseline: `deceefdb568990fe273cc3f6cdf849726d8380bb`

The task is preparation ticket 02, delegated by `/root`. No production claims
or gameplay runs have occurred in this task.
The parent and ticket 01 agent encountered encrypted-output decoding failures
after delegation. The surviving ticket 02 and 03 agents retained their work;
ticket 01 was resumed to complete its durable reconciliation bridge.

Replacement coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`. Its
`preparation/recovery-20261013.md` explicitly authorizes starting the existing
worker's exact isolated maintenance paths after checked liveness/adoption.
`preparation/approved-maintenance.toml` binds this actual surviving worker and
the replacement ticket 03 worker. The recommendation SHA-256 is
`1338edb2e6d648d293aba30bc5b0b4d3f0f30c6170f0ca1aa8542b88a6a8bb39`;
direct liveness inspection SHA-256 is
`fe863b9a8b0929a46692667fb7ae4087a4225cede0f6812d7d284581122bde40`.
This is narrow preparation adoption; old gameplay claims stay held.

## File ownership

This worker owns `crates/pipeline/src/assignment.rs`, the new sibling
`authority.rs`, `crates/pipeline/tests/assignment_queue.rs` and the new
`session_authority.rs`. Ticket 03 owns session/CLI/native adapter files and
consumes the additive authority API. Main Git/index and ticket status belong to
the integration coordinator. This handoff has one writer.

## Agreed implementation boundary

- Retain `Job`, `Claim`, `Update` and `WorkResult` shapes and legacy tuple/artifact
  bytes. Add optional `Request.authority` with real `session_id` and `generation`.
- Expose `assignment::authority` as a sibling-file child module and reuse the
  queue's main-checkout global mutation lock for role publication and claims.
- Store prepared/active role policy in `.scratch/decomp-parallel/authority.toml`.
  A prepared policy admits only declared disposable drill efforts; production
  canonical mutation remains pending. One explicit integrator and persistent
  dispatcher tokens replace first-submitter authority for managed efforts.
- Scan every durable assignment ledger and sealed legacy holdings for writable
  path prefix conflicts, duplicate ticket identity and active integration.
  Reserved/running/submitted holdings persist. Shared assignment capacity is 3.
- Revalidate the current real session/generation before all queue mutations and
  identical retries. Checked transfer advances generation, keeps original tokens
  and claim tuples, and preserves surviving children. Time does not expire claims.
- Admit narrow preparation maintenance exceptions only when the sealed bridge
  identifies original hold ids, exact paths, admitted effort, real session and
  checked stop/adoption observation. Exceptions retain original gameplay claims.

Ticket 03 accepted the typed authority request/receipt route and read-only
`validate(checkout, effort, coordinator, credential, integration)` before native
gates. The engine revalidates under the mutation lock at durable recording.

## Next action

Implement the engine boundary under the checked adoption. Read the finalized
`.scratch/decomp-parallel/reconciliation/legacy-owner-bridge.toml` and its hash
before staged policy validation. Make a scoped WIP commit, run
existing assignment tests plus focused session-authority tests in an isolated
target/run directory, publish receipts here and perform scoped cleanup. Tests
must mutate only disposable repositories. Shared execution configuration remains
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`.

## Verification and cleanup

Read-only design completed; no generated run/build artifacts or verification
processes exist for this task. No cleanup is required at this boundary. Exact
test, commit, reclamation and remaining-space evidence will be appended after
implementation.

## WIP checkpoint

`a10736a90c0011f99c145baa492cd38afb938429` commits the authority implementation
and additive engine enforcement in the assigned worktree. This is a recoverable
WIP, not integration-ready source. An initial no-run build found the isolated
checkout lacks the local RustySNES patch package. The same locked/offline build
is continuing with an explicit Cargo path patch to the retained main dependency
checkout; no project source/Cargo changes were needed. Target is
`/tmp/sm-decomp-preparation-ownership-target-20261013`; build stdout/stderr is
`/tmp/sm-decomp-preparation-ownership-check-20261013.log`.

Next: finish compile, extend focused disposable-repository tests for global
ownership and managed real-session fencing, checkpoint again, run the existing
assignment tests and session-authority controls, then scoped cleanup. Runtime
policy/claims remain unchanged in production. The source API is recorded in
`02-authority-api.md` for the replacement session tools worker.

For compilation only, this worktree has an unstaged read-only dependency copy of
ticket 03's committed `coordination.rs` from
`c98e1e4d099fb29eff0ea029c72f75dba212bf1f`. This worker does not edit or commit
that file; the integration owner will use ticket 03's real commits. The source
copy fixes the expected new optional Request constructor during focused tests.
The existing assignment test binary ran successfully: 18 passed. Full Cargo
invocation and new controls follow after that dependency is present.

Second scoped WIP commit: `f67265de2aea088988ed4baf7944fa99b9564597`.
Locked/offline focused Cargo tests passed: assignment queue 21/21; managed
session authority 5/5 (one ignored child-process harness is invoked by the
controls). Exact stdout/stderr is
`/tmp/sm-decomp-preparation-ownership-focused-20261013.log`. The managed controls
exercise two real child processes with separate thread ids, shared capacity,
explicit integrator, generation rejection, checked role retry, surviving child
claim retention and sealed bridge/source/hash rejection. These are automated
controls; ticket 04's independent live-session drill remains separate.

Use successor bridge `reconciliation/legacy-owner-bridge-v2.toml`, SHA-256
`cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`, and
maintenance approval v2 SHA-256
`bcc953cb2d2a1459b62ef477cfaf49703652f3fdbf704a3828066dc2ed4b8264`.
The original worker identity is retained; ticket 03's successor identity is
`01a13474-5fcc-7861-84f8-5c353938bdd2`.

Next: review concurrency gaps around retries/stopped proof/current source,
add final rejection/activation/maintenance controls, freeze final source and
send commit/evidence for integration. The unstaged coordination dependency
SHA-256 is `d846f7f18ea8d25ac99b4ea0148d967af721bbf30481b8b9ffa3294c9b109686`.

## Final submission and verification

Final source HEAD: `6d23fdab55213f6101c337d12f163550817364d8`. The complete
scoped commit sequence after baseline is:

- `a10736a90c0011f99c145baa492cd38afb938429`
- `f67265de2aea088988ed4baf7944fa99b9564597`
- `24808b54afaa96501f456ce7fc8db85edc841170`
- `80e3d7870343a3550eab44195b91a18db3e666b6`
- `a020d572d78ac20bbe6ff38f3ea77ff2bbe51d6a`
- `6d23fdab55213f6101c337d12f163550817364d8`

Final focused command used the explicit retained dependency override:

```sh
env SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 cargo test -p pipeline --locked --offline --config 'patch."https://github.com/doublegate/RustySNES".rustysnes-core.path="/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core"' --target-dir /tmp/sm-decomp-preparation-ownership-target-20261013 --test assignment_queue --test session_authority
```

**23 assignment tests and 9 session-authority controls passed**. One ignored
child-process harness is invoked explicitly inside the controls and is not a
missing gate. Final passing stdout/stderr is retained and commit eligible at
`02-ownership-evidence/final-focused.txt`, SHA-256
`d4ba3f5a44a8dfb7652fe72b2e2e41649410abf8eebd7cbec1a25ce7bd232304`.
An initial maintenance control accidentally selected another allowed queued
job; its exact failed receipt is retained in
`02-ownership-evidence/initial-maintenance-test-failure.txt`. The control was
isolated correctly and passed on the next and final runs. All retained receipts
are checked by `02-ownership-evidence/artifacts.sha256`.

Source now rejects cross-effort ancestor/descendant paths and duplicate tickets
under the global lock, including submitted holdings and symlink aliases.
Managed shared capacity is 3; generations fence copied/old real sessions before
all mutations and retries. Explicit integrator registration preserves legacy
token/claims; checked role transfer accounts for every held child while retaining
the child's original tuple. Same-attempt completion retries recheck sealed
evidence. Exact maintenance exceptions remain pinned to approved path/session
and original holding ids. Canonical mutations and unlisted efforts remain
blocked while policy is prepared; activation requires a current sealed drill.

The final visibility-only commit exposes the same mutation lock to ticket 03
for scoped Git/handoff publication. It changes no tested runtime behavior.
The foreign compilation-only coordination copy has been restored; this worker's
worktree is clean. The primary can integrate the six exact commits above and
remove this worktree after integration. No sealed evidence depends on it.

## Final cleanup

All test processes finished. Both maintained cleanup passes scoped to
`preparation/02-ownership-evidence` succeeded and reclaimed **0 bytes**, preserving
receipts. `cargo clean` removed 1,825 private target files, reported **1.3 GiB**;
the measured target tree before cleanup was **1,260,555,955 bytes** and the entire
target directory is now absent. The retained `cargo-clean.txt` receipt has
SHA-256 `96c023252ddec18dae9590e12544eb1963086e0a823d12da2aa0fddfda7a7413`.
Disposable original `/tmp/` log copies were unlinked after their retained copies
authenticated. Remaining free space observed after cleanup was
**220,019,093,504 bytes**. Retained dependencies are the main checkpoint bridge,
original ledger/source/evidence and small test receipts; production/private
gameplay data and readers were untouched.

Next command belongs to the preparation integrator: incorporate the exact
worker commit sequence, integrate ticket 03, run the independent disposable
session drill and frozen combined-source verification, then prepare the pinned
real role registrations without launching gameplay. Tickets/status/main Git are
coordinator owned. Production authority has not been mutated by this worker.
