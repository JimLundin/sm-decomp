# Worker validation

Worker `/root/ceres_route`, branch `restart/room-scroll`, baseline `a82fe50`.
Owned scope: `restart/room-scroll/**`; restart ticket 06. The integration owner
publishes shared stages and owns actual-C, live, and original-ROM acceptance.

Commands from `/home/jim/sm-decomp-restart-scroll`:

```sh
cargo check --manifest-path restart/room-scroll/Cargo.toml --offline
cargo fmt --manifest-path restart/room-scroll/Cargo.toml
cargo test --manifest-path restart/room-scroll/Cargo.toml --locked --offline
cargo clippy --manifest-path restart/room-scroll/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo fmt --manifest-path restart/room-scroll/Cargo.toml -- --check
git diff --check
```

Nine tests passed: tile quadrant flips; low-byte multiplication and wrapping
column stride; 17-block VRAM row wrapping; disabled background ratio; signed
layer versus unsigned register block positions; 17 sequential display flushes;
owned failure at the second flush; bounds errors at actual tile reads; and a
frozen root preserving all WRAM. The test build took 0.44 seconds, tests finished
in under 0.01 seconds, and Clippy completed in 0.14 seconds. Formatting and diff
checks passed. No actual-C/native game run was launched by this worker.

The selected source was compared directly with pinned `src/sm_80.c`:
DisplayViewablePartOfRoom at line 1657; layer-two position/scroll/block helpers
at lines 1730–1859; row and column preparation at lines 2083–2295. Root confirmed
preserving the source's Mode7 upload guards rather than adding a blanket refusal.
No mutable globals, ROM dependency, full-RAM copy or speculative preflight remains.

The standalone Cargo manifest contains `[workspace]` for isolated worker checks;
the integrator removes that section when registering this crate in `restart/`.

Cleanup is intentionally pending integration: root explicitly requested retaining
the worker target until integration and owns the combined live-run cleanup.
No processes remain from these bounded commands. Only the ignored Cargo target
requires cleanup; no native run directories, ROM copies or external evidence
dependencies were created. Keep this receipt, source and upstream notice when
removing the worker target/worktree, and record measured reclamation in ticket 06.
