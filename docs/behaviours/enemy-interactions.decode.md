# Ceres enemy interactions: decode notes

## Door contact sources per entry

| Entry | Source |
| --- | --- |
| ContactAllowed | $A0:8FD4, 901E..903E pre-AI gates; $A0:9758..9778 normal-box selection; $A0:A143..A17F count, spritemap, intangible, DAFF and invincibility guards |
| BoxOverlap | $A0:A1A6..A1D6 wrapped per-axis absolute difference, borrow and strict radius comparison |
| FirstPowerShot | $A0:A181..A225 ascending five slots, ordinary power type, consumed marker and first hit return; $A0:A226 response dispatch; E23F header shot response $A6:F920 (RTL) |

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
actor contact words and all five global permission words agree. The pre-contact watch is
`boundary-c1.txt`, SHA-256
`16620f616496ff75e1ce8fed6f5219fb1b8540d4ac04cdcd6297d5da66b02202`, captured by the
execution callback in `boundary-server.lua` at A0:A143. The selected replay uses preceding
cuts156..159: eight ordered visits, 72 actor-word comparisons and 40 global-word comparisons.
The globals cover gameplay mode, actor updates, interactions, Samus stopped and enemies
stopped; the post-W1 shot count is not admitted from the preceding cut. The trace stays in
private oracle evidence; this source pin and the immutable record cross-check establish
the bounded input tape, not actor selection outside it.

The original watched replay's existing evidence is 345/345 WRAM
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

Shot contact retains no State. Marker agreement and its stateless free accounting establish
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

The reusable bridge applies returned consumption to the complete owned W1 State. The checked
ordinary runtime calls it per processing actor before that actor's protection decrement and AI.
The contact entry is stateless; the engine keeps the projectile owner and the transient ordinary
pass.

## Shaft picture sources

| Entry | Source |
| --- | --- |
| ShaftActorDraw | $A0:8EB6..8FD3 establishes selection. $A0:90EF..9118 tests the ordinary box window ($A0:ADE7..AE29) but skips it for extended figures. Both formats test deleted/invisible and the draw-enable result. |
| ActorShake | $A0:8A6D..8A87 clears the allocated actor slots. $A0:90A6..90B3 increments the frame count on ordinary actor turns. $A0:947B..9494 reads the updated count and spends the prior timer on a draw visit. $A0:8702..872C refreshes the active list after drawing. |
| EffectSlots | $86:8016..8026 clears the live identities. $86:8097..8103 scans the 18 slots downward, gives the chosen slot an initial timer and an empty map, and ignores the request if full. $A6:F834..F850 issues the stationary effect using $86:E509, with no initial screen test. |
| EffectClip | $86:8104..8158 advances the live pool when enabled. $86:E4FE..E507 checks $86:E6E0..E721 before each advance, including on the spawn turn. $86:E138 and E1EA are the two finite clips. $86:8154 deletes at the end. |
| PictureWindow | $A0:9456..9494 places the actor origin. $A0:9538..956C checks the widened origin window for each mounted vent's one extended art entry. $86:83D6..8426 checks the same widened window for effects, using the quake query at $86:8427..846A. The overlay is ordinary and goes straight to $81:8AB8 from $A0:94EE. |
| ShaftPictureOrder | $A0:884D..88CE draws high-priority effects before ordered actor layers, and Samus before actor layer 3. $A0:9423..9449 appends actor visits within a layer. $86:8390..83B1 draws high-priority projectile slots from highest to lowest. |

Original banks [81](https://patrickjohnston.org/bank/81),
[82](https://patrickjohnston.org/bank/82), [86](https://patrickjohnston.org/bank/86),
[A0](https://patrickjohnston.org/bank/A0) and [A6](https://patrickjohnston.org/bank/A6).
Static clip art is in bank 8D, separate from the 86 clip durations. See the
[B8 burst decode](boss-ceres-ridley.decode.md) and [room quake](world-quake.decode.md)
for the separately owned requests.

### Actor mapping and turn order

- In this placement, overlay is actor slot 0, with layer 2 and a simple figure.
  Mounted vents are slots 2 through 12, layer 5, extended figures. Native stride is
  40h at 0F78. Current layer is actor word 0F9A, properties 0F86, extended-figure bit
  0004 at 0F88, figure 0F8E, horizontal radius 0F82, centre 0F7A/0F7E, and projected
  offset 7E7010/7E7012. Each shown mounted vent's extended figure has one art entry and
  its local origin is zero. B8's figure ID resolver and AF3 identify the static art.
- Actor clocks are 0FA4 and 0FA2 per slot. Placement produces zero values. Do not
  rebuild the frame count from global time or from the count of visible puffs. The actor must
  have participated even if the overlay is offscreen or the vent is hidden. B8 already owns
  the checked participation window for mounted vents. Capture its actual result as a typed
  receipt. The overlay is selected offscreen via 0F86 bit 0800. A0:ADE7's ordinary draw
  gate uses the overlay's starting view and unprojected position, with inclusive endpoints.
  The shaft's extended vents skip that draw gate and use their projected origin
  later. The raw actor draw gate tests 0F86 bits 0300 and 17A2 bit 0001. The shaft's
  supported drawing actors never disable drawing independently of their selected shown
  figure. The overlay's placed empty figure draws no pieces.
- Main gameplay 82:8B44 uses the starting layer-1 camera (0911/0915) for selection.
  The actor work and then enemy projectile work precede 90:94EC scrolling.
  Drawing reads the scrolled camera and the quake after B8's new starts but before
  its countdown. 82:8BAF performs the room quake after drawing, so the actor clock's
  draw spend uses the existing 0FA2 value. The quake overwrites 0FA2 with 2 for each
  selected actor, including invisible ones. Both elevator states 20h and 21h call this
  ordinary sequence, starting with the timer picture at 82:836F and 82:8390.
- During room-arrival fade-in, 82:E737..E76A does actor selection, actor work, effect
  advancement and drawing, with no room quake update. A B8 door can start the quake
  during this sequence. At hall 239, slot 0 runs the B8 overlay opening
  A6:F7BD..F7DB and lights A6:F850..F86F. The separate shaft door in slot 1
  runs A6:F76B and starts the quake through A6:F773..F7A4, the existing B8
  `EscapeShakes` intent. Apply that intent before querying the effect draw offset.
  The draw uses that quake even though it does not count down yet.
  Hall 239 executes the first actor turn; hall 240 only completes pending work.
  The next fresh turn is hall 241. Use the placement and fade-in caller and the pending vblank phase to establish
  which non-wait cuts execute actor work. Retaining prepared state through suspension belongs to the scheduler.
- This route has no damage flash, frozen/paralysis override or debug targeted shake
  (A0:8FE4..8FF9). The checked ordinary runner must admit those facts or report missing
  coverage. Enemy palettes, invulnerability and other contact or drawing families are outside
  the shaft capability.

### Effect mapping and drawing

The live projectile identities start at 1997 with 18 two-byte slot offsets. This scope
admits only zero or stationary dust/explosion E509. Reject unrelated nonempty slots in a
scoped production replay. Centre words are 1A4B/1A93. Clip next-entry cursor is 1B47,
current figure is 1B6B, and remaining frames are 1B8F. The room initializer supplies clip
entry E138 for SmallExplosion and E1EA for Smoke based on the B8 request's kind. The outgoing
cursor points one entry past the currently drawn figure. At initial placement the cursor is on
the first entry, 1B8F is one and 1B6B points to the empty figure on bank 8D. The enabled flag
is 198D bit 8000. The contents are mount-independent and use zero extra graphics selectors
(19BB) and high drawing priority (1BD7 bit 1000).

An outside lifetime centre clears identity before picture choice. The original can still
change some scratch payload words in the same turn after it clears identity. The empty slot
projection compares identity only, because occupancy and the next insertion are the only
consumers here. Reuse initializes every modeled quantity. The wider draw window is independent
of this lifetime cull. Use signed wrapped coordinates for clipping and preserve the exact layer
order across both drawing stages. For the effect and each vent figure, the exact writer
adds signed piece Y to the admitted origin; out-of-range Y gets F0h while its X, tile,
attributes and place in the picture are still written (81:8B22/8B96, 81:8C0A/8C7F).
The ordinary overlay uses 81:8AB8 and preserves byte wrapping of piece Y. Within each
figure the exact writer preserves piece order. OAM capacity, remaining owners, residual
hidden entries, and hardware publication stay with the exact writer and caller.

### Shaft evidence and harness

Source records are the locked lean tiers for `chain-any-0010-DF8D` and
`chain-any-0011-DF45`.

Seed at hall 225 after native room placement. Compare hall 226 through 278 and shaft
0 through 239 in order, including held cuts. Take actor participation inputs from the
starting camera and B8's actor placement and shown figure observations. The independent
reference carries every modeled effect slot and actor clock across all of these cuts.
Use the observer's B8 overlay and RNG rules for arrivals and the room-quake rules for its
draw offset and refresh. The model reads B8's resolved art and projected relocation, and
the draw-time layer-1 camera as borrowed inputs from the comparison cut; those facts do
not prove the B8, AF3, camera or scheduler producers. No next actor clock, native effect
slot, effect phase, or artwork output is admitted to predict the corresponding C1 result.

The reference matches 293 of 293 complete pool projections (222 frozen-seed matches),
5,274 of 5,274 stable slot projections (5,044 frozen-empty matches), and 7,032 of 7,032
actor clock fields (2,154 frozen-seed matches). It sees 255 running turns and ten B8
requests; nine survive to their first figure. The first slot appears at hall 241, holds
through 242, shares the picture with the second slot at 256 and changes figure at 265.
The first slot finishes at shaft 18. Shaft 31 is a due burst whose new slot is immediately
culled. The pool and slot comparisons include empty slots.

Exact attribution matches 485 prepared and 485 published effect pieces. The effect
publisher comparison includes pictures prepared at an overrun and published when the
pending turn completes. Omitting the first arrival or its draw shake changes hall 241.
Holding clip time still changes hall 244, and reversing the effect slots changes hall 256. The effect
prefix starts at piece 0 on fade-in turns and at piece 17 after the ordinary timer. It
matches 528 prepared and 528 published overlay pieces, with the overlay following the last effect piece on
43 cuts. Shaft 1 distinguishes the overlay's shake and shaft 29 distinguishes its draw
gate. The locked-standing vent suffix comparison (shaft 165 through 239) matches 192
prepared and 192 directly published vent pieces, starting after the timer and Samus at
piece 24. The earlier independent [B8 vent decode](boss-ceres-ridley.decode.md) and
full-suffix attribution also use the native sprite writer's preserved hidden entries
through 241. These projections cover the stated picture groups. Whole-engine replay must still check
the other OAM owners and complete publication.

| Translated vector | Native discriminator |
| --- | --- |
| R-vent-first-left | Shaft 166, slot 4. Updated actor count is 87 and incoming shake is 2. It is the first visible vent on the locked suffix and the first effect of dropping vent shake in that suffix. |
| R-overlay-first-right | Shaft 1. Overlay count goes from 16 to 17, spends its incoming two-turn shake for +1 X, and the room quake refreshes it to two. |
| R-overlay-leaves | Shaft 29. The overlay keeps running while its ordinary draw gate withholds its picture. |
| R-effect-first | Hall 241, slot 17 starts Smoke at its first frame. |
| R-effect-hold | Hall 242 retains the cut 241 slot. |
| R-effect-order | Hall 256 paints existing slot 17, then new slot 16, then the overlay. |
| R-effect-change | Hall 265, slot 17 advances from its first Smoke figure to its second. |
| R-effect-finishes | Shaft 18, the final Smoke figure in slot 17 ends and disappears. |
| R-effect-cut-off | Shaft 31. The fresh effect is outside the pre-scroll lifetime rectangle and never supplies a picture. |

Vectors translate positions and replace static art and clip content with invented data.
The authenticated records exercise the actual clips. Full pool, no-update continuation,
wrapping coordinates, and wide-picture versus lifetime edges use constructed witnesses.
