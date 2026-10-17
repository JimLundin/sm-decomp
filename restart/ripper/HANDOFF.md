# Ticket28 material source handoff

Owner: existing material agent `/root/material`.
Worktree: `/home/jim/sm-decomp-restart-ripper-rust`.
Branch: `restart/ripper-rust`.
Baseline: `aa272bda62c9175f0b99f6779f9a0781e4127971`.
Exclusive writable source: only new `restart/ripper/` files.

## Interface and exact scope

Implemented Entry repr(u32): JetInit0, JetMain1, RipperInit2, RipperMain3,
with ALL, address() and TryFrom<u32>. Public execute takes Entry, mutable full
WRAM, immutable runtime ROM and mutable Services, returning
Result<Option<()>,String>. Services has immutable available() and one
move_horizontal(ram,slot,amount)->Result<u8,String>. This shape was sent to
native before implementation, along with the stable shared fixture contract.

The four original bodies are sm_a2.c:3215–3254 and3273–3312 at recovered revision
578f90b3cc49557bb70060ad033bb90b8cf8ac50. Pinned sm_a2.c SHA256:
d62422845f0fee20c929ed7a635a0eb397d376a6b2d62b9c0aab21e3c18a315e.
Main enemy_ai.rs was inspected only as selected reuse context. Its old row63
limit and pre-service row snapshot are not copied. License notice is retained.

No shot/frozen/lifecycle definition, terrain solver, shared registration or
runtime scheduling file is changed. Root publishes translated/verified/integrated
stages separately; this source handoff does not imply those later gates passed.

## Runtime data and preserved behavior

Full65-row520-byte table extent is checked before mutation. Native owns binding
the exact prepared runtime table; source preparation records pristine slice
SHA256 b8c7ce12514576f554227bdf8a4e22d00bee41cd4b7b3aeea5dac2a594ecbcfb.
The crate reads the supplied runtime bytes. Unit models explicitly change row64
to prove there is no baked-in arithmetic substitute. These models do not assert
that modified bytes have the pinned native identity.

Init includes endpoint64 and wrapping8*parameter aliases; old E is irrelevant.
Main uses stored E, not parameter1. On collision it rereads both E and D after
service copyback from the original captured actor record, as the original C
pointer does. D==0 takes the nonnegative branch. There is no actor write on
clear movement. Unsupported state/extent/row and unavailable Main service refuse
unchanged; errors after an accepted request remain errors with effects retained.

## Shared46-case matrix

Public feature fixture-support exports BodyFixture{name:String,entry:Entry,
wram:Vec<u8>} and body_fixtures(). Native received exact names and count:

- Both prefixes jet/ripper:10 Init each, parameter0000/0001/003f/0040/2001
  crossed with direction0000/8000. Row64 uses current slot07C0; others0040.
- 13 Main each:air-positive/negative/carry/borrow/zero;
  wall-zero-integer/mismatch-positive/mismatch-negative/row0/row63/row64;
  spike-allocate/full-pool.

All Main cases use slot0040,16x16 room, Y0040/radii8. Default X0040;
right obstruction/spike X0038 with cells52/68, left obstruction X0048 with
cells51/67. Spike is only cell52 A000/BTS15, free or full40-slot PLM pool.
Rows/C/D are explicit fixture fields; native supplies real movement and D094
setup. Whole-WRAM sentinels preserve unrelated actor records and state.

The suggested native inherited chain configures Init row64/direction8000 at
X0038 with a right wall before the first call, then runs Init->Main(collision)
->Main(clear left movement) on inherited state. Native owns that control and
actual CallEnemyAi dispatch; no stage is inferred from this suggestion.

## Source checks, pending verification and cleanup

Source-only rustfmt and git diff --check passed. No Cargo/build/test/native/game
producer was started. Root owns workspace/lock/native registration, all focused
and actual-C execution, recipe freezing, stage publication and cleanup.

Ten authored unit tests cover the Init matrix, runtime endpoint reads,
immutable refusals, full copyback, table reload versus negation, zero/end rows,
post-service field reads, fatal errors and inherited patrol state. None has
been run by this worker. Root's focused command after registration is:

`cargo test --manifest-path restart/Cargo.toml --locked --offline -p sm-ripper`

Native supplies the exact46-case/terrain/transport/coexistence control CLI.
All semantic, actual-C, adapter-fatal and any live claims remain pending until
root's evidence exists. No private run outputs or active producers exist;
local cleanup0 bytes. Root retains sources and performs both cleanup passes,
authentication and integrated worker retirement.
