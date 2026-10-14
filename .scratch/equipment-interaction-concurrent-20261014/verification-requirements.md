# Private equipment interaction actual-C verification

This is the fresh Offline verification contract for the completed source
bootstrap described in [requirements](requirements.md) and
[the source handoff](bootstrap-handoff.md). It does not assert an IsolatedProof
Installation, confined rewrite, production claim or managed integration.

Source commit: `847d76f60d8a373b7b051f0be5c441f6be2342f4` in the private
`/home/jim/sm-decomp-equipment-interaction-20261014/source` repository.
The authored paths are `crates/port/src/pause_equipment.rs`,
`crates/port/tests/pause_equipment.rs` and `tools/port/pause-equipment.toml`.
The retained source snapshot is
`/home/jim/sm-decomp-equipment-interaction-20261014/source-snapshots/847d76f`.

Use the existing confined Native runner and a private catalog containing only
`pause-equipment`. Freeze the runner's matching harness, dependency pins and
source inventory with the exact private source before execution. Reuse the
existing pinned original C and ROM as inputs, never historical passing verdicts.
The main catalog and original production ownership remain outside this scope.

## Required comparisons

Execute every one of the manifest's 427 declared Fixture pairs across its
28 directly callable entries, including 181 complete-root cases. This covers
the complete 29-routine boundary: static `ChangePaletteValues` is exercised
through its button-response and Plasma/Spazer callers. Manifest SHA-256:
`b08cf0c2693d353f1662f45f26e211aebce446a14554304918e89010f5e0aed7`.

Compare complete 128-KiB WRAM and raw returns from Rust and actual compiled C.
The canonical state includes serialized SFX queues, OAM output and VRAM queues;
no byte masks, C behavior stubs or sampled case subset are permitted. Preserve
the manifest's input domain, source quirks and fixed dependencies (`audio`,
`pause_equipment_render`, C support `sm_80.c`, `sm_81.c`, `sm_rtl.c`).

Preserve all 411 independent roots and 16 continuation nodes, including the
15-step menu chain and 3-step reserve-transfer chain. A continuation inherits
the matching predecessor state; its dependency cannot be skipped to increase
parallelism. All admitted producers share the four-process execution budget.
Use the unchanged 180-second compiler and five-second fixture limits. This
first existing-runner experiment has a finite 600-second family bound.

All 13 independent Rust controls already passed under the separate Cargo test
contract recorded in the source handoff. Native verification does not rerun or
replace those controls. Never call unsafe C for the Rust-only invalid-input
refusal cases.

## Result and cleanup

Retain the exact inputs, source, reader, helper, full comparison result and any
bounded failure. Exercise unchanged replay through the existing Native API,
complete both scoped cleanup passes and authenticate the retained result.
Publish `verified` only after all 427 declared comparisons pass. Source review,
main integration, Live replay, hardware effect ordering and original-ROM parity
remain separate requirements.
