# Native Ceres elevator platform preinstruction
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate A364 pure platform preinstruction, observed524 actual C calls in88.
Preserve exact samus_y_pos equality72 timer/list writes and canonical projectile
slot validation; expose direct symbol and scheduler route. Actual-C WRAM oracle
and independent positive/negative/invalid-slot checks. Coordinate direct live
allowlist after source freeze, require fresh exclusiveRustnonzero measured hits
for live ownership. No pad A328 SamusCodecallback or nativeallocator claim.

## Offline implementation evidence

A364 preinstruction and direct `EprojPreInstr_CeresElevatorPlatform` void(uint16)
dispatch implemented. Exact global samus_y_pos equality72 writes only caller slot
instruction timer1 and listA28B. Other heights preserve fullWRAM. Canonical18-slot
validation occurs before writes. No allocator or A328 pad callback translation.

378 actual-C cases authenticated in
`out/port/ceres-platform/20261011T072815.113664Z/verdict.json`.
Every18 slots ×7 heights ×3 timer states, adjacent-slot sentinel preservation,
fullWRAM/rawreturns match. Independent test verifies allWRAM equality for match,
nearby/nonmatch/signed-high heights and invalid35slot rejection without writes.

`cargo test -p port --quiet`, `cargo clippy -p port --all-targets -- -D warnings`,
`cargo fmt --all -- --check` pass; fixture builder py_compile passes.
Cleanup --apply reclaimed0 bytes; --deduplicate --apply reclaimed96,731,136 bytes.
Verdict reauthenticated after cleanup; disk2.6GiB. No worktree. Gameplay source
frozen before final oracle. Live tool exactleaf addition owned by effect_storage;
wholemovie compressed verification is recorded below.

## Completion evidence

Full129712 movie, four fresh C/mixed processes, authenticated current A364 oracle:
`out/port/ceres-platform-live/20261011T073246.635709Z/verdict.json` reports
`mixed-C-Rust-integration`, `state_effects_match:true`. Both C processes execute
524 C calls and zero Rust calls; both mixed processes execute524 Rust calls and
zero C calls. Each mixed process exclusively exercises419 Rust decompressions
and125529 Rust RNG calls. All990 sealed artifacts match across four processes,
including periodic serialized hardware/game state, terminalWRAM/SRAM, captures
and every ordered hardware event. Measured CPU fallback0 across all subjects.
This proves C/mixed parity and live ownership for this direct A364 leaf only;
original-ROM parity remains explicitly not-tested.

Lossless gzip stream416900864bytes preserves4967833433 event bytes, decompressed
SHA256 `ade76f110aa3833e707b549a07b22a746f359a8cb1a614f4bed4fed9d72621eb`.
Completed per-process ROM fixture sharing reclaimed3506438144bytes while retaining
each actual view and every snapshot path. Finished same-run hardware/ROM sharing
reclaimed1263285504bytes with immutable baseline inventory/byte authentication.

Cleanup scoped to the final run: `python3 tools/port_cleanup.py --root <run>
--apply` reclaimed0bytes; `--deduplicate --apply` reclaimed923528410bytes. All four
artifact sets, linked binary, host dependency snapshots, transformed bank source,
retained distinct ROM exemplars and offlineoracle were authenticated after cleanup.
The gzip stream was fully decoded through CRC/trailer validation. Authentication
summary is `storage-authentication.json`; diskfree3738198016bytes (~3.48GiB).
Retained dependencies: linked live binary, generated host/families/transformed C,
headless snapshots, frozen gameplay modules, actual-C predecessor evidence, original
ROM/movie identities and all sealed observations. No separate worktree created.
No new runs after the user's stopping-point request; existing final run completed.
