# Prepare optimized current tools and a sealed installation candidate

Status: open
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

The earlier stripped-plan timeout is not a passing performance experiment.
