# Prepare optimized current tools and a sealed installation candidate

Status: claimed
Blocked by: 03, 04, 06, 07
Triage: ready-for-agent
Type: task

## Scope and ownership

Own controller/sandbox build-profile selection, narrowly justified CLI startup
changes, tool identity reporting and preparation of the installation candidate.
Use existing production installation/registry machinery and current ownership.

## Required work

Compare matched optimized profiles for the controller, its dependencies and the
separate sandbox. Measure build time separately from repeated verification time;
keep debug information separately where useful. Defer executable hashing only for
commands whose contract does not require it. Retain exact producer/admission
identity checks and demonstrate which tool paths the command actually launches.

Build from frozen final source and run real relevant controls with the selected
tool pair. Prepare an installation candidate binding binaries, source, readers,
recipe/storage/scheduling contracts and unchanged required coverage. Do not replace
active pinned binaries in place or mark prerequisite-stopped work accepted.
Activation belongs to the final existing-owner transition after ticket 09's gates.

## Acceptance

Controlled startup and representative full-verification comparisons identify a
profile with useful repeatable benefit. Both selected binaries are actually used
by the candidate installation. Installation/readiness/status checks preserve
original lineage, requirements and refusal behavior; tampered tools are rejected.
All concrete installation inputs and commands are reviewable before activation.

## Completion evidence

Pending. Record source/tool hashes, flags, profiles, matched measurements and
candidate-installation pointers, plus build-output cleanup and retained readers.

## Comments

2026-10-15: provisional isolated profile/startup implementation is assigned to
`/root/shared_preparation` after its completed ticket 04 handoff. Worktree:
`/home/jim/sm-decomp-performance-tools`; branch
`performance/optimized-tools-20261015`; starting checkpoint `83c0f4f`.
Exclusive target: `/home/jim/sm-decomp-offline-verifier/target`, released by 04
after retaining and authenticating its evidence. Proof parent:
`/home/jim/sm-decomp-performance-tools-proof`. Owner session remains
`01a13e7f-ab01-7413-a1c2-288b08a4b719`.

This refines only the start of independent profile/startup work: final matched
tools, complete verification and the sealed installation candidate still wait
for the complete 03/04/06/07 source and Ceres recipe repair. Own root Cargo
profile configuration, narrow pipeline startup/identity code and corresponding
tests, plus this ticket's candidate preparation documents. Do not write active
installed tools, policies, registries, claims or gameplay source. Existing
installation machinery remains the only activation path. Root will supply the
final combined revision before the accepted tool-pair build.

The earlier stripped-plan timeout is not a passing performance experiment.
