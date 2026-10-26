# Shared visual detached composition

The maintained harness composes player, room, enemy and visual checks.
Current work and validation follow [PLAN](../../../../docs/PLAN.md).

World.player is the only Session. Current/target colors, palette effects, animation
slots, Samus palette fields and statue animation now live in that Session. Removed
old World.animation, World.palette and World.target_colors allocations. The same
SessionAccess implements all gameplay Hosts. No Game or second RAM-backed owner is
constructed. Palette alias writes use the actual session_encoding projected-byte
implementation, with whole-word preflight. Tests therefore reach real shared
mechanism timers through the maintained codec rather than a fabricated timer map.

The harness includes actual animation/palette execution, Samus palette, player
charge feedback, enemy initialization and room scheduler files. Composition methods
call those cores on the same Composed value. Content adapters implement the frozen
Record interface on the maintained RomRecord. Prepared loaded/native sources share
one synthetic cartridge allocation here; source-rebinding/low-window/native-runtime
coverage belongs to the separate native/defined carriers. Unprepared actor/backend
operations panic explicitly. Upload records a request and applies the prepared
synchronous callback; it does not emulate DMA registers or claim native DMA coverage.

Selected cases and labels:

- eye: actual animation command creates actual EyeGlow enemy, whose four colors
  immediately reach shared palette; animation then spawns actual palette effect
  and its interpreter replaces just one of those shared colors.
- alias: actual palette script writes displacement1E1C; actual aggregate projection
  updates mechanism0 timer, then actual room scheduler executes the awakened script.
- heat: fractionF001/amount7 becomes3001/8 before sound; sound changes equipment
  and heat, and actual palette pre selects gravity's new program and consumes it.
- upload_ack: callback redirects selected, reuses captured source, disables
  animation and queues a later slot; captured slot is acknowledged and the later
  zero-size request still executes, while inactive slot0's source stays intact.
- upload_failure: intentional callback panic is caught; source remains pending
  and callback inventory mutation remains visible.
- samus: actual inventory-driven suit index/current/target copy, real player charge
  decisions into Samus output, then real uncharged feedback into visor timer/color.
- samus_carry: selector2 hurt803B reaches803D through two actual misc calls. Real
  source-capture observation proves no intervening default9300 copy despite the
  negative signed subtraction; hurt8 independently verifies two increments to10.

The old animation color/target host notifications were internal adapter events,
not retained visual effects. Their state/order checks now assert shared values at
the real music effect and afterward. Canonical palette color_written notifications
remain in the event trace; exceptional aliases must not synthesize them. Existing
snapshot/replay comparisons now include the full shared visual Session.

Mutation-author contract: `--visual-mutation <case>` runs exactly one selected case
and prints `visual composition case passed: <case>`. Assertions carry exact
`visual.composition.*` labels in visual_checks.rs. upload_failure emits the expected
caught panic `intentional upload failure` from visual_host.rs before success; other
selected cases must not panic. The full run retains its existing success text.

Legacy entry preservation: palette_independent_checks.rs (A) and
animation_independent_checks.rs (B) preserve the additional old standalone
assertions. The old CI/Cargo entry files become thin wrappers around this host via
run_legacy_palette/run_legacy_animation. Full main and each legacy wrapper run their
shared assertions once. Optional animation_invalid_result preserves the old
explicit unknown-dispatch return fixture and defaults to the prior panic behavior.
