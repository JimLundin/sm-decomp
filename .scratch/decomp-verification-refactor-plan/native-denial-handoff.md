# Native negative-control adaptation

Status: claimed; focused compiler/runtime gates and independent readers passed;
fresh complete full-real gate pending.
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

The complete full-real invocation (exec 6474) stopped after 54.49s at the
pre-existing runtime-denial fixture: `std::fs`, `std::fs::read` and
`std::fs::write` are unmodeled runtime inputs, so its injected candidate now
refuses before Native intent publication. Source remains `1b0b75c`. This gate
already passed all 16 original pairs / 34 producers in 26.626s, replay and
tampering, the explicit include capture refusal, actual compiler file denial
and durable replay, and the separate rejected Native compiler receipt with
two real producers and unchanged retry. The remaining runtime and reservation
controls were not completed; no complete full-real acceptance is claimed.

Both cleanup passes completed after the stopped test; deduplication reclaimed
4,194,304 bytes across 48 snapshots. The cleanup binary is retained at
`/home/jim/sm-decomp-performance-bulk-proof/b1f8691-20261015/tools/sm-pipeline`;
its cleanup source is identical to this checkpoint. Logs are
`logs/failed-full-real-{cleanup,deduplicate}.log`. The separate compiler-free
reader passed every completed retained case in 1.48s afterward, recorded in
`logs/failed-full-real-reader.{log,time}`. Remaining disk space was
95,679,139,840 bytes. No writer remains in either proof.

The root integration owner required the complete second 16-pair/34-producer
Native run and a diagnostic operation in every admitted Rust fixture namespace.
That test-only adaptation is now authored. The original std::fs mutation has
its own exact capture-refusal record. A trusted standalone probe compiles through
the admitted Native Rust compiler with a single explicitly bound source. Every
runtime Program copies the independently admitted original argv, inputs, runtime,
helper, outputs and five-second/512-KiB limits, replacing only the executable.
All 16 probes require the parent read/write, sibling-state and C-program read
denials, then copy declared before.wram to declared after.wram and emit one exact
success marker. These diagnostic results grant no additional Native Fixture pass.

The probe compiler and all 16 executions have separate durable results, exact
retry/hash/mtime checks and a retained reader binding them to the admitted Native
invocations and compiled executable. The real protected host parent, sibling
state and C executable must exist and remain unchanged. A focused runtime gate
can reuse the already admitted 1b0b75c positive as its namespace seed, before a
fresh complete full-real gate. No production scanner/runtime source changed.

Current tested source checkpoint: `955ffc3b9e86e0290301e435ef646c4ffa412dac`.
Its retained proof is
`/home/jim/sm-decomp-performance-native-denial-proof/955ffc3-20261015`.
The full tracked snapshot and prerequisite copies in `frozen-source/` pass
their SHA-256 checks. The pipeline test build passed in 54.44s. The helper
matches the earlier successful build; `logs/helper-source-unchanged.diff` is
empty. Retained tool SHA-256 values are:

- `sm-pipeline-tests`: `64aafcc12256a4f16f340784828da5427628260b6c9bbbdcc6f9d5b3c5387c52`.
- `sm-translation-sandbox`: `a49f79b9a613c96a5c8dc386a942065d1ae191fbed0d724b066435a013b4dce9`.

The focused `confined_runtime_probes_replay_and_authenticate` gate passed in
21.86s. It independently admits the completed positive Native run retained in
the older `1b0b75c-20261015/full-real/` proof, then compiles and executes all
16 runtime namespace probes with durable replay. This focused gate creates
17 diagnostic operations and no new Native family producers. Both automatic
cleanup passes reclaimed zero bytes, recorded in
`runtime-focused/cleanup.toml` and `runtime-focused/cleanup-deduplicated.toml`.
The separate retained reader passed in 2.63s with `PATH=/unavailable` and no
compiler/helper environment, authenticating the Native seed and every original
Rust fixture namespace. Logs are `logs/runtime-focused{,-reader}.{log,time}`;
reader exec 40810 completed with exit 0. No focused writer remains.

Pending: repaired full-real gate, its cleanup and independent authentication,
retirement-reference audit and final handoff. Compiler,
Fixture and family budgets remain 60, 5 and 180 seconds. Cargo is locked/offline,
jobs 1; shared execution root is `/home/jim/sm-decomp/out/pipeline`, workers 4.

The new worktree uses the existing RustySNES dependency via an ignored link.
Pinned C, its source archive and the private ROM are independent real copies
from the frozen final worktree. Do not retire this worktree or target until the
integration owner audits retained evidence references.
