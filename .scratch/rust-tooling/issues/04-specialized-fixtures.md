# Specialized verification and fixture generation

Status: resolved
Blocked by: 01, 03

Port dispatcher, projectile-host, callback and PLM spawn verification and their
fixture generators. Preserve canonical WRAM, pointer banks, shared Rust host
types, actual callback ordering and independent host-state comparisons.

## Completion evidence

Complete typed config/all44domain audits, adapter regressions and cleanup are in
[completion.md](../evidence/completion.md). This supersedes pending notes below.

Dispatcher and PLM generator implemented. Native ROM matches the reference SHA
`66891cd1701badac52f5f1ac65a5e71c9b6c33c5c0497fa8b166111f8e69c457`;
all 1,444 fixture declarations match the predecessor exactly after removing the
generated asset's absolute path. Native adversarial tests cover wrong pointer
banks, host-only mismatch, missing host sidecar, and malformed eight-word state.
Native callback and PLM spawn adapters passed actual C/Rust controls. All 44
fixture domains reproduce checked-in manifests with a native golden test
(`/tmp/sm-native-crawler-fixtures.log`). All remaining Python fixture generators
were removed. Strict pipeline Clippy passed. Final integration and cleanup remain
pending.

## Typed migration follow-up

2026-10-11 fixture generators, callback/PLM adapters and owned public command tests
now use typed Rust domain records and TOML; C callback stdout is a numeric protocol.
Native config validation parses every family and detects contradictory return
annotations. Golden fixture tests compare typed domains against converted TOML
matrices, while the PLM test still pins the 4 MiB synthetic ROM digest and verifies
1,444 bounded cases across six routines. Compiler checks, real C/Rust regressions
and cleanup are orchestrator-owned and remain pending final source freeze.
