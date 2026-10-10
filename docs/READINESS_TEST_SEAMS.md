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

Ticket 02 tests public `diff` invocation and its structured parity evidence, then
public `readiness-parity` invocation and immutable run evidence. Synthetic ROMs
exercise exact requested scope and fallback execution without commercial inputs;
real any% prefixes exercise generated dispatch and identity-bound build artifacts.
These tests establish shared-backend parity/report honesty, not independent
hardware synchronization or full readiness.
