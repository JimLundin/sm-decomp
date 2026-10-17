# Sealed ordinary item instructions

This crate implements the thirteen item instruction/trigger bodies selected by
ticket18 from recovered C pin578f90b. It owns instruction control, the pickup
bit, wrapping missile inventory effects and queued-message request. Native code
retains allocation/collision setup, timed drawing, HUD/music services and message
presentation. The native adapter owns hook composition and transactional commit.

`execute` accepts only EF2F/argument0022 at the exact operand sites in the two
sealed bank84 scripts. It does not constrain the game state to normal play, so
the same initialization can execute through native room-load service52. No raw
pointer or arbitrary native function is exposed. Safe foreign identities return
`Ok(None)` unchanged; malformed identity-local requests return errors before
effects. Required services and queue indices are preflighted. A later service
error is fatal and must never retry the body through C.

The original Entry discriminants 0..12 and Service discriminants 0..2 retain
their meanings. Outputs distinguish preinstruction completion, Stop/null,
and Continue(bank84 offset), including odd offsets. Input/link validation does
not confuse the scheduler's stored instruction pointer with its currently
executed operand pointer. The exact semantics, service order and matrix remain
in root's frozen `restart/out/item-instructions/planning/semantics-contract.md`
SHA25607f9978238a4e7ae3b0a6d8f8635a4f76dbe46fa2a852da628727feb0b6e3e95.

Enable `fixture-support` to share `fixtures::body_fixtures` and
`refusal_fixtures` with native actual-C controls. They validate the fixed ticket17
revealed-WRAM SHA256 and both script seals, then apply declarative overlays for
exactly25 body and14 refusal cases. R07 uses `entry=None/raw_entry=21`; never
construct an invalid enum. Each fixture has expected output/direct writes and
service order. Direct-write expectations intentionally exclude native service
effects: actual-C controls must compare complete C/Rust WRAM plus ordered real
service pre/post snapshots, not substitute those expectations for C execution.

Unit tests use those same cases with inert service auditing, plus explicit
service-effect propagation/failure checks. They require the retained private
inputs (no ROM/WRAM capture is committed):

- `SM_ITEM_FIXTURE_SEED`, default
  `/home/jim/sm-decomp-restart/restart/out/first-missile/author-05/missile-revealed.wram`
- `SM_ITEM_FIXTURE_ROM`, default `/home/jim/sm-decomp/rom/sm.sfc`

Root owns execution, workspace/native registration, actual-C fixture runs and
the new fixed24832-pad C/Rust live pair. Root's selected live pair proves the
normal lifecycle/message return and actual room-load clear-item composition;
the compact native fixture proves the already-collected room-load branch.
No standalone fake512-resumption message loop is an acceptance substitute.
Ticket17 raw captures remain sealed and unchanged. Other items and original-ROM
parity are outside this crate's acceptance claim.

Ticket19 adds EF23/argument001A at exact Morph script sites, reusing eight
instruction bodies and adding graphics loading, equipment pickup and both
item-frame roots. Entry values 13..16 are LoadItemPlmGfx, DrawItemFrame0,
DrawItemFrame1 and PickupEquipmentAndShowMessage. Both frame roots execute the
private Rust Common body. Service values 3..5 are ProcessPlmDrawInstruction,
CalculatePlmBlockCoords and DrawPlm, in that order. Common saves the global slot
after Process and retains that slot for Draw after Calculate. The native
renderer and message presentation remain dependencies.

`fixtures::morph_body_fixtures` and `morph_refusal_fixtures` use the same sealed
ticket18 seed and existing Fixture type. They expose the finite 31 body and 20
refusal cases listed in [MORPH_HANDOFF.md](MORPH_HANDOFF.md). All four graphics
rings, both frame tables, valid clipped/full-capacity results, equipment OR,
Morph's D873 item bit, and changed admission/safety bounds are represented.
The runtime seals both Morph scripts, graphics/frame tables and referenced
draw records. Every capability and static memory bound is checked before
effects. No private build or verification run was performed by this worker;
root owns the new actual-C controls and fixed-input live comparison.

Ticket20 adds the exposed energy, missile, super-missile, power-bomb and reserve
tank headers EED7/EEDB/EEDF/EEE3/EF27, with permanent item IDs 0000..01FF.
The shared bit helper computes the map byte/mask from the admitted argument;
the exact prior Morph and Chozo missile argument domains remain unchanged.
Positive out-of-map IDs fail before effects; negative IDs are safe foreign
refusals. Entry values 17..20 add the four new collectors, and Service values
6..7 add the two explicit super-missile/power-bomb HUD services. Reserve graphics
reuse the existing item frame/Common implementation and memory bounds.

`fixtures::tank_body_fixtures` and `tank_refusal_fixtures` return the same Fixture
type using the same authenticated seed. The finite 36 body and 21 refusal cases
are documented in [TANK_HANDOFF.md](TANK_HANDOFF.md). The selected milestone is
source translation and actual-C family verification. The unchanged route is a
compatibility check and does not establish live reach of the four new collectors.
