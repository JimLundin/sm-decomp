# Isolated whole-Ridley live control tests

Status: passed; candidate file only, shared source untouched.
Owner: `/root/progress_finish`, delegated by `/root/ridley_live_finish`.

The only changed candidate file is
`/tmp/sm-ridley-whole-live-source/crates/pipeline/tests/live_ridley_whole.rs`.
The test compiles actual production whole-Ridley host and native projectile C
templates with small bus/native stubs, and separately compiles the actual
production Rust live ABI with the real port module and immutable original and
execution ROM files. ROM-derived bytes remain local.

Six compiled C controls pass: selector refusal falls back once; nested native
calls add no nested whole-root counters; accepted success publishes all host
words and unwinds depth; support error and accepted execution error both exit
four without a native C retry or after marker; Mode7 writes/read order is exact;
and native137 allocation before/after captures preserve source1995, WRAM and
host words, report one Rust allocation attempt without native/ABI reentry, and
distinguish a saturated pool from material allocation.

The actual Rust ABI invokes all thirteen production controls, adds adjacent ROM
mutations at 0x29659/0x2965b, ROM/WRAM and ROM/host overlap, and wrong ROM length.
No-callback panic stubs verify support probes/refusals never publish state,
invoke C services/bus, emit effects/opcodes or allocate. State and all eight host
words remain unchanged. The one-byte runtime ROM guard and foreign-bank/non-Ridley
refusals are exercised through the unmodified production controls.

Validation:

```sh
cargo test -p pipeline --locked --offline --test live_ridley_whole -- --nocapture
cargo clippy -p pipeline --locked --offline --test live_ridley_whole -- -D warnings
rustfmt --edition 2024 --check crates/pipeline/tests/live_ridley_whole.rs
```

Cargo result: two tests pass in 7.74 seconds; strict test-target Clippy passes
without exemptions in 0.96 seconds. Logs are `recovery-whole-live-host-cargo-controls.txt`
and `recovery-whole-live-host-clippy.txt`. Exact tested template/test hashes are
`recovery-whole-live-host-control-source.sha256` and passed checks.

Rust test RAII removed every generated C/Rust source, executable, capture and
temporary run directory after success. The standalone test executable was
removed (7,356,200 apparent bytes). Both scoped retained-artifact project cleanup
passes succeeded with zero supported disposable bytes; final receipts are
`recovery-whole-live-host-final-cleanup.toml` and
`recovery-whole-live-host-final-dedup.toml`. Available disk bytes are recorded in
`recovery-whole-live-host-disk.txt`. Candidate shared Cargo target is retained for
the owning live candidate's active build/replay and remains its owner's cleanup.
No gameplay fixture run, shared packed store, original ROM or source freeze was
changed by these controls.
