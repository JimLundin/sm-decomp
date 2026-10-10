# Workspace notes

## User's intended loop

- The user wants this project ready to start the recompilation loop and wants
  sufficient feedback to sustain that work.
- `loop-me` is being used to specify workflows. This session produces specs;
  implementation follows agreement on the complete design.
- Agreed: specify both readiness to start and recurring recompilation iteration.
- Agreed: an agent owns routine selection, changes, validation, and diagnosis.
  The user receives a short brief when a design decision or review is needed.
- Agreed: useful feedback establishes that changed compiled code executed,
  checks correctness, measures recompilation progress, and explains a first
  failure well enough to drive the next change.
- Agreed Q4–Q9: readiness may cover a fresh 18,000-frame any% domain while
  recording later hardware synchronization failures as debt. Expand one
  routine/width slice or one timed instruction pattern per increment. Require
  automatic first-observable-disagreement diagnosis, controls, faulty-code and
  uncovered-code feedback checks, identity-bound immutable evidence, and a
  preserved accepted state. Target focused replay within 60 seconds and cold
  acceptance within 20 minutes. Explicit requests start autonomous sessions;
  the user reviews late when needed. Commit/publish/merge needs separate
  authorization.
- Agreed Q10–Q15: prioritize measured hot interpreted work, first with supported
  patterns; newly accepted address/width keys must execute. Readiness requires
  fresh fallback/generated and strict original-core evidence through 18,000;
  increments require fresh generated cold acceptance and relevant regressions.
  CPU-only changes may reuse verified unchanged independent evidence. Focused
  diagnostics compare transient instruction/bus effects with cold-validated
  paired caches and predecessor/disabled-entry controls. New patterns require
  the complete affected-opcode corpus and applicable independent timing cases.
  Use repository commands and JSON/Markdown evidence with short linked briefs.
  Unspecified iteration requests default to one accepted increment; three
  consecutive experiments without new evidence trigger a decision brief.

## Canonical terms

- **Recompilation increment:** a bounded change that moves CPU execution into
  generated Rust, or makes such execution correct. Expand one routine/width
  slice with supported instructions, or one timed instruction pattern.
- **Mixed execution:** generated instructions and interpreter fallback using the
  same emulated SNES hardware.
- **Parity:** mixed execution agrees with the interpreter on the same backend.
  This does not independently establish SNES hardware accuracy.
- **Original-core synchronization:** strict comparison with the movie's recording
  emulator, lsnes rr2-beta23 / bsnes v085 Compatibility, at the actual opcode
  fetch of `RunOneFrameOfGame` (`$82:8948`). Observations contain frame/event
  order and ten initialized game-state words, not complete emulator state.
- **Validated domain:** a particular ROM, movie, initialization/profile, and
  frame interval supported by current evidence. Evidence does not generalize
  silently to other movies, later gameplay, or another hardware version.
- **Changed-entry coverage:** execution evidence for the addresses and M/X width
  variants affected by an increment. A global generated count is insufficient.
- **Cold acceptance:** replay from clean initialization. Cached replays can speed
  diagnosis but need separate validation before they support acceptance.

## Repository facts observed 2026-10-10

- Rust workspace statically recompiles Super Metroid's 65816 CPU and emulates
  PPU, audio, DMA and other hardware. Commands are in `justfile`.
- `just iterate` regenerates/builds and runs a default 9,000-frame any% prefix.
  It currently compares frame-end CPU/clocks, complete WRAM, and framebuffer.
- `config/functions.toml` currently selects `WaitForNMI` in all four M/X widths.
  The compiled whitelist is immediate LDA/LDX, REP/SEP, absolute STA, NOP and DEX.
- Mixed reports currently provide aggregate generated/interpreted counts and
  frame/region/byte differences. They lack changed-entry coverage and automatic
  first-instruction diagnosis.
- A local 19,000-frame parity artifact reports 48,345 generated and 272,286,496
  interpreted steps, six entries, and 819.72 seconds. It predates later hardware
  changes and cannot certify the current dirty source tree.
- The full original any% capture covers 129,712 frames and is hash-bound.
  Latest saved candidate evidence passes 15,184 strict visits through 18,000;
  first strict mismatch is event 15,292 at original frame 18,195 versus 18,196.
  A later first semantic mismatch occurs at frame 35,203 (X=44 versus 43).
- CI runs ROM-free Rust/Python/hardware checks. It does not replay local TAS
  movies. The external fuzz corpus is inactive in ordinary checks; a retained
  one-case-per-opcode sample has four ADC overflow failures among 226 cases.
- Shared-backend parity, independent recording-core checks, instruction
  semantics tests, and hardware timing regressions answer different questions.
- There are many pre-existing uncommitted implementation changes. This spec
  session must preserve them; an implementation must capture their exact state
  when establishing accepted evidence.

## Specification state

Q1–Q15 are settled. `workflows/recomp-readiness.md` and
`workflows/recomp-iteration.md` contain the complete proposed operational
contracts. Final confirmation of shared understanding is pending under the
invoked grilling discipline. The session only changed workflow specs and
workspace notes; feedback implementation and fresh readiness validation have
not been performed.
