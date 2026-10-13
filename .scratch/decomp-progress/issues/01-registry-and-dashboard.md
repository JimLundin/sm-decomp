# Publish shared translation inventory and interactive progress

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task

Owner: recovery coordinator and `/root/progress_registry`, `/root/progress_dashboard`.
Recovery publication owner on 2026-10-13: `/root/recovery_audit`, delegated by
the coordinator. Original source owners are preserved; generated inventory,
registry publication and local completion notes are delegated here.
Subsequent recovery publication owner: `/root/progress_finish`, resuming durable
artifacts after the interrupted audit; maintained main/batch ownership stays frozen.

## Required work

- Index all recovered C definitions, bank/address/body extent and dependencies.
- Rust refresh/update/status/serve commands; concurrent writes, atomic publication,
  source identity checks, idempotent updates and hashed evidence pointers.
- Interactive HTML with bank, stage, agent and search filters, remaining work,
  per-routine scope/evidence details and refresh from shared JSON.
- Register the recovered completed and in-progress gameplay lanes with accurate
  stage boundaries. Publish agent instructions and a durable session checkpoint.
- Verify registry integrity, concurrent updates, rendered browser behavior and
  scoped cleanup; preserve original agents and authenticatable evidence.

## Comments

2026-10-12: resumed from the crashed coordinator's logs. Temporary planning
script `/tmp/sm-progress-report/src/main.rs` produced
`.scratch/decomp-port/progress-inventory-observation.toml`; no registry or HTML
was completed. Existing active Ridley/Samus-frame lanes remain independently owned.

2026-10-13: recovered the maintained Rust registry and dashboard plus the four
previous trail/Samus translated/verified observations. The coordinator delegates
current inventory regeneration and stage publication. The older observations
will remain as historical records if their inventory identity differs after
regeneration; new evidence-linked observations will state the recovered stages
and frozen verification domains explicitly. Gameplay integration and live stages
wait for the coordinator's passing post-cleanup receipts.

## Completion evidence

2026-10-13 refresh recovery: the real declaration seam failed at 49.324 seconds
on a four-megabyte fixture payload. A private progress metadata projection reduced
that scan to 58.8 milliseconds; all fourteen focused progress tests pass. The real
fresh-source refresh finished in 57.78 seconds under ongoing batch/live load and
published the complete current inventory. Six recovered observations were then
published through the real CLI, preserving the four older historical observations.
Current counts are translating 220, translated 228 and verified 8 definitions.
Integration/live stages await coordinator post-cleanup receipts. Exact source,
timing, scope and pending cleanup details are in
`../recovery-refresh-checkpoint-20261013.md`.

After all twelve current affected Offline families authenticated following
cleanup (20,943 fixture pairs), eighteen current translated/verified/integrated
observations were published for the installed six scopes. Shared registry has
28 observations, four historical, and current translating220/translated247/
verified34/integrated34 native definitions. The complete broad220 Ridley scope
retains its separate translation stages; live exercise awaits independent replay
receipts. Release current-registry concurrency passes six writers in 2.629 seconds
with 421 exact-byte atomic reads, and release status takes 0.16 seconds. Both scoped
progress cleanup passes succeeded (zero supported disposable bytes); identified
helper binaries and isolated JSON/HTML copies were removed while preserving
Rust source and evidence. The checkpoint records exact receipts and byte counts.

The maintained Rust progress commands provide refresh, update, status and serve
with mandatory pinned source/inventory identity, routine membership checks,
hashed evidence, bounded operating-system file locks, idempotent observations
and atomic JSON/HTML publication. These commands were exercised through the
real CLI in isolated verification and are being used for shared publication.
The generated pinned source inventory now includes 6,053 definitions and 75,229
body lines; exact SHA-256 is
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.

Browser/CLI verification succeeds against this current inventory and recognizes
all 220 symbols in the complete Ridley/Ceres closure. Six separate concurrent
CLI writers completed in 5,645 milliseconds, preserving all six observations
and their identities after an identical retry. Fifty-one reads during publication
saw complete valid JSON. Browser assertions cover combined bank/agent/scope and
symbol filters, exact independent stages and remaining counts, expanded evidence
digests, themes, seven-second live refresh with preserved filters, stale evidence
exclusion, file picker loading, malformed JSON errors, standalone offline data,
and a 390-pixel phone layout without horizontal overflow. Desktop and phone
screenshots were visually inspected.

Retained proof: `../recovery-browser/report.md`, `verification.json`,
`desktop.png`, `phone.png`, helper Rust source/Cargo lockfile and inventory,
`source-hashes.sha256`, and checked `artifact-hashes.sha256`.
The earlier 24-process stress attempt hit the intentional ten-second lock bound
and retained nineteen complete observations. That bounded contention limit is
recorded; no maintained implementation fix is claimed. The existing six-writer
unit test remains unchanged.

Cleanup: verifier, CLI server and browser processes exited first. Both required
project cleanup passes succeeded on four isolated roots and the temporary Cargo
target, reporting zero bytes for these unsupported run formats. After retaining
the small evidence/source artifacts, identified unsealed roots/build output were
removed, reclaiming 471,583,052 measured apparent bytes. Available filesystem
bytes afterward were 246,840,741,888. Ten cleanup receipts and the before/after
measurements are under `../recovery-browser/cleanup/`. No implementer worktree,
shared evidence store or gameplay run was modified by this verification.

2026-10-13 final local resolution: all required progress feature work is complete.
The shared refreshed registry includes recovered achieved stages, accurate
limitations and historical records. Browser, concurrent/atomic CLI publication,
source identity, fresh inventory, idempotency, full current evidence and cleanup
verification pass. The durable checkpoint identifies pending independent live
replay and broad Ridley acceptance frontiers; those gameplay gates are outside
this progress tooling ticket and will use new observations when achieved.

Integration branch: `affected-family-assignment`. Exact maintained source hashes
are progress.rs `06aefc6c5b598bff9cfc659cfc814b8fbc83837bff724761a99dca1f2f9e87b3`,
private declarations.rs `35181a610e5ee9e37edb00841bdd058bd90c04116650067dab5d07e88d42fb81`,
and tests.rs `204dcc0ab93f060ccc7a10bcde978997689eb224f5a58ce3e59af1d1f3630e4c`.
`../recovery-current-publication.sha256` checks the exact source and current
published artifacts. Final shared wiring commit remains the coordinator's work;
this ticket is resolved locally against the retained source/evidence, with no
new commit or live achievement inferred at that checkpoint.

2026-10-13 coordinator commits: lexical inventory correction is `c8f161d`
(three pinned/lexical regression tests pass); maintained progress tools,
dashboard template and agent instructions are `b8c3ffc` (fourteen progress
tests, retained browser/concurrency verification). Shared gameplay registration
and live wiring are separate pending integration changes. Generated dashboard,
registry, binaries and retained large evidence remain local. The trail root
received its independent live observation after passing post-cleanup
authentication; broader composed stages still wait for their final receipts.
