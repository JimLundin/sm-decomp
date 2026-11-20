# Quiet room quake: decode and harness

Companion to the [quiet quake spec](world-quake.md). Keep these notes outside
confined exports.

## Sources

### QuietQuakePhase

Source: [Bank 86](https://patrickjohnston.org/bank/86), `$86:8427..846A`
for the earlier enemy projectile Drawing's quake displacement query. The
zero-duration branch returns zero displacement on both axes. Its positive
duration path can derive displacement from duration and content. The phase
has no read in Drawing here.

Source: [Bank A0](https://patrickjohnston.org/bank/A0), `$A0:8687..8711` for
the room update. At zero duration it skips the countdown, displacement edits,
and enemy shake requests. It still increments the phase with full-width wrap.
Active branches can change the background scrolls, decrement the duration and
affect active enemies. Which active cases do so, quake content, pause, expiry,
and any other room producer remain outside the admitted update.

The accepted final-history v4 source packet pins checker SHA-256
`754f50ef1458ed80b4d3ec22234895a55e05a0904e049d1d9ff386f4ada55d48`.
Its fixed physical reference is `feb96d3dd93c1bf49ba859cb622568ebd7c65787`.

## RAM mapping

| Quantity | Source | Retained opportunity |
| --- | --- | --- |
| remaining | `$7E:1840` | whole little-endian count, read by Drawing at 6395 and the room update at 6763, never written in the interval |
| phase | `$7E:1842` | whole little-endian count, read and written only at the room update's increment at 6765 |

The table uses native callback coordinates. The corrected origin and the
pre-increment snapshot agree on both quantities. Near frame end, the game
clock opportunity precedes the room update. Hurt timers and actor-list
termination follow it. The v4 bind proves that the actual guard uses the
admitted duration and that both IRQ histories preserve both quantities. The
sole phase read in the interval is the increment's read-modify-write. Drawing
queries the duration only. Give it the earlier current duration. The quiet operation leaves the room
positions and any existing actor effects to their current owners.

## Harness contract

R-quiet-frame observes row `chain-any-0003-DFD7` in the retained
`postRTI9232 -> postRTI9233` interval, corrected native seed and local end cut
1. Sample both quantities at the start, immediately before the increment at
6765 and after it by 6769. Both post-increment components persist at the last
pre-Wait snapshot at 6999. Compare both quantities in full without masks.
Seed once from independent before values and compare the actual production
update with independently observed after values.

Bind each actual scheduled query and update to the current room's owners.
Give Drawing its quiet displacement from the current valid duration. Only
the actual room update changes the phase. Require one actual update, including
at wrap. Compare a distinct independent initial phase, chained opportunities
and a fresh restoration carrying both fields. Reject A-active-duration before
a write or publication and preserve both quantities across the rejection and
restoration. Actual displacement from another owner, other actor producers,
and active quake work are excluded from the quiet scope. A required unknown
context stops before any dependent effect.

The reference observes one changed phase in one matched retained vector
against zero for a frozen phase. Duration remains zero. It matches five
constructed updates, including the wrap and both continuations, and both
omission and duplication fail on all six updates. The active-duration probe
is a boundary check. The quiet picture earns no output gain. Production
scheduling, fresh adapter restoration, and indexed replay remain separate
integration checks.

## Budget

Fixed base `d41c8769aaa13fcf7301f4be9c0d3a54d92ba9cd`. No attributable old
production quake owner is available for this area.

k = 37.5 weighted tokens per spec unit, the frozen coefficient.

- Area quiet-quake: QuietQuakePhase

The spec-units counter reports one entry, one arm, two counted operators and
no cell terms. This gives 1.7 units and a reported budget of 62 weighted tokens.
