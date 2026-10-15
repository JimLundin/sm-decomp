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

Provisional source `a643f8b` preserves an open executable descriptor and hashes it
lazily once when the command requires identity. The utility control changes from
365,502,352 executable bytes read to zero; Offline identity still reads the exact
executable once. Descriptor replacement controls pass. The `verification` profile
uses opt-level 2, 16 codegen units, no LTO and separately retained line-table debug
information. Initial optimized pair build: 755.27 seconds, 2,771,584 KiB peak RSS.

Matched source `5b6e59d`, complete 12-case workload with four continuations:
three alternating warm dev samples 12.639/11.770/11.426 seconds versus
verification 1.117/1.106/1.103 seconds. Median 11.770 to 1.106 seconds is about
10.65 times faster for this bounded workload. Fresh Rust/actual-C executions,
private artifact caches and shared four-worker capacity were retained; outcomes
were byte-identical and cleanup plus independent/cross-profile admission passed.
This is not a complete-batch multiplier. Proof:
`/home/jim/sm-decomp-performance-tools-proof/family-profile-comparison/alternating.toml`.

Remaining: helper/runtime and disposable-image tamper controls, final assembled
source/tool pair, complete gates and sealed installation candidate. Candidate
staging root is `/home/jim/sm-decomp-performance-installation-candidate`.
Root supplied the exact existing production request, two Installations, launch
plan and prepared/active migration references from the documented production
checkpoint. Preparation preserves original policy/claim tuples, acceptance
requirements and the held pause blocker. No active registry, fence, policy,
tool or attempt changes before the supported final ownership transition.

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
