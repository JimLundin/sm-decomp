# Research complete Samus movement/contact frame continuation

Status: open
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumers:07,09
Read-only research; implementation depends on required13 hardware contract

## Work

Read [catalog](../catalog/subsystem-catalog.md) and [later lanes](../catalog/later-lanes.md).
Reuse current samus/samus_control/collision/projectile flight/special/power-bomb,
the accepted whole projectile roots and resolved139 trail root. Identify the
complete player beta/frame movement/input/contact root and its exact dynamic
pose/handler/collision dispatch, ROM tables, integer/byte aliases and host calls.
Do not infer a caller hit from existing helper or projectile counters.

Candidate exact envelope: `crates/port/src/samus.rs`, `samus_control.rs`,
`collision.rs`, `samus_projectile_frame.rs`, `projectiles.rs`,
`projectile_flight.rs`, `projectile_special.rs`, `projectile_trails.rs`,
`power_bomb.rs`; matching exact tests under `crates/port/tests/` and proposed
`tools/port/samus-frame.toml`. Select only needed files into one reviewed scope;
camera_scroll/room modules remain read-only room dependencies. Integrator owns
lib.rs, canonical state/ROM, live registration and shared callback adapters.

## Acceptance / Answer

State existing translated/verified/integrated/live-exercised domains and the
remaining complete root; pin source+observed root exercise and each dependency.
Current native bank86-only exercise schema is a blocker to non-bank86 promotion:
create a reviewed Rust extension task if no matching supported receipt exists.
Publish exact1–32 coherent routine task/scope with writable/input hashes when
facts exist. Require actual-C complete WRAM/return/transient/order behavior,
wrap/alias/collision controls, material independent mixed route exercise,
original-ROM matching boundaries and visible fallback. Use60s initial and
3600s merged-source gates. Add new implementation children to07/09 and needed13
dependencies; preserve accepted projectile/trail domains and explicit Ceres
drawing refusal rather than copying upstream undefined-local behavior.

## Completion evidence

Pending Answer/child/scope/artifact hashes, no-run or scoped-run cleanup and
retained source/store/readers/private-input authentication.
