# Next playable milestone: accumulated Rust through Ceres escape

Status: recommendation consumed; ticket 11 resolved at `4c8ffaa`
Date: 2026-10-16
Owner: scope/research session `01a142f2-9750-79f3-9d1a-e34bcc44a445`

After ticket 10 finishes its existing comparison and cleanup, exercise the
retained full Ceres route once with the accumulated Rust consumers. Prefer this
small milestone before another pre-escape ownership scope: it can expose the
next actual gameplay problem while extending useful coverage of the door actors,
room transitions and directional scrolling just integrated. The normal-camera
packet remains the next implementation candidate if that run has no blocker.
The integrator owns scheduling; this recommendation adds no ticket 10 gate.

## Reusable evidence and input

The existing [ceres-06 result](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/result.toml)
records recovered-C escape and area 0 gameplay, 16,991 host steps, 16,068 actual
polls/completed updates and zero CPU fallback. Its exact native pads remain in
[inputs.txt](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/inputs.txt):
16,991 lines, 67,964 bytes. No new input discovery is needed.

The retained [observations.csv](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/observations.csv)
contains these distinct checkpoints; the row after each header has ordinal1:

| Reached behavior | Native host step |
| --- | ---: |
| Computer room E021 resumes gameplay | 9,497 |
| Short corridor E06B resumes gameplay | 9,732 |
| Ridley room E0B5 resumes gameplay | 9,983 |
| Return reaches E06B gameplay | 12,898 |
| Return reaches E021 gameplay | 13,159 |
| Return reaches DFD7 gameplay | 13,422 |
| Return reaches DF8D gameplay | 14,004 |
| Return reaches DF45 gameplay | 14,337 |
| Escape state 20, then 21, then 22 (hex) | 15,003 / 15,063 / 15,078 |
| Area 0 room 91F8 appears during loading | 16,885 |
| Area 0 normal gameplay state 8 | 16,991 |

These are observations of the old recovered-C route, not expected timing
constants to force onto a new candidate. The return route offers reached
contexts beyond the two outbound crossings; which private routines and branches
actually execute must be measured in the new run. BA48, escape-door behavior,
parameter 0 reopening and leftward directional coverage must not be promoted
merely from room names or source predictions.

The old [invocation](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/invocation.toml)
took 15.153 s with a 90 s deadline. The independently recorded replay is documented
as 19.404 s in [ticket 04](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/04-runnable-baseline.md).
Those are historical baseline CLI costs, not current Rust timings, trace costs
or rebuild-plus-compare measurements.

## What the existing CLI can establish now

Source inspected at restart commit
`4d6c541f5cdedc9667a32ef286850e1bb908989e`:

- [runtime_cli.rs:249](/home/jim/sm-decomp-restart/restart/src/runtime_cli.rs:249)
  reads the explicit saved input file. Rust mode installs scrolling, Ceres doors,
  coordinator and directional scrolling at boot; projectiles activate on reached
  DF45 gameplay at line 367.
- [route.rs:42](/home/jim/sm-decomp-restart/restart/src/route.rs:42) observes Ceres,
  ordered 20→21→22 (hex) with the loading/timer conditions, then normal area 0 gameplay.
  [runtime_cli.rs:317](/home/jim/sm-decomp-restart/restart/src/runtime_cli.rs:317)
  ends the named scenario on that predicate.
- [runtime_cli.rs:397](/home/jim/sm-decomp-restart/restart/src/runtime_cli.rs:397)
  reports the milestone and separate ownership/refusal counters. The CLI's
  `passed` milestone by itself does not enforce every ownership counter and
  does not compare per-step full WRAM/external state/effects with C.

Use the integrator's final checked ticket 10 executable and prepared library,
ordinary `run --gameplay rust --scenario ceres`, the unchanged `ceres-06/inputs.txt`,
16,991-step bound, existing 90 s deadline and a fresh output directory. This is a
proposed run, not a command executed by this research session. Preserve any
first failure and hand the actual boundary problem to its existing owner.

A successful result supports “the mixed C/Rust game escapes Ceres from reset
and reaches area 0.” Inspect the reported counters to state which installed
consumers ran and whether any refused; isolate projectile prelude C calls as
already done by the existing comparison runner. The unmodified baseline inputs
do not carry the pilot's added shot at 8,380, so do not demand or claim its shot
lifecycle from this route. Keep that material evidence in the existing 9,380-step
comparison. Samus, Ridley and other foreign gameplay remain C-owned.

## A complete C/Rust comparison is a separate bounded extension

Passing a longer `--prefix` to current `sm-pilot` is insufficient:

- [scrolling.rs:4](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot/scrolling.rs:4)
  truncates all scrolling-family input to 9,380 and adds a shot at 8,380.
- [sm-pilot.rs:250](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs:250)
  selects that authoring path for the current scrolling/door/transition cases.
- [door_scrolling.rs:239](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot/door_scrolling.rs:239)
  requires exactly two crossings into DF8D/DFD7. Those predicates belong to the
  short scenario and cannot be treated as full-route acceptance.

If a full differential milestone is selected, extend the existing runner with
one named full-Ceres scenario: preserve these exact inputs, reuse the existing
route predicate, and capture one current-host C reference before checking Rust.
Keep the short scenario intact. Do not create a second runner or trace format.

The old observations CSV contains sampled gameplay counters, not the current
SMTRACE1 full-WRAM/external-state/ordered-effect hashes. It cannot replace that
new C reference. Current [identity construction](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs:177)
binds source, prepared host, ROM, inputs, observation schema and named scenario.
The [input reader](/home/jim/sm-decomp-restart/restart/src/bin/sm-pilot.rs:157)
admits up to 40,000 pads/200,000 bytes; the retained route fits. The existing
[trace format](/home/jim/sm-decomp-restart/restart/trace/src/lib.rs:14) has 104-byte
records and a 64 MiB file limit, so 16,991 records require about 1.77 MB plus the small
identity/footer. Reader capacity is not a reason for new infrastructure.

Measure that longer scenario separately from the short edit loop. One passing
milestone can be reused until affected source, host, inputs or a new question
warrants another run. No whole-replay original-ROM claim follows; the existing
441-byte Fire comparison and original shot capture keep their present scope.

## Research completion

Only retained metadata, input extent and source were read. No builds, tests,
replays, capture, authentication, worktrees or run outputs were created. No
cleanup operation applies; reclaimed bytes: 0. Implementation, run retention,
shared progress and ticket 10 review remain with their existing owners.

## Implementer result observed after this recommendation

Ticket 10 closed at `71cf03b`; ticket 11 was claimed at `44861a4`. The existing
integrator then ran [escape-01](/home/jim/sm-decomp-restart/restart/out/full-ceres-rust/runs/escape-01/result.toml):
16,991 steps, 16,068 polls/completed updates, successful escape and area 0 state 8,
zero CPU fallback, 27.445 s. Its retained
[comparison log](/home/jim/sm-decomp-restart/restart/out/full-ceres-rust/evidence/baseline-comparison.log)
reports observations CSV, final WRAM and final PNG byte-identical to the old
recovered-C baseline. This is useful additional baseline agreement, without the
per-step full-state/ordered-effect claim of SMTRACE1.

Projectile refusals are `[0,706,707,707]`; Rust completions are
`[0,4891,5343,5343]`. The retained
[projectile domain](/home/jim/sm-decomp-restart/restart/out/full-ceres-rust/evidence/projectile-domain.md)
explicitly excludes the Ceres escape high bit and Mode7 rendering. The comparison
log observes that bit on 706 host steps, from 14,372 through 15,077, but does not
identify every individual call's refusal reason. Therefore this milestone
retains mixed projectile ownership and does not expand the pilot's admitted
domain.

The other four consumers record zero C/refusals. Doors reach all 13 scheduled
entry kinds, including solid setup and ten puff services. The coordinator
reaches 22 of 27 routines with 630 pending/resumed waits. Directional scrolling
reaches eight of 13 routines, with five right and five left setups, 320 movement
calls each way and 645 balanced scrolling children. These extend live exercise;
they do not grant new actual-C verification stages for those branches.

The integrator's completion evidence records both cleanup passes succeeding and
all 97 pre-cleanup hashes plus shared runtime/source dependencies authenticating.
The existing independent reviewer checked the result without another replay.
This session only read those results. The completed normal-camera packet is
[next-camera-scope.md](next-camera-scope.md); no new gameplay failure from this
milestone blocks that successor.
