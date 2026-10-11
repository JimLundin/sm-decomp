# Original movie controller boundary alignment
Status: open
Blocked by: None
Triage: ready-for-agent

Raw video-frame inputs diverge from original-ROM update timing before Ceres
frame9000. Resolve the bounded controller seam before claiming original movie
progression. Preserve prelude polls, waits and input edges; no corrective state
injection or later-state matching.

## Existing diagnostic evidence

`docs/DECOMP_PREFIX.md` and
`out/decomp/ticket07/20261010T182149.168780Z` retain native input-register
reads, timeline, full completed states and changed-gameplay control. First
unsupported boundary is original prelude poll frame19, before native main
dispatch. First compared mismatch frame272: original controller words0 vs
C0080, while game_state1/RNG758 match.

Pinned runtime removes original NMI ReadJoypadInputs at8095E1. Replaying every
poll as RtlRunFrame therefore cannot preserve the original schedule. Investigate
actual ReadJoypadInputs entry/return with hardware register samples separately
from completed update/continuation boundaries; avoid silently dropping polls.

Legacy `out/sync/full-any-lsnes.csv` indicates original frame43000 still has
health62/roomCF80 while raw C replay is in gameover. Legacy capture metadata
lacks current provenance seals and is diagnostic only. Current replay evidence
is tracked in ticket73; original-ROM parity remains unproven.

## Verification required

Reproduce bounded320-frame diagnostic. Establish an explicit controller/dispatch
mapping that handles frame19 and frame272 without altering gameplay state.
Require read-only original-ROM observer, sealed assets/harness/binaries, changed
gameplay rejection control and cleanup. Extend through first Ceres divergence
only after bounded mapping is sound.
