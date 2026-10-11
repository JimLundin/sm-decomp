# Ki-Hunter acid spit
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate CF90/CFA6 initializers and CFD5/CFE6/CFF8 callbacks, preserving
vertical-first collision order, wrapping coordinates and signed gravity cap.
Require actual-C full-WRAM allocation/collision evidence and run cleanup.

## Work evidence

CF90/CFA6 initializers and CFD5/CFE6/CFF8 callbacks translated. Full port suite
and strict Clippy pass. All 1,340 actual-C callback cases pass and authenticate:
out/port/ki-hunter-acid-callbacks/20261011T103414.978335Z/verdict.json.
Ordinary cleanup zero; deduplication 344,195,072 bytes. First allocation run
passed but included instruction bytes misidentified as a header; cleaned
(zero ordinary, 11,796,480 deduplicated bytes) and rerun with native CF18/CF26.
Ten additional cap/wrapping gravity cases pending. Retain ROM, source snapshots,
binaries and full-WRAM cases. Available disk 266,119,581,696 bytes.

## Completion evidence

Corrected native allocation: all 54 cases pass and authenticate after zero
ordinary cleanup and 11,796,480 bytes deduplication:
out/port/ki-hunter-acid-allocation/20261011T103604.222393Z/verdict.json.
All ten targeted gravity cap/wrapping cases pass and authenticate after zero
ordinary cleanup and 1,310,720 bytes deduplication:
out/port/ki-hunter-acid-gravity/20261011T103639.438532Z/verdict.json.
Combined with 1,340 callback cases, full port tests and strict Clippy, verifies
these routines offline. Retained ROM, snapshots, binaries and case inventories;
available disk 265,525,190,656 bytes. Live ownership separate; decomp still active.
