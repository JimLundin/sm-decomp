# Preparation integration status

Status: frozen tooling verified; actual drill integration and final validation active
Coordinator: `01a13460-4aa9-7fe0-865e-a8f54ffca310`
Worktree: `/tmp/sm-decomp-preparation-integration-20261013`
Branch: `decomp-preparation-20261013`

Recovery bridge/docs are committed at `c46b730`. Engine WIP
`a10736a90c0011f99c145baa492cd38afb938429` is incorporated at `ffac1cd`;
session WIP `c98e1e4d099fb29eff0ea029c72f75dba212bf1f` at `1495593`.
This section records the earlier integration history. The current freeze and
remaining work are recorded below. Gameplay and original queues remain held.

The engine worker can read/copy the three session WIP files from this integration
worktree for compiling focused engine tests. These are temporary foreign
dependencies, excluded from its scoped commits. Ticket 03's current replacement
is `/root/session_tools_finish`, actual thread
`01a13474-5fcc-7861-84f8-5c353938bdd2`.

Use the normalized sealed `legacy-owner-bridge-v2.toml` and
`approved-maintenance-v2.toml` in the main checkout for authority preparation.
The former original bridge/approval remain immutable predecessors. The engine
must expose `assignment::mutation_lock` to ticket 03, which holds that lock for
checkpoint/Git/publication to fence concurrent generation transfers. Ticket 03
records the child start first, then revalidates/publishes under the same lock;
interrupted publication retains its running claim and typed caller request.

Initial combined `cargo check` compiled the engine and native adapter, then
found checkpoint's local `credential` binding shadowing its revalidation
function in the early session WIP. Ticket 03's replacement is correcting it.
The isolated checkout uses the existing locked dependency through a temporary
Cargo config override for `patch."https://github.com/doublegate/RustySNES"`;
no Cargo source change is needed. Coordinator build target:
`/tmp/sm-decomp-preparation-integration-build`.

Worker source/test commits and private verification artifacts are retained in
their own handoffs. The coordinator incorporates only exact submitted commits
and performs final merged-source checks plus measured cleanup before completion.

## Recovery continuation at 09:14 UTC

The engine corrections through `a020d572d78ac20bbe6ff38f3ea77ff2bbe51d6a`
and session lifecycle/adoption controls through
`8f17e0b8042a88baf7d1641eb898747c3f3e8606` are incorporated at integration
HEAD `282970b`. Combined queue/session/authority tests passed; original failing
maintenance test is retained in the earlier logs and corrected in the scoped
engine commit. Latest logs are `/tmp/sm-decomp-preparation-integration-tests-final.stdout`
and `.stderr`, pending retention and cleanup. The final session preflight and
owner cleanup remain in progress. Freeze final source before actual drill.

Catalog ticket 05 is resolved after applying its map/milestone proposal; its
children 11–20 remain future research or held continuation work. No gameplay
workers have launched. Ticket 04's Rust harness is being prepared by
`/root/drill_preparation`, with actual isolated CLI sessions required. The
production authority and original Ceres ledger remain unchanged.

Engine owner: latest `a020d57` is incorporated and passing combined tests.
Please finish final focused validation/cleanup and handoff without additional
unrelated source changes. If a concrete correctness defect remains, report it
and preserve a scoped fix before declaring source frozen.

## Review corrections before drill

Ticket 02 is submitted through `6d23fdab55213f6101c337d12f163550817364d8`;
the coordinator incorporated this and session `9892cb2` at `8f66410`.
Read-only review found activation only authenticates booleans and arbitrary
sealed text, and a submitted-result retry requires a removed implementation
worktree. Root now owns the narrowly scoped authority activation validator/test
correction; engine owner has completed source and cleanup. Session worker owns
the retired-worktree retry correction. Drill waits for these concrete fixes.
No production authority file has been created or legacy ledger changed.

## Implementer retirement

Session source through `9a5516c` is incorporated at `11f13b4`; combined
23 queue, 15 session and 9 authority controls pass. Engine and session
implementer worktrees were clean and are now removed from Git registration.
Their authored source remains in exact commits. Session's small evidence was
copied byte-for-byte into `preparation/03-session-evidence/` and retained at its
historical `/tmp/sm-decomp-preparation-session-tools-20261013/out/session-preparation-verification/`
path without Git/build metadata. The original `logs.sha256` authenticates from
that historical root; `relocation.sha256` authenticates the main retained copy.
Evidence descriptor SHA remains
`653a75179074422b39f283687862a61f99ddec1470082ccf3e4e82659cfb3e2a`.
Both manifests passed after removal. Engine main receipts also reauthenticated.
Measured source-only worktree trees before retirement were 411,247,105 and
411,288,619 bytes (session's 37,889-byte evidence retained). Free space observed
after retirement: 221,698,600,960 bytes. Original Ceres controller submission
worktree and reporting processes remain held.

Catalog reviewer owns narrow activation evidence and submitted-only legacy
integrator checks in `/tmp/sm-decomp-preparation-activation-20261013`, branch
`decomp-preparation-activation-20261013`, from `ba9728b`. Session worker prepares
production cutover request artifacts only; no authority/ledger mutation.

## Validator recovery and integration handoff

Catalog/reviewer authored validator WIP was interrupted twice by the same
encrypted-output stream error. The session-tools worker now owns that worktree's
authority/drill-evidence/tests and preserved the WIP immediately at `2db3318`,
incorporated at `9610535`. No work was lost or duplicated. Final validators
bind actual isolated Git, launch JSONL/process identity, frozen source/binary,
typed per-control command/claim/role receipts, live-child/recovery and applied
cleanup/integration evidence; booleans/text alone are rejected. Final focused
gate remains in progress; no actual drill sessions launched yet.

Exact integration baseline propagation `0b46487` is incorporated at `adf0155`.
Typed integration Start/retry/Resume tests `05a165a` and `a338409` are incorporated
at `45318f4`. Combined 23 queue/15 session/9 authority controls pass with this
source. The test-only implementer worktree was removed after authentication;
small evidence remains at its historical root and
`preparation/integration-handoff-evidence/`. Both original/relocation manifests
passed. Measured pre-retirement worktree tree: 411,555,234 bytes; worker removed
1,218,133,244 private target bytes; final free space observation was
220,182,654,976 bytes. Source remains in scoped commits.

Cutover request package records fresh stopped original Ceres integrator and all
three exact held claims, with separate reporting/main Git ownership. Original
ledger seal and normalized bridge remain unchanged. Production validation will
use the same evidence checks through a read-only validation route while keeping
canonical claims pending until the separately requested gameplay launch.

## Frozen actual drill and latest recovery

The actual execution subject is frozen at
`c8799f326ffe198b652301f08866f1b7a7bd9ebc` in the worktree above, with source
fingerprint `4d87e457e895a03f82f5a7ccdd891f7769b83d25d48caf2fd3a55432e5929338`.
Reporting source `263ad5a` is included. The separately retained engine executable
is `../drill/bin/sm-pipeline`, SHA-256
`89e3c46596ea556011cfcbb679cace5b96277bbf56abb8a7a2a61db7048268d9`.
The frozen Rust harness is `../drill/main.rs`, SHA-256
`fabe7ff02289fee04e23707b80bb592daef6523802c12d226a11650a47237e15`.
Final combined locked/offline verification passed 23 assignment, 15 session
and 10 authority controls; formatting and build passed. Logs and cleanup
receipts are sealed in `integration-evidence/artifacts.sha256`.

Actual independent sessions A `01a134aa-705a-7c30-b33a-9471d5a7b27a`
and B `01a134aa-705a-7931-a8c6-a30b9e8eeef9` execute against the independent
tiny Rust fixture repository `/tmp/sm-decomp-concurrency-drill-20261013`.
Ticket 04 owns their exact launch/process/command and acceptance receipts.
Observed controls include disjoint workers, capacity 3, stopped parent with
live child retained, checked adoption/resume, replacement attempt preserving
scoped WIP, stale attempt/authority rejection, submitted exact retries, and
four actual shared lease holders with one waiting fifth client. Serial
integration, final manifest validation and cleanup remain active.

An actual duplicate-ticket request was correctly deferred with `ticket is
claimed`. The frozen validator expected the later engine diagnostic `ticket
ownership`. A separate validator-only correction will accept that earlier
diagnostic only with sealed evidence tying the candidate to the original exact
held ticket/claim and disjoint writable paths. Its source/build version is
separate from the unchanged actual execution subject; no drill receipt is
relabelled. The corrected validator consumes this frozen checkout before any
final source/docs integration.

Reporting finished its scoped source/evidence save and released the main
Git/index to this coordinator after both `main` and
`affected-family-assignment` reached
`9ed933daa151f52e3cf922e6559c7681330e0f85`. Preserve those commits intact.
The production readiness check will prepare a policy only inside a private
mount namespace and call `validate-activation`; host canonical authority and
ledger remain absent and the original Ceres ledger seal remains unchanged.
