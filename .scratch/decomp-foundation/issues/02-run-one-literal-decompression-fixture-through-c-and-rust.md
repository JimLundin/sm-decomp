# 02: Run one literal decompression fixture through C and Rust

**What to build:** One public offline job command compiles the actual pinned C routine and a readable Rust replacement, executes a synthetic literal stream from identical initial state and compares declared outputs.

**Blocked by:** 01: Retain the executable C baseline

Status: resolved
Triage: ready-for-agent

- [x] Declare the routine address, source revision and notices, dependencies, memory reads/writes, ROM mapping, numeric behavior, fixture scope and exclusions in a small versioned job manifest.
- [x] Use bounded byte-addressed WRAM views, explicit little-endian access and validated LoROM reads, preserving aliasing and source rollover semantics without broad state restructuring.
- [x] Compile the selected upstream C routine with the minimal necessary shims instead of retyping a second reference algorithm.
- [x] Execute literal and terminator cases with independently specified expected output and unchanged surrounding memory; retain C/Rust results and machine-readable failures.
- [x] Expose a deterministic prepare/build/compare/verdict command; incomplete algorithm coverage is labeled partial and cannot accept the whole routine.

## Completion evidence

Implemented on `decomp-foundation-ticket-02`, based on integration commit
`2e35773`. Public job: `just decomp-routine`; API/CLI and limitations documented
in `docs/DECOMP_ROUTINE.md`. Literal and FF fixtures match independent expected
whole-WRAM states in actual pinned C and Rust, with only partial status and
whole-routine acceptance disabled. Unsafe C fixture admission is bounded to
literal streams; unsupported Rust commands return explicit errors.

Verification: public-job test failed before implementation, then passed;
invalid-destination fixture control failed before admission validation, then
passed. `python3 -m unittest tools.test_decomp_routine` (2 tests),
`cargo test -p decomp --locked --offline` (2 public memory/error tests), and
`git diff --check` pass. Fresh retained command evidence:
`/tmp/sm-decomp-foundation-context/ticket02-verdict.json` (points to immutable
run directory with binaries, inputs, outputs, commands, identities and notices).
Implementation commit is the commit adding this completion record.

Follow-up verification corrected a relative script-path identity error discovered
by the merger: the runner now resolves its own source path. The public-job tests
invoke `python3 tools/decomp_routine.py` from repository root. Both that exact
relative invocation and `just decomp-routine` were run and their retained JSON
verdicts explicitly checked for `partial`, all four completed stages and disabled
routine acceptance. Evidence pointers:
`/tmp/sm-decomp-foundation-context/ticket02-verdict.json` and
`/tmp/sm-decomp-foundation-context/ticket02-just-verdict.json`. The earlier error
run remains retained; it was not accepted or reused.
