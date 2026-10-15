# Native negative-control adaptation

Status: claimed; test adaptation authored, compilation and gates pending.
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

Pending: compilation, focused compiler denial, repaired full-real gate, both cleanup passes,
independent retained authentication and exact source/tool hashes. Compiler,
Fixture and family budgets remain 60, 5 and 180 seconds. Cargo is locked/offline,
jobs 1; shared execution root is `/home/jim/sm-decomp/out/pipeline`, workers 4.

The new worktree uses the existing RustySNES dependency via an ignored link.
Pinned C, its source archive and the private ROM are independent real copies
from the frozen final worktree. Do not retire this worktree or target until the
integration owner audits retained evidence references.
