# Mother Brain blue-ring lifecycle
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate native blue-ring allocation, eight-frame head tracking, movement,
Shitroid/Samus/room collision and contact explosion through the existing
projectile host seam. Preserve full WRAM, external spawn scratch, wrapping
arithmetic, signed collision comparisons, damage and scheduled execution.
Verify against pinned actual C with valid even projectile slots and aligned
enemy slots; retain offline evidence and complete scoped cleanup/authentication.
Live ownership and original-ROM parity remain separate follow-up requirements.

## Completion evidence

Added native C2F3 allocation and C335 lifecycle, C320 head positioning,
C3A9 Shitroid collision/death sentinel, C3C9 room collision, C404 quake,
C410 contact explosion and C42E graphics clear through the existing host seam.
The implementation preserves eight-frame tracking, original velocity/subpixel
arithmetic, signed C integer promotion in enemy overlap, collision priority,
slot reuse, enemy/Samus damage, suit division, frozen-health behavior,
knockback, dust allocation and sound queues. Missing host and malformed linked
enemy slots reject explicitly. No C fallback was added.

Current-source actual-C full-WRAM/raw-return/eight-word-host comparisons:

- 1,044 allocation cases:
  `out/port/mother-brain-blue-ring-allocation/20261011T115700.709969Z/verdict.json`.
- 4,663 lifecycle/boundary/native-entry cases:
  `out/port/mother-brain-blue-ring-lifecycle/20261011T115700.709332Z/verdict.json`.
- 253 scheduled frame cases:
  `out/port/mother-brain-blue-ring-frames/20261011T115700.715277Z/verdict.json`.

Fixtures are reproducible with `python3 tools/port_blue_ring_fixtures.py`.
Run each manifest with `python3 tools/port_plm_instructions.py --manifest
tools/port/<family>.json --source deps/snesrev-sm --output out/port/<family>`.
All 5,960 current-source cases pass. Scheduled coverage includes delay/timer
boundaries and contact explosions allocated into lower slots in the same frame.
Preliminary 3,764 cases remain separate historical artifacts.

All 247 port tests pass (`cargo test -p port --locked --offline --quiet`),
including three independent blue-ring tests. Strict all-target Clippy and
diff whitespace checks pass; the new test file passes rustfmt. Nine family
harness tests and three pointer/host evidence tests pass. The pointer/host test
runner requires `PYTHONPATH=tools`; the initial missing-path import failure was
corrected and rerun. Temporary harness test directories clean themselves.

For each of the three family output roots, ran `python3 tools/port_cleanup.py
--root <root> --apply` followed by `--deduplicate --apply`, covering both
preliminary and final runs. Ordinary cleanup reclaimed zero bytes;
dedup reclaimed 680,001,536 allocation bytes, 2,165,047,296 lifecycle bytes
and 132,907,008 frame bytes: 2,977,955,840 bytes total. All six retained
artifact inventories authenticate afterward; final runs also pass the
current-source full-WRAM/return/host evidence authenticator. Results are in
`.scratch/decomp-port/blue-ring-authentication.json`. Retain source snapshots,
binaries, ROM and all before/after states/host sidecars. No implementer
worktree created. Free disk at authentication: 286,913,589,248 bytes.

This is offline translation equivalence. Live blue-ring replacement and
original-ROM parity remain unproved; whole-decomp translation remains active.

## Combined-source refresh

After tickets 130–132 changed shared dispatcher/module dependencies, reran all
5,960 cases on the frozen combined source. Allocation, lifecycle and frames
pass at `out/port/mother-brain-blue-ring-<family>-combined/` with timestamp
`20261011T120706` (full verdict paths and current-source authentication are in
`.scratch/decomp-port/combined-blue-ring-authentication.json`). All complete
WRAM, raw return and external host comparisons authenticate after cleanup.
For each output root, cleanup `--apply` reclaimed zero bytes; scoped
`--deduplicate --apply` reclaimed 269,484,032 allocation bytes, 1,194,983,424
lifecycle bytes and 33,423,360 frame bytes: 1,497,890,816 total. Retain the sealed
source snapshots, binaries, ROM and before/after states. No worktree created.
