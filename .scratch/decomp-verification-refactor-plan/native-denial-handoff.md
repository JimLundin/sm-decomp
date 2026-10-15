# Native negative-control adaptation

Status: claimed; focused actual compiler denial passed, full-real gate next.
Owner: `/root/bulk_context_resume`, parent `/root`, session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Main tracking: verification refactor ticket 01, integration-owner claim.
Worktree: `/home/jim/sm-decomp-performance-native-denial`.
Branch: `performance/native-denial-controls-20261015`.
Baseline: `45b89f844df928478b890c4b4814208dfe44bfb8`.
Exclusive Cargo target: `/home/jim/sm-decomp-performance-admission-target`.
Proof root: `/home/jim/sm-decomp-performance-native-denial-proof`.

## Scope

Only `crates/pipeline/src/translation_native_tests.rs`, its test-only retained
authentication records/reader and this handoff belong to this worker. Ticket
08 separately changes identity initialization in that test file; preserve those
changes at integration. No production scanner, lifecycle or runtime changes.

The existing direct `include_str!` family mutation now refuses during retained
input capture, before a Native intent is published. Preserve that early refusal
and add a durable actual compiler file-access denial through the public Program
and operation APIs. Separately preserve Native rejected-receipt/retry coverage
using understood compiler-failing syntax. All existing 16-pair/34-producer,
runtime-denial, reservation-denial, tamper and replay controls remain required.

## Verification and cleanup

The test now records capture refusal, a separately attributed actual compiler
denial and a rejected Native compiler receipt. The retained reader authenticates
all three without executing a compiler. A focused ignored compiler-denial test
is available before the complete unchanged-domain Native gate.

Tested source checkpoint: `1b0b75c`. Its full tracked snapshot and prerequisite
copies are under
`/home/jim/sm-decomp-performance-native-denial-proof/1b0b75c-20261015/frozen-source`.
Pipeline test compilation passed in 1m 58s; matching helper compilation passed
in 16.48s. Retained tools and hashes are in that proof's `tools/` directory.

The focused `confined_compiler_denial_replays_and_authenticates` control passed
in 26.91s. Both cleanup passes reclaimed zero bytes. A separate retained-reader
process passed in 0.03s with `PATH=/unavailable` and compiler/helper environment
variables absent. Exact logs are `logs/focused*`, with cleanup records under
`focused/`. No focused writer remains; exec 63226 completed successfully.

Next: run `full_real_family_retains_authenticates_and_reuses_every_producer`
using the retained `tools/sm-pipeline-tests`, matched `sm-translation-sandbox`,
the installed nightly-2026-09-29 rustc, `SM_NATIVE_FULL_SOURCE` pointing to the
frozen snapshot and `SM_NATIVE_PROOF_NAME=full-real`. `SM_NATIVE_PROOF_ROOT`
is the proof directory above. `SM_NATIVE_PERFORMANCE_ONLY` must be absent.
Retain and independently authenticate all resulting cases after both cleanup
passes. No implementation source will change during that gate.

Pending: repaired full-real gate, its cleanup and independent authentication,
retirement-reference audit and final handoff. Compiler,
Fixture and family budgets remain 60, 5 and 180 seconds. Cargo is locked/offline,
jobs 1; shared execution root is `/home/jim/sm-decomp/out/pipeline`, workers 4.

The new worktree uses the existing RustySNES dependency via an ignored link.
Pinned C, its source archive and the private ROM are independent real copies
from the frozen final worktree. Do not retire this worktree or target until the
integration owner audits retained evidence references.
