# Normal projectile terrain flight and firing
Status: resolved
Blocked by: 12, 13, 14
Triage: ready-for-agent

Own normal beam/wave/missile/super missile flight, six terrain entry points and
all tile reactions/slopes/extensions; native trail placement including Mode7;
core beam/missile firing and linked-super collision. Actual C SpawnPLM callback
oracle must preserve null cur_coll_amt32 for shot reactions. Top-level
HandleProjectile remains C until SBA/bomb/power-bomb handlers are owned.

## Implementation

Normal firing/flight forms one owned dependency closure: six bank94 collision
entry points with all16 shot tile reactions, square/nonsquare slopes and bounded
extensions; normal beam/wave/missile/super pre-instructions and direction wrappers;
linked-super trailing collision; initial shooting collision including left-speed
minus-one quirk; uncharged/charged/hyper firing and missile ammo/HUD cancellation;
trail allocation and placement with native Mode7 signed multiply transforms.

`Host::spawn_plm(ram,header)` deliberately has no movement amount: shot reactions
leave the upstream `cur_coll_amt32` null. Harness manifest selects
`plm_amount_context: none`; callback copies full owned WRAM, performs actual C
setup, records before/after full WRAM and counts remaining C gameplay. Unknown
setup dispatch and broken no-context quicksand branch reject before callbacks.
Source uses existing projectile_core lifecycle/audio with canonical WRAM.

Eight independent synthetic-ROM tests cover fixed-point negative movement,
wave reaction without destruction, room/slot rejection, bounded extension
cycles, native Mode7 rotation/signed trail offsets, no-context quicksand
rejection, and initial left firing transient speed. Broad final fixture family
is moderate42routines/1233cases; commercial fixture data remains local/ignored.

## Completion evidence

- Shared actual-C null-context callback harness:42routines/1233fixtures match
  return, all128KiB WRAM and ordered full before/after callback state/arguments.
- Verdict: `/tmp/sm-flight-final-shared/20261010T203804.856231Z/verdict.json`;
  acceptance consumer reverified current retained source/dependency/harness hashes.
- Eight independent flight tests and nine predecessor projectile tests pass;
  rustfmt and targeted clippy `-D warnings` pass.
- Every terrain category, thirteen square/nonsquare shape representatives with
  four flips, all ten flight directions and timer/trail phases, beam combinations,
  missile HUD/ammo/cancellation, real Mode7 trails and linked-super collision pass.
- This is offline equivalence only; live replacement/original-ROM parity remain
  untested. C PLM setup callbacks are explicit and measured. Full HandleProjectile,
  SBA/speed echoes/spread bombs, bomb tile reactions and power-bomb HDMA remain.
