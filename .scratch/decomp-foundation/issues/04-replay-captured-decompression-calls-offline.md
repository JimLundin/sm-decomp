# 04: Replay captured decompression calls offline

**What to build:** The same offline command restores real calls captured from the C game and verifies their Rust replacements against the actual C routine.

**Blocked by:** 03: Complete synthetic RAM decompression coverage

Status: open
Triage: ready-for-agent

- [ ] Validate fixture source, ROM, destination, initial state and expected-state digests; restore fresh isolated state for each implementation and replay.
- [ ] Run every declared captured call, compare the complete output and unchanged surrounding state, and preserve the first mismatch with routine and fixture identity.
- [ ] Use original-ROM routine execution on a bounded set of captured valid calls to independently check the C algorithm; distinguish C/Rust equivalence from original-ROM evidence.
- [ ] Repeat fixtures deterministically; identify unavailable branches and ROM-data dependencies explicitly.
- [ ] Keep ROM-derived fixture material local and ignored; only synthetic examples and reproducible capture procedures belong in source control.
