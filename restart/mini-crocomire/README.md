# mini-crocomire actor library

This restart workspace target exposes the existing [translated actor source](../../translations/mini-crocomire/src/lib.rs) (14 recovered definitions). Cargo uses that file directly; the standalone source candidate and this workspace target share one implementation.

The library exposes its typed entry and service boundary. A caller must satisfy the actor's admission contract, retain complete canonical WRAM and ordered native effects, and treat errors after ownership as terminal. No runtime dispatch or native adapter is registered by this package.

Status: translated source registered for workspace checks; compilation, executed tests, actual-C parity, live replay and original-ROM acceptance remain pending. See [candidate scope and acceptance requirements](../../translations/mini-crocomire/README.md).

Focused batch commands, from the repository root:

```sh
cargo fmt --manifest-path restart/Cargo.toml -p sm-mini-crocomire --check
cargo clippy --manifest-path restart/Cargo.toml -p sm-mini-crocomire --all-targets --locked --offline -- -D warnings
cargo test --manifest-path restart/Cargo.toml -p sm-mini-crocomire --locked --offline
```
