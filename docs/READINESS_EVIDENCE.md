# Immutable readiness evidence

Run `just readiness-preflight` from the repository root. It allocates and prints
an `out/runs/<UTC timestamp>-<unique suffix>/` directory and its `brief.md` before
checking setup. This command verifies setup only; even a pass has
`acceptance_eligible: false`. Later readiness orchestration must complete its
mandatory replay and feedback checks before acceptance.

The versioned manifest binds maintained source/config, verified ROM/movie/P1
inputs, seed 0, clean reset, NTSC recording profile, actual patched hardware,
lockfile and compiler. Hardware verification compares build inputs against the
checksum-verified pinned archive with the maintained final patch identities.
Existing generated tables provide optional prior provenance; the generated
output actually used by a subsequent build must receive its own identity.
Binary identities are added only when an artifact exists after a successful
build. ROM-derived inputs and generated material stay under ignored `out/`.

`source/files/` preserves the exact maintained candidate, including dirty and
relevant untracked source/config. The ordered `source/inventory.json` records
path, SHA-256 and mode. Downloaded trees, ROMs, generated/build/run output and
credential files are excluded. `Run.restore_source(destination)` validates the
snapshot and restores into a new directory; an existing destination is rejected
to preserve user edits. Downloaded material is identified separately and must
be supplied to a restored checkout before building.

Every stage records its argument array, working directory, separate stdout and
stderr, underlying return code/signal and elapsed time. Source and guarded input
changes invalidate checks. Orchestrators should build from isolated run-local
generated inputs and use `Run.shared_mutation()` around any shared generation or
build operation. It holds the checkout's advisory build lock and verifies source
and input identity before and after the operation; external tools that ignore
the lock remain subject to mutation checks.

A run without a valid terminal `verdict.json` is incomplete. Caught errors,
interruptions and timeouts preserve terminal evidence; an uncatchable kill
leaves a running manifest and incomplete initial brief. `read_verdict()` rejects
missing, malformed or mismatched verdicts. Completed evidence cannot be
finalized again. Exit codes are 0 for a scoped pass, 1 for divergence/uncovered,
and 2 for infrastructure/incomplete failure. A recorded failed or incomplete
check cannot be omitted from a requested terminal pass.

The shared API is `Run.create()`, `capture_source()`, `guard_input()`,
`check_sources()`, `run_stage()`, `record_check()`, `record_artifact()`,
`shared_mutation()`, `restore_source()` and `finalize()`. `preflight(run)` performs
setup checks without finalizing, leaving later orchestration to define its full
mandatory-check set. Evidence consumers must verify the declared checks and
scope as well as the terminal result before changing accepted state.

`run_stage(..., exit_statuses={1: "divergence", 2: "error"})` may classify
known tool exit codes before recording immutable stage reports. Signals and
timeouts remain incomplete, and source/input mutation remains error regardless
of that map. Other callers retain the default nonzero infrastructure status.

Run `just readiness-parity fallback-only` and `just readiness-parity generated`
for separate 18,000-frame cold parity runs. `tools/readiness_parity.py` exposes
`run_parity(run, mode, frames=18000, timeout=None)` for the full suite. Each mode
uses isolated generated inputs and a target directory, preserves generator and
candidate binaries, and verifies the actual replay report against all input and
binary identities. The fresh generated table binds the ROM and must be nonempty
and execute. Untimed whole-subroutine output is preserved separately and does
not supply cycle evidence. Reports include first frame/region/byte disagreement,
decoded CPU/clocks, aggregate execution totals and stage timings. The explicit
`--timeout` is a hard limit; a completed replay over the 20-minute target reports
an overrun and keeps its actual outcome. Scoped parity never updates acceptance.
Direct `diff` commands require a fresh output directory and write `parity.json`
before input validation; incomplete/old/absent reports cannot certify a pass.
