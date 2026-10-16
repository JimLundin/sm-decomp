# Ceres door verification execution handoff

Status: released sequence stopped at first failure; execution assignment closed
Verifier: `01a14300-1d4f-77e1-b198-a47e8d902bb7`
Integrator: `01a1414e-7fc1-73c3-969e-a7294824368a`
Ticket: restart pipeline 08; monitor ticket 05

## Ownership

Verifier may create only the reserved run outputs under
`/home/jim/sm-decomp-restart/restart/out/ceres-door/runs/`:

- `reference-01`
- `rust-01`
- `cli-rust-01`
- `fault-01`

Integrator owns source/build/cache mutation, warm dev tests and publication.
The exact source, binaries, library, argv, environment and timeout will be
recorded here from its release. Do not infer the commands from older consumers.
Do not launch until explicitly released. Stop after the first failed check;
preserve output and report without an unapproved rerun or source fix.

## Frozen execution inputs

Released source `ca4f20959a1b9ced89d8e731deff6b494e195473`; exact handoff copied
to `restart/out/ceres-door/evidence/verifier/released-handoff.md`. All five
released runtime file hashes matched before execution. Root reports release
10:52:43 UTC; first command began 10:58:28 UTC.

## Results

`reference-01` completed 9,380 steps but failed material admission at 10:58:40
UTC, exit 1, 11.93 seconds. No candidate/CLI/fault commands were launched by
this verifier. Trace, final WRAM/PNG, door/lifecycle observations and failed
result are preserved; seven output hashes recorded. Source diagnosis showed
the required BA48 setup is conditional escape behavior, absent on this route.

Integrator explicitly closed the old execution assignment and launched the
corrected dev/CLI/fault runs directly at source `bd4e28c`; it retains all run,
cache and source mutations. This verifier now owns only read-only analysis and
its reports in `evidence/verifier/`. No stale reserved command may be launched.

Independent review found the C trace's authenticated footer contains 9,380
records. Writer.finish precedes the failed material evaluation; the existing
Reference reader checks its checksum, identities and consecutive ordinals.
Root's passing dev-01 completes that strict reference comparison under unchanged
host/gameplay/input identities. All seven original output hashes still match,
and the original result remains failed. No second C replay was required.

## Cleanup

Verifier's shell and reference child exited. The process observation captures
later integrator-owned dev processes, not verifier writers. No verifier build
or worktree exists; no artifacts were removed. Failed reference and reports
are retained for the integrator's final cleanup after its readers/writers exit.
