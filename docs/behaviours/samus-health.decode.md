# Current energy: decode notes

Observer, adversary and integrator evidence only; never part of a confined export.
The [spec](samus-health.md) describes player-visible quantities and deliberately
excludes source structure, hardware accounting and native coordinates.

## Sources by entry

Bank log [90](https://patrickjohnston.org/bank/90), pinned SHA-256
`0b1aa7a6e6e839d6dd3e96543509f6508620105f9448bf5854eba1568db66290`.

### PeriodicLoss

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

### ProtectionTime

`$A0:9169..9171` reads active protection and decrements it once when positive.
It executes after status in this acquired active Ceres window. Both Drawing
markers retain the count that the later timer close reads. At 993 the sequence is
begin 0, grant 96, Drawing 96, close 95. At 994 it is begin 95, Drawing 95, close 94.
The native bus witness includes the physical word load and read-modify-write
against the same word. The adjacent decrement for knockback is separate.
This source speaks for the active frame tail; it does not admit stopped time,
other grants or resets.

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
video numbering is inferred. Its before/after health snapshots are reconstructed
at the actual maintenance owner opportunities, not treated as frame endpoints.
Both current energy components are written even though the reached pending loss
is zero. Scalar output equality alone cannot establish completed work.

The vector file holds 16 model vectors (one recorded, 15 constructed) and three
separate `IntegrationAdmission` contrasts. Every constructed expected result is
worked by the spec's arithmetic: borrowing one fractional unit from 31 whole gives
30 whole plus 65535 fractional units; equal fractions do not borrow; overspending
sets both components to zero. A second opportunity consumes zero, whereas restoration
before that loss repeats it. The reference uses a combined integer energy amount,
independent of the component arithmetic used to work the expected vectors.
It agrees with all 16 model vectors. The do-nothing comparison matches five;
constructed gain is 11, recorded scalar gain zero. Borrow, consumption and threshold
mutants are rejected. Five reference DTO omission contrasts distinguish all retained
quantities; complete clean-model restoration and faults still require implementation
and integration proof. Those DTO contrasts are not a completed model checkpoint claim.

The independently pinned recovered row cache covers local cuts 0..183: 184 records
and 183 eligible adjacent pairs. Reference health compatibility and null equality
both match all 183 pairs. This provides no gain and no health opportunity selection
proof; the reference must not manufacture gameplay opportunities from these records.

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
reference reads those same operands and compares the carried before/after result,
pre-Maintenance energy and Drawing values. The reference agrees with both native
opportunities direct and restored. The do-nothing comparison agrees with neither.

The expanded set has 24 model vectors (3 recorded and 21 constructed) and 3
admission contrasts. The reference matches all 24; the do-nothing comparison
matches 6. The duplicate, fraction-clearing, early-age, omitted-age, stale-energy,
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

The private adjacent source manifest has SHA-256
`b3f99e3019a7c238f992ee5fb24aa25e686c7e068b3248972c3efd8339a1803d`.
The retained frame fixture pins the
origin and both controller schedules. Both source captures reconstruct complete
WRAM across both native returns. The checked controller
schedule and origin identify the finite borrowed context. Only the first
production Maintenance supplies the carried Health result.

## Harness contract

Call the actual production health entry with carried complete State, selected by
actual running-time and equipped-suit producers. Seed once from the independent
record. Compare whole/fraction current energy, whole/fraction pending damage and
warning, and warning request when independently observed. There are no scalar masks.
Do not seed from the oracle after the first observation or feed HUD history back into
health. Native executing write evidence is private causal qualification, not a clean
model input or an alternative production clock.

Warning audio playback is skipped until the audio producer/acceptance contract is
proved; only requests and warning State are health outputs. Whole-frame comparison
must name any intervening health writers and qualify their real owner exchanges.
Animation reads earlier current energy, Drawing later current energy; Appearance's
previous-draw energy stays separate. Missing pending-loss accumulation, pickups,
alternate protection sources, reserves, refill or death remains Unsupported. A scope ending
before such a required neighbor does not claim that neighbor implemented.
All six scalar quantities need complete set/read/save/restore checks and a
checkpoint continuation over repeated opportunities. Never infer the zero-loss
operation was omitted because its final scalar values match its initial values.

For the Ceres tail segment, source the eligible impact before applying any mutation
and carry Health through Maintenance, both later Drawing reads, status and the
actual active-frame close. The active close schedules one model ageing operation;
it cannot seed the value, use the zero-timer frame-tail profile, or substitute the
escape clock. Feed the same carried protection to actor contact. Seed once at
992, continue directly and from a checkpoint after 993, and compare complete
Health State, the warning request, energy and history at both Drawing opportunities,
and late gauge change and hold. Add scoped wrong controls for duplicate impact,
stale current energy and early ageing. If the post-adjustment amount is borrowed,
declare that scope and give no actor producer credit. Wider actor behavior, physical
Drawing output and VRAM publication require their own qualification. Keep the
indexed previous Maintenance and retained-time requirements.

## Budget

Fixed base: `5f47bb69187ed694464e7addc1813bd48cd0a133`. There is no existing H1 clean
model or periodic-loss production owner to replace; planned H1 rounds are not an
implemented baseline. An attributable old-code `tools size` baseline is unavailable
for this new owner; no zero-size receipt or passing size assessment is inferred.
Retained pickup and HUD models are neighboring responsibilities, not complete old
implementations of this bounded health area.

k = 37.5 weighted tokens per spec unit, the frozen coefficient.

- Area current-energy: PeriodicLoss, LowEnergyWarning
- Area protection: TailHit, ProtectionTime

Budget counter: four entries, eight arms, no cell terms or counted operators.
The counter reports 8.0 units and 299 weighted tokens after conversion by area
(current-energy 187, protection 112). A private std-only CLI wrapper calls the
byte-identical `verify::spec_units` implementation. The previous periodic Health
owner is retained and extended. At fixed base
`0ede0e619848bc630c24bf45c853136c276d49fe` the production Health and frame-tail
admission are limited to inactive protection. There is no implemented tail-hit
or active protection gameplay owner to count as replaced.
