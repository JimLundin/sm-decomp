# Sprite object rendering
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate DrawSpriteObjects and shared base-tile spritemap OAM semantics,
descending ordering, signed clipping, wrapped coordinates, tile/palette words
and extension-bit OR behavior. Actual-C full-WRAM oracle and live verification;
retain failure evidence and complete scoped cleanup/authentication.

## Work evidence

DrawSpriteObjects translated with complete base-tile spritemap-to-OAM writes,
signed clipping, descending slot order, coordinate/tile wrapping, extension-bit
OR preservation and cursor wrap. Rendering has no C callback in Rust.
486 actual-C full-WRAM cases pass in
out/port/sprite-rendering/20261011T083848.006352Z/verdict.json.
Clippy requested as_chunks iteration; source changed and final oracle refresh
running via session 64748. Strict Clippy and existing sprite tests pass.
Initial missing sm_81.c support link failure retained at 083841.149528Z and
cleaned (zero bytes). Successful first oracle cleanup ordinary zero, dedup
109707264 bytes. Final oracle/live verification and authentication required.
Allocation full-movie run remains active via session 90788; do not restart.

Final oracle out/port/sprite-rendering/20261011T083906.671146Z passes all 486
cases and authenticates after cleanup (ordinary zero; dedup 109707264 bytes).
Full-movie four-process verification started via session 7959 under
out/port/sprite-rendering-live. Poll existing process before restarting.
Live verification and final cleanup/authentication remain required.

Independent synthetic two-entry full-WRAM rendering test passes: cursor 508
wraps to zero, signed sprite size/X bits OR into existing extension words,
coordinates truncate and tile addition wraps. All five sprite tests and strict
all-target Clippy pass. Live rendering run is
out/port/sprite-rendering-live/20261011T083926.700735Z, session 7959, still active.
Gameplay implementation unchanged by test addition.

First full-movie C rendering replay completed with 96336 DrawSpriteObjects
calls and zero CPU fallback; Rust ownership/parity awaits mixed subjects.
Existing live session 7959 remains active.

## Completion evidence

Full 129712-input four-process run:
out/port/sprite-rendering-live/20261011T083926.700735Z/verdict.json.
Status mixed-C-Rust-integration; all 990 state/effect artifacts match across
four subjects. Both mixed subjects execute 96336 DrawSpriteObjects calls
exclusively in Rust, matching C counts. CPU fallback zero in all subjects.
Original-ROM parity remains not tested. All 486 offline actual-C cases and
five independent sprite tests pass; strict Clippy, formatting and diff checks pass.
Fixture generator reproduces the exact oracle manifest.

Live cleanup --apply reclaimed zero bytes; --deduplicate --apply reclaimed
927521092 bytes. Automatic finished ROM/observation sharing reclaimed
4769723648 bytes. Each successful offline rendering run reclaimed 109707264
bytes; failed-link cleanup zero. All four artifact sets, binary, host snapshots,
transformed source and current oracle authenticate after cleanup. Gzip fully
decoded 4967833433 bytes, SHA256
ade76f110aa3833e707b549a07b22a746f359a8cb1a614f4bed4fed9d72621eb.
Summary: live storage-authentication.json. Free disk 274498740224 bytes.
No implementer worktree or relocation. Retain linked binary, sealed observations,
host/transformed sources, oracle, accepted predecessor evidence and ROM/movie pins.
Allocator live ownership remains open under 94; this closes rendering only.
