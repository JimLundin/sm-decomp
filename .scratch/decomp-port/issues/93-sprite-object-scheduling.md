# Sprite object scheduling
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate B4 sprite-object frame scheduler, repeat/terminate/goto controls and
exact overlapping ClearSpriteObjects range. Full actual-C WRAM and ROM oracle,
independent branch tests and live replacement required. Allocation and rendering
remain explicit follow-up work, not covered by scheduler ownership.

## Work evidence

Sprite scheduling module implemented; 1221 actual-C cases pass in
out/port/sprite-objects/20261011T082038.746437Z/verdict.json.
Independent scheduler, clear-range alias and frozen/invalid-slot tests pass;
strict all-target Clippy passes. Ordinary cleanup reclaimed zero bytes and
snapshot dedup reclaimed 214827008 bytes. Initial duplicate fixture-name run
20261011T082001.345179Z cleaned with zero reclamation; fixed unique names.
Family live-admission oracle initially failed linking g_fail at
out/port/sprite-objects-family/20261011T082102.243508Z (cleaned, zero bytes).
Minimal runtime-global declaration added; rerun is active via session 69597.
Live scheduler admission and final authentication remain required.

Live-compatible family oracle passes all 1221 cases:
out/port/sprite-objects-family/20261011T082226.510766Z/verdict.json.
Nine family harness tests pass. Family cleanup ordinary zero bytes, dedup
214827008 bytes; current sprite oracle authenticated. Pad authentication rejected the changed
family harness seal; refreshing pad oracle again with current harness.
Fresh four-process full movie sprite verification running via session 67632.
Poll that existing process before restarting. Final live verification, cleanup
and evidence authentication remain required.

## Completion evidence

Fresh four-process full 129712-input verification:
out/port/sprite-objects-live/20261011T082259.543000Z/verdict.json.
Status mixed-C-Rust-integration; all 1006 artifacts match across four subjects.
Both mixed subjects execute 96336 HandleSpriteObjects calls and 64 clears
exclusively in Rust, matching C subject counts. Direct control-instruction
wrappers have zero movie calls: their coverage remains the 1221-case offline
oracle and independent tests; no separate live-control exercise is claimed.
All subjects report zero CPU fallback. Original-ROM parity remains not tested.

Current family oracle authenticates after cleanup. Cargo tests, strict Clippy,
nine family tooling tests, formatting and diff checks pass. Live ordinary
cleanup reclaimed zero bytes; dedup reclaimed 934732137 bytes. Automatic
finished ROM/observation sharing reclaimed 4769723648 bytes. Both successful
sprite offline runs each reclaimed 214827008 bytes; failed runs reclaimed zero.
All four artifact sets, linked binary, host snapshots, transformed C and current
family oracle authenticate after cleanup. Gzip CRC/trailer validation decoded
4967833433 bytes with SHA256
ade76f110aa3833e707b549a07b22a746f359a8cb1a614f4bed4fed9d72621eb.
Summary: live storage-authentication.json. Disk free 277480787968 bytes.
No implementer worktree created or relocation required. Retain sealed binaries,
observations, source snapshots, actual-C oracle, accepted decompressor/RNG
predecessors and pinned ROM/movie identities. Sprite allocation/rendering remain
untranslated follow-up work; scheduling completion does not close bank B4.
