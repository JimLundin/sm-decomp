# Focused Rio host checks

This standalone package compiles the exact `native-host/src/rio_hooks.rs` and
`rio_native.rs` used by the native host. It selects Rio transport behavior and avoids unrelated
Unix host preparation dependencies. It does not build or exercise native C.

From `restart/`:

```sh
cargo test --manifest-path native-host/rio-checks/Cargo.toml --locked --offline -j 1 --lib
```

Four tests compare complete actor fixture outputs and service sequences, unchanged
admission refusal, terminal native failure/invalid results, and terminal commit
failure. Scripted transports establish wrapper behavior only. Full native host
build, actual-C comparison, admission/fatal controls, production composition and
natural live Rio encounters retain their independent gates.

`--lib preparation_tests` selects two checks that run the maintained Waver,
Ripper and Rio preparation stages with the exact shared replacement utility.
They check wrapper/fence preservation and refusal of missing/duplicate scheduler
anchors before Rio source publication. Their synthetic intermediate source tree
establishes patch integration only; it supplies no recovered-C gameplay evidence.
