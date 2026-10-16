# Restart progress watch

Status: resolved
Started: 2026-10-15 21:05 UTC
Scope: observe the restart until its first playable baseline, as requested by
the user. Report drift here; this watch does not introduce an acceptance gate.

Work: [01 — Watch the playable baseline](issues/01-watch-baseline.md).

Watch `/home/jim/sm-decomp-restart`, branch `restart/playable-baseline`, and
coordinator session `01a1414e-7fc1-73c3-969e-a7294824368a`. The implementation plan
is that worktree's `.scratch/restart-pipeline/map.md`; ticket 04 owns the first
runnable baseline. This monitor owns only `.scratch/restart-monitor/` in the
original checkout. It does not dispatch workers or mutate another owner's work.

Use existing commits, work status, command results and retained observations.
Do not launch duplicate builds, verification, authentication or gameplay runs.
Record unknowns instead of creating new infrastructure to measure them.

Report boot, controllable play and Ceres escape independently. The plan's
stronger baseline reaches normal area 0 gameplay after Ceres. Original-ROM
comparison and Rust gameplay ownership are separate facts; unfinished comparison
must not erase a demonstrated playable increment. File validation, compilation,
test counts and documents are enabling work, not gameplay milestones.

Watch for these concrete departures:

- Broad verification after documentation, presentation or unrelated refactors,
  without an identified behavioral dependency.
- Repeated expensive checks after a passing result, without a relevant change,
  new failure or specific unresolved question.
- General orchestration, evidence machinery or architectural expansion before
  the first boot/input experiment, without a demonstrated need for that experiment.
- Repeated failures without a changed hypothesis or bounded next experiment.
- A completed planning handoff with the executable task left unowned or idle.
- Claims of playability from input EOF, frame counts, build success or fixtures
  without the named gameplay postcondition.

The sketch's 5-second routine and 30-second subsystem feedback targets remain
provisional measurement targets, not added prerequisites for baseline progress.
Distinguish initial cold build cost from repeated edit-to-result cost.

## Completion

The requested native playable baseline is complete on `restart/playable-baseline`,
implementation `91f84a76e489a40f3e93a974ee6b5421a9f4827e`. Ordinary boot, controller
response, desktop/audio presentation, Ceres escape and normal area 0 arrival are
demonstrated. Saved-input replay reproduces the successful route exactly.
The runnable build and evidence remain available after scoped cleanup.

The watched restart delivered gameplay without a broad legacy verification gate.
Original-ROM comparison and Rust gameplay translation remain separate unfinished
work. This bounded watch is complete; no recurring monitor is installed.
