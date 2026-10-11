# Escape room setups
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate AfterSavingAnimals, AutoDestroyWallAfterEscape and TurnWallIntoShotblocks
room callbacks with complete hardcoded PLM behavior and earthquake writes.
Translate missing BB09 PLM initializer. Verify actual-C full WRAM for occupied
and free tables and valid room widths. Complete cleanup/authentication; offline
scope remains separate from live movie ownership.

## Completion evidence

Three room callbacks and BB09 event-gated PLM initializer translated. All 54
current-source actual-C full-WRAM cases pass:
out/port/escape-room-setups/20261011T094912.776196Z/verdict.json.
Covers rescued event unset/set, full/low/high free PLM slots and widths 16/32/64.
Independent test verifies event-gated allocation and unconditional earthquake.
Full port suite and strict Clippy pass. Offline only; no live ownership claim.
Earlier 54-case run 20261011T094810.999108Z cleaned ordinary zero and dedup
13369344 bytes; historical source seal after map/quake additions. Refreshed
run cleanup ordinary zero, dedup 13369344 bytes; all cases authenticate afterward.
Free disk 268072321024 bytes. Retained source/binary/pinned ROM dependencies;
no implementer worktree created. Branch decomp-port-integration.
