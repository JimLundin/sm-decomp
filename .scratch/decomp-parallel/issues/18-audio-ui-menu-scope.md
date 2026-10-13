# Research complete audio acknowledgment and UI/menu boundaries

Status: open
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumers:07,08
Read-only work; future implementation waits required13APU/reset and player inputs

## Work

Read [later lanes](../catalog/later-lanes.md). Reuse resolved02system/03audio
bounded WRAM/queue routines. Research HandleMusicQueue and whole sound-effect
acknowledgment state machine first, then separate pause/file/options menu roots.
Pinned C modifies acknowledgment behavior; declare actual source choice and
original-ROM effect instead of treating faithful C parity as native semantics.
No complete Rust UI/menu or audio-frame module/receipt was found in preparation.

Exact candidate files: `crates/port/src/audio.rs`, `crates/port/src/system.rs`,
`crates/port/tests/audio.rs`, `crates/port/tests/system.rs`; proposed separate
`crates/port/src/audio_frame.rs`, `crates/port/tests/audio_frame.rs`,
`tools/port/audio-frame.toml`. UI exact new files are chosen only after root/
closure research, not a blanket module claim. Integrator owns state/SRAM/lib.rs,
APU/hardware callbacks and shared catalog/ABI registration.

## Acceptance / Answer

Record each coherent boundary's source/observed-root/dependency/input hashes,
current bounded stages and missing host interface. Current bank86-only native
exercise is a blocker; generate a reviewed Rust provenance extension task and
exact separate1–32 routine implementation children when facts exist. Require
real ordered audio port reads/writes/queue/ack states and controls, UI inventory/
menu resource state, actual-C parity plus original-ROM boundary comparisons,
independent material live ownership and visible fallback. Concrete60s initial/
3600s merged argv and required13dependencies; add resulting tasks to07/08.

## Completion evidence

Pending Answer/task/scope/hash pointers, scoped cleanup/authentication or
no-run result with bytes/free space and retained dependencies.
