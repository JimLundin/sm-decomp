# Pipeline improvement delivery

Implementation `c86c946`, measurement revision `7c9a766`, integrated into
`decomp-port-integration` by merge `3b53c74`. No gameplay modules or case matrices
changed. Actual decomp continues separately.

Exact legacy rendering uses a bounded88-byte stack line, pinned itoa decimal
formatting and one write per event. Canonical formats, byte seals and historical
readers remain compatible. Detailed telemetry is off by default for ordinary
verification; `SM_PIPELINE_METRICS=enabled` and benchmark scopes opt in. Normal
execution retains one-owner wait4 CPU/exit accounting, elapsed receipts and
timeout/cancellation. Disabled sampled peaks are explicitly unavailable.

The native benchmark runner freezes source/tool/compiler/input/predecessor and
cache/store identities, excludes runtime coordination locks from immutable
seeds, records complete command/cleanup/authentication boundaries and refuses
drift, unavailable resources, cache misses or external CPU interference. The
user-directed bounded scope supersedes automatic execution of the original
large matrix. Binary transport remains deferred; conditional04–08 unclaimed.

## Measured limits

[Short measurements](short-measurements.md) records exact old/new observations,
binaries and host/warmth limits: 24MB restoration median442.695→64.429ms (85.4%
less time), short complete boundary15.467→12.468s (19.389% less time). No
large-workload speedup or <=2% detailed-telemetry claim. Long telemetry tuning
was stopped at the user's request; its diagnostics remain, excluded from passing
performance reports. The2% threshold remains for future default-on promotion.

## Final-source regression

`/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-live/1791814326101526304-2/verdict.toml`
accepts four fresh subjects through all129,712inputs. Exact periodic hardware/
game state, terminal WRAM/SRAM, ordered effects and material captures match;
remaining CPU fallback0. Both mixed subjects record Rust projectile entries
16/95,325/8, zero refusal; all nine IDs617/50/50/201/75/152/177/528/524 and
allocations16Ridley/4pad/4platform. Original-ROM whole-game parity is not asserted.
Verdict SHA256:
`b54db24d3e7332b51e00d40055e51b2877420d5637e4a48d4e415af19186a0fe`.
Final reader SHA256:
`408b5aa21ba2240568df73cec822c14e13a4f72769245b504c1e62361cdf5ba7`.
The live command completed outcome=success after its automatic cleanup/dedup and
post-cleanup authentication; elapsed1,394.304s under a shared host, not admitted
performance evidence. All four subjects published to `final-evidence-store`.

Fresh prerequisite evidence: projectile1,840existing cases; decompressor40/
23/11faults/three original checks; RNG65,536seeds/8,783livecalls. Their exact
paths and validation controls are in short-measurements. Strict all-target
Clippy, fmt/diff and the public process/resource/capture/failure suites passed.
One SIGPIPE test race was corrected with a parent/child handshake; full8controls
pass. Default mode was independently demonstrated red before changing it.

## Cleanup and retention

Final scoped cleanup/dedup under `out/ticket-03` succeeded:0 and131,072bytes;
dedup checked7,682snapshots. Predecessor cleanup reclaimed554,696,704projectile
bytes and176,081,290RNGbytes; decompressor0. Check targets cleaned2.8GiB and
release126.1MiB; earlier targets are recorded in the checkpoints. Retained
reader-comparison harness target cleanup70.6MiB. Automatic live cleanup removed
build intermediates; final scoped utility receipts are `logs/final-scope-*`.

Integrated worktree `/tmp/sm-pipeline-upgrade-integration` was removed using
`git worktree remove --force` after preserving all evidence and exact sources.
The same path now holds a34.8MB source/manifest/movie snapshot and the original
evidence, no Git metadata or build tree. Source files and manifests referenced
by absolute seals are restored unchanged at their original paths. Historical
ticket01/ticket02source freezes, original failure evidence, packed stores,
subject pointers, readers, ROM/dependency references and capacity-two worker root
remain. The unrelated trace-storage worktree is untouched. Final disk available:
217,131,274,240bytes (subject to other host activity).

Post-removal authentication receipts: `logs/final-live-auth-after-removal.toml`,
`final-projectile-auth-after-removal.toml`, `final-system-auth-after-removal.toml`,
`final-decomp-auth-after-removal.toml`, `final-rng-auth-after-removal.toml`.
Family authentication of the frozen snapshot uses an explicit TOML checkout/
verdict request; Git-dependent argument shortcuts are for active checkouts.

All post-removal checks completed successfully: live authenticated four subjects
with1,479artifacts each, binary=true, same verdict hash; system/projectile are
offline-verified and decompressor/RNG accepted. No verification writers remain.
