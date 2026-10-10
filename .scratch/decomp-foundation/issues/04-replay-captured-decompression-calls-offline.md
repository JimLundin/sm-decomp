# 04: Replay captured decompression calls offline

**What to build:** The same offline command restores real calls captured from the C game and verifies their Rust replacements against the actual C routine.

**Blocked by:** 03: Complete synthetic RAM decompression coverage

Status: resolved
Triage: ready-for-agent

- [x] Validate fixture source, ROM, destination, initial state and expected-state digests; restore fresh isolated state for each implementation and replay.
- [x] Run every declared captured call, compare the complete output and unchanged surrounding state, and preserve the first mismatch with routine and fixture identity.
- [x] Use original-ROM routine execution on a bounded set of captured valid calls to independently check the C algorithm; distinguish C/Rust equivalence from original-ROM evidence.
- [x] Repeat fixtures deterministically; identify unavailable branches and ROM-data dependencies explicitly.
- [x] Keep ROM-derived fixture material local and ignored; only synthetic examples and reproducible capture procedures belong in source control.

Implementation: `9323f87` on `decomp-foundation-ticket-04` (based on integration
`e0cdfb7`). Public-command TDD covered retained capture identity errors and real
replay before implementation, then green with seven focused tests. Local
validation: `/home/jim/sm-decomp/out/decomp/ticket04/20261010T182850.050044Z/verdict.json` checked all
23 calls twice, all 23 source streams against unmodified ROM, and three bounded
original instruction executions. Verdict remains partial/nonaccepting.
