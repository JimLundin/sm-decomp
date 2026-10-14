# Migration formatting

The scoped Rust formatting check passed for `translation_production.rs`,
`translation_production_tests.rs`, `translation_legacy.rs` and
`translation_production_prepare.rs`; stdout/stderr are retained beside this file.
The source/documentation `git diff --check` passed.

The broader `cargo fmt --all -- --check` exited 1 on the existing formatting of
the `samus_projectile_frame` match arm in `crates/pipeline/src/live.rs:28`.
Rustfmt proposes wrapping that `matches!` expression in a block. That file was
unchanged by this migration and remains unchanged. No test failure resulted.

Raw Cargo output under `verification/` retains its original trailing blank
lines. Exclude that raw-output directory when checking source whitespace.
