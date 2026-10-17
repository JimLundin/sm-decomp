# Room display and scrolling

`sm-room-scroll` implements three scheduled bank-$80 operations:
`DisplayViewablePartOfRoom`, `UpdateScrollVarsUpdateMap`, and
`CalculateLayer2PosAndScrollsWhenScrolling`. All helpers are private. The crate
has no dependencies, reads no ROM, stores no state outside canonical WRAM, and
does not copy WRAM or perform speculative execution.

The module owns layer-two ratios, scroll-register and block positions, update
direction selection, tile quadrants and flips, row/column tile buffers, and
their VRAM transfer descriptors. The native host owns actual
`NMI_ProcessVramWriteQueue` execution and ordered PPU/DMA effects. Initial display
prepares and synchronously flushes 17 columns; each service finishes and returns
updated RAM before the next column is prepared.

The source preserves the native guards: frozen
`CalculateLayer2PosAndScrollsWhenScrolling` does nothing; Mode7 skips row/column
preparation while roots still update their other state and initial display still
performs its flushes. This is not Mode7 rendering ownership. Camera movement,
room loading, decompression, door scheduling and graphics uploads outside these
three roots remain native. No general camera or door module is imported.

Full 128 KiB WRAM is required. Tile addresses are checked at actual reads; an
out-of-bounds source or failed service is an owned error, never a request to
repeat the operation in C. This module does not provide rollback. The native
integration owns one private WRAM transaction and must terminate after errors,
especially after the first external transfer. `Outcome::Refused` is retained
for the existing interface vocabulary; these roots currently handle valid
inputs directly rather than speculatively checking or silently skipping them.

Provenance: snesrev/sm revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, `src/sm_80.c` at `$80:A176`,
`$80:A2F9..A51F`, and `$80:A9D6..AD1C`. Selected algorithms derive from this
project's `crates/port/src/room_scroll.rs` and `room.rs` at baseline `a82fe50`.
String dispatch, old-crate imports, full-state preflight, and duplicate RAM
transactions were removed. The full upstream notice is preserved in
`LICENSE.upstream.txt`.

Worker tests target quadrant flips, signed block positions, byte-width multiply
and word-width stride, VRAM row wrapping, disabled layer ratios, service ordering
and errors. The integrator runs actual-C entry comparisons and a bounded
pre-escape Ceres route including loading, horizontal/vertical scrolling and door
transitions. Implemented cases beyond that route are not blanket verification.
Original-ROM comparison remains a separate unresolved milestone.
