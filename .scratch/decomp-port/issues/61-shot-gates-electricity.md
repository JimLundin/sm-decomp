# Spawned shot gates and save station electricity
Status: resolved
Blocked by: 60
Triage: ready-for-agent

Port E5D0/E5D5/E5DA/E6AD allocation and E605 gate movement, preserving PLM
coordinate globals and fractional movement without carry. Verify C and cleanup.

## Completion evidence

Implemented E5D0/E5D5/E5DA gate and E6AD electricity allocation using native
PLM coordinates, preserving global coordinate writes, gate block E and closed
up/down Y offsets. E605 timer integration advances the script at 0x1000;
fractional Y addition deliberately does not carry into the signed high-byte Y
movement. Independent Rust test verifies fraction wrap and threshold behavior.

513 actual pinned C full-WRAM/raw-return cases pass (256 allocations, 128
preinstructions, 128 scheduler frames and sleep control):
`out/port/shot-gates-electricity/20261011T042609.120806Z/verdict.json`.
Rebuild through `tools/port_gate_fixtures.py`. Full port tests, fmt and strict
Clippy pass. Offline verification; live gameplay ownership remains open.

Cleanup: `port_cleanup.py --root out/port/shot-gates-electricity --apply`
reclaimed zero bytes; `--deduplicate --apply` reclaimed 93,454,336 snapshot
bytes. Accepted verdict authenticated after cleanup; immutable ROM/source
snapshots retained. No implementer worktree created. 33 GiB available.
