# 05: Prove and freeze offline translation acceptance

**What to build:** A correct exercised decompression job can be accepted, while wrong behavior, hidden fallback, missing coverage or unreliable evidence fails.

**Blocked by:** 04: Replay captured decompression calls offline

Status: resolved
Triage: ready-for-agent

- [x] Report complete write sets and ordered writes where the contract requires them; compare output, surrounding state and relevant transient writes.
- [x] Execute intentional wrong-output and transient-write candidates and prove the real comparison command rejects them.
- [x] Execute disabled or unexecuted replacements and missing/reduced fixture or branch declarations; each must fail coverage or remain explicitly nonaccepting.
- [x] Prove build errors, crashes, timeouts and altered identities preserve failing evidence and cannot reuse a successful verdict.
- [x] Freeze a minimal versioned offline contract with exact job/source/fixture identity, predecessor lineage and accepted exercised coverage, separate from recomp acceptance.

## Completion evidence

Implemented the complete ordered-store comparison using the unchanged pinned C bank
as final-state oracle and a separately digested six-store instrumented bank. The
safe Rust WRAM observer records all stores, including write-and-restore changes.
Frozen acceptance binds the canonical job, 40 synthetic cases, exact 23-call
capture manifest, three bounded original-ROM calls, source and executable digests,
observed coverage, negative verdicts and predecessor lineage. All 11 real fault
controls reject with their intended runtime/build/admission cause.

Validation: `python3 -m unittest tools.test_decomp_routine` (9 tests),
`cargo test -p decomp --locked --offline` (6 integration tests), and diff whitespace
check. Retained accepting evidence:
`out/decomp/ticket05/20261010T183439.571835Z/verdict.json`,
`out/decomp/ticket05/20261010T183439.571835Z/acceptance.json`, and separate pointer
`out/decomp/ticket05/decomp-acceptance.json`. Public `verify_accepted` successfully
verified all 63 fixture identities, 38 observed cases and 11 negative controls.

Reproduce with `python3 tools/decomp_routine.py --accept --captures CAPTURES
--original-rom ROM --output OUTPUT`. Synthetic-only runs remain partial. Historical
acceptance survives a failed later attempt; its own latest verdict remains failing.
Live Rust ownership and whole-game ROM parity are explicitly unproven.
