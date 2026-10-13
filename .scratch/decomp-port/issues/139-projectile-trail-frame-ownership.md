# Native Samus projectile trail frame ownership

Status: resolved
Blocked by: 16
Triage: ready-for-agent
Type: task

Owner: `/root/independent_gameplay`
Worktree: `/tmp/sm-projectile-trails`
Branch: `decomp-projectile-trails`
Baseline: `b6f30bd`

Translate `HandleProjectileTrails` ($90B6A9) and its three movement opcodes
($90B525/$90B587/$90B5B3) as one complete scheduled script/render closure.
Ticket16 already owns trail creation/placement. Preserve eighteen-slot reverse
traversal, left/right ordering, wrapping timers and coordinates, native script
termination, frozen rendering, clipping, signed OAM capacity and transient
unaligned word writes, including aliases into later trail state.

## Scope and dependencies

Writable ownership is new `crates/port/src/projectile_trails.rs`,
`crates/port/tests/projectile_trails.rs`, `tools/port/projectile-trails.toml`
and `tools/decomp/projectile_trails_inventory.rs`. The integration owner handles
shared public registration, catalog and live harness changes. This scope is
independent of ticket138's Ridley instruction ownership.

Pinned actual-C flight replay
`out/port/flight-fixed-live/20261010T204643.443282Z/verdict.json` records nineteen
natural trail allocations in each C baseline. Its SHA256 is
`367bf4ce28ff4519581ab52e1411097c3184099c180e90f8a7469080c8f174be`.
Creation counters establish planning provenance; they do not establish native
whole-frame ownership. Fresh Rust-produced actual-C active-frame observations
must establish the remaining scheduler/script/render behavior.

See [scope](../parallel-independent-scope-projectile-trails.md) for source
identities, exact behavior and verification boundaries.

## Required verification

- Actual compiled pinned C versus Rust: full WRAM/raw returns for all eighteen
  slots, direct opcodes, native-ROM lifecycle chains and naturally active captures.
- Independent controls for partial writes, ordering, timer/movement wrap,
  frozen rendering, clipping, OAM saturation/aliases and support refusal before
  mutation. Malformed C pointers are outside the oracle domain.
- Four fresh C/mixed processes with exclusive nonzero Rust whole-frame ownership,
  active material evidence and matching state/effects. Preserve current native
  projectile admission. Broader original-ROM parity remains a separate frontier.
- Scoped ordinary cleanup and deduplication, exact-source retention, integrated
  worktree removal and post-cleanup authentication before resolution.

## Comments

2026-10-12: root assigned this independent implementation lane after the user
confirmed parallel decompilation orchestration. Shared worker root
`/home/jim/sm-decomp/out/pipeline`, capacity four; bounded initial experiments
are at most sixty seconds. All new orchestration and tests use Rust.

## Recovery completion evidence in progress, 2026-10-13

Implementation integrated at `2b62134bfbee7c5dcd4c81616edc5d91e781d44f`.
Current merged actual-C family passes all 3,316 pairs across four routines and
post-cleanup authentication passes in
`../recovery-integrated-projectile-trails-post-cleanup-authentication.toml`.
Recovered natural family retains fourteen active captures in its exact source
context. The current 9,000-input four-subject native live verdict is
`out/recovery-session/trails-live/1791860457650339558-98848/verdict.toml`,
status `mixed-C-Rust-integration`, full state/effects match, zero fallback.
Each mixed subject records 3,824 exclusive Rust frames, 167 active calls and
zero refusals; baseline subjects match traversal and active counts. Final
post-cleanup authentication is queued behind the active composed shared workers.
Ordinary cleanup reclaimed 47,310,242 bytes; dedup zero. Stale implementer Git
registration was retired and exact source-only dependencies retained.
Ticket resolution waits for the final authentication receipt and scope
reconciliation preserving native projectile admission in its independent gate.

## Completion evidence

2026-10-13 final retained trail authentication passes in
`../recovery-trails-live-post-cleanup-authentication.toml` (four subjects,
94artifacts each). The independent composed129712gate also passes, preserving
native137 admission and all16observed9642allocations; its post-cleanup receipt is
`../recovery-composed-post-cleanup-authentication.toml`. The complete scheduled
trail root is published live; the three movement opcodes retain Offline stages
because this prefix has no direct natural hits. Shared live wiring commit:
`3ce28ba58aba319b60bbc949153baafd4433132e`.

Both scoped cleanup passes succeed: trail47310242bytes/dedup0, composed48176296/
dedup0. Exact integrated source contexts remain without Git/build trees;
implementer registrations are retired, packed pointers/readers/shared store
retained. Full evidence and domain limits:
`../recovery-live-completion-20261013.md`; disk `../recovery-live-final-disk.txt`.
The rawinput route does not establish original-ROM movie completion.
