# Ceres-door shot contact: decode notes

## Sources per entry

| Entry | Source |
| --- | --- |
| ContactAllowed | A0:8FD4, 901E..903E pre-AI gates; A0:9758..9778 normal-box selection; A0:A143..A17F count, spritemap, intangible, DAFF and invincibility guards |
| BoxOverlap | A0:A1A6..A1D6 wrapped per-axis absolute difference, borrow and strict radius comparison |
| FirstPowerShot | A0:A181..A225 ascending five slots, ordinary power type, consumed marker and first hit return; A0:A226 response dispatch; E23F header shot response A6:F920 (RTL) |

Original source: [bank A0](https://patrickjohnston.org/bank/A0) and
[bank A6](https://patrickjohnston.org/bank/A6); selector writers and caller order are in
[bank A5](https://patrickjohnston.org/bank/A5) and
[bank 82](https://patrickjohnston.org/bank/82). Records were inspected first.

## RAM mapping and integration boundary

- Processing list 17AC is rebuilt by A0:8EB6; main enemy runner iterates it in order at 8FFF.
  Interactive list 17EC is a different receipt, used by solid-enemy Samus motion.
  The preceding NMI-return records' processing list is empty: do not decode it as next-frame
  membership. Production must retain the actual selector result for the current frame.
- Global off-screen enemy processing enabled is 183C: nonzero admits every live,
  non-DAFF, nondeleted slot regardless of the window, ProcessedOffScreen or Frozen.
  Both selector branches exclude empty, DAFF and deleted before admission, in ascending
  slot order. Intangible affects interaction, not processing membership. Generic decoded
  commons default species, so retain a separate raw DAFF check.
  The main-gameplay selector is the first call at 82:8B47. Direct override writers are
  enemy initialisation's A0:8ABF clear and A5:86BC/EACB enable; a normal same-room
  consecutive gameplay iteration has no intervening writer. Other live routes must
  supply their actual selector-time facts or report an input defect. The production
  bridge receives actual membership, and supports relocated doors anywhere; Ceres
  filtering declares this harness's coverage only.
- Enemy processing 0E4C, suppression 0E12, stopped-time 0A78/185E and current processing
  membership are separate named facts combined by ContactAllowed in the clean model. The initially positive 0FA0 branch decrements and
  skips contact even when the result reaches zero. The bridge must retain that opportunity's
  protection decision; a post-decrement zero alone is insufficient. Intangible 0F86 bit 0400
  skips interaction before this protection decrement. Timer ownership remains the runner's.
- Current actor is 0E54, slot stride 40h. Its pre-contact centre is 0F7A/0F7E, radii 0F82/0F84,
  properties 0F86, secondary properties 0F88, figure 0F8E, protection 0FA0, species 0F78.
  0F88 bit 0004 selects multibox, unsupported. Missing/804D figure is WithoutFigure;
  intangible is Passable; invisibility 0100 alone is irrelevant. Empty/DAFF species is not a
  supported placed Ceres door. Normal power shots do not have plasma piercing or special
  missile/super-missile responses; property 1000 cannot change their consumption.
- Shots: count 0CCE; five slot strides 2; centres 0B64/0B78; radii 0BB4/0BC8;
  kind 0C18; direction/consumed 0C04. Ordinary power type is nonzero with category bits zero;
  consumed sets 0010. The search does not test the existing consumed marker. Impact and Free
  are not eligible. W1's current clean bodies supply the positions and radii; never the next
  record's bodies. Consumption changes no other W1 quantity and retains its body until update.
- Invoke the decision per current processing actor, after W1 and before that actor's AI.
  Apply the returned marker immediately. Earlier actor AI and response side effects are
  visible to later calls: do not freeze all collision properties at selection or frame start.
  The function owns no State and no selection/countdown logic; it owns the eligibility conjunction; the production bridge must reject other
  species, multibox and unsupported weapon families explicitly, not report no hit for them.
- Door response F920 writes nothing. No damage, shake or sound accompanies ordinary power
  contact. Door AI/instructions remain B8; the bridge must not duplicate them in C1.

## Ceres temporal proof and limits

For B8's parameters 0, 1 and 4, its instruction arms F56C..F63C, conditions F63E,
F66A/F678, and property operations F68B/F695/F6A6/F6B3 update the current slot's phase,
figure, visibility and tangibility only. F6BD requests opening sound. The selected main
function F76B/F773 writes global shake timer/type only. Door centre, radii, protection,
another door's properties and owned-shot bodies are not written. Header response F920 is
no-op. Therefore an earlier supported door can change global shake/sound, but does not
change a later door's contact geometry/permission. This is a narrowly sourced temporal
proof, not a claim about E23F's other parameters or arbitrary enemies. Unsupported parameter
actors need their own proof or actual pre-contact observation. The production per-actor seam
preserves evolving state even where this narrow harness can reconstruct it.

Revalidated existing independent original watch against recovered immutable records:
chain-any-0003-DFD7 preceding cuts156..160, ten entries in order slots0 then1, all nine
actor contact words agree. The original watched replay's existing evidence is 345/345 WRAM
cuts; this observer did not rerun that oracle session. The new checks decoded all345 records,
confirmed shot live157..159, consumed160, absent161, and independently predicted post-W1
bodies from preceding records with the observer W1 reference. Its input_frame ignores the
following-record argument; the new proof passes None explicitly. Expected marker bytes are
read only for comparison. This artifact validation is not a new original execution.

## Honest harness contract

Compare the five raw live-pointer AND direction-0010 markers after C1. This predicate
includes every observed kind. Reject restored occupied Impact+0010 and count0+Flying from
preceding inputs rather than narrowing the expected predicate. No masks or following-output
admission. The Ceres region declares coverage; unsupported inputs inside it are defects.

Every admitted interval requires the exact preceding ordinary main-wait return, state8/debug
inputs, independently completed endpoint and finite HDMA source capabilities. C1 additionally
rejects reachable reverse-DMA producers and validates DMA hardware destinations. Its marker
capability is separate from the inherited broader W1 quiet assumptions: search inactivity does
not certify caller or actor validity.

The ordinary suffix admits typed door headers A6:F765/F920/F920, normal AI handler0,
parameter-derived dynamic functions and instruction cursors in imported reachable graphs.
Live contact remains single-box parameters0/1/4. Quiet modes3/5/6 have marker-preserving
source closure; mode2 and steam touch are unsupported. Steam touch can install knockback
and change POST. Other actors, alternate handlers, multibox and unclosed loading/cinematic
routes are explicit defects, including quiet opportunities.

Normal PRE/POST dispatch, imported pose/animation/radius quantities, movement speed content
and ordinary momentum bounds establish a conservative collision rectangle with horizontal bound 32 and vertical bound 128.
Read every actual raw cell in that rectangle, including out-of-loaded-room aliases; classes
0/1/8/9 exclude collision-created PLMs. Empty callback pools, no-call room/graphics hooks,
bounded explored-map/OAM/graphics queues and absent reverse-DMA producers compose the suffix.
The source-based radius16/19/21, E913/E926 and no-item run-extra bounds do not use record IDs.
E926 alone requires its jump-binding/AF4 predicates; E913 retains its ordinary input contract.

Measured local M0 receipt coverage is136 compared/held versus135 null, with208 explicit
defects/unreached among344 transitions. The first supported interval ends at1; the first
unsupported interval ends at8. A segment seeded at125 holds57 receipts through182, including
contact160, and stops at183. This is a local stateless marker certificate, not an uninterrupted
engine prefix. All63 rows account for40835 transitions:295 compared/held,293 null,31630
outside Ceres and8910 unreached, with zero masks, folds or divergences. All unreached inputs
remain visible. The null refreshes the same independently predicted post-W1 markers each
opportunity, on identical scope.

ContactAllowed receives actor-updates, interactions, Samus/enemy stopped time, current
processing membership and initial protection separately. Negative vectors independently
disable each condition. The30 constructed vectors include eight translated observation cases
and22 other constructed cases. Inclusive edges fail C-edge and C-y-edge. Four live searches
and one hit are within the supported M0 receipt scope; W1 removes the consumed body at161.
No expected body, frame whitelist or hash grants admission.

C1 has no retained State. Marker agreement and its stateless free accounting establish
neither whole W1 State, whole OAM, sound, health nor whole-M0 play. A separate executed
OrdinaryPower production replay compares the complete modeled State, projectile owner and
all 544 OAM bytes across 183 accepted cuts through the Ceres door contact. That bounded
engine prefix does not extend the C1 marker-only certificate to sound or whole-M0 play.
Independent planted-fault proof must be rerun against this source-certified scope before
its final integration gate.

## Recorded cases behind vectors

C-record-157-door-0 through C-record-160-door-1 use chain-any-0003-DFD7 cuts157..160,
with post-flight W1 geometry predicted from preceding cuts156..159. Both actor and shot
centres are relocated together to invented centres; radii and relative offsets are observed
quantities, with no asset bytes. The eight observation cases are honestly labelled constructed: their coordinates were translated,
not copied unchanged. C-inside and C-passable are additional constructed representatives of
these recorded overlap/passability arms. All other cases are hand-worked from the arithmetic:
exact x/y edges fail, negative and wrap offsets strictly inside succeed, half-world fails,
zero door radii still overlap when the shot encloses its centre, first slot wins, and an
already consumed beam may be selected by a second door. No imported content is in vectors.

## Budget

k = 37.5 weighted tokens per spec unit.

- Area contact: ContactAllowed, BoxOverlap, FirstPowerShot

The reusable bridge applies returned consumption to the complete owned W1 State. The checked
ordinary runtime calls it per processing actor before that actor's protection decrement and AI.
The C1 model is stateless; the engine keeps the projectile owner and the transient ordinary pass.
No old-engine deletion baseline is available for this new contact model. The replaced verifier
contact helper is accounted separately from production model and bridge code.

### Translated observation provenance

All rows in this table are `chain-any-0003-DFD7`; preceding cut R supplies W1 inputs and
pre-AI actor fields. The following cut supplies expected consumption only. Actor slots use
processing order from the independent watch. Quantities translated are both centres; radii
remain observed values. Values are therefore constructed translations, not unchanged records.

| Vector | Preceding frame | Compared frame | Actor slot |
| --- | --- | --- | --- |
| C-record-157-door-0 | 156 | 157 | 0 |
| C-record-157-door-1 | 156 | 157 | 1 |
| C-record-158-door-0 | 157 | 158 | 0 |
| C-record-158-door-1 | 157 | 158 | 1 |
| C-record-159-door-0 | 158 | 159 | 0 |
| C-record-159-door-1 | 158 | 159 | 1 |
| C-record-160-door-0 | 159 | 160 | 0 |
| C-record-160-door-1 | 159 | 160 | 1 |
