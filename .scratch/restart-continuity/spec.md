# Keep the restart agents aligned and moving

Status: active — process improvement, structure and continuity
Started: 2026-10-16
Owner: session `01a143c7-78d1-7033-8eff-a361c16993dd`

The user requested coordination of the existing restart agents so work stays
aligned with the project goal and continues. The goal is one playable vanilla
Super Metroid port, reset through credits, with readable Rust owning gameplay
and hardware emulation remaining separate. Map Rando follows vanilla. Project
tools, orchestration and tests remain Rust.

This effort owns only `.scratch/restart-continuity/` in the original checkout.
It coordinates existing sessions and records continuity; it does not take source,
integration, shared progress, or legacy production ownership. No additional
verification gate or coordination framework is introduced.

Completed coordination: [01 — Establish continuing implementation ownership](issues/01-continuity.md).
Completed original watch: [02](issues/02-maintain-structure.md).
Completed door watch: [03](issues/03-ceres-doors.md).
Completed transition watch: [04](issues/04-room-transition-process.md).
Completed directional watch: [05](issues/05-directional-scrolling-process.md).
Completed full-Ceres watch: [06](issues/06-full-ceres-process.md).
Completed camera watch: [07](issues/07-camera-process.md).
Completed projectile escape watch: [08](issues/08-projectile-escape-process.md).
Completed original Handle watch: [09](issues/09-original-handle-process.md).
Completed Parlor progression watch: [10](issues/10-progression-process.md).
Completed Morph Ball watch: [11](issues/11-morph-progression-process.md).
Completed ability progression watch:
[12](issues/12-ability-progression-process.md).
Completed item-instruction watch: [13](issues/13-item-instruction-process.md).
Completed Morph instruction ownership: [14](issues/14-morph-instruction-process.md).
Completed exposed-tank family: [15](issues/15-resource-tank-family-process.md).
Completed improvement: [16 — Batch publication](issues/16-batch-publication-process.md).
Current follow-up: [17 — New tank live milestone](issues/17-live-tank-process.md).
Parallel execution assignments: [work distribution](parallel-work.md).
Ongoing process ownership and experiments: [process improvement](process-improvement.md).
Concrete follow-ups and recovery: [updates](updates.md).

## Owners

| Responsibility | Existing session | Durable record |
| --- | --- | --- |
| Restart code, scopes, workers, integration, checks and cleanup | `01a144ca-27ba-7130-a9e2-a615c7c80e3a` | Energy live22 claimedf479add; disjoint policy/runtime workers active |
| Exclusive continuation/liveness | `01a144fc-f095-7d71-aa00-10358e7f716e` | Healthy reserve took over after roadmap01a144f0 timeout; [transfer](watch-transfer-20261016.md) |
| Independent evidence analysis, cost/coverage and retention | `01a14300-1d4f-77e1-b198-a47e8d902bb7` | `../restart-verification/progress-reporting-cost.md` |
| Cross-session continuity and unresolved ownership | `01a143c7-78d1-7033-8eff-a361c16993dd` | This directory |

The user opened the replacement integrator after the old session
`01a1414e-7fc1-73c3-969e-a7294824368a` failed twice before executing its next
task. Old-parent retries are stopped. Replacement ownership and supported child
dispatch are recorded in restart ticket 13; no source or evidence was lost.

The previous baseline monitor completed its bounded work and remains idle;
there is no need for another duplicate gameplay watch. Current collaborators
were explicitly asked to retain these separate responsibilities and report
concrete findings, rather than repeat generic review reminders.

## Direction

Preserve the native Ceres escape/area 0 baseline and completed projectile and
scrolling Rust pilots. Ticket 06 is complete at clean checkpoint `f39ad13`.
Ticket 07 is also complete at `a987df5`: original shot lifecycle captured and
first Fire routine compared over its declared 441-byte original projection.
Ticket 08 is complete at `ca64dfc` for the reached Ceres proximity-door actor.
Room transitions completed at `cee31cf`, directional scrolling at `71cf03b`.
Producer ticket 11 completed mixed full-Ceres exercise at 4c8ffaa, camera
ticket 12 at 099af04, projectile escape updates at 4161074 and original Handle
fixtures at a8a8c97. Parlor progression completed at eefb09a. Current ticket 16
completed the bounded route to Morph Ball at316f557 with pickup, presentation
completion, resumed control and exact strict replay. The prepared successor
completed actual Morph use and first missile pickup at67b234c. The13-body Chozo
missile instruction/trigger family completed707c5de with actual Rust ownership.
Ticket19 completed the ordinary Morph family at1b382ce, with controls and
fixed-input live pair passing on first candidate64f135c. Root immediately
selected the prepared exposed-tank family for source/actual-C acceptance;
the old route supplies compatibility only, not new collector live execution.
Each failure should lead to a changed, bounded
experiment. Run checks because behavior changed or a specific question remains;
keep passing results reusable and feedback cost measured.

Keep four facts separate: named gameplay reach, Rust ownership and remaining C
behavior, original-ROM comparison extent, and measured feedback cost. Native C/Rust
agreement is not original-ROM agreement. Source or test counts are not gameplay.

Before ending a delivery, preserve the exact source, results, cleanup and next
owner/command. Continue the first dependency-ready restart task within the
authorized design. The roadmap's early original-ROM feasibility work remains
open before broad translation; continued work must not silently waive it or
reintroduce the legacy production gates into the separate restart.

## Observation method

Read existing tickets, thread status, commits, run results and process state.
Do not repeat producer builds, gameplay runs, tests or authentication. Retain
live or ambiguous owners; an `active` thread flag alone does not prove progress.
Send concrete recovery guidance when an owner stops or a ready task is unowned.
Do not release claims or kill another session's processes to manufacture activity.

This is current-session coordination, not an installed recurring background
monitor. Existing monitoring sessions remain independently responsible for their
requested watches.

At 11:15 the user explicitly renewed continued active improvement rather than
stopping after one milestone. Continue evaluating and acting on the next
measured bottleneck while useful authorized project work remains. Preserve
independent execution and use real result/ownership events for interventions.

## Maintained structure

The user's 09:08 continuation explicitly asks to maintain structure. Keep one
integrator, isolated writable scope ownership, read-only research and the
existing separate monitor responsibilities. Keep gameplay and synchronous host
work behind their existing boundaries; original-reference capture is a separate
Rust tool. Reuse the working trace/comparison implementation. Any new boundary
must address a concrete observed need, not anticipated full-game requirements.

Use the existing local ticket/map, scoped commits, separate progress claims,
bounded experiments and cleanup. A monitor finding is advisory evidence and
direction, not another acceptance gate. First capture stays at 8,850 original
frames and a 120-second process deadline. Record exact commands and input
identities; distinguish capture, routine comparison and full replay parity.

## Initial handoff (historical, 08:21 UTC)

At 08:21 UTC, source edits were observed in the isolated room-scroll worker and
the integrator's shared gameplay/runner files. The implementer and both monitors
were active, and all three had acknowledged their responsibilities. The roadmap
monitor retains the gameplay watch through delivery and next owner/command;
the verification monitor retains cost/coverage/cleanup review. Concrete result,
blocker and successor notifications are requested back to this session.

At that handoff scrolling was in progress. It subsequently completed, one model
socket timeout was recovered without losing work, and the integrator claimed
the original-capture successor. See `updates.md` for the current evidence.

Cleanup for this coordination work: Markdown notes and read-only inspection
only, with no tests, builds, gameplay runs, worktrees or run intermediates created.
Utility cleanup is not applicable; reclaimed bytes: 0. Available disk at the
final observation was 106,962,345,984 bytes. All other sessions' source,
worktrees, active processes, warm caches and retained evidence are preserved.
