# Validated paired checkpoints

`just diagnose --cache out/cache/focus --start-input N --start-instruction N
--instructions N` preserves separate complete reference and subject checkpoints.
The reference envelope retains the recording-profile system scheduler, raw CPU
bookkeeping and full hardware. The subject envelope retains interpreter CPU,
hardware, runtime counters, dispatch table, misses, coverage and cached WRAM.
Supplemental hardware serialization restores pending DMA scratch and PPU per-dot
pixels, carries, cursor positions and overflow state omitted by ordinary saves.
The formats are distinct and cannot be interchanged.

A cache binds ROM, original movie, canonical inputs, maintained source, actual
patched hardware, profile, serializer, compiler, build features, seed, candidate
binary and dispatch identities. Creation verifies the cold candidate prefix and
records its observed boundary/history digest. For every cache, both restored
sides are compared with their own cold ordered instruction/bus/pending-state
traces through the remaining partial frame and one complete following frame.
Inputs advance exactly at each side's own frame boundary. Raw ending checkpoint
bytes must agree, including counters. A faulty generated candidate can pass
restoration fidelity while the validation report records candidate/reference
divergence; focused replay still reports its first incorrect instruction.

`pair.bin`, `validation.toml` and digest-bound `cache.toml` are retained together.
Unknown, truncated, corrupt, incomplete or incompatible caches are moved to a
unique `.rejected-*` sibling before a fresh candidate cache is built cold.
Cross-candidate reuse is conservatively rejected: a source identity change
requires new cold creation and validation. No caller-supplied history label
establishes equivalence or permits relabeling state.

The focused report distinguishes cache creation/validation from warm replay
seconds and reports the 60-second target overrun. Cache execution is diagnostic
feedback only; the mandatory 18,000-frame synchronization and parity gates remain
cold. `--cache` is optional so ordinary cold diagnostics retain the same public
command behavior.
