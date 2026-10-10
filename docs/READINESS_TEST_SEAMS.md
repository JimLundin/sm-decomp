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
