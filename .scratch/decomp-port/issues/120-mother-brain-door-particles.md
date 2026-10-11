# Mother Brain escape door particles
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate C961 eight particle initializers and C9D2 complete lifecycle, including
signed horizontal deceleration, wrapping gravity/motion, signed timer expiry,
host point update and native dust allocation. Verify actual C and clean runs.

## Work evidence

C961/C9D2 complete lifecycle translated. Full port suite and strict Clippy pass.
All 24 native allocation cases pass and authenticate after ordinary cleanup
zero and deduplication 5,898,240 bytes:
out/port/mother-brain-door-particle-allocation/20261011T102517.402248Z/verdict.json.
Initial motion run rejected the old host field name; preserved diagnostic,
cleaned with zero bytes reclaimed. Corrected 487-case motion run pending.
Retain source snapshots, ROM, binaries and full-WRAM/host inventories.

## Completion evidence

All 487 actual-C motion/native instruction cases pass including full WRAM,
returns and host state:
out/port/mother-brain-door-particle-motion/20261011T102722.820481Z/verdict.json.
Finished run cleanup ordinary zero; deduplication 120,717,312 bytes. Evidence
authenticates after cleanup. Available disk 267,265,019,904 bytes. Combined
with 24 initializer cases, full port tests and strict Clippy: complete offline
translation of these callbacks. Live ownership remains separate.
