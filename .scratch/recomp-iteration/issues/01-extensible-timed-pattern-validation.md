# 01: Make timed-pattern validation extensible

**What to build:** A candidate can establish complete, independently supported validation for its declared timed instruction patterns without acceptance depending on a fixed list of seven opcodes or 70,011 cases. This preparatory refactor preserves verification of the already accepted readiness baseline and keeps the existing compiled scope unchanged.

**Blocked by:** None (can start immediately).

**Status:** implemented; final integrated full-suite verification pending

- [x] Bind a versioned pattern declaration to the exact candidate source and production emitter, covering opcode/addressing behavior, applicable M/X widths, modes, timing branches, supported conditions and explicit exclusions. Reports cannot narrow the declared production scope to omit a failure.
- [x] Derive required corpus and independent-fixture coverage from that declaration and verified available inputs. Preserve corpus identities, ordered case names/digests and expected/run/passed/failed/skipped counts; require every supported case to complete and pass.
- [x] Execute the actual generated step bodies through the production fetch-and-PC path. Require independently specified fixtures for applicable conditions absent from the corpus, with explicit coverage substitutions for control/stack/interrupt behavior when needed.
- [x] Verify applicable ordered bus effects, exact timing and hardware boundary conditions against independent expectations. Explain inapplicable conditions instead of requiring unrelated cases.
- [x] Carry the declared scope through standalone pattern checks, integrated validation, terminal evidence verification, reuse decisions and acceptance. Missing modes, branches, cases or evidence, development samples and altered declarations cannot establish acceptance.
- [x] Continue to verify historical versioned readiness evidence under its original contract, including the existing seven-opcode, 70,011-case baseline. Fresh candidates use the extensible contract without rewriting completed evidence.
- [ ] Demonstrate that the existing complete enabled-pattern suite still passes and that incomplete or falsely reduced declarations are rejected through the public command/evidence seam. A scoped pattern pass still requires the remaining cold and independent gates before acceptance.
- [x] Keep unsupported instructions interpreted and known affected-pattern failures blocking support; this refactor does not enable a new production pattern.

## Completion evidence

- Source-owned version 2 declaration drives production support and dynamically declares corpus/fixture/timing scope. Existing seven opcodes remain the complete enabled set.
- Public command/evidence regressions exercise actual generation/execution, wrong independent expectations, executable mutation, reduced declarations, altered declarations, development acceptance rejection, and historical contract downgrade rejection.
- `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 python3 -m unittest discover -s tools -p 'test_readiness*.py'`: 51 tests passed, 2 optional independent original captures skipped.
- `cargo fmt --all --check` and `cargo clippy -p recomp --all-targets --locked --offline -- -D warnings` passed.
- Full 70,011-case plus independent timing execution remains scheduled on the final integrated candidate; this ticket's demonstration criterion stays open until that real suite passes. No shared accepted evidence or pointer mutated.
