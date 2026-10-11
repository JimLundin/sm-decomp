# Native projectile script leaves
Status: resolved
Blocked by: None
Triage: ready-for-agent

Expose native delete, set/clear preinstruction and absolute goto entry points using existing Rust instruction semantics. Actual C inventory observes 36/16/44/827 calls respectively. Verify all slots, WRAM preservation and ROM pointer return semantics offline, then fresh full-movie four-process C/mixed parity with exclusive nonzero Rust hits. Cleanup and authenticate retained evidence. Original-ROM parity remains separate.

## Completion evidence

Native bank-86 dispatch reuses instruction semantics and canonical slot validation.
Live wrappers preserve exact C byte-pointer signatures and bank-86 normalization;
admission requires matching nonzero exclusive counts for every leaf.

Offline oracle: `out/port/projectile-script-leaves/20261011T075927.005484Z/verdict.json`.
All 342 actual-C cases across all 18 slots match full WRAM and returns, including
odd operand addresses and adjacent-slot sentinels. Independent Rust tests cover
WRAM preservation, pointer returns and invalid-slot rejection.

Fresh full 129712-input four-process evidence:
`out/port/projectile-script-leaves-live/20261011T075955.123199Z/verdict.json`.
Status mixed-C-Rust-integration; all 1038 artifacts match across all four subjects.
Both mixed subjects exclusively execute 36 Rust deletes, 16 set-preinstructions,
44 clears and 827 gotos, with zero C calls to those leaves. Both C subjects have
matching C counts and zero Rust leaf calls. Each mixed subject also exclusively
executes 419 Rust decompressions and 125529 Rust RNG calls. Measured CPU fallback
is zero in all four subjects. Original-ROM parity remains not tested.

Cargo tests, strict all-target Clippy, cargo format check, 16 live-tool tests
(one opt-in control skipped), and git diff --check pass. Wrong pointer-bank
admission has a negative control. Fixture builder: tools/port_script_leaf_fixtures.py.

Cleanup: port_cleanup.py --root <run> --apply and --deduplicate --apply on both
runs. Ordinary cleanup reclaimed zero bytes; offline sharing reclaimed 115605504
bytes and live sharing 942450550 bytes. Automatic finished live ROM/observation
sharing reclaimed 4769723648 bytes. Total: 5827779702 bytes. Approximately 261 GiB
remains free. No implementer worktree created or source relocation required.

After cleanup all four artifact sets, linked binary, host snapshots, transformed
C sources, offline predecessor and retained ROM exemplars authenticated. Gzip
CRC/trailer validation decoded 4967833433 event bytes with SHA256
ade76f110aa3833e707b549a07b22a746f359a8cb1a614f4bed4fed9d72621eb.
Summary: live run storage-authentication.json. Retained dependencies: sealed
binary/observations, transformed C and host snapshots, frozen gameplay sources,
actual-C oracle, accepted decompressor/RNG evidence and pinned ROM/movie identities.
