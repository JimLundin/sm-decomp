# Spore spawner positioning
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate DCD4 four spawner positions and DCC3 stalk head tracking; verify
actual-C full-WRAM allocation/callback cases and finish run cleanup.

## Completion evidence

Existing DCD4 implementation retained; added direct native DCC3 stalk tracking.
All 12 allocation and 72 callback full-WRAM cases match pinned C. Port suite
and strict Clippy pass. Current verdicts:
out/port/spore-spawner-allocation/20261011T111132.686220Z/verdict.json
out/port/spore-stalk-tracking/20261011T111131.382886Z/verdict.json
Ordinary cleanup zero; dedup respectively 2,752,512 and 18,350,080 bytes;
both authenticate afterward. Earlier allocation and incorrect dispatcher probe
cleaned (2,752,512 dedup bytes, zero ordinary); failure diagnostic preserved.
Retain ROM, binaries, snapshots and full-WRAM cases. Free disk
263,581,659,136 bytes. Offline equivalence only; whole-decomp remains active.
