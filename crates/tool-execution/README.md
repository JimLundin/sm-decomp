# Tool execution

Shared execution infrastructure for project tools: bounded filesystem input,
authenticated compiler preparation and artifact caching, cross-process worker
capacity, child-process ownership, atomic metadata publication and accounting.

`builds` accepts tool invocations, standalone Rust build descriptions and Cargo
artifact requests with explicit source inputs. Its callers own Cargo package
selection and the fixed Fixture and RNG recipes. `bounded` shares byte/path
counters and deadlines across nested operations and captured worker scopes.
It contains no Recovered C identity, Fixture model or Translation policy.

The crate depends on `pipeline-model` only for existing telemetry records. It
cannot depend on the pipeline application or Offline family verification.
`IMPLEMENTATION_SOURCES` supplies the compiled source stamps that higher-level
evidence contracts must retain and authenticate.

Run the library's behavior checks with:

```sh
cargo test --locked --offline -p tool-execution
```

The optional `test-support` feature exposes filesystem fixtures and current
input accounting to consumer tests; it is not needed by production callers.
