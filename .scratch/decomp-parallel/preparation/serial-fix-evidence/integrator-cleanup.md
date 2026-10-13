# Integrated fix cleanup

Source `2d59536c059a1ca9c31d9278bff2d9222eafa56c` was integrated in the main
checkout as `23fde6a`. A clean coordinator-owned frozen subject remains at
`/tmp/sm-decomp-preparation-final-source-20261013` on branch
`decomp-preparation-final-source-20261013` for fresh drill/readiness validation.
Required source and executable bytes are preserved there and in
`drill-final/bin/sm-pipeline`; the frozen executable SHA-256 is
`df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84`.

All build/test writers finished. Both worker evidence utility cleanup passes
returned zero bytes. The clean integrated implementation worktree
`/tmp/sm-decomp-serial-integration-20261013` measured 412,354,934 logical bytes
and was removed using `git worktree remove`. The private target measured
1,508,742,193 logical bytes and was cleaned with `cargo clean --manifest-path
/tmp/sm-decomp-preparation-final-source-20261013/Cargo.toml --target-dir
/tmp/sm-serial-fix-target-20261013`; cargo removed 2,042 files (1.6 GiB reported).
Filesystem available space afterward: 222,050,205,696 bytes.

All thirteen worker evidence hashes authenticate from their containing directory
after retirement. Original frozen drill subject, original validator worktree,
legacy Ceres submission and reporting server remain held for their own evidence
or acceptance requirements. Final drill/readiness cleanup is recorded separately.
