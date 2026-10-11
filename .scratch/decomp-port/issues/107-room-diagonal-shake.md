# Room diagonal shake transitions
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate both native horizontal/diagonal shake callbacks, preserving timer
decrement, RNG threshold, aliased room state and complete fourth-frame explosion
caller behavior. Verify actual-C full WRAM, tests, cleanup and authentication.

## Completion evidence

Both callbacks translated with unsigned countdown, native RNG thresholds,
42-frame burst reset, normal/strong state alias differences and existing Rust
fourth-frame explosion caller. All 240 actual-C full-WRAM cases pass:
out/port/room-diagonal-shake/20261011T095059.040077Z/verdict.json.
Covers countdown 0/1/2/42/FFFF, six RNG seeds and freeze/frame gates. Independent
test verifies timer expiry and RNG burst while frozen. Full port suite, strict
Clippy and whitespace check pass. Offline scope only; no live ownership claim.
Ordinary cleanup zero; dedup reclaimed 49545216 bytes; all cases authenticate
afterward. Free disk 270163480576 bytes. Retained binary/source/pinned ROM
dependencies; no implementer worktree created. Branch decomp-port-integration.
