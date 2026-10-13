# Current integrated translation stages

Status: twelve affected Offline families authenticate after cleanup; independent live gates pending.
Owner: root integration coordinator; registry publication delegated to `/root/progress_finish`.

Pinned source revision is `578f90b3cc49557bb70060ad033bb90b8cf8ac50`; shared
inventory SHA-256 is `7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`
with 6,053 definitions and 75,229 body lines. All twelve current integrated
Offline families passed 20,943 actual-C fixture pairs and authenticated after
both cleanup passes. Full exact verdict pointers, family counts and cleanup
receipts are in `recovery-integrated-final-authentication-summary.toml` and
`out/recovery-session/integrated/batch.toml`. Six coordinator authentication
reports and six `progress-integrated-*` reports preserve the returned current
source seals; these reports are evidence records, not a new verification run.

| Translation scope | Exact installed gameplay commit | Current actual-C domain |
| --- | --- | --- |
| Native projectile trail script/render frame, four routines | `2b62134bfbee7c5dcd4c81616edc5d91e781d44f` | 3,316 fixture pairs |
| Scheduled Samus projectile update/render passes, four routines | `c711faea56f20d42113eb63393cc4a76d4940557` | 476 fixture pairs |
| Complete camera scrolling frame and helpers, four routines | `7512b6479afe55da270931a9f3ae26c9d558e5a5` | 467 fixture pairs |
| Normal room scrolling and tilemap queue subsystem, eleven routines | `97afab2f1dc712a2433a1ef35bb9dd8901ca6011` | 1,166 fixture pairs |
| Bounded native Ridley attack instruction frames, eight routines | `e947e968086a15c170f58472764bf9244b403a2a` | 1,804 controlled pairs and 90 natural captures |
| Native projectile allocation/scheduling, three inventory definitions plus one adapter control | `692805ce76eb73763a36cdf24082f829b98248cd` | 1,840 fixture pairs plus authenticated initializer/frame/render/contact prerequisites |

The installed gameplay commits are integrated into main at baseline
`c711faea56f20d42113eb63393cc4a76d4940557`. Shared Rust verification/live wiring
is currently uncommitted and authenticated at its exact frozen source hashes;
final wiring commit remains coordinator work. `integrated` observations describe
the installed scope and passing affected-family gates at this freeze. They do
not imply independent live ownership or original-ROM parity.

The broad uncommitted Ridley/Ceres recovery covers 220 definitions and retains
its separate translating/translated observations. The eight-definition attack
scope includes generic `ProcessEnemyInstructions` only within the accepted
Ridley attack script. It does not establish verification of all enemy scripts
or the broad lifecycle/AI/rendering closure. Ninety natural captures supplement
the controlled attack scope; explicit C services and refused scripts remain.

Native137's four-entry family includes synthetic `EprojInstruction`, which is
not a recovered C definition. Its adapter control remains authenticated but
contributes no lexical C inventory coverage. Native progress uses the three
actual definitions `SpawnEprojWithRoomGfx`, `SpawnEprojWithGfx`, and `EprojRunAll`.
Existing historical resolved native137 status and evidence remain preserved.

Automatic batch deduplication reclaimed 4,522,901,504 bytes across 64,952
snapshots. Subsequent ordinary cleanup and deduplication reclaimed zero further
bytes, with 63,551 snapshots checked. All stale implementer Git worktree
registrations were pruned, with required source-only snapshots preserved;
`recovery-worktree-retirement-20261013.toml` records this. The passing retained
post-cleanup authentication permits the achieved stage publication. Trail and
full composed four-process live replay gates remain pending. Camera and room
input bounds, Samus PLM/HDMA services and Ceres refusal, and external callback
closures remain subject to their authenticated fixture and admission domains.
