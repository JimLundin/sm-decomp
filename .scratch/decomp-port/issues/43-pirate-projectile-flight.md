# Pirate laser and thrown claw script flight
Status: resolved
Blocked by: 34
Triage: ready-for-agent

Translate A050 script callback setup and A05B/A05C/A07A/A0D1/A124 movement.
Preserve unchanged continuation pointer, camera-relative signed bounds,
laser speed flag and claw reversal. Host-global spawn coordinates remain open.
Verify actual C instruction/preinstruction/frame seams and clean runs.

## Completion evidence

Implemented A050 callback setup with unchanged operand pointer, no-op A05B,
left/right laser speed-flag motion and thrown-claw reversal/gravity. Camera
bounds retain signed wrapping comparisons and edge deletion. Initializers using
host-global spawn point/rectangle remain explicitly unsupported.

Actual pinned C full-WRAM/raw-return comparison passes 644 cases (four callback,
320 preinstruction, 320 scheduled frame) at
`out/port/pirate-projectile-flight/20261011T035343.310192Z/verdict.json`.
Independent Rust test verifies claw reversal and camera wrapping. Full port suite,
fmt and strict Clippy pass. Offline fixture equivalence only; live ownership open.

Cleanup: scoped `port_cleanup.py --apply` reclaimed zero bytes;
`--deduplicate --apply` reclaimed 191,758,336 snapshot bytes. No implementer
worktree created. Retain synthetic script ROM at
`out/port/pirate-flight-fixtures/pirate.sfc`, immutable source/run dependencies;
final verdict authenticated after cleanup. 35 GiB free.
