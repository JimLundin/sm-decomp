# Rust bootstrap

This independent Cargo package prepares the patched hardware source before the
root workspace can resolve its path dependency. It calls host download/archive/
patch tools directly and verifies the pinned SHA-256 values; it has no Python
runtime dependency.

```sh
cargo run --manifest-path tools/bootstrap/Cargo.toml --locked -- prepare-rustysnes
cargo run --manifest-path tools/bootstrap/Cargo.toml --locked -- setup --offline --skip-mesen
cargo run --manifest-path tools/bootstrap/Cargo.toml --locked --offline -- symbols
cargo run --manifest-path tools/bootstrap/Cargo.toml --locked --offline -- doctor
cargo run --manifest-path tools/bootstrap/Cargo.toml --locked --offline -- hardware-check
```

`SM_PROJECT_ROOT` selects another checkout for source preparation or checks.
The hardware command copies the APU/savestate packages into a temporary minimal
workspace, preserving root-lockfile registry versions and checksums. Its source
copy is removed on success or failure; compiled tests use `target/hardware-tests`.
