# Original Ceres wrapper build cleanup

Status: complete
Blocked by: None

The private locked/offline release build compiled the unchanged original Rust
entry point with the missing serde_json dependency supplied by its private
manifest. The retained executable is `../bin/ceres-assignment-driver`, SHA-256
`dcb26ac047a0ebfa5b043e5740c406b422606dd532de9253068647e406c7ac1a`.
`source/` retains the exact entry point and its three project dependencies,
compiler dependency list, manifest and lockfile identities in `source.sha256`.
Both the original failed build and successful private build logs remain in
`../receipts/`.

## Completion evidence

The finished build's apply and deduplicate/apply utility passes both exited0
and reclaimed0 bytes; receipts are `../receipts/wrapper-cleanup1.toml` and
`wrapper-cleanup2.toml`. `lsof +D` found no remaining writer. This Cargo-only
build has no parent gameplay verdict, so the utility has no supported target
selection. Under docs/agents/run-cleanup.md's unsupported-format fallback, the
identified disposable `/tmp/sm-decomp-run-20261013-ceres-target` was removed
after retaining the binary and sources. It occupied49,071,794 logical bytes,
recorded in `../receipts/wrapper-target-before.txt`. The guarded Rust removal
checks its canonical path, absence of writers, cleanup receipts and byte-identical
retained executable. Its stdout and disk observations are retained in receipts.
The active host batch was outside this cleanup scope.
