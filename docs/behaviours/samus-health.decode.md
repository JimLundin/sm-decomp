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
The [retained fixture loader](../../tools/ci/fetch_retained_frame.sh) pins the
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
protection freshness, reserves, refill or death remains Unsupported. A scope ending
before such a required neighbor does not claim that neighbor implemented.
All five scalar quantities need complete set/read/save/restore checks and a
checkpoint continuation over repeated opportunities. Never infer the zero-loss
operation was omitted because its final scalar values match its initial values.

## Budget

Fixed base: `5f47bb69187ed694464e7addc1813bd48cd0a133`. There is no existing H1 clean
model or periodic-loss production owner to replace; planned H1 rounds are not an
implemented baseline. An attributable old-code `tools size` baseline is unavailable
for this new owner; no zero-size receipt or passing size assessment is inferred.
Retained pickup and HUD models are neighboring responsibilities, not complete old
implementations of this bounded health area.

k = 37.5 weighted tokens per spec unit, the frozen coefficient.

- Area current-energy: PeriodicLoss, LowEnergyWarning

The unchanged `verify::spec_units` counter reports two entries, five arms, no cell
terms or counted operators: 5.0 units and a reported budget of 187 weighted tokens.
The coefficient product before the counter's integer conversion is 187.5. Its current
CLI dependency graph cannot build at the fixed base because the unrelated HUD event
shape fails compilation. A private std-only CLI wrapper calls byte-identical copies
of that counter and `verify::size`; wrapper, copied-source and DLL hashes are retained
with the private evidence. This is the existing counter implementation, not a new
counter. Budget material stays out of independent review copies.
