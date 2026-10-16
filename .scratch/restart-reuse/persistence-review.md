# Independent review: bounded SRAM codec

Reviewer: `01a145c8-9bc8-78a2-b0f9-8519ad5f75f5`, 2026-10-16.
Disposition: **the declared four-routine source and actual-C control scope is
supported, with source/library integration only. Ordinary gameplay does not yet
install the Rust persistence callback.** Live save/reopen/continue, durable
filesystem persistence and original-ROM parity remain unclaimed.

Evidence root: `/home/jim/sm-decomp-restart/restart/out/persistence/`.
Original candidate `230fd9ebec6f92a151316202ffee3a823374ecf0`; final control
candidate `f00f2069e6547f5931f15f0f334fb0a40e1fa160`.

## What the retained results establish

The shared fixture definitions supply 30 body pairs: nine Saves spanning slots
0–2, all six admitted save areas and high-bit aliases; nineteen Loads spanning
checksum alternatives, failures, aliases and wrapping; two complete map-helper
cases. One additional Save pair covers a missing `saves/` directory. Direct byte
comparison of all 31 C/Rust pairs passes for input/output WRAM and SRAM, ordered
effects, observation images and typed result records. The retained read-only
Rust auditor also passes after retirement, authenticating result/log hashes,
128 KiB WRAM, 8 KiB SRAM, ownership deltas, disk bytes and backup state.

Native preparation preserves each original C body under its renamed definition;
control C invocations clear the Rust hook. The compiled native manifest records
recovered C `578f90b3cc49557bb70060ad033bb90b8cf8ac50`. Its measured library SHA
`3838386dfaa1aa8e1c6fb06f4a4f208409cbc702bb9f55646133ccf8268668d3`
matches both the manifest and the successful suffix result. The controls check
per-entry C versus Rust completion, zero admitted refusal, canonical generation
and exact service ordering. Save's disk phases see the fully mutated state;
non-Save entries emit no disk request. This is material Rust codec execution,
not an unchanged C result labeled Rust.

Three Save→Load chains preserve the complete canonical SRAM output between
invocations. Their WRAM intentionally changes four declared regions before
Load: player fields, explored-map region, load station and area. The retained
audit checks exactly those perturbations rather than demanding an unmodified
WRAM continuation. Each chain compares both implementations' states/effects and
restored values. It does not reopen a process or resume a playable save.

Nine pure Rust admission checks distinguish unchanged refusal from errors and
from Load's invalid-data byte. They cover unsafe masked slot3, unowned save
areas, missing service, short buffers, changed consumed ROM data and a nonzero
helper argument. These are host/domain controls, not nine additional actual-C
equivalence pairs. Unsafe slot3 is never admitted to the original C body.

The frozen interface's finite matrix is represented. Existing semantic-test
logs report 12 passes; final affected Clippy and executable build pass. No
broader game replay is required by this source/control contract or was run by
this reviewer.

## Harness correction and retained failure

The initial run stopped on fatal control3, `wrong-wram`. Its process record
remains `verified=false`, exit2, no timeout. The actual stderr is:

> Rust persistence failure: persistence invocation poisoned by service failure or changed generation
> native fatal: Rust persistence owned failure; C retry prohibited

The callback deliberately swallowed the immediate service error; expecting that
immediate message at process exit was wrong. The final code delta changes only
the control harness among executable sources: expected poison messages, exact
exit2 admission, and a suffix mode. The other changed files are tracking and
translated-stage metadata. Codec, native transport and library are unchanged.

The second run reports `fatal_start_index=3`, `skipped_fatal_controls=3`,
`fatal_controls=12`, `body_pairs=0`, and reuses the same library. There are exactly
15 unique retained fatal modes across the two runs, all exit2 without timeout.
The read-only audit checks their raw stdout/stderr hashes and expected reasons;
it explicitly interprets the old wrong-WRAM record without rewriting it. Dirty
WRAM/SRAM, wrong buffers/generation, refused or failed post-service invocation,
callback/observer/sibling recursion, unsafe slots and malformed output are covered.

This is appropriate reuse of valid failed-run evidence. A corrected diagnostic
expectation did not justify repeating 31 pairs, three chains or the first three
fatal processes. The initial audit's mistaken unchanged-WRAM-chain assumption was
also corrected in the reader, preserving the producer artifacts.

## Integration limit and next concrete work

`sm-persistence` is in the workspace and `Game` exposes the dedicated SRAM,
callback, observer and counter seam. The control executable installs the Rust
callback and exercises all four bodies. `Game` initializes that callback to
`None`, and ordinary `runtime_cli.rs` does not install it. Therefore describe
the integrated stage as bounded source/library/control integration; normal-game
Save/Load still follows C. Do not infer four runtime-owned routines merely from
the successful control counts.

The integrator was notified and acknowledged this limitation. Record ordinary
runtime hook installation as unfinished implementation work; live save/reopen/
continue is its separately scoped behavioral evidence. This does not invalidate
the completed codec comparisons or require replaying old gameplay gates.

## Cost, identity and cleanup

The 15 fatal-process records total **1,465 ms**. This is only that subprocess
slice, not total native preparation or codec verification. Existing logs record
semantic compilation0.59s/test execution0.74s, final Clippy0.16s and final build
4.52s. No controlled speedup or complete wall-time figure is inferred. The
read-only retained auditor completed successfully in the review; it launches
hashing tools, not native gameplay/control producers.

All five candidate archive/executable seals pass. Declared ROM and recovered-C
archive hashes pass. Before/after-retirement audit JSON is byte-identical at
`39397d40b1f5dfe1c290012f279e98a17944e15cc84d8734088cc64f1fddf88f`.
Reader source SHA is
`f04fd4bf657f9ea37d44447c6bd6a54330a5c88301ee6634f78ef8459f02df07`;
reader executable SHA is
`181b08a8df55af6792d56773719b11e9a34a5168f8524589864a747cb89b75e6`.

Both maintained cleanup passes on native01 and the complete evidence root
succeed with zero disposable bytes. Retired worker directories fall from
444,612,608 to135,168 allocated bytes: **444,477,440 bytes reclaimed**.
They are absent from the Git worktree list, have no `.git`, and their retained
source files match the `worker-source/` copies. Producer available-disk record
is **99,499,503,616 bytes**. The warm root cache remains outside sealed evidence.

Preserve both candidate archives/binaries, failed and successful control records,
full state/effect/disk outputs, native manifest/source/library, reader and source,
frozen contracts and private dependencies. Root owns the final encompassing
manifest after copying this review and any stage publication. This review does
not claim that a not-yet-published final manifest passed.

Review operations were read-only comparisons, source/receipt inspection and the
retained read-only auditor. Only local Markdown was written; no build, native
body/fatal/gameplay run, worktree or disposable output was created. Reviewer
cleanup is0bytes. Completed energy/main-reuse work was not reopened.
