# Offline RAM decompression pilot

Run `just decomp-routine` (or `python3 tools/decomp_routine.py --output DIR`).
Each invocation creates fresh prepare/build/compare/verdict evidence. A successful
initial job reports **partial**, exits zero, and sets `routine_accepted: false`.
Errors exit nonzero and retain an error verdict. The checked-in literal and
terminator fixtures specify expected bytes independently and compare every byte
of canonical 128 KiB WRAM, including unchanged surrounding memory.

The versioned job manifest declares the selected $80:B119 routine, pinned
upstream revision and archive identity, memory/numeric contracts, dependencies,
notices, scope and exclusions. The runner checks the bank, all upstream headers
and complete license against that archive. It builds the unchanged whole
`sm_80.c` bank with function/data sections and discards unrelated sections at
link time. Its only retained external dependency is the checked `RomPtr` shim.
Every run retains source, shim, implementation, fixture and executable digests,
compiler versions, exact commands, stdout, final states and complete notices.
The original C algorithm is never retyped into the harness.

`crates/decomp` is independent of generated gameplay code. Its public seam is
`decompress_to_mem(&RomView, &mut WramView, source, destination)`. `RomView`
validates LoROM addresses and immutable extents; `WramView` provides checked
byte operations and explicit unaligned little-endian words against one full
WRAM allocation. Source reads preserve bank rollover. Unsupported command modes
return an error. Errors may leave stores already performed; a fresh process and
initial state are required for each comparison. The candidate CLI accepts
`ROM WRAM SOURCE DESTINATION OUTPUT` and emits JSON diagnostics.

This first command's C fixture admission permits only bounded literal streams
and termination. The shim validates ROM reads but cannot bound arbitrary stores
inside unchanged C; malformed or unsupported fixture streams are rejected
before it runs. Future synthetic coverage must expand this admission contract
alongside tested Rust modes, including preexisting WRAM before the destination
for valid signed relative references. Rust's memory bounds remain checked.

Full algorithm coverage, captured game calls, ordered stores, acceptance fault
controls, the live C/Rust ABI and original-ROM execution belong to later tickets.
No offline partial result establishes whole-game Rust ownership or original-ROM
parity.

Validation: `python3 -m unittest tools.test_decomp_routine` and
`cargo test -p decomp --locked --offline`.
