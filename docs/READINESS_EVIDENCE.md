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
relevant untracked source/config. The ordered `source/inventory.toml` records
path, SHA-256 and mode. Downloaded trees, ROMs, generated/build/run output and
credential files are excluded. `Run::restore(owner, destination)` validates the
snapshot and restores into a new directory; an existing destination is rejected
to preserve user edits. Downloaded material is identified separately and must
be supplied to a restored checkout before building.

Every stage records its argument array, working directory, separate stdout and
stderr, underlying return code/signal and elapsed time. Source and guarded input
changes invalidate checks. Orchestrators build from isolated run-local generated
inputs and target directories. `Run::stage` acquires a shared worker lease and
verifies source and guarded input identities before and after execution.
`Run::stage_controller` lets a controller wait for leased child stages without
holding a worker itself. External tools remain subject to mutation checks.

A run without a valid terminal `verdict.toml` is incomplete. Caught errors,
interruptions and timeouts preserve terminal evidence; an uncatchable kill
leaves a running manifest and incomplete initial brief. Evidence consumers must
validate the native manifest, source inventory and terminal verdict together;
accepted-evidence verification rejects missing or mismatched records. Completed
evidence cannot be finalized again. Exit codes are 0 for a scoped pass, 1 for divergence/uncovered,
and 2 for infrastructure/incomplete failure. A recorded failed or incomplete
check cannot be omitted from a requested terminal pass.

The Rust implementation provides `Run::create`, `capture_source`, `guard`,
`check_sources`, `stage`, `stage_controller`, `record_check`, `record_typed_check`,
`record_artifact`, `restore` and `finalize`. Preflight performs setup checks
without finalizing, leaving later orchestration to define its full mandatory-check
set. Evidence consumers must verify the declared checks and scope as well as the
terminal result before changing accepted state. Native manifest, source inventory,
stage reports and verdict metadata use versioned TOML.

The `exit_statuses` argument to `Run::stage` maps known tool exit codes to statuses
such as `divergence` and `error` before recording immutable stage reports. Signals
and timeouts remain incomplete, and source/input mutation remains error regardless
of that map. Other callers retain the default nonzero infrastructure status.

Run `just readiness-parity fallback-only` and `just readiness-parity generated`
for separate 18,000-frame cold parity runs. `sm-pipeline readiness-parity` accepts
`--mode`, `--frames` and `--timeout`; frames default to 18,000 and the hard timeout
defaults to 2,400 seconds. Each mode uses isolated generated inputs and a target
directory, preserves generator and
candidate binaries, and verifies the actual replay report against all input and
binary identities. The fresh generated table binds the ROM and must be nonempty
and execute. Untimed whole-subroutine output is preserved separately and does
not supply cycle evidence. Reports include first frame/region/byte disagreement,
decoded CPU/clocks, aggregate execution totals and stage timings. The explicit
`--timeout` is a hard limit; a completed replay over the 20-minute target reports
an overrun and keeps its actual outcome. Scoped parity never updates acceptance.
Direct `diff` commands require a fresh output directory and write `parity.toml`
before input validation; incomplete/old/absent reports cannot certify a pass.

Generated parity now writes `coverage.toml`: actual instruction counts by
address/MX, opcode/pattern and native/emulation mode; generated table executed/
unexecuted inventory; supported deferred and unsupported interpreted work;
aggregate instruction totals and fallback fraction. Hardware interrupt entry and
idle wait/stop dispatches are excluded from instruction totals. Call-target
`misses.csv` remains discovery evidence and is never a coverage rate.

`crates/pipeline/src/coverage.rs` preserves exact generated bodies and bytes, binding
source/config/table digests. `sm-pipeline readiness-parity --predecessor <run directory>`
requires a completed passing predecessor with intact source and generated-table
artifacts, then requires every added or changed body key to execute. Shared
execution-source changes conservatively require all current keys. Initial
readiness retains legacy unexecuted keys as unvalidated inventory; those keys do
not count as validated progress. Empty compilation and zero compiled execution
remain uncovered. Progress deltas require identical ROM/movie/input/profile/
reset/seed/controller and cold interval. Full integrated readiness still needs
all independent gates beyond these shared-backend coverage observations.
