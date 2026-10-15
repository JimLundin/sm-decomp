# Recovered C source

Authenticates dependency-pinned Recovered C source and exposes its source
identity, C unit list, lexical definition facts and source inventories.
Inventory publication and CLI output belong to the application.

`verify_source` checks the dependency pin, original archive and current source
files. `authenticate_inventory` checks a retained inventory against those
inputs. `inventory` produces typed discovery facts without granting Translation
admission. `DefinitionSource` and `recovered_definition` serve source consumers
while lexical masking and token discovery remain private.

Compose related source operations with `context::limited` or `context::until`.
`context::shared_scope` captures a `SourceContext` for worker execution. It
shares the existing bounded input counters and deadline with an operation-local
typed identity cache. Reuse still rehashes the archive and every source file;
new operations and reconciliation start with fresh caches.

The only project dependency is `tool-execution`. `IMPLEMENTATION_SOURCES`
provides the compiled source stamps needed by retained evidence contracts.

```sh
cargo test --locked --offline -p decomp-source
```
