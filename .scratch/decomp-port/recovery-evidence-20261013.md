# Crashed-session evidence recovery

Date: 2026-10-13
Status: recovery and prerequisite verification complete
Owner: `/root/evidence_recovery`

The reboot removed every original `/tmp` source snapshot, implementer directory
and native evidence run. Main retains the integrated translations and the
`out/port` artifacts below. Root owns source changes, new merged family/live
verification, progress publication, worktree pruning and commits.

## Retained evidence recovered

Exact source files were reconstructed by matching the dependency/harness SHA256
maps in each original family verdict to its sealed root-level artifacts. Each
copy was hashed before publication and existing destination files were required
to match. Original absolute candidate and manifest paths were restored. No Git
metadata or build directory was reconstructed.

| Evidence | Source snapshot | Recovery receipt |
| --- | --- | --- |
| Trails 3,316 controlled cases / four routines | `/tmp/sm-projectile-trails` | `recovery-trails-controlled-source.toml` |
| Trails fourteen naturally active full-WRAM cases | `/tmp/sm-projectile-trails-natural-source` | `recovery-trails-natural-source.toml` |
| Samus projectile update/render 476 cases / four routines | `/tmp/sm-samus-projectile-frame-source-final` | `recovery-samus-source.toml` |

The candidate files remain at `/tmp/sm-projectile-trails` and
`/tmp/sm-samus-projectile-frame`; candidate tests came from commits `cf5f2d8` and
`0f59de1`. Recovered contexts link their pinned C source and download archive to
the retained main checkout. Preserve `deps/snesrev-sm` and `deps/downloads`.
Each source context has `retention.toml`; durable recovery inputs and procedure
are in `.scratch/decomp-port/recovery-*-source.toml` and
`recovery-source-snapshots.rs`.

All three original family verdicts successfully reauthenticated against their
recovered source contexts. Receipts are `recovery-*-authentication.toml`.
The trail actual-C inventory also authenticates all 352 sealed artifacts using
its recovered exact producer/template, recorded in
`recovery-trails-inventory-authentication.txt`. Its 9,000-input evidence remains
at `out/port/projectile-trails/natural-9000-corrected`.

## Lost evidence requiring fresh verification

Camera source/test/manifest and actual-C inventory producer survive in main,
but its old `/tmp` Offline family, captured natural frames, logs and cleanup
receipts do not. No duplicate camera verdict or capture was found in retained
`out/port` or local Markdown. Run the current native camera family and capture
fresh material camera activity in the merged replay.

Room dependency source from older trail/Samus seals is recovered, but no new
native room Offline family exists in retained `out/port`. Historical JSON room
families remain evidence of their frozen implementation; a current native family
is required for merged admission.

Ticket137's completed native complete-route replay was solely
`/tmp/sm-pipeline-upgrade-integration/out/ticket-03/final-live/1791814326101526304-2/verdict.toml`
and is lost. Its old native authentication receipt and historical JSON evidence
remain. No surviving native137 Offline family was found under `out/port`; fresh
merged evidence must replace this dependency.

Native decompressor acceptance and NextRandom prerequisite TOML evidence also
did not survive in `out/decomp`. A new Rust `decomp-baseline` 9,000-input run is
in progress under `out/recovery-session/prerequisites/baseline`, to produce
23 genuine full-state capture fixtures before native acceptance and NextRandom
exhaustive/offline plus live prefix verification. The completed actual-C baseline
is `out/recovery-session/prerequisites/baseline/1791858402896633736-2`:
both 9,000-input processes agree, each reports 23 decompression calls and zero
CPU fallback. Its capture manifest is `replay/fixtures/manifest.toml`.

## Completion evidence

Built current `sm-pipeline` using
`cargo build -p pipeline --locked --offline --bin sm-pipeline`; cached build
completed in 0.05 seconds. Source recovery used only Rust, Git read operations
and standard byte-copy/hash commands. No Python source or runtime was used.

Finished old trail runs received both project cleanup passes:

```sh
target/debug/sm-pipeline cleanup --root out/port/projectile-trails --apply
target/debug/sm-pipeline cleanup --root out/port/projectile-trails --deduplicate --apply
```

Both reclaimed zero bytes; dedup checked 10,004 snapshots. Receipts are
`recovery-trails-cleanup.toml` and `recovery-trails-dedup.toml`. Naturally active
and controlled family reauthentication after cleanup both succeed. Receipts are
`recovery-trails-natural-after-cleanup-authentication.toml` and
`recovery-trails-controlled-after-cleanup-authentication.toml`.

Samus received the same two passes scoped to `out/port/samus-projectile-frame`.
Dedup reclaimed 164,888,576 bytes across 4,530 snapshots. Its ordinary pass
completed and reclaimed zero bytes. All writers had stopped before cleanup.
Final Samus reauthentication succeeds against the recovered source snapshot;
receipt `recovery-samus-after-cleanup-authentication.toml` verifies all 476
cases. Shared stores/caches were preserved.

Available disk after source recovery: 250,430,615,552 bytes. Later measurement
while the fresh baseline was compiling: 249,517,977,600 bytes available. Final
prerequisite cleanup is complete; latest disk measurement below.

The fresh decompressor acceptance pointer is
`out/recovery-session/prerequisites/decompressor/decomp-acceptance.toml`, naming
run `1791858611765960898-2`. Its verdict SHA256 is
`d3e76d56807dde61de98aa37904ca972b903548ea0d0d7d86300e35f9b7205b1`.
It passes 63 fixtures (40 controlled plus 23 genuine captured calls), three
bounded original-ROM calls, complete ordered destination writes and twelve
acceptance fault controls. Receipt: `recovery-decomp-acceptance-final.toml`.

An earlier attempt `1791858566801568155-2` matched all 63 C/Rust fixtures but
the original-ROM subprocess received relative `rom/sm.sfc` inside a fixture
working directory and exited 2. Repeating with absolute ROM/capture paths
completed acceptance without changing source. The failed verdict remains
diagnostic evidence. Fresh NextRandom predecessor verification now runs against
the accepted pointer above under `out/recovery-session/prerequisites/next-random`.

Both baseline cleanup passes complete and report zero bytes; its native
`replay-completed` format is outside ordinary cleanup eligibility and its
snapshot inventory is outside the dedup format. Compile outputs referenced by
the baseline build receipt, both repeated subjects and all capture evidence are
retained. Receipts: `recovery-decomp-baseline-cleanup.toml` and
`recovery-decomp-baseline-dedup.toml`.

Both cleanup passes scoped to the decompressor attempt batch complete and
reclaim zero bytes, checking no dedup-supported snapshots. The accepted format
seals every retained compiler output, fixture and diagnostic control, so those
artifacts remain. `decomp-authenticate` succeeds after cleanup; receipt
`recovery-decomp-after-cleanup-authentication.toml`. The accepted pointer and its
retained output dependencies are ready for root's merged live verification.

The native NextRandom predecessor is accepted at
`out/recovery-session/prerequisites/next-random/1791858692092144850-2/verdict.toml`.
It proves every 65,536 input word against actual C, sixteen natural before/after
captures, all 8,783 observed call effects and compiled ABI controls. Its live
child `live/1791858714116893753-2/verdict.toml` compares four fresh processes,
each with two independent 9,000-input reset epochs. Both mixed subjects have
8,783 exclusive Rust NextRandom calls and 23 exclusive Rust decompression calls
in each epoch, with zero CPU fallback and matching state/hardware effects.
The run's built-in final native authentication succeeds; command report is
`recovery-next-random.toml`. Both scoped cleanup passes complete: ordinary pass
reclaims zero bytes; dedup reclaims 176,081,290 bytes across 737 snapshots. The
verdict SHA256 remains
`47f845601ad0e8609c535153115da67126fc949799a09a5443bfbc354a4d4b19`.
Receipts are `recovery-next-random-cleanup.toml` and
`recovery-next-random-dedup.toml`. Post-cleanup native authentication succeeds:
`recovery-next-random-after-cleanup-authentication.toml`.

Source-recovery and inventory-authenticator binaries were temporary one-off Rust
tools. After their work finished, removed only `/tmp/sm-source-recovery`,
`/tmp/sm-recovered-trails-producer` and `/tmp/sm-source-recovery.rs`, totaling
9,641,137 measured logical bytes. The reconstruction Rust source and all
recovered evidence source files remain under the recorded durable receipts.
No shared store or build cache was removed. Latest available disk is
244,130,078,720 bytes while root's merged families/live outputs are growing.
Required project cleanup reclaimed 340,969,866 bytes total in this recovery
lane. All lane-owned processes have finished. Integrated Git worktree pruning
remains root's separate completion responsibility; restored paths are source
snapshots and are not Git worktrees.
