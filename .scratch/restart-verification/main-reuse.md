# Selective reuse from main for the projectile restart

Status: resolved
Scope: source inspection only, 2026-10-16. Main HEAD:
`193cad941713b3539d23e08238a79e7fb09dde61`.

Main contains useful gameplay translations and counterexamples. Reuse those
pieces with fresh comparison against the restart's actual compiled C. Its old
acceptance decisions, dependency graph and evidence publication process should
not become prerequisites for the restart pilot. The following **facts** are
visible in source; **reuse** entries are recommendations, not new verification.

## Seven useful pieces

1. **Projectile initialization, animation and removal are separate stages.**
   `InitializeProjectile` sets damage, instruction pointer, radii and timer 1;
   `RunProjectileInstructions` then executes goto/delete or consumes an eight-byte
   animation record. `ClearProjectile` resets fifteen fields, installs the empty
   preinstruction and decrements the appropriate projectile/bomb counter with the
   native signed-underflow guard. Rust implementations and small counterexamples
   are already readable. **Reuse:** adapt `initialize`, `run_instructions`,
   `clear` and the ordinary-beam path of `kill`; correlate those stages for the
   same slot in the pilot. An impact changes the projectile into an explosion;
   it need not clear the slot immediately.
   Sources: [C initialization/scripts](../../deps/snesrev-sm/src/sm_93.c),
   `InitializeProjectile`, `RunProjectileInstructions`, `Proj93Instr_Delete`;
   [C cleanup](../../deps/snesrev-sm/src/sm_90.c), `ClearProjectile`, `KillProjectile`;
   [Rust](../../crates/port/src/projectiles.rs), `initialize`, `clear`, `kill`,
   `run_instructions`; [counterexamples](../../crates/port/tests/projectiles.rs),
   `initialization_and_instruction_data_are_distinct_stages`,
   `goto_then_delete_clears_bomb_and_preserves_other_slot`.

2. **The scheduler's order and global index matter.** C `HandleProjectile` visits
   slots 18 down to 0, invokes preinstruction before animation, then reloads
   `projectile_index`. The Rust `update` follows that order; its ten-slot test
   exercises deletion and animation in one pass. **Reuse:** preserve this small
   scheduler contract when selecting the admitted projectile root. Do not infer
   lifecycle execution merely from an empty scheduler invocation.
   Sources: [C](../../deps/snesrev-sm/src/sm_90.c), `HandleProjectile`;
   [Rust](../../crates/port/src/samus_projectile_frame.rs), `update`;
   [test](../../crates/port/tests/samus_projectile_frame.rs),
   `ten_slot_scheduler_executes_delete_goto_and_data_in_reverse_order`.

3. **Fractional motion and initial firing have cheap, valuable counterexamples.**
   `move_axis` combines position/subposition into a wrapping 32-bit value;
   `collide` sign-extends speed and multiplies by 256. The test with speed
   `0xff80` moves X=100 to X=99, fraction `0x8000`. `initial_collision` also
   installs speed `0xffff` for direction 7 before regular initial speed is set;
   the left-shot test expects X=88 and speed `0xfc00`. **Reuse:** these focused
   regression inputs when adapting normal-shot motion, instead of importing
   every beam/missile/terrain fixture family. Their current tests are synthetic
   Rust assertions; they are not evidence that a restart candidate matches C.
   Sources: [implementation](../../crates/port/src/projectile_flight.rs),
   `move_axis`, `collide`, `initial_collision`, `fire_beam`;
   [tests](../../crates/port/tests/projectile_flight.rs),
   `signed_fixed_point_flight_preserves_fraction`,
   `firing_uses_actual_initial_left_direction_minus_one_speed_quirk`.

4. **Drawing can be intermittent while the shot remains alive.** Ordinary beam
   visibility depends on slot bit 1 and NMI frame parity; plasma uses a different
   parity rule. Both C and Rust separate that decision from the active instruction
   pointer. **Reuse:** the ordinary-beam part of `flicker_visible` and the normal
   camera-relative drawing path, plus the existing parity counterexample.
   Observe actual drawing during the lifecycle rather than requiring a sprite
   every frame. Defer Mode7, echoes and deliberately aliased OAM fixtures unless
   the admitted scenario needs them.
   Sources: [C](../../deps/snesrev-sm/src/sm_93.c), `DrawPlayerExplosions2`;
   [Rust](../../crates/port/src/samus_projectile_frame.rs), `flicker_visible`,
   `player`, `draw_spritemap`; [test](../../crates/port/tests/samus_projectile_frame.rs),
   `plain_and_plasma_beams_follow_the_two_native_flicker_patterns`.

5. **Ownership counters prevent an unchanged C run from masquerading as progress.**
   Main checks `total == c + rust`, active counts bounded by total execution and
   zero Rust calls in a C reference; `frame_admission` requires material execution.
   **Reuse:** these invariants in a small pilot-specific execution record, alongside
   its actual spawn/move/impact/draw/remove observations. Keep foreign calls
   explicit. Counter equality alone does not prove behavior or connect events to
   one projectile slot.
   Sources: [live implementation](../../crates/pipeline/src/live.rs), `counters`,
   `frame_admission`, `counter_exclusivity_rejects_double_execution_and_unexecuted_predecessors`.

6. **Observation identity can be separated from producer provenance.**
   `canonical::identity` hashes a domain tag, format version, object kind and
   payload. `put_run` stores an observation root separately from provenance, so
   the observed bytes can retain identity when producer metadata changes.
   `Reader` rejects truncation, overflowing lengths, noncanonical varints and
   trailing bytes. **Reuse:** these small design ideas in the reference format;
   bind C source/ROM/host/input/scenario/observation schema to reference identity
   and record candidate Rust identity separately. Main's optional `Provenance`
   fields are not a ready-made required reference-cache key.
   Sources: [identity/reader](../../crates/evidence/src/canonical.rs), `identity`,
   `Reader`; [run identity](../../crates/evidence/src/run.rs), `put_run`;
   [metadata](../../crates/evidence/src/metadata.rs), `Provenance`, `validate`.

7. **Compact binary observations and localized differences are reusable ideas.**
   `trace::Event` represents ordered PPU/APU writes as typed values.
   `diff_images` descends only unequal image subtrees and identifies differing
   256-byte pages; its own contract requires authenticating external images
   before trusting equal roots. **Reuse:** emit typed effects directly, compare
   canonical frame bytes and report the first frame/field/effect difference.
   For this short pilot, simple frame records may be cheaper than importing the
   tree/store implementation. The image helper is a storage diff, not an existing
   fail-first live comparator. Also, `TraceWriter::push` still renders legacy text
   to update its legacy byte seal, so copying it wholesale retains that cost.
   Sources: [events/writer](../../crates/evidence/src/trace.rs), `Event`,
   `TraceWriter::push`; [image diff](../../crates/evidence/src/image.rs), `diff_images`.

## Work amplification to leave behind

- **Four complete captures before comparison.** `live.rs::SUBJECTS` fixes
  `c-0`, `c-1`, `mixed-0`, `mixed-1`; `verify` captures all four before comparing
  artifact inventories. It also authenticates decompressor, exhaustive RNG and
  family predecessors. These are concrete dependencies, not an estimate of their
  runtime. For the restart warm loop, use one immutable, correctly keyed C
  reference and one candidate run, with an explicit host/scenario determinism
  check when those inputs change. Keep old predecessor publication outside this
  loop. Source: [live.rs](../../crates/pipeline/src/live.rs), `SUBJECTS`, `verify`.
- **Whole-catalog conservative change selection.** `affected::shared` treats
  all `crates/pipeline` and much tooling as shared inputs; `select_changed` adds
  that reason to every considered family. `HOST_SOURCES` includes cleanup,
  documents, batch and a Ridley fixture generator in the Offline contract.
  For the pilot use explicit behavior-affecting inputs and separate report/docs
  edits from gameplay identity; do not recreate generalized catalog discovery
  before there is a second consumer.
  Sources: [affected.rs](../../crates/pipeline/src/affected.rs), `shared`,
  `select_changed`; [source_stamps.rs](../../crates/pipeline/src/source_stamps.rs),
  `HOST_SOURCES`.
- **Per-call full-WRAM speculative copies as a default integration architecture.**
  The old frame module copies all WRAM in `handle` before publishing success;
  `supports_call` separately simulates against a copy for selection. This protects
  its fallback contract but is extra work to assess before transplanting it.
  Keep narrow pre-call eligibility and explicit failures; if a Rust call already
  performed an external effect, never retry that same call in C. Source:
  [samus_projectile_frame.rs](../../crates/port/src/samus_projectile_frame.rs),
  `copy`, `handle`, `supports_call` and the `handle` host-failure contract.

## Completion evidence

Read actual main Rust, tests and local recovered C; the latter's declared pin is
`578f90b3cc49557bb70060ad033bb90b8cf8ac50` in
[tools/dependencies.toml](../../tools/dependencies.toml). No builds, tests, captures,
source authentication runs, or acceptance updates were performed. No original-ROM
feasibility claim is made. No verification artifacts/worktrees were created;
cleanup reclaimed 0 bytes, with 107,790,618,624 bytes available at inspection.
No retained evidence dependencies were added. One overly broad read-only path
search was interrupted; its process exited 130 before handoff.
