# Ticket 10 Cargo adapter handoff

Status: implementing

Owner: `/root/cargo_acceptance`; integration owner `/root`.
Worktree: `/tmp/sm-ticket10-cargo-20261014`; branch `ticket10-cargo`.
Baseline: `808efe2ee473e633a42bbf330f3872f5a0499df9`.
Owned source: `crates/pipeline/src/translation_cargo.rs` and optional
`crates/pipeline/src/translation_cargo_tests.rs`. Permanent CLI registration
belongs to the integration owner. No production authority, claim, original
submission or gameplay source is writable by this task.

The requested module runs original Cargo acceptance commands against candidate
and integrated frozen source with exact test identities and a hard 60-second
whole-command deadline. It uses durable `translation_execution::operation`
and confinement, sealed offline inputs, bounded evidence, and authenticated
unchanged replay. Cargo success establishes only Rust control acceptance.

Agreed test seams: the public prepared-contract/run/authenticate module boundary
and installed supervisor command. TDD skill read before implementation.

Initial API proposed to root: `CargoKind`, `CargoContract`, `PreparedCargo`,
`StrictRuntime`, `CargoReceipt`, `CargoVerdict`; `prepare`, `run`, `authenticate`,
`references`, `producer_cli`. Legacy readiness contracts remain untouched.

Next: implement exact original command/test inventories and the sealed Cargo
offline closure, then prove real confined success, retained failure/replay and
protected-input refusal. Use private target/run roots with
`SM_PIPELINE_WORKERS=4` and `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`.

Preparation red/green: the first public-seam control-inventory test failed on
the unimplemented adapter, then passed after preparation validation. It requires
the exact original host controls and rejects a renamed control even with an
updated source seal. The real original Cargo proof has reached its intended red
(`original Cargo run is not implemented`) in
`/tmp/sm-ticket10-cargo-proof/real-red-02.stdout`; an earlier fixture-inventory
failure is retained under `real-red.*`.

Current implementation includes original argv/identity tables (31 host, 8+2
compact, 10 pause), frozen embedded compact test-module hashes, public typed
contracts/receipts/authentication, a durable operation per original command,
whole-command deadlines, quota reservations, complete source snapshots and the
installed `translation-cargo-producer --request /inputs/cargo-request.json`
entrypoint. Typechecking passed with temporary local CLI/module registration.
The normal producer binary and public-seam proof test are being compiled.

Checkpoint: complete Cargo and environment APIs typecheck. Root may integrate
the source checkpoint while real proofs continue. `prepare_environment` takes
the prepared contract, source checkout, output, runtime, context, timeout seconds
(at most 600), and `build_dependencies`; `authenticate_environment` returns
`Result<()>`, `environment_references` returns retained references. Its separate
producer is `translation-cargo-environment-producer`. Runtime-only preparation
credits no original test gate. Dependency preparation exports vendor libraries
only and excludes source-tree project artifacts.

The first real original31 attempt at `real-01/candidate/cargo-receipt.toml`
retains a 60-second input-discovery exhaustion before any Cargo process started.
It is a bounded failure, not a passing substitute. The new explicit environment
preparation imports the same compiler identity into the same operation root
before the unchanged 60-second original command. Preparation helper typecheck
passed; full proof with the helper is next.

Offline `cargo vendor --locked --offline --versioned-dirs` succeeded at
`/tmp/sm-ticket10-cargo-proof/vendor`: 189,236,115 logical bytes. Its exact file
hash map is persisted in each `prepared.json`. Compiler/proof target:
`/tmp/sm-ticket10-cargo-target`. No production original gate has been run.

Cleanup: pending completed proof runs and both utility passes; root owns final
worktree retirement after preserving referenced source snapshots. The local
untracked ignored `deps` symlink points to the original read-only dependencies;
it is for authoring builds and is never mounted in a Cargo producer namespace.
