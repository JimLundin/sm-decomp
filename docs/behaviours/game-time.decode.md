# Play time: decode and harness

Companion to the [play-time spec](game-time.md). Keep these notes outside confined
exports.

## Sources

### PlayTimeAdvance

Source: [Bank 82](https://patrickjohnston.org/bank/82), `$82:8BA0..8BB3`
for the caller, `$82:DB69..DC0F` for positive whole Health, the frame/second/minute
carries, and saturation. For all legal values the comparisons after increment
are ordinary numerical threshold tests. Crossing 59 carries into the next
component. Crossing 99 hours writes 99 hours and 59 to all three lower digits.
Return from the legal ceiling therefore leaves the same four observables.

The retained first CPU frame uses accepted S6 remainder v2 and the final
history v3 private observer packet. Pin the v3 checker to SHA-256
  `e09af7f35f4b40e79ae6495d12a242f7b57545920eb83ad76657f0fc3611bb84`.
Its accepted physical reference is `feb96d3dd93c1bf49ba859cb622568ebd7c65787`.
The accepted v3 review identifies both handled IRQ ranges and the earlier
Health writer. The bank log supports the constructed cases beyond this trace.

## RAM mapping

All values below are complete unsigned little-endian observations. Health enters
the reached branch as a positive signed whole-energy word.

| Quantity | Source | Retained opportunity |
| --- | --- | --- |
| play-time frames | `$7E:09DA` | read at 6746, full increment store at 6749 |
| play-time seconds | `$7E:09DC` | carried untouched |
| play-time minutes | `$7E:09DE` | carried untouched |
| play-time hours | `$7E:09E0` | read at 6752, below 100, carried untouched |
| current whole Health | `$7E:09C2` | Health Maintenance store at 4247 supplies read at 6743 |

Indices are private callback coordinates. Before the increment, late HUD and
background work and the current room handler complete. The Health read tests
the just-completed owner`s value. The next room-shake opportunity follows the
advance, and the caller later requests the display wait. The positive Health
word lies in 1..32767. Reserve transfer, refill, death and signed-negative
Health lie outside the reached branch. Both independent HUD and Drawing clocks
belong to earlier controller/NMI work.

## Harness contract

R-ordinary-advance observes row `chain-any-0003-DFD7` in the retained
`postRTI9232 -> postRTI9233` interval. It uses the corrected native-origin seed
and local end cut 1. The vector takes the clock before the read at 6746 and
after the store, at 6752. The corrected origin has the same complete before
clock. The last pre-Wait snapshot at 6999 has the same complete after clock.
The pinned checker proves source widths, same-frame Health, and sole writers
through the corrected frame, including both handled IRQs. Seconds and minutes
are admitted from the origin and preserved without reads at the opportunity.
Observed values are witnesses. Schedule from the current qualified gameplay
work and Health, and seed the complete independent clock only at admission.

Compare four components in full. There are no masks. Compare complete admitted
and post-advance State and check one actual production opportunity before the
wait. A fresh full restoration must give the same complete State and next
result. Check different legal starts and carry the post-advance result across
subsequent qualified opportunities. A skipped or repeated increment changes
the frame observation in the retained witness. Use an independent expected
provider. Independently admit C-ceiling-arrival with positive current whole
Health and an established ordinary opportunity. The advance from 99:59:59.58
to 99:59:59.59 has no carry and must succeed, preserving Health and the
other clock components. A further required opportunity from that maximum
must surface Unsupported. The full model carry and ceiling are covered by
constructed vectors. Runtime requests for carry, unestablished scheduling, invalid
clock state, or nonpositive or stale Health at this required opportunity
remain Unsupported before clock effects. Save-slot encoding, stopped time
scheduling and additional original gameplay contexts are outside the witness.

The reference evaluates a single saturated count of earned frames and decomposes
it back to all four quantities. It agrees with all 11 vectors. The retained
record agrees at the qualified opportunity with gain one over a frozen clock.
Constructed vectors agree 9/9 against a frozen clock`s 2/9. Omitted and doubled
advances each fail eight comparisons, including the retained one. The two
continuations restore from complete reference results. Each of the four
single-component snapshot omissions is rejected. These are reference checks.
Production scheduling, production checkpoint restoration and indexed replay
remain integration obligations.

## Budget

Fixed base `67ed6b7b591ba0b586a10149147b1044d57b37bc`. There is no existing
production gameplay-time owner to replace, so there is no attributable
old-code size baseline.

k = 37.5 weighted tokens per spec unit, the frozen coefficient.

- Area clock: PlayTimeAdvance

The spec-units counter reports one entry, five arms, no cell terms or counted
operators, 5.0 units and a reported budget of 187 weighted tokens.
