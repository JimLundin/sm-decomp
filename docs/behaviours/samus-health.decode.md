# Current energy: decode notes

Observer, adversary and integrator evidence only; never part of a confined export.
The [spec](samus-health.md) describes player-visible quantities and deliberately
excludes source structure, hardware accounting and native coordinates.

## Sources by entry

### The frame's order

Bank log [82](https://patrickjohnston.org/bank/82), page SHA-256
`a925e5aca5ee4596616fad8ff0c3f628bc86451ab583bb704a26f5ac412417f6` as fetched on
2026-11-23. Main gameplay at `$82:8B44..8BB8` runs, in order, the Samus current-state
handler (`$90:E692`), Samus/projectile interaction, the main enemy routine (`$A0:8FD4`
at `$82:8B65`, where Ridley's tail contact runs), the Samus new-state handler
(`$90:E722` at `$82:8B69`), drawing (`$A0:884D`), the HUD (`$80:9B44`), and last the
hurt-timer decrement (`$A0:9169`). This is the spec's "Order that matters": hit, loss,
warning, drawing, then protection spent. The spec states it as one whole-frame step;
the routine boundaries stay here.

Health's first reader inside the new-state handler is animation. In bank log
[90](https://patrickjohnston.org/bank/90), page SHA-256
`04dbc3f75171959ffbd9dc0f40ed39fe2c90ddedc3799cfbdd342a779798faa1` as fetched on
2026-11-23, the normal new-state handler (`$90:E725..E758`) runs Animate Samus
(`$90:E73B JSR $8000`), then periodic damage (`$90:E74D JSR $E9CE`), then the low
health check (`$90:E753 JSR $EA7F`). Animate Samus reaches the animation delay
(`$90:8062 JSR $82DC`), whose instruction at `$90:8346` compares whole energy with
30: that is `Breathing`'s read. The current-state handler (`$90:E695..E6C8`) calls no
animation. So `Breathing` sees energy after the tail hit and before periodic loss.

### PeriodicLoss

Bank log [90](https://patrickjohnston.org/bank/90), pinned SHA-256
`0b1aa7a6e6e839d6dd3e96543509f6508620105f9448bf5854eba1568db66290`.

Source: `$90:E9CE..EA44` establishes whole/fraction subtraction, borrow, signed-negative
clamp and clearing pending damage. This round admits only running time and neither
Varia nor Gravity. The original's suit scaling and frozen-time clearing are source
evidence outside this bounded model, not silently implemented paths. The accepted
actual-source reconstruction separately proves execution of the reached zero-loss
owner writes and pending consumption. `$90:8346..8355` is a whole-energy animation
consumer; `$90:EA7F..EAAA` consumes the later energy for warning. These consumers
establish the observable fractional boundary and warning contrast, not extra model calls.

### TailHit

Bank logs [A6](https://patrickjohnston.org/bank/A6),
[A0](https://patrickjohnston.org/bank/A0) and [91](https://patrickjohnston.org/bank/91).
The trace is checked against the pinned unheadered ROM in
[ORACLE section 7](../ORACLE.md#7-data-the-rom-bundles-and-the-cipher).

Ridley's tail check at `$A6:CB0E..CB20` consults tangibility, Samus's protection and
whether Ridley holds Samus. Its unprotected branch checks tail-tip overlap at
`$A6:DFD9..DFEF` with `$A6:DF29..DF57`. A hit takes the actor's configured amount from
`$7E:7838` through `$A0:A45E..A46F`. The recorded suit read at `$A0:A460` clears both
reduction bits. The pinned `$91:DF51..DF7F` path checks time before subtracting from
current whole energy and does not touch the fraction. The pre-subtraction live X and
the subtraction amount share the entry value. The observed return stack and
`$A6:E000` store tie that debit to the tail caller and its separate grant of 96.
`$A6:E006` sets a separate knockback timer of five. Knockback direction, hurt flash
and their producers remain outside Health.

At 993 the native actor reads zero protection; at 994 it reads 95 at the same
`$A6:CB14` operand and bypasses the overlap and debit path. There is no native
second overlap result in this window. A derivative passive capture on the same
locked pads observes the actual eligibility operands, tail-tip and Samus centres,
Samus's radii, both ordered overlap tests and their successful return at 993.
It then observes the configured pre-suit damage feeding the same adjusted debit
independently proved in the original capture. Cut 992 has an ineligible actor
check. Cut 994 has a tangible, non-holding Ridley and carried protection, and its
check returns without geometry. The three cut records, all 91 phase images and
the earlier Health trace are byte-identical across the acquisitions.

The derivative manifest SHA-256 is
`efceaaf979a3bd55c38024b59af279c42355c77d778f45972f5e75c7c5bbd1bf`.
Its raw actor trace SHA-256 is
`14a932cc8a340455837540dab885107fe06626dee4bb73dcb876044a4b938003`.
The private qualification runs in `$TEMP/collab/tail-geometry-671/`. It verifies
the operands at their live reads, alongside observed results and call/return
evidence. Tail-tip publication, actor pose updates and configured damage writers
remain actor-side dependencies for integrated replay. The independently observed
post-adjustment amount is available as a declared borrowed Health stimulus for
scoped production replay.

### The hit's worth: the escape's steam

The frame's impact carries its worth from the hitter. Ridley's tail takes the actor's
configured amount above (15); the escape's steam vents reach the same debit with a
species damage of 0. In bank log [A0](https://patrickjohnston.org/bank/A0), a puff's
touch at `$A0:A562..A596` reads the species damage (`$A0:E1FF` + 6: 0) through
`$A0:A45E` into `$91:DF51`, which takes 0 from current whole energy and leaves the
fraction. The same touch stores 96 in `$18A8`, as the tail's `$A6:E000` does, plus
the push time and direction that Movement owns. The vents' contact skips the touch
while `$18A8` is not 0, so Health never sees a vent's impact while protected; the
constructed C-steam-protected states what Health does if one arrives. The worth is
therefore a value of the hitter, not a constant of Health: `TailHit`'s arms subtract
whatever the impact carries, with the floor of zero, and a 0 grant is the same grant.

R-steam-hit is row `chain-any-0009-DFD7`, the interval ending at cut 137 (room 3 of
the escape, the vent touching her from the left): cut 136 holds the State before and
cut 137 the State after. Whole energy 24, fraction 0, pending 0 and warning On at both;
protection 0 at 136 and 95 at 137; neither suit; time running. The lean record cache
has no Drawing marker, so the vector omits `protection_drawn`; the warning request
None follows from the unchanged warning word and is not independently observed.
The second puff hit, `chain-any-0010-DF8D` cut 66 (room 2, from the right), has the
same Health State before and after and is not a separate vector.

### ProtectionTime

`$A0:9169..9171` reads active protection and decrements it once when positive.
It executes after status in this acquired active Ceres window. Both Drawing
markers retain the count that the later timer close reads. At 993 the sequence is
begin 0, grant 96, Drawing 96, close 95. At 994 it is begin 95, Drawing 95, close 94.
The native bus witness includes the physical word load and read-modify-write
against the same word. The adjacent decrement for knockback is separate.
This source speaks for the active frame tail; it does not admit stopped time,
other grants or resets. The count both Drawing markers read is the spec's
protection while drawn: 96 at 993 and 95 at 994, the `protection_drawn` of
R-tail-hit and R-tail-protected.

### Drawing and status

`$90:EAD7` compares current energy to Appearance's previous drawing energy.
`$90:EAFA` reads current energy again and `$90:EAFD` saves it as drawing history.
At 993 both energy operands read 24. The snapshots at both entry markers still carry
39 in history, 96 in protection and inactive diagnostic invincibility. At 994 both
energy operands and prior history read 24, and protection is 95. No health write
by Drawing occurs in either opportunity. The wider debug restore and hurt or flash
branches are outside this admission. `$80:9B8B` reads current energy for the gauge
at both opportunities. At 993 the gauge redraw re-reads 24, updates previous shown
energy and its digits, then carries at 994 without a duplicate redraw. Both the
Appearance history and the HUD cache have their own owner.

### LowEnergyWarning

Source: `$90:EA7F..EAAA`, external wrapper `$90:EAAB..EAB2`, plus the energy result
of `$90:E9CE..EA44`. The threshold comparison is signed after subtracting 31:
within admitted whole energy `0..32767`, it means at most 30 versus greater than 30.
The warning word changes independently of whether the audio queue accepts the request.
Audio library 3 requests 2 (start) and 1 (silence), with a maximum queued count of 6,
are exact-encoding/audio details, not model codes. No queue acceptance is asserted here.

## RAM mapping

All locations below are WRAM words. Fractions count `1/65536` energy. Warning is
0 Off and 1 On in qualified records; wider warning codes are not admitted by this
decode. Keep the model's typed complete State distinct from these encodings.

| Model scalar | WRAM location |
| --- | --- |
| `pending_fraction` | `$0A4E` |
| `energy_fraction` | `$0A4C` |
| `pending_whole` | `$0A50` |
| `energy_whole` | `$09C2` |
| `warning` | `$0A6A` |
| `protection` | `$18A8` |

Equipped suit bits in `$09A2` are Varia bit 0 and Gravity bit 5. `$0A78` is the
time-frozen context. These are production admission inputs, not a new owner for
equipment or time. Other equipment remains Run Progress's responsibility.

## Witnesses and reference

Private evidence is retained under
`$TEMP/collab/retained-time-479/health-spec-565/`: `observe.py`, `create.py`,
`reference.py`, `record_check.py`, source metadata and RAM mapping. Original text,
native records and source receipts remain outside exports. Source prerequisite
`timer-source-563/manifest.toml` is pinned
`8af8a5601abeefb1a17ccfd029014a3a8e27db3a557d83f01ec1b4030380bf9e`;
its independently reviewed `timer-source-review/REGATE.md` is pinned
`2f1a51e3bb9a89efde92f70e492ec4b05562bc8c0c45b074ac70586f79859986`.
The observation reconstruction validates the bank pin, 14 executing read groups and
eight persistent write bytes: two each for current whole/fraction energy and pending
whole/fraction loss. It rejects a poisoned physical energy read. No warning write occurs
in this reached interval.

R-maintenance-current observes the native `postRTI9232 -> postRTI9233` interval.
The selected row context is `chain-any-0003-DFD7`, whose selected case uses native
seed input and local end cut 1. No equivalence between that local cut and native
video numbering is inferred. Its before/after health snapshots were reconstructed at the maintenance
opportunities. They are also the State at the frame's two cuts: the interval runs
from one NMI return to the next, and the adjacent captures below show Maintenance is
the interval's only Health writer. So the vector is one whole frame. Both current
energy components are written even though the reached pending loss is zero. Scalar
output equality alone cannot establish completed work.

The periodic-loss and warning vectors are 16 model vectors (one recorded, 15
constructed); the file also holds three separate `IntegrationAdmission` contrasts. Every constructed expected result is
worked by the spec's arithmetic: borrowing one fractional unit from 31 whole gives
30 whole plus 65535 fractional units; equal fractions do not borrow; overspending
sets both components to zero. A second frame consumes zero, whereas restoration
before that loss repeats it. The reference uses a combined integer energy amount,
independent of the component arithmetic used to work the expected vectors.
It agrees with all 16 model vectors. The do-nothing comparison matches five;
constructed gain is 11, recorded scalar gain zero. Borrow, consumption and threshold
mutants are rejected. Five reference DTO omission contrasts distinguish all retained
quantities; complete clean-model restoration and faults still require implementation
and integration proof. Those DTO contrasts are not a completed model checkpoint claim.

The independently pinned recovered row cache covers local cuts 0..183: 184 records
and 183 eligible adjacent pairs. Reference health compatibility and null equality
both match all 183 pairs. This provides no gain and no proof of which frames admit Health; the reference
must not manufacture admitted frames from these records.

### Auxiliary Ceres contact and protected carry

R-tail-hit and R-tail-protected use row `chain-any-0006-E0B5` at cuts 992–994,
with the hit in the first interval. The immutable encrypted observer capsule is
`ceres-health-hud-observer-7fabae8cb45825a4.tar.gz.enc`, SHA-256
`7fabae8cb45825a4ed4cfc8ffd4584e3cce06782bafcbfd6bdd368befc2a3b13`.
Its acquisition manifest SHA-256 is
`2959332acb217e387f0d7b5d32cddef31e01b5e16d6028983a55403a300cde5f`.
The native access trace SHA-256 is
`cd926656dabb61c12f32ab5af50773756066cc102f912febc9dc0b88b770f6de`.
The capsule's independent controls realign all three full WRAM, CPU, latch and pad
cuts to the pinned cache and reconstruct 37 scoped writes. The later Health
reference read those same operands and compared the carried before/after result,
pre-Maintenance energy and Drawing values. The vectors keep the cut State and the
protection both Drawing markers read; the energy both Drawing markers read equals
the State at the frame's end. The reference agrees with both frames direct and
restored. The do-nothing comparison agrees with neither.

The file's tail and periodic vectors are 24 model vectors (3 recorded and 21
constructed) beside 3 admission contrasts; the steam's three are counted under the
hit's worth below. The whole-frame reference (below) matches all 24; the do-nothing
comparison matches 6. In the earlier staged reference the duplicate, fraction-clearing, early-age, omitted-age, stale-energy,
borrow, repeated pending-loss and warning-threshold controls fail. Trace controls
also reject stale protection at contact, wrong equipment at the actual impact,
a changed grant, a changed late count, stale Drawing energy and early timer
publication. The earlier Maintenance witness and the independent incoming-register
and caller-stack controls still apply. Restoration here is a reference check;
the clean model and actual production checkpoints must separately prove it.

### Adjacent Animation

On the recorded and held controller schedules, the four stores of the first
Maintenance are the last Health writers before the adjacent Animation. Both
captures preserve the five Health words and six context words across the prior
NMI, both current drawing interrupts and the returned no-hurt actor operation.
The adjacent actor reads the same invincibility word at +43,020 ticks.
Neither Animation branch reads whole energy. Its first physical read in the
adjacent interval occurs at Maintenance, +108,126 ticks after the native return.
This capture does not place animation within the frame. The bank logs do (the
frame's order above): animation runs in the new-state handler (`$90:E73B`), after
the main enemy routine (`$82:8B65`, Ridley's tail contact) and before periodic damage
(`$90:E74D`) and the low health check (`$90:E753`). So Appearance's animation reads
current energy after the frame's tail hit and before its periodic loss. On R-tail-hit
whole energy goes from 39 to 24 on the hit frame, across `Breathing`'s threshold of
30, and the original's animation sees 24. Health's one-frame step does not give back
that value; the gap list records the "energy as animated" output an integrated
Health-to-Appearance read needs. No vector checks it yet.

The private adjacent source manifest has SHA-256
`b3f99e3019a7c238f992ee5fb24aa25e686c7e068b3248972c3efd8339a1803d`.
The retained frame fixture pins the
origin and both controller schedules. Both source captures reconstruct complete
WRAM across both native returns. The checked controller
schedule and origin identify the finite borrowed context. Only the first
production Maintenance supplies the carried Health result.

## The whole-frame step

MODEL §1 decisions 35 and 36 leave intra-frame timing to T1. The spec now states
Health as one step per frame: the frame's tail impact and admission in, Health's
State as the frame leaves it, the warning sound and protection while drawn out. The
staged surface it replaces had three caller-sequenced entries within one frame (the
tail contact, Maintenance and the active close), which existed to stage Health for
the Ceres tail sequencer.

The catalogue keeps its four entries (PeriodicLoss, TailHit, ProtectionTime,
LowEnergyWarning) and their eight arms; none existed only for staging. Removed or
reworded text, and where each rule lives now:

- "A Maintenance call, a Drawing read, a repeated contact check or a display
  interrupt never spends a protection frame": removed; only the frame step spends
  protection, already stated at ProtectionTime's arm and "Reading State changes
  nothing" (Memory and boundary).
- "The optional tail impact and the completion of an active gameplay opportunity
  are distinct from Maintenance": removed; the tail impact is the frame's input
  (Memory and boundary) and spending protection is ProtectionTime's arm.
- "The model may always be queried for current energy": moved to "Reading State
  changes nothing" (Memory and boundary).
- Old "Order that matters", Ridley sees the protection carried into contact: moved
  to TailHit's arms ("as the frame starts") and the header's Reads.
- The hit takes energy and grants protection before Maintenance judges the warning:
  already stated at "Order that matters" (hit, loss, warning) and LowEnergyWarning's
  arms ("after the frame's hit and loss").
- Both Drawing reads and the late gauge see the maintained energy: moved to "Order
  that matters" (drawing and the gauge read the energy the frame leaves).
- Protection granted by contact stays live through Drawing and ages after it: moved
  to ProtectionTime's arm as the output protection while drawn.
- 95 after a hit, one remaining frame still blocks, a hit lands after expiry:
  already stated at TailHit's arms and "Order that matters".
- Animation can read current energy before periodic loss: moved to "Order that
  matters" (animation reads energy after the frame's tail hit and before its loss).
- Appearance's previous drawing energy and the flash after damage are not Health's:
  already stated at the header's "Leaves to others"; repeated in "Order that matters".

The value another domain read part-way through the frame, protection at Drawing, is
the named output protection while drawn. Energy at Drawing and at the gauge equals
the State the frame leaves, and Ridley and animation read the State as the frame
starts, so neither needs an output.

Vectors: 27 to 27 (24 model vectors, 3 admission contrasts). Dropped: C-tail-duplicate,
two eligible impacts within one frame, which tested only the staged contact entry
(the frame's tail impact is one input, and a hit already blocks any later overlap). Added in its place, converted to a whole
frame: C-tail-repeat, an eligible overlap on the frame after a hit (continuing
C-tail-after-expiry), blocked by the 95 frames left. Every tail vector's
`tail_impacts` list became the single `tail_impact`, its `drawing` pair became
`protection_drawn` (each drawn energy equalled its `after` energy), and
`before_maintenance_whole`, energy between the staged entries, was dropped; no
neighbour reads that value. The 16 periodic-loss and warning vectors and the 3
contrasts are unchanged.

Reference: `$TEMP/collab/health-wf-units/reference.py`, one step per frame on a
combined-integer energy, agrees with all 24 model vectors and checks every
`continues` chain; the do-nothing step matches 6 of 24. Mutants rejected
(`mutants.py`): ageing before the hit (17 of 24 pass), protection drawn after ageing
(17), the warning judged before the hit (20), and a hit ignoring protection (22).

## The hit's worth

The frame's impact is "absent, or an eligible hit worth `d` whole energy", `d` from
the hitter, so one input holds both Ceres hitters: the tail's hit worth 15 and the
steam's worth 0, each with the same protection. `TailHit` keeps its name for both; its
first arm subtracts the impact's `d`. The vectors write the impact as `impact_worth`,
omitted when absent: the tail vectors at 15, the steam vectors at 0.

The file has 32 vectors: 29 model vectors (4 recorded, 25 constructed) and 3 admission
contrasts. The steam's: R-steam-hit (recorded, above); C-steam-hit, a 0-worth hit at protection 0 on 31 whole
and 300 fractional units with 1 whole and 100 fractional units pending: the hit keeps
31 and 300 and the pending loss, loss leaves 30 and 200, the warning goes On with
Start, 96 while drawn and 95 at the frame's end; C-steam-protected, continuing it, a
0-worth impact with 95 left: nothing but the frame's ageing to 94, drawn 95. Two
more hold the worth to neither hitter's value: C-hit-worth-40 takes 40 from 52 to
leave 12, so the warning goes On with Start; C-hit-worth-most takes 32767 from 32767
and 9 fractional units, leaving 0 whole and keeping the 9. Both grant 96 while drawn
and leave 95.

Reference: `$TEMP/collab/health-worth/reference.py`, one step per frame on a
combined-integer energy, agrees with all 29 model vectors and checks every
`continues` chain; the do-nothing step matches 6 of 29. Mutants rejected: a worth
fixed at 15 (25 of 29 pass), a worth taken as 0 when 0 and 15 otherwise (27; only
C-hit-worth-40 and C-hit-worth-most reject it), a 0-worth hit granting no protection
(27) and a 0-worth hit skipped (27). Per frame from each cached record before (`records.py`), with an
impact worth 0 at the two puff hits and none elsewhere, it matches 168 of 168 admitted
pairs in `chain-any-0009-DFD7` 0-168 (do-nothing 136) and 111 of 111 in
`chain-any-0010-DF8D` 0-111 (do-nothing 3).

## Harness contract

Call the actual production health step once per admitted frame with carried complete
State, admitted by actual running-time and equipped-suit producers. Seed once from
the independent record. Compare at each frame cut whole/fraction current energy,
whole/fraction pending damage, warning and protection, and the warning request when
independently observed. There are no scalar masks. Do not seed from the oracle after
the first observation or feed HUD history back into health.

Warning audio playback is skipped until the audio producer/acceptance contract is
proved; only requests and warning State are health outputs. Whole-frame comparison
must name any other health writer in the frame and qualify its real owner exchange.
Appearance's previous-draw energy stays separate. Missing pending-loss accumulation,
pickups, alternate protection sources, reserves, refill or death remains Unsupported.
A scope ending before such a required neighbor does not claim that neighbor
implemented. All six scalar quantities need complete set/read/save/restore checks and
a checkpoint continuation over repeated frames. Never infer the zero-loss frame was
omitted because its final scalar values match its initial values.

For the steam hit, seed at `chain-any-0009-DFD7` cut 136 and play 137 with the frame's
impact worth 0, from the vents' owner or as a declared borrowed input with no actor
producer credit; compare complete Health State at 137 and on through the protection
the hit grants. Production hands Health the worth the hitter gives; it never derives
the worth from the hitter's identity inside Health.

For the Ceres tail frames, the frame's tail impact comes from Ridley's owner, or is a
declared borrowed input with no actor producer credit. Feed Ridley's contact the
protection as the frame starts. Seed once at cut 992, play 993 and 994 directly and
from a checkpoint after 993, and compare complete Health State and the warning
request at each cut, and protection while drawn where Appearance's drawing consumes
it. Wider actor behavior, physical Drawing output, the gauge and VRAM publication
require their own owners' comparisons.
