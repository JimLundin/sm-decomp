# Animated tile allocation
Status: claimed
Blocked by: None
Triage: ready-for-agent

Translate native bank-87 enable/disable, clear and descending six-slot allocation
with real ROM header reads. Verify full WRAM against actual C and prove live
ownership before extending the interpreter/frame closure. Scoped cleanup required.

## Work evidence

Native enable/disable/clear and SpawnAnimtiles translated with descending
six-slot search and real header list/size/VRAM reads. All 43 actual-C full-WRAM
cases pass at out/port/animated-tiles/20261011T085332.485292Z/verdict.json.
Independent full-WRAM allocation/refusal test and strict all-target Clippy pass.
Offline cleanup --apply zero bytes; --deduplicate --apply 11403264 bytes;
current evidence authenticates afterward. No implementer worktree created.
Full-movie four-process live verification active via session 32896 under
out/port/animated-tiles-live. Poll existing process before restarting.
Live results, final cleanup and authentication remain required. Instruction
interpreter/frame handler remain follow-up closure, not translated by this ticket.

## Completion evidence (partial live scope)

Full-movie four-process run passed:
out/port/animated-tiles-live/20261011T085347.505558Z/verdict.json.
All 998 artifacts match; both mixed subjects execute 4 enables and 64 clears
exclusively in Rust. Disable and SpawnAnimtiles have zero movie hits, so required
allocator live ownership is unproved and ticket remains claimed. CPU fallback
zero. Original-ROM parity not tested.
Live cleanup ordinary zero bytes; dedup reclaimed 926787303 bytes. Current-source
43-case oracle refreshed at out/port/animated-tiles/20261011T090908.318328Z;
cleanup ordinary zero, dedup 11403264 bytes. No implementer worktree created.
Post-cleanup run authentication is active; retain frozen live binary, original
and refreshed oracles, source snapshots and all sealed observations.

All four artifact sets, binary, host snapshots, transformed sources and refreshed
current oracle authenticate after cleanup. Gzip decoded 4967833433 event bytes,
SHA256 ade76f110aa3833e707b549a07b22a746f359a8cb1a614f4bed4fed9d72621eb.
Summary live storage-authentication.json; automatic sharing reclaimed 4769723648
bytes. Disk free 272617402368 bytes. Spawn/disable live coverage remains open.
