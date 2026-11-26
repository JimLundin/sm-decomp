# Play time

The elapsed play time the player can see and keep with a run. Vectors:
[game-time.toml](../../scenarios/vectors/game-time.toml) and
[recorded Ceres cases](../../scenarios/vectors/game-time-ceres.toml).

## Header

**Scope.** One advance of play time at a qualified gameplay opportunity.
The clock has a complete rule for all legal times. Production qualification
covers the active Ceres arrival, ordinary room play, and the locked shaft
passes through the final room work. The same clock carries across room passages
and into the escape save.

**Reads.** The gameplay scheduler supplies the current ordinary opportunity.
[Health](samus-health.md) supplies current whole energy after same-frame
maintenance for admission. The clock rule takes a qualified opportunity and its
own complete time.

**Leaves to others.** [Run Progress](run-progress.md) owns pickups and progress.
[Door passage](world-doors.md) and the gameplay scheduler own when time can
advance. Health owns reserve, refill and death decisions. Display formatting,
save-slot persistence and the other gameplay contexts for advancing or
stopping time are outside this rule. The selected fresh file supplies zero
elapsed time. Other gameplay
counters and hardware timing have their own owners.

## Memory and boundary

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| hours | `PlayHours`, 0..99 whole hours | PlayTimeAdvance, output |
| minutes | `PlayMinutes`, 0..59 minutes within the hour | PlayTimeAdvance, output |
| seconds | `PlaySeconds`, 0..59 seconds within the minute | PlayTimeAdvance, output |
| frames | `PlayFrames`, 0..59 clock frames within the second | PlayTimeAdvance, output |

These four quantities are complete clock State. A caller can set any legal
combination, read it back, carry it forward, and restore it into a fresh clock.
Each quantity is independently observable and retained, including the fraction
of a second the player has already earned. The model returns the complete clock
after one qualified opportunity. It is total across the stated ranges.

The qualified Ceres work uses positive whole current Health and a legal
clock. It includes the ordinary carry from frame 59 into the next second
[R-ceres-first-carry]. The minute, hour and ceiling cases retain their existing
model proof. Advancing from 99:59:59.58 to 99:59:59.59 is in scope
[C-ceiling-arrival]. An attempt to advance from 99:59:59.59 requires
the ceiling arm, which is outside this production scope. In the supported
scope Health remains unchanged by PlayTimeAdvance. An
unqualified request for the ordinary advance must surface Unsupported before
any clock effects. A frame where the scheduler has established there is no
opportunity makes no clock call. A required but unestablished opportunity
remains Unsupported.

## Rules

### PlayTimeAdvance

- kind: mechanic
- summary: one ordinary opportunity earns one frame on the clock, capped at the maximum.
- params: none
- switch: fixed
- Qualified opportunity with frames below 59 → add one frame and keep seconds,
  minutes and hours [R-ordinary-advance, C-independent-a, C-independent-b,
  C-continued-after-restore, C-ceiling-arrival].
- Qualified opportunity at frame 59 with seconds below 59 → set frames to zero
  and add one second. Keep minutes and hours [C-second-carry].
- Qualified opportunity at frame 59 and second 59 with minutes below 59 → set
  frames and seconds to zero and add one minute. Keep hours [C-minute-carry].
- Qualified opportunity at frame 59, second 59 and minute 59 with hours below
  99 → set frames, seconds and minutes to zero and add one hour [C-hour-carry,
  C-last-hour].
- Qualified opportunity at 99:59:59.59 → keep the entire time at
  99:59:59.59 [C-ceiling, C-ceiling-again].
- checked: oracle (R-ordinary-advance, R-ceres-first-carry); constructed
  (other carries and saturation).

## Order that matters

The reached opportunity uses Health completed for this same gameplay frame.
The scheduler advances the clock once before the completed clock for that frame
can be observed. Each independent vector begins from its own full time. A
continuation uses the previous result or a fresh restoration of that same
complete result. Other counters and hardware refreshes supply no opportunities.

### Ceres work opportunities

- The first-file arrival advances once on each of its 30 active room turns,
  including the turn that hands on ordinary play. Preparation holds the clock.
  The next ordinary room turn continues from the carried time
  [R-ceres-arrival-work].
- A fresh ordinary room turn advances once, including a turn that touches a
  door or reaches the elevator. Work during the door passage holds play time.
  The first ordinary turn after the passage advances the carried clock once
  [R-ceres-return-clock, R-ceres-shaft-resume].
- A fresh locked shaft turn advances once during the wait and fade. The final
  room turn earns time while publication is pending. Vblank-end work, the
  interrupted escape save, and its publication continue that turn without a
  second advance [R-ceres-last-room-work].
- Restore the whole clock with the scheduler's work state. A resumed turn
  keeps the advance it already earned [R-ceres-last-room-work].
