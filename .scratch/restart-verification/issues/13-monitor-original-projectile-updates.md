# Monitor original projectile updates

Status: resolved
Blocked by: None (12 resolved)
Triage: ready-for-agent
Owner: session `01a14300-1d4f-77e1-b198-a47e8d902bb7`

## Scope

Independently review completed artifacts, measured feedback cost, retention and
publication for restart ticket14. Integrator session
`01a144ca-27ba-7130-a9e2-a615c7c80e3a` owns implementation, source freeze,
execution, cleanup and publication. This watch performs no source audit,
capture, game run, build or fixture execution and adds no acknowledgment gate.

Authoritative ticket:
`/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/14-original-projectile-updates.md`.
Claim commit `1ff7457`; baseline `41610741f80777a52cbfa1b97a631fedbd0008a1`.
Evidence root `restart/out/original-updates/`; final report path awaits the
integrator's assignment. Completed ticket13 reports remain immutable.

## Frozen comparison boundary

All34 retained original HandleProjectile pairs, frames8661..8694, independently
seeded from their own original entry. Original selected output: existing441 Fire
bytes plus360 trail bytes `[0xd658,0xd7c0)`, total801. Full131072-byte C/Rust
WRAM equality is a separate comparison. This establishes neither continuous
replay nor drawing, inter-call evolution, CPU/stack or hardware/audio parity.
Impact is frame8676; removal is8694.

Root freezes eligibility, complete call ledger, CPU linkage and source/input
identities before execution. Review declared34-call completeness, actual
results/first differences, negative-control meaning, batch authentication and
timings, final reuse, cleanup and retained manifest. No repeated13 checks or
standalone progress-status scan are needed.

## Comments

Startup: integrator, continuity and roadmap observed active. Two existing
workers own disjoint selector and executor worktrees; root owns CLI/integration.
The user's crash-recovery instruction remains in force. Roadmap owns liveness
coordination; relay confirmed failures there, retain live/ambiguous ownership,
preserve source/evidence before scoped cleanup and supported replacement.

## Completion evidence

Immutable report written at integrator-assigned
`/home/jim/sm-decomp-restart/restart/out/original-updates/evidence/independent-review.md`;
REPORT WRITES FINISHED sent to root. Frozen source
`cbe2b38c9dc1ecda2c640ecd86c04a073d778782`. All34 raw C/Rust131072-byte WRAM
outputs and three-way801-byte projections independently agree; all471 batch
and42 fault-map hashes pass, plus Fire's retained outputs/executable. Batch
7.134213918s; default Fire command2.837s; deliberate output fault3.118215258s
rejects call2/frame8661 at0xb64 (145 versus144). It tests comparator rejection,
not a gameplay mutation. Focused12 passing tests and Clippy complete.

Cleanup confirmed: both worker trees retired with small source/harness snapshots,
442847232 allocated bytes reclaimed apart from earlier89.5MiB Cargo clean;
104237481984 bytes available. Material/batch ordinary and deduplicate utility
passes report zero recognized disposable bytes; warm target and dependencies
preserved. Post-cleanup pre-manifest authentication succeeds in0.588552409s.
No reviewer process, worktree or build output exists. Final manifest/publication
await root closeout; the report will not be edited afterward. A missing-path
progress-update failure is root's publication task, separate from passing outputs.

Final closeout `a8a8c97`: clean scoped checkout;563 retained plus530 dependency
hashes authenticate (1093 OK lines), including the immutable review. Absolute-path
progress retry succeeds, publishing only bounded Handle verification;18.301260922s.
Final manifest/dependency authentication1.575905215s executes no gameplay. Actual
closeout command history confirms no standalone progress-status scan. No14 work
remains; completed source, negative control and evidence remain sealed.
