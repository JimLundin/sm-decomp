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
constructed vectors. The extended ordinary Ceres trace proves the first second carry and
qualifies the work listed below. Other unestablished scheduling, invalid clock
state, and nonpositive or stale Health remain Unsupported before clock effects.
Save-slot encoding remains in the save owner.

The reference evaluates a single saturated count of earned frames and decomposes
it back to all four quantities. It agrees with all 11 vectors. The retained
record agrees at the qualified opportunity with gain one over a frozen clock.
Constructed vectors agree 9/9 against a frozen clock`s 2/9. Omitted and doubled
advances each fail eight comparisons, including the retained one. The two
continuations restore from complete reference results. Each of the four
single-component snapshot omissions is rejected. These are reference checks.
Production scheduling, production checkpoint restoration and indexed replay
remain integration obligations.

## Ceres arrival, return and departure work

| Case | Source and comparison |
| --- | --- |
| R-ceres-first-carry | `chain-any-0001-DF45` cuts 28 to 29. The complete clock changes from 00:00:00.59 to 00:00:01.00 during ordinary play with positive whole Health. |
| R-ceres-arrival-work | `any-ceres-arrival` cuts 0 to 222 hold zero through preparation and then earn 30 opportunities in state 7, including the turn that publishes state 8. The next row's cut 0 is the next ordinary turn. |
| R-ceres-return-clock | `chain-any-0006-E0B5` cut 1610 advances once on the touch. Cuts 1611 to 1776 hold 00:00:39.54, including D1's five ready/publish pairs. `chain-any-0007-E06B` cut 0 resumes at 00:00:39.55. |
| R-ceres-shaft-resume | `chain-any-0010-DF8D` cuts 221 to 278 hold at 00:00:47.55 after the touch. Shaft `chain-any-0011-DF45` cut 0 advances to 00:00:47.56. Its cuts 164 and 165 separately qualify the reach turn and first locked turn. |
| R-ceres-last-room-work | Shaft cuts 238 to 242. The fresh room pass earns 00:00:51.55 at cut 239, while the HUD and finished-frame counts hold. Cuts 240 through 242 hold elapsed time through the save, final publication and dispatch. |

The clock uses `$82:8BA0..8BB3` after the room's gameplay work, including the
locked shaft work from `$82:8367..83A6`. Native game states and NMI cuts
distinguish arrival preparation, active arrival, ordinary work, passage and
locked work. The late overrun is inside `$80:82CE` after the room pass.
The subsequent save work at `$81:8066` and requested publication do not
run the elapsed clock. The scheduler already qualifies completed work
separately from NMI publication. Do not use record clock values as input.

The observer reference compares the complete clock through arrival and all
eleven consecutive room rows. It counts 3,115 advances from zero through
shaft cut 239, checking positive whole Health on advances, every ordinary
door touch, all passage holds, and each adjacent row boundary. The recorded
schedule tables are integration witnesses. The Ceres case file keeps the recorded first carry beside the schedule. The
existing clock model tests cover the same second carry arithmetic with
C-second-carry. Production must compare the
entire carried clock against native readback, including after restoration of an in-flight shaft turn and before P2 captures
the escape save.

The source mapping in `game-time-ceres.sources.toml` pins each fixture ID and the
complete native row sequence. The production verifier must register an indexed
`game-time/ceres-continuous-clock` case and a `played::play_time` consumer for
all four components at every actual cut across that sequence, including held
cuts and row changes. Start with the selected file's actual clock. Use the
native clock as readback only. Require 3,115 scheduler advances, with one per
qualified work opportunity and none during preparation, unready work, resumed
save or publication. At 239 the incoming locked work is fresh and the native
overrun has reached the completed room work's vblank wait. This combines a
prior work boundary and the authenticated interrupted site. NMI kind alone
does not establish that work ran.

Restore the actual production checkpoint at cut 238 before work, at cut 239
before the save payload, and at cut 240 before save completion. Compare the
complete clock and next output on each continuation. Plant skip and duplicate
controls on the ordinary carry and final ready room turn. Plant a second
advance on resumed cut 240 and on publication cut 241, including after
restoring cuts 239 and 240. Keep the first differing clock field and prove the
healthy continuation again. The observer audit checks native inputs and
arithmetic. These production controls and checkpoint receipts remain required
integration evidence.

## Budget

Fixed base `67ed6b7b591ba0b586a10149147b1044d57b37bc`. There is no existing
production gameplay-time owner to replace, so there is no attributable
old-code size baseline.

k = 37.5 weighted tokens per spec unit, the frozen coefficient.

- Area clock: PlayTimeAdvance

The spec-units counter reports one entry, five arms, no cell terms or counted
operators, 5.0 units and a reported budget of 187 weighted tokens.
