# Crawler enemy movement and gravity closure
Status: claimed
Blocked by: 15
Triage: ready-for-agent

Translate bank A3 Stone/Fire Zoomer, Big Eye Bugs and Wrecked Ship Orange Zoomer
initialization, script preinstruction selection, horizontal/vertical crawling,
slope-adjusted displacement, earthquake falling and landing state restoration.
Use pinned actual C and native enemy terrain movement. Preserve integer widths,
aliasing, ordered PLM callbacks and script pointers; reject invalid ROM indexes
and dispatches. Own enemy_crawler.rs, tests/contracts and crawler documentation.

Verify complete WRAM, returns and callbacks at the existing approved family
seam. Include independent examples for corners, gravity carry and landing,
all orientations, movement misses/hits, earthquake and speed-table sentinel.
Record translated/offline/live status separately. Integrate public crate and
batch catalog, with no hidden gameplay fallback.

## Comments
Claimed on ticket22-crawler worktree based on decomp-port-integration 2161df6.
Uses the existing approved canonical full-WRAM/actual-C/ordered-PLM seam and TDD.

## Implementation
Implemented all sixteen crawler routines (including both script-pointer handlers)
and shared native A0 horizontal slope processing. Independent expected examples
verify corners, gravity carry/landing, all property orientations, sentinel,
earthquake/fourth miss, script aliasing and input rejection. BTS15 actual D094
PLM allocation is admitted through the existing explicit callback host.
