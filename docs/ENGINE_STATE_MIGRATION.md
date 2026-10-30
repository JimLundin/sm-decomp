# Moving gameplay state out of RAM

Read this when changing state ownership, reset/load, replay or CPU/oracle boundaries.
[PLAN](PLAN.md) and [REVIEW](REVIEW.md) own current work and verification;
[the target architecture](target-architecture/README.md) owns the destination.
This consolidates the earlier ownership, engine-design and movie-state audits.
Their experiment narratives remain in Git history, not the current reading chain.
The invariants below are migration constraints, not claims that ownership is complete.

## One authority per execution interval

RAM remains authoritative until a complete producer/consumer interaction migrates.
Decoding a typed value on each operation and writing it back is a compatibility
adapter. Moving bank files or sharing algorithms does not establish persistent
Session ownership. Never keep an always-live typed copy beside independently
writable bytes for the same state.

During a typed-owned interval, all participating transitions use that owner.
Export before legacy, CPU or native callbacks can read or write its fields;
import the **actual selected image** before resuming typed execution. Revoke or
rebuild the model after external mutation, including partially failed loads.
The byte image is an encoding during typed ownership, not a competing authority.

Close the complete interaction: creation, setup, update, callbacks, suspension,
collision, deletion/reuse, reset and persistence. Identify callers and consumers
before removing their RAM communication. A bridge must disclose remaining writers
and the condition that permits its removal.

## Memory and identity constraints

- Game and the retained Backend share one stable WorkRam allocation. Gameplay,
  CPU/physical access and snapshot capture/adoption use that allocation; snapshots
  are separate byte copies. Keep its SNES byte layout compatible while consumers
  depend on it. WorkRam copies Cell<u8> values without exposing live u8 references
  or a raw RAM base. Preserve this value-copy model and audit remaining raw aliases
  before introducing references or slices.
- Indexed/native operations can reach another nominal field or array. Preserve
  full-u16 and odd identities, wrapping arithmetic, unknown values, native versus
  little-endian words, allocation provenance and off-table alias behavior.
- Captured identities and live selectors differ. A callback may change the current
  projectile, block or actor while an in-flight operation retains its original
  identity. Reacquire only where the original does; account for slot reuse and
  generation when introducing handles.
- Preserve lazy/fallible read order and the exact effects before failure. An eager
  predecode, broader validation or sorted/batched traversal can change behavior.
- Keep entry contracts unsafe while reachable unchecked backend or ROM operations
  remain. Checked RAM, passing replays and a safe-looking wrapper do not prove
  transitive safety. See the archived [M1 audit](archive/M1A-REVIEW.md).

## Reset, import and external writers

Audit these paths for every promoted state cohort; this is a checklist of known
boundaries, not proof that every generic native access is disjoint:

| Boundary | Required treatment |
| --- | --- |
| `Game::new`, ROM hard reset, `RtlReset`, Rust workspace clears, Samus initialization | Establish the model from the resulting initialized image. |
| Every `RestoreSnapshot`, retries and partial failures | Discard stale typed values and respect the actual restore/failure prefix. |
| Frontend replay base/final snapshots, SRAM-only/reset replay, checkpoint machine load | Reconstruct at the correct phase; these formats restore different state. |
| Replay patches, `RunAsmCode`, CPU/DMA/WRAM-port writes | Export before access and import after external writes, including aliased fields. |
| Scanline, HDMA and IRQ activity after gameplay; native off-table accesses | Do not assume the Rust gameplay return is the final writer of the frame. |

Follow the current code in `game_state.rs`, `cpu_infra.rs`, `rtl.rs`,
`snapshot_io.rs`, `verify/checkpoint.rs`, `ram_map.rs` and
`game/session_runtime.rs` under `sm/src`, and in `session.rs`,
`persistent_session.rs` and `session_encoding.rs` under `crates/sm_game/src`.
Audit current callers; historical bank filenames are not an ownership map.

## Oracle rewind and adoption

A paired frame saves the before image, runs CPU/theirs, restores before, runs
Rust/mine, normalizes comparison exceptions, compares and selects/restores the
completed state. Hardware, audio and native-player selection is owner-specific;
do not universally adopt theirs. Import the selected **normalized** state, not
an earlier CPU model or the Rust value retained across rewind.

The continuation region `$0611..$0616` (including `$0613`) has normalized fields;
those fields are not independently validated by byte comparison. Inspect the
current masks and `PatchBugs` when touching them. A passing comparison validates
only its observed fields, reached paths and ordered effects.

## Checkpoints, saves and movie observations

A save-game restore, full verifier checkpoint, comparison snapshot and movie
input log are different objects. The `.smvchk` codec explicitly serializes state
and compatibility metadata; adding a Rust field does not add it to a checkpoint.
When state leaves RAM, extend the codec and roundtrip checks deliberately. Keep
ROM, input, source and native identity guards and partial-load failure semantics.

A semantic observation projection is not automatically a resumable checkpoint.
Resume needs hidden timers, fractional positions, interpreter cursors, suspended
continuations, input phase and hardware latches, as applicable. Do not invent
unknown emulator fields or default them to claim a complete state conversion.
Preserve capture provenance, observed PCs and the exact capture phase.

Movie input words contain inputs, not game state. Derive state by executing and
capturing the oracle. Alignment includes lag, subframes, controller selection and
input-poll boundaries. Use the current capture/conversion recipes in
[movies/README](../movies/README.md); old importer limitations and experiment
results are not current validation. Self-replay proves determinism, while fidelity
needs independent original-game observations for the compared behavior.

## Scheduling and domain boundaries

Session execution is resumable and order-sensitive. Preserve the actual phase
order and message suspension; resuming must not replay earlier preparation,
awards or effects. Effects consumed during the current update cannot be postponed
to the end of the frame.

Mode, captured in-flight dispatch, caller phase, PLM pass and active dialogue/child
cursors have different lifetimes. Changing mode does not automatically replace
suspended dispatch; clearing an activation does not necessarily cancel a running
pass. A live message request differs from an active message; completion clears
the live request. Shared continuation bytes have context-specific meanings, such
as startup, Kraid and PLM execution, rather than one universal state enum.

Keep inventory and ammunition under the Samus owner, world facts under progress,
and saves as a selection from owners. Player and hostile projectiles have distinct
lifecycles and script schedulers; share only complete matching contracts. Preserve
first-hit traversal and ordered collision outcomes. A synchronous shot-to-room
callback must publish/import around mechanism allocation/setup before flight
resumes, including mutations to selectors and aliased state.

The completed core exposes owned domain state and explicit effects, independent
of `Game`, RAM and backend addresses. Content, room/door topology and opt-in
mechanics remain explicit inputs as required by the Map Rando target. Existing
compatibility algorithms are migration steps; coverage and current completion
claims belong in PLAN and the named segment results.
