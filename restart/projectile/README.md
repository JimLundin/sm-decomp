# Ordinary Samus projectiles

`sm-projectile` implements four real scheduled entry points: `FireUnchargedBeam`,
`HandleProjectile`, `DrawPlayerExplosions2`, and `DrawBombAndProjectileExplosions`.
They share private firing, fixed-point flight, terrain collision, instruction,
trail, impact, sound-queue and OAM code. All state belongs to the supplied full
128 KiB WRAM; ROM is read-only. This crate has no dependencies and no RAM copies.

The admitted inventory is the ordinary uncharged beam (`equipped_beams == 0`,
no Hyper Beam), its `0x8700` terrain-impact explosions, empty bomb slots and no
shinespark crash echoes. `HandleProjectile` admits either value of the Ceres
escape high bit (`ceres_status & 0x8000`); Fire and both draw entries require it
to be clear. Active beams use
preinstruction `0xAEF3`; explosions use `0xB169`. Unsupported inventory returns
`Refused` before mutation. Once admitted, errors terminate ownership; they never
request fallback. The native host supplies a private transaction and enforces
that distinction around calls.

`SpawnPLM` is the only foreign service. It observes and modifies the latest full
WRAM synchronously and returns the actual native carry/overflow bits. Unsupported
or context-dependent setups fail. No service may bind `cur_coll_amt32` to a Rust
pointer. The first pre-escape Ceres wall-shot scenario expects zero PLM requests.
OAM and sound queue writes are WRAM effects. The existing host owns PPU transfer
and sound playback, input/HUD/cooldown scheduling, and enemy contact reactions.
Enemy reactions can mark a beam for deletion; the next Rust projectile pass
performs that deletion. These remaining C responsibilities are not Rust ownership.

Ceres Mode7 drawing remains excluded: the pinned recovered-C player renderer
passes an uninitialized Y local, and its explosion renderer discards the computed
Mode7 point. `sub_9382FD` (elevated rendering), other weapons and charged beams
also remain C-owned. This crate does not implement their algorithms.

During Ceres escape, the complete ordinary update still runs the existing
ten-slot scheduler, beam flight/collision and impact instructions. A trail
allocation tick transforms its point with `CalcExplosion_Mode7` before applying
the signed left/right offsets. The transform uses signed 16-bit products shifted
by eight, truncates each product to 16 bits, and wraps sums and differences. It
uses M7A in both equations and adds the layer position back for stored trail
coordinates. This calculation uses canonical WRAM directly and needs no foreign
service. The branch selector is Ceres status, independently of PPU Mode7 flags.

Source provenance: snesrev/sm revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, banks `$90/$93` projectile routines,
`$94` no-wave terrain collision, `$9B:A3CC` trail placement, `$8B:8AD9` projectile
Mode7 transform, `$81` spritemap
emission and `$80` SFX queues. Selected algorithms were adapted from the previous
project's `crates/port/src/{projectiles,projectile_flight,projectile_trails,
samus_projectile_frame,audio,memory}.rs` at restart baseline `c177b4b`.
Its generic dispatchers, whole-WRAM transactions, weapon variants and old crate
dependencies are not retained. The upstream notice is in `LICENSE.upstream.txt`.

The bounded unit checks cover refusal without mutation, negative fractional
velocity, the initial left-shot probe, deletion ordering, malformed bytecode and
ordinary OAM flicker. Actual recovered-C comparisons and native replay are run by
the integration owner against these exact entry points. Passing unit checks do
not establish original-ROM comparison or acceptance outside the tested domain.
Escape checks additionally cover entry-specific unchanged-state refusals, retained
inventory/error rules, signed transform arithmetic, allocation of both trails,
and subsequent flight/instruction progression. Synthetic unit inputs grant no
live-from-reset or active escape-rendering claim.
