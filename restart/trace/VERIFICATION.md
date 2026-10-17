# Trace worker verification

Date: 2026-10-16. Worker: `/root/presentation`, scope `restart/trace/**`.
Worktree: `/home/jim/sm-decomp-restart-pilot-trace`.
Branch: `restart/pilot-trace`, baseline `c177b4b`.
This is a trace-format/comparison result; no gameplay was executed.

## Focused checks

All final commands exited 0:

```sh
cargo test --manifest-path restart/trace/Cargo.toml --locked --offline -j1
cargo clippy --manifest-path restart/trace/Cargo.toml --locked --offline -j1 --all-targets -- -D warnings
cargo fmt --manifest-path restart/trace/Cargo.toml --check
git diff --check
```

Ten tests passed, none ignored. They cover compact round-trip and hash retention;
all identity fields; every byte's corruption/truncation in a small reference;
appended data; count/order/extent/identity-length rejection after recomputing a
checksum; individual WRAM/external/effect changes; missing/extra/reordered
candidate observations; sticky first failure; invalid WRAM shape; incomplete
writers; existing-file preservation; immutable loaded hashes and the size cap.

The initial strict Clippy check requested the newer `as_chunks` iteration idiom
instead of `chunks_exact`; this was the only code correction after passing tests.
Strict Clippy and the focused tests both passed on the final source.

## One cost measurement

```sh
cargo run --manifest-path restart/trace/Cargo.toml --locked --offline -j1 --example hash-cost -- restart/trace/out/hash-cost.trace
```

The synthetic workload is 10,000 steps, each with 128 KiB WRAM, 64 KiB external
state and 32 effect bytes. The writer and candidate comparator each hash about
1.97 GB. Only `sha2` and `digest` dependencies have dev `opt-level = 3`; this
crate remains in the ordinary debug profile.

Observed output:

```text
records=10000 trace_bytes=1040249 write_seconds=1.161386 load_seconds=0.002647 compare_seconds=1.155208
```

This single measurement preceded the iteration-idiom cleanup above. The format,
hashing and comparison logic were unchanged. The cost example authenticated its
complete trace, compared all candidate records, and required matching final
summaries. No broad legacy suite or repeated performance run was invoked.

## Cleanup and retention

All worker Cargo/test/example processes finished; `pgrep` found no matching
active trace process. Tests removed their owned temporary fixtures automatically.
`cargo clean --manifest-path restart/trace/Cargo.toml` removed 663 files, reported
as 123.1 MiB. The target directory's measured apparent size beforehand was
121,760,499 bytes. No legacy run format or shared store was created.

Ignored `out/` retains 12,114,478 bytes: the focused test executable, synthetic
cost trace, and a small exact source/manifest/lock snapshot. Available filesystem
space after cleanup was 107,793,506,304 bytes. Source copies were checked with
`cmp`, and retained artifact hashes were rechecked after cleanup:

| Artifact | SHA-256 |
| --- | --- |
| `out/retained/trace-tests` | `3785abb96d75acfc1f13f900c2c8a030144c5c0f4a7922ca364dc809a103ce90` |
| `out/hash-cost.trace` | `e67502b9fe94ff92daa4334a48aec1afa545b438f60177e609c96f3a95e41102` |
| `src/lib.rs` | `632c8f90ed3728b896adb99793527a68abcf88ffc571e29d9b6107e17a85d0f0` |
| `src/tests.rs` | `c1f13950a663e25ec4e93acc5750a1ea9fbed08f76fe3ea48a508e073813192e` |
| `examples/hash-cost.rs` | `634c5849131cfe44930e3f7db56cf62fb3fe77178121704b9bcc847a37d4b9ec` |

The integrator owns removal of this worktree after source integration and should
preserve the retained `out/` artifacts first. README records the small manifest
changes needed to join the parent workspace and keep hash dependencies optimized.
There are no ROM, SDL or recovered-C runtime dependencies in these artifacts.
