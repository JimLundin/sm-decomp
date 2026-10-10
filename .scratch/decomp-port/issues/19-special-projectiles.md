# Bomb terrain and special beam attacks
Status: resolved
Blocked by: 14, 16
Triage: ready-for-agent

Own complete normal bomb five-tile terrain reaction and spread-bomb bounce;
wave/ice/plasma/spazer SBA initialization, dispatch and lifetime phases;
exact byte-product trigonometry and speed-echo radial lifetime. Reuse native
projectiles/flight trails and audio. No-amount PLM callback preserves null.
Power-bomb radius/HDMA remains separate; no full HandleProjectile replacement.

## Implementation scope

Native normal bomb fuse and five-tile explosion, all16bomb terrain reactions
including synchronous special/shootable/bombable PLMs and extensions; spread
bomb allocation, gravity, terrain bounce and explosion; complete wave/ice/plasma/
spazer SBA creation, FireSba selection/cost, orbit/lifetime/phase/end handling;
exact original320entry signed sine table with byte multiplication; radial speed
echo and unknown8027 shinespark trajectories. Existing native projectiles,
projectile_flight trail placement/Mode7 and audio are reused.

No-amount Host callback preserves upstream null movement context, records full
WRAM before/after and visible C setup counts. Invalid dispatch and quicksand
movement-context requirements reject before callbacks. Full HandleProjectile
and powerbomb radius/HDMA remain untranslated. Do not globally replace generic
BombOrPowerBomb_Func1 because its powerbomb branch is deliberately unimplemented;
normal bomb PreInstr wrappers are eligible after live verification.

Seven independent synthetic examples cover exact cardinal byte products,
plasma and spazer transitions/pair cleanup, wave signed fractional movement,
malformed phases/slots, fuse versus explosion and ordered five-tile once-only
callbacks. Focused actual-C suites passed without mismatch; final full837case
seal runs once after named-source stabilization.

## Completion evidence

- Exact final actual-C shared no-amount callback oracle accepted30routines/
  837fixtures, every full128KiB WRAM byte/return and ordered callback snapshots.
- Verdict `/tmp/sm-special-final/20261010T205450.319106Z/verdict.json`; current
  acceptance consumer reverified retained source, dependency and harness hashes.
- Seven independent synthetic tests pass; rustfmt and clippy `-D warnings` pass.
- All16bomb tile categories/fuse phases, selected square/nonsquare slope flips,
  real special/shootable/bombable PLM reactions, complete four SBA phases/firing,
  speed-echo/shinespark lifetimes tested. No hidden C/CPU fallback in candidates;
  remaining C PLM callbacks are measured.
- Offline equivalence only; live replacement and original-ROM parity untested.
  Full scheduler and powerbomb radius/HDMA remain separate ownership work.
