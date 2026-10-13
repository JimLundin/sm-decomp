# Progress accuracy review

Status: resolved
Blocked by: None
Owner: `/root/progress_accuracy_review`
Date: 2026-10-13
Implementation: `66cde48`.

The review confirmed that global and evidence-context stage totals deduplicate
native symbols independently and exclude stale observations. Evidence context
remains caller-reported provenance; a hashed report does not automatically
authenticate current source, gameplay acceptance or internal call closure.
Legacy observations retain their unspecified context and byte-stable JSON.

Immutable corrections now use optional `supersedes` IDs. Targets must exist and
share the stage and source inventory; self-reference, duplicate IDs and cycles
are rejected. Superseded observations retain their original evidence and remain
visible with a replacement notice. Their suppression persists when replacement
evidence becomes stale, including after persistence and reload, so inaccurate
historical claims cannot silently return to stage totals. Empty new fields are
omitted for compatibility before the first correction is published.

## Source and evidence findings

The initial 1,452-definition source map omitted eight definitions listed in the
older 220-definition Ridley translated observation. Independent comparison of
pinned C and Rust supports the inlined caller correspondences documented in
`ridley-inlined-source-alias-review.md`: `Ridley_Func_17`, `18`, `24`, `25`, `27`,
`41`, `48` and `127`. The same 220-definition translated replacement is supported
with those source-review pointers and explicit caller-domain limits. In
particular, Func48 covers its caller's fixed 4-by-4 rectangle. This source review
grants no independent wrapper exercise or verified stage; the controlled family
still names 33 native roots and requires reached-branch evidence separately.

Independent hashing of the captured registry's evidence pointers found one
mismatch: `.scratch/decomp-progress/recovery-live-ticket-audit-20261013.md`,
retained digest `ea6f058fd61871ce27be2699291c2f7b7ed5363bbf269ad522196adc24cb3658`
versus observed digest
`b6e9b9dcba902952ac124eacc9f6e16b2d7e1d454638a230ead4251ae6e4a1be`.
The associated legacy `projectile-trails-root-6053-live-20261013` observation was
already stale and excluded. Its context sibling uses intact verdict,
authentication and cleanup evidence. No other pointer mismatches or global/context
count discrepancies were found in that snapshot. Publication continued during
the review; these findings do not claim a final exhaustive registry recheck.
Compressed historical live imports explicitly rely on retained authentication
receipts and should not be described as fresh storage reauthentication.

## Completion evidence

The correction regression first failed on the unsupported `supersedes` field.
After implementation, this command passed all 19 progress tests in 0.16 seconds:

```sh
cargo test -p pipeline --locked --offline --bin sm-pipeline progress::tests:: -- --nocapture
```

Tests cover independent context counts, legacy serialization/retries, actual
update-command publication, permanent suppression after changed evidence and
reload, same-stage/inventory validation, invalid targets and corrupt graph cycles.
Owned `git diff --check` and `rustfmt --check --edition 2024` passed for
`crates/pipeline/src/progress.rs` and `crates/pipeline/src/progress/tests.rs`.

Strict Clippy reported four existing failures outside the owned files:
`filter_next` in `enemy_instruction_inventory.rs`, `collapsible_if` and
`chunks_exact_to_as_chunks` in `fixture_execution.rs`, and `too_many_arguments`
in `metrics.rs`. The established exemptions passed with this exact command:

```sh
cargo clippy -p pipeline --locked --offline --bin sm-pipeline -- -D warnings -A clippy::filter_next -A clippy::collapsible_if -A clippy::chunks_exact_to_as_chunks -A clippy::too_many_arguments
```

The coordinator subsequently reported that actual Chromium and phone-layout
supersession checks passed for committed implementation `66cde48`; browser
verification belongs to the viewer/coordinator lane.

All `sm-progress-test-*` temporary directories were removed by test RAII,
including the failed regression run. No gameplay verification runs, worktrees
or standalone helper executables were created; shared Cargo build outputs were
retained for coordinator builds. No shared registry publication was performed by
this lane. Final measured filesystem availability was 231,351,574,528 bytes.
