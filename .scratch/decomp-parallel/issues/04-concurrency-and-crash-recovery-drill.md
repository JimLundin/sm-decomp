# Prove concurrent sessions and crash recovery

Status: resolved
Blocked by: 02, 03
Triage: ready-for-agent
Type: task
Owner: recovered preparation coordinator `01a13559-e517-72d1-88b5-e554fed61da4`
Source: `57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`

Existing queue tests and the runbook do not prove that independent live
sessions can cooperate and recover. Exercise the implemented contracts from
02 and 03 with two actual sessions before activating production authority.

## Work

- Prepare a disposable isolated checkout and effort with no writable overlap
  with production claims, private inputs or retained evidence. Retain the drill
  source and receipts outside disposable build trees. Production
  `.scratch/decomp-parallel/ledger.toml` remains unactivated during this drill.
- Launch two actual independent sessions with distinct real identities and
  admitted dispatcher roles. Both read the same queue, request work concurrently,
  and dispatch subagents with disjoint worktrees, handoffs and run directories.
  Prove capacity and the shared `SM_PIPELINE_WORKERS=4` budget hold across them.
- Crash or interrupt a parent after its worker starts; the other session must
  observe and retain that live child's claim. Then stop a worker, preserve its
  scoped WIP commit/evidence, check its exact identity and claim, and launch a
  replacement only after the supported checked-stop/fenced transition.
- Interrupt after durable result publication and retry the exact update.
  Reject a stale attempt and old integrator authority. Transfer an observed
  stopped integrator while retaining submission ownership and verify one
  integration at a time. Record the resumed exact command and worktree.
- Authenticate retained receipts after scoped cleanup/removal. Use the passing
  drill and ticket 01's ownership inventory to seal production cutover with
  admitted roles, current source, capacity and legacy claim bridge. Validate
  that unresolved legacy paths remain held before any canonical claim launches.

## Acceptance

Retained evidence identifies both actual sessions and their workers and proves
exclusive disjoint work, global capacity, surviving-child ownership, stopped
worker recovery, exact retries, stale rejection and fenced integration transfer.
A replacement resumes from source/handoff files without reconstructing a chat.
Only then may a validated cutover activate production canonical authority;
new gameplay scopes still require ticket 05. An unrun drill leaves the workflow
unproven and the production queue unactivated.

## Completion evidence

Append session/thread ids, full source commits, role/claim tuples, commands,
process identities, handoff hashes, run/queue/rejection receipts, evidence
authentication and cutover/legacy bridge hashes. Complete scoped
[run cleanup](../../../docs/agents/run-cleanup.md), remove disposable worktrees
after preserving dependencies, and record measured bytes/free space before
resolution. Keep production source, claims and retained evidence intact.

### Recovered corrected-source validation subject

Current root is `01a13559-e517-72d1-88b5-e554fed61da4`. Execution and validator
use clean frozen `/tmp/sm-decomp-preparation-retry-source-20261013`, full commit
`57f93c5ce4e6adac2dfe181b852d4aad6e0776bd`, fingerprint
`bd103af4b5d563904a3cbdbe62e971c46c6b4a4e907b60bbbb8575a2373355ec`.
Engine SHA-256 is
`04f4a10aec71b9654cdc1c7f1aca6936f9a9bb7ce810f8232c955f3b11e7d573`;
[build receipt](../drill-retry/engine-build.toml) SHA-256 is
`cdf1295fdc8ad86b574866636f68966205e5c46bf2fd19649fa9a4dd2f5bdbb9`,
checkpoint `9acff17`; main integration is `0d042e0`.

The prior [drill-final outcomes](../drill-final/outcomes.md) completed two serial
integrations but A's completed replay failed after B advanced HEAD. The current
[Rust correction](../../decomp-integration-retry/handoff.toml) retains red/green
diagnosis and passes 23 assignment, 17 session and 11 authority controls. It
replays sealed historical acceptance without rerunning gates or accepting later
source. Original failure receipts remain unchanged. New actual drill
`concurrency-drill-retry` uses
`/tmp/sm-decomp-concurrency-drill-retry-20261013`; follow its
[recipe](../drill-retry/recipe.md) and exact surviving session/claim state.

Actual sessions A `01a13515-9a51-7c73-9913-1deb499ba15c` and B
`01a13515-9a51-7af3-bc9f-a144afb06090` completed the drill. Exact worker/claim/
generation/process identities are retained in `.scratch/decomp-parallel/drill-retry/execution.toml`, SHA-256
`805dde33a142cc10dc9d9c3c2d6791d1589639aebaebb9feaba7a71dbbccb67a`. [Outer manifest](../drill-retry/concurrency-drill.toml)
SHA-256 `63912c67e38680fd163d09126ba285f8ce26da8a2445c801f489f8971b74751f` records all eleven controls. A integration source:
`0812fbcd0857b6845f2ef966f6b2f601c0b4a949`; final B merged source: `2b1400ef758907c9aee50f997bb9bca05216ec52`.
Both completed exact replays at final HEAD reused byte-identical publications,
authentication and gates, with zero gate reruns; proof `.scratch/decomp-parallel/drill-retry/post-b-retry.toml`,
SHA-256 `f8ab8db23c0e795f574b4c59be8cadfc5b3349c5300a77070152cd249030a954`.

Root's maintained validation and
[readiness completion](../preparation/completion-20261013.md)
record passing private `prepared` validation, unchanged host authority/canonical
ledger absence and unchanged legacy bridge/Ceres ledger seals. Output manifest
`.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013/private-dev-fix-20261013/production-validation-1/artifacts.sha256`, SHA-256 `241018cd062ef561c5650c90bce7840bec5e0749e9ebc4d2b220203fa1e4269d`.
The original readiness package manifest remains unchanged; completion is a new
receipt document beside it.

Both cleanup passes succeeded for drill/validation/worker scope; all integrated
implementer worktrees were removed after retained source snapshots. Cleanup
receipts `.scratch/decomp-parallel/preparation/completion-20261013.md`; reclaimed `2,481,607,746` logical bytes across the retry worker and recorded helper targets,
available `221,230,100,480` bytes. Post-cleanup authentication
`.scratch/decomp-parallel/drill-retry/metadata-closure-20261013/independent-authentication.json` passed. Retain exact frozen source/path,
engine/readers, small fixture Git history, sealed requests/gates/source snapshots
and shared evidence/private-input dependencies. Root checkpoint:
`.scratch/decomp-parallel/preparation/final-checkpoint-20261013.md`.

The explicit post-publication interruption also passed: the same B controller
was observed live, interrupted after durable publication, observed stopped,
and resumed under generation 2. Both exact updates succeeded with all 18 receipt
files unchanged. [Supplemental proof](../drill-retry/post-publication-interruption.toml)
SHA-256 `6735bb4c95e4ec1581ec5b13fb61b96ed2e5621b261f181decff49c68eb6b451`.
The final launch metadata uses sealed absolute prompt paths and authenticates
318 artifacts after cleanup; original launch metadata remains retained.

The historical section below records unchanged `c8799f3` execution and its
separate validator epoch. Its second-integration failure is preserved as the
originating diagnosis for the earlier serial fix; `drill-final` replay failure
is retained separately. Fresh actual corrected-source receipts above resolve
this preparation gate.
Production authority/canonical ledger remain absent, legacy Ceres
claims/seals unchanged, and gameplay launch remains separately requested.

### Actual frozen execution retained 2026-10-13

Executed engine source `c8799f326ffe198b652301f08866f1b7a7bd9ebc`, fingerprint
`4d87e457e895a03f82f5a7ccdd891f7769b83d25d48caf2fd3a55432e5929338`, and
binary SHA-256 `89e3c46596ea556011cfcbb679cace5b96277bbf56abb8a7a2a61db7048268d9`
remained frozen. Independent disposable repository:
`/tmp/sm-decomp-concurrency-drill-20261013`, original fixture baseline
`e26c7160706233a3d517b91ff87ed0cf4b6e669f`. Rust harness source, frozen build
metadata, exact stdout/stderr requests/process receipts and source snapshots are
retained in [drill](../drill/recipe.md) and the isolated `.scratch/drill-control`.

Actual A `01a134aa-705a-7c30-b33a-9471d5a7b27a` and B
`01a134aa-705a-7931-a8c6-a30b9e8eeef9` independently reserved disjoint lane A/B
attempt 1 claims and dispatched actual workers with scoped collaboration
subagents. Worker identities were A `01a134ac-e876-7193-a450-3a6900d6242b`, B
`01a134ad-1ea9-7850-9a5c-8179e7a091da`, replacement A
`01a134b4-77dd-7d30-93e3-82ffe1b47c27`. Live child A survived interrupted A;
B retained its exact held claim, transferred dispatcher generation 1 to 2,
adopted/resumed the original WIP, then recorded a checked stop before replacement
attempt 2. Original scoped WIP `ddcdae9770cd17d9aa8d133fbac26b999549bab0`
and submitted replacement `340e346ccc8d3bd7712fdf66326b3edcdbc55bda` are retained.
Old attempt and old dispatcher/integrator generations rejected mutations.

Capacity 3 held two running workers plus one unlaunched reservation; a fourth
request assigned no work. Writable overlap deferred. A pure duplicate-ticket
next request, with zero available slots and disjoint sealed candidate paths,
assigned no work while retaining exact B attempt 1 and reporting
`control-overlap = "ticket is claimed"`. Five actual native lease clients shared
`SM_PIPELINE_WORKERS=4`: four held leases while the fifth waited; all five
released successfully, including observed contention. Both real Rust submission
gates ran once; byte-identical retries succeeded after durable publication and
controller interruption.

Strong integration recovery used actual independent integration child
`01a134bf-913f-7270-ba31-e1b7c3280eea`, PID 441189. A started
`integrate-lane-a` attempt 2 under integrator generation 3, then only its
controller stopped. B observed that exact live child, transferred generation 3
to 4, adopted and resumed the same attempt at original fixture source/worktree.
Old A generation 3 rejected resume. After the independent child finished, B
merged exact A source, ran the maintained Rust integration gate (one test
passed), authenticated the result and completed exact integration/submission
claims under generation 4. Integrated source is
`340e346ccc8d3bd7712fdf66326b3edcdbc55bda`; authenticated evidence is in the
isolated `receipts/integrate-lane-a-2-infrastructure-result.toml`.

[Typed execution](../drill/execution.toml) and
[outer drill manifest](../drill/concurrency-drill.toml) retain all 11 required
controls with actual command/process/source artifacts. The original literal
ticket validator expected `ticket ownership`; actual maintained same-effort
queue returned `ticket is claimed`. A separately reviewed validator-only
correction consumes exact original reservation, competing sealed scope/request,
held nonstale claim, disjoint paths and pure refusal. Executed engine and
historical receipts were unchanged. Independent reviewer confirmed all 11
predicates and exact integrated authentication/gate/role/retirement evidence.

[Observed limitations](../drill/integration-limitations.md) retain two honest
diagnostics. Adoption of a stopped controller-as-worker correctly required a
surviving child; checked stop/new attempt plus independent child recovered.
Subsequent lane B integration Start rejects A's already merged path because
baseline-to-HEAD handoff scope validation includes earlier integration changes.
B's submission `ee2fca390aaa53d51fcc70694afa54eabdaa1567` remains held, with its
small diagnostic worktree; no B integration was admitted or falsely completed.
General sequential integration needs a separate behavioral fix and fresh actual
drill. Production remains unactivated.

[Cleanup](../drill/cleanup.md): all writers stopped; both scoped utility passes
applied and returned zero bytes. Integrated A attempt 2 and original stopped A
attempt 1 worktrees removed after snapshots. Identified private build/setup
intermediates totaling 256,874,607 logical bytes removed, then temporary helper
copies totaling 46,363,952 bytes removed (303,238,559 total logical bytes).
Filesystem available space measured 222,921,195,520 bytes. Small isolated Git,
21,175-byte failed B submitted worktree, frozen binary/source,
scoped snapshots, historical requests/logs/gates remain authentication
dependencies. Recursive retained-seal check authenticated 304 artifacts after
worktree/build removal. Final readiness validation and ticket disposition remain
with the parent integration owner.
