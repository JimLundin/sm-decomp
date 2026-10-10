# Readiness evidence test seams

The user confirmed these public interfaces before implementation: repository
readiness/replay commands and their evidence files; reference/subject instruction
and checkpoint APIs; real generated-step corpus/timing runner.

Ticket 01 tests the command and evidence seam, plus the reusable `Run` public
lifecycle consumed by those commands. Tests use real temporary repositories,
files and subprocesses. They verify source restoration, unique evidence runs,
explicit incomplete/error outcomes, logs, identity rejection and mutation
invalidation. They do not establish emulator correctness or a readiness pass;
later tickets exercise those independent seams.

Ticket 04 tests the public strict synchronization command and capture evidence:
identity-bound ordered visits and completion; rejection of missing provenance,
shifted/reordered/truncated captures and incorrect semantic words. It also tests
the public original observer generation and replay command, including requested
scope rejection. The user confirmed these command/evidence interfaces before TDD.
