# Ceres elevator pad and Samus release
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate native A328 pad timer and movement, including actual SamusCode_0E
frame-handler transition. Preserve signed wrapping and zero-timer short circuit.
Verify all slots and boundary values against full actual-C WRAM, then fresh
full-movie C/mixed ownership and parity. Clean and authenticate evidence.

## Work evidence

Implemented A328 timer/movement and complete SamusCode_0E plus
CallSomeSamusCode transition clearing. Independent full-WRAM Rust test checks
wait, zero/one/signed-high timer cases, height wrapping and invalid slots.
All 1296 actual-C full-WRAM cases pass:
`out/port/ceres-pad/20261011T081730.543614Z/verdict.json`.
Initial failed oracle retained at 20261011T081651.271247Z: exposed missing
CallSomeSamusCode clearing at A28, corrected before final oracle.
Failed-run cleanup --apply and --deduplicate --apply reclaimed zero bytes.
Cargo tests, strict all-target Clippy and live tooling tests pass.
Full-movie four-process live verification is running via exec session 45174
under out/port/ceres-pad-live; poll existing process before restarting.
Final live verification, cleanup and authentication remain required.

Refreshed current-lib offline oracle after adding sprite module:
out/port/ceres-pad/20261011T082103.180389Z/verdict.json (1296 cases pass).
Previous final offline cleanup reclaimed 233570304 bytes and authenticated
before the lib module addition. Running live binary is frozen and independent
of the new module. First full C subject finished with 528 pad calls and zero
CPU fallback; second C subject is running. Existing session remains 45174.

Current-harness offline oracle passes and authenticates after cleanup:
out/port/ceres-pad/20261011T082330.511009Z/verdict.json.
Ordinary cleanup reclaimed zero bytes; dedup reclaimed 233570304 bytes.
Refresh was required because the sprite harness fix changed family_harness_sha256.
Both previous refreshes remain historical source-bound evidence, not current seals.

## Completion evidence

Full 129712-input four-process run:
out/port/ceres-pad-live/20261011T081814.286687Z/verdict.json.
Status mixed-C-Rust-integration. All 990 state/effect artifacts match across
four subjects; both mixed subjects execute 528 exclusive Rust pad calls and
zero C pad calls. Both C subjects execute 528 C pad calls and zero Rust.
All subjects report zero CPU fallback. Original-ROM parity remains not tested.

Final current-source oracle is 20261011T082330.511009Z (1296 passing cases),
authenticated after cleanup. Full crate tests, independent boundary tests,
strict all-target Clippy and live tooling tests pass.

Live cleanup --apply reclaimed zero bytes; --deduplicate --apply reclaimed
922479990 bytes. Automatic finished ROM/observation sharing reclaimed
4769723648 bytes. Each of the three successful offline oracle cleanups reclaimed
233570304 bytes (ordinary zero); failed oracle cleanup reclaimed zero.
After cleanup four full artifact sets, binary, host snapshots, transformed C
and predecessor immutable seal authenticate. Current-source oracle authenticated
separately because shared lib/harness additions postdate the original predecessor.
Gzip fully decoded 4967833433 bytes, SHA256
ade76f110aa3833e707b549a07b22a746f359a8cb1a614f4bed4fed9d72621eb.
Summary is live storage-authentication.json. Disk free 276449165312 bytes.
No implementer worktree created; no relocation required. Retain original and
refreshed offline evidence, binary, host/transformed sources, sealed observations,
accepted decompressor/RNG predecessors and pinned ROM/movie identities.
