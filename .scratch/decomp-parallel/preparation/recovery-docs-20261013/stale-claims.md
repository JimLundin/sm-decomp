# Current documentation corrections

Status: proposed corrections; final actual receipts pending
Owner: root `01a13500-1ef8-73d3-9a0d-3446b6050eba`

Only operational claims change. Preserve historical source epochs, sealed
requests, snapshots, observations, failure logs and their manifest bytes.

| Original document/location | Stale current claim | Exact correction |
| --- | --- | --- |
| `preparation/status.md`, Exact source and ownership | Current execution is `c8799f3` at `/tmp/sm-decomp-preparation-integration-20261013`; validator correction active separately | Set current execution/validator to clean `/tmp/sm-decomp-preparation-final-source-20261013`, commit `2d59536c059a1ca9c31d9278bff2d9222eafa56c`, fingerprint `e5e412053f6dbe2cbadaba077192e885684fbb9f2699cafeaa6acd694879d78d`, engine SHA-256 `df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84`. Retain old subject/validator as historical links. |
| `preparation/status.md`, Ownership | Current coordinator is `01a13460`; current Git ownership asserted from earlier checkpoint | Set current root `01a13500-1ef8-73d3-9a0d-3446b6050eba`; reference fresh root inspection for Git/index/process ownership. Prior `01a13460` and `01a134cd` remain historical. |
| `preparation/status.md`, Preparation gates/cleanup | Final integration/validator cleanup active against earlier epoch | Keep 04 claimed until actual final eleven controls, two serial integrations, validation, current-root readiness/host-preservation, both cleanup passes and reauthentication are attached. Set preparation ready only then. |
| `spec.md:38`, Current checkpoint and limits | `d8a9045` table is titled current | Title it historical checkpoint and identify table as observed source/claim state at that checkpoint; use preparation status for current source/owners. |
| `spec.md:63–67`, Capability table | Cross-effort inspection, real-session fencing, infrastructure admission and durable handoffs missing/future | Describe implemented cross-effort path/ticket/submission/capacity fences, current role generations, sealed infrastructure admission and typed Start/Checkpoint/Inspect/Resume/Adopt. Actual liveness remains external inspection. |
| `spec.md:75–76`, Authority cutover | Tickets describe future work; no explicit separate launch boundary/private readiness distinction | Tickets 01–03/05 resolved, 04 pending; private validation preserves host absence. Production prepare/register/validate/activate follows separate gameplay launch instruction. |
| `spec.md:130–136`, Handoffs | Manual `.md` convention; maintained handoff/WIP/resume ticket03 future | Make typed TOML authoritative, rendered Markdown derivative, and describe maintained Start/Checkpoint/Resume/Adopt. Keep gameplay evidence/gates separate. |
| `map.md:39–40`, Decisions | 02/03 implement disjointly | 02/03 integrated/resolved; old adoption remains historical. Add current corrected-source and actual final receipts pending. |
| `map.md:50`, Ticket04 row | Integration/final authentication active without current subject | State corrected frozen-source final validation/readiness/cleanup pending; set resolved only with final receipts. |
| `spec.md`/`map.md`, top-level status | Preparation completion could be mistaken for overall spec completion | Keep spec/map open for gameplay06–20. Add preparation-specific status; remove resolved preparation blockers only after 04 actually resolves if root maintains blockers as unresolved-only. |
| `issues/04-…md:7–8`, Owner/source | Old display owner `/root/drill_preparation` and `c8799f3` current source | Current root UUID and `2d59536…`; append fresh-source section before unchanged original evidence. Retain historical failure section. |
| `issues/04-…md`, acceptance disposition | Old B serial-integration rejection remains latest evidence | Add fresh two sequential authenticated integrations and actual passing final verdict when available; retain original rejection as diagnosis. Add exact cleanup/retirement/reauthentication. |
| `docs/agents/decomp-sessions.md:7–12`, startup | No direct launch handoff pointer | Point later-launch branch to final handoff/actual receipts, while preparation uses status. |
| `docs/agents/decomp-sessions.md:274–294`, integration/completion | Integration source/submission distinction unclear; checkpoint observation reads current | State exact submitted writable scope versus frozen merged integration source; label Ceres checkpoint states historical and require current status. Preparation01–05 and overall gameplay completion have different gates. |
| `preparation/launch-handoff-proposal.md:59`, prospective root | Final readiness observation binds `01a134cd` | Adopt concise `launch-handoff.md` bound to recovered root/current-root readiness successor after actual receipts. Leave proposal unchanged as previous session's artifact. |
| `preparation/readiness-final/README.md`/`helper-build.toml`/requests | READY/caller/paths describe prior root `01a134cd` and old helper only | Preserve package unchanged; use sealed `readiness-recovery-20261013` successor with `01a13500`, fresh observations, current helper build/source and actual private validation. Copied predecessor text must be corrected in successor before validation. |
| `preparation/cutover/activate-request.template.toml` | Old checkout/session and placeholder drill SHA | Historical template stays unchanged; generate fresh reviewed launch request with current real caller/generation and exact final seals. |
| `catalog/subsystem-catalog.md`/`later-lanes.md` | Observed HEAD/owners/hashes can be mistaken for live authority; suggested `door_transition`/`room_transition`, `persistence`/`save` differ | Label observations dated; inspect at launch. Each research Answer selects exact reviewed module/files/contract before native scope creation. Existing research evidence stays unchanged. |

Root reports no source delta with `git diff 2d59536 HEAD -- Cargo.toml Cargo.lock
crates tools`; append the exact checked HEAD/receipt before treating main source
as equivalent. The retained frozen source path/HEAD/fingerprint remains validation
authority. Harness-v2 adaptation requires its own review receipt; root reports
review limited to typed Start `submitted_commit`/caller corrections and pointer
recovery. No adaptation equivalence is asserted by these docs.

## Final adoption changes after actual validation

Replace each **PENDING** block in proposed status, spec, map, ticket04 and launch
handoff with exact artifact paths/SHA-256, session/claim/generation identities,
passing verdict state and cleanup measurements. Set ticket04 `Status: resolved`,
status `Status: ready for separately requested gameplay launch` and all01–05 rows
resolved. Keep spec/map `Status: open`; gameplay06–20 have not launched/resolved.
Host authority/ledger absence and unchanged Ceres/bridge seals must be backed by
actual readiness preservation receipts. Fresh later launch still reinspects
owners and creates current production credentials.
