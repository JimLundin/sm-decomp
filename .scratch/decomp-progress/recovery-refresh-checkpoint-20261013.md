# Progress refresh recovery checkpoint

Status: progress feature complete; remaining independent gameplay live frontiers recorded.
Owner: `/root/progress_finish`, delegated by `/root` after interrupted recovery audits.

Only maintained `crates/pipeline/src/progress.rs`, `progress/tests.rs`, and the
new private `progress/declarations.rs` were edited. Frozen main SHA-256 remains
`1c2076de53340f6fd0af0489db696da299134f11d9a07bfd997fc9f10e47b3f4`;
batch remains `4aa716117c3cfd0fae0597a13f0fdfac7a0520c0cf99f4ef8905325763201219`.
The coordinator's retained frozen binary is `/tmp/sm-recovery-bounded-source/sm-pipeline`,
SHA-256 `9d91d2aa83942bbc30c8048dfa82db40fc3f5641d10a08355f922ae19fcc0ae1`.
The updated debug binary/source digests are `recovery-refresh-fixed-source.sha256`.

## Diagnosis and verification

The original progress refresh PID 46322 had exited before this recovery began,
without publishing the shared registry. Its retained stdout shows successful
6,053-definition source indexing; its performance log records more than eight
minutes of CPU-active declaration parsing. Old registry bytes remain in
`recovery-registry-before-refresh.json` and the interrupted logs are retained.

The red-capable real `add_declarations` test scans a four-megabyte fixture array
and requires completion within two seconds. Before the fix it failed at
49.324007705 seconds (`recovery-refresh-red-test.txt`). After the private lexical
metadata projection it completed at 58.795711 milliseconds, with all 14 progress
tests passing in 0.28 seconds (`recovery-refresh-progress-tests.txt`). The tests
compare family/symbol results and metadata failures against complete TOML decoding
for table, inline, quoted, dotted, duplicate and invalid-type layouts. Ignored
fixture comments, arrays and multiline strings cannot create fake declarations.
Fixture field semantics remain the family verifier's responsibility.

The fresh maintained debug CLI command `progress refresh --root /home/jim/sm-decomp
--source /home/jim/sm-decomp/deps/snesrev-sm` completed in 57.78 wall seconds,
39.07 user CPU seconds and 116,760 KiB maximum RSS while the coordinator's batch
and live runs were active. It atomically published source SHA-256
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`,
6,053 definitions, 75,229 body lines and 834 fixture-declared definitions.
Logs and measurements are `recovery-refresh-fixed.*` and
`recovery-refresh-fixed-performance.txt`.

## Shared publication

The retained Rust publisher `.scratch/decomp-port/progress-publish-recovered-20261013.rs`
published all six recovered stage updates with the real maintained CLI. It took
55.75 wall seconds / 39.05 user CPU seconds under verification load, and its
disposable executable removed itself. Logs are `recovery-publish.*`; update TOML
and individual CLI outputs are `.scratch/decomp-port/progress-*-6053-recovered-*`.
All sixteen pending Ridley source hashes matched immediately before publication.

The shared registry now preserves four earlier observations as historical and
has ten total observations. Independent current counts are translating 220
definitions / 3,129 body lines; translated 228 / 3,393; verified 8 / 264;
integrated 0; live-exercised 0. Whole Ridley/Ceres uses the exact 220-definition
closure and uncommitted source snapshot. Trail and Samus translated/verified
updates use main commits `2b62134bfbee7c5dcd4c81616edc5d91e781d44f` and
`c711faea56f20d42113eb63393cc4a76d4940557`, with frozen family domains stated
explicitly. Coordinator current dependency and live receipts remain pending.

Read-only update inspection confirms updates do not decode fixture declarations.
Each current request rehashes a 3.26 MB inventory, reads a 2.4 MB registry and
rehashes approximately 53.06 MB of unique evidence pointers; the debug CLI also
hashes its 173 MB executable at startup. A real debug status took 11.37 wall /
5.95 user CPU seconds; an identical update retry took 11.36 wall / 6.35 user CPU
seconds. Existing observation IDs/timestamps remained unchanged. These timings
motivate measuring six concurrent full-registry retries with the coordinator's
release build, while preserving the ten-second lock bound. No main/source
authentication contract changes are needed for this performance check.

The release binary SHA-256 is
`7fd44cf8a43d1b9896891fc029a9ebdebb74ad448846c725ee3bc97cece4d762`;
`recovery-release-source.sha256` records the unchanged main/batch hashes and
progress sources. Real release status completed in 0.16 wall / 0.13 user CPU
seconds. Six independent release CLI retries used an isolated exact copy of
the 2,479,320-byte current registry and its actual evidence pointers. All passed
in 2,629 milliseconds; writer completions were 377–2,624 milliseconds. During
publication 421 exact-byte reads saw unchanged valid registry bytes, including
all observation timestamps. The shared registry was unchanged. Retained proof
is `recovery-real-registry-concurrency/verification.json`, six CLI stdout/stderr
pairs, and the Rust helper source. No maintained update change was necessary.

Both required project cleanup passes succeeded for the isolated concurrency
root, each reporting zero bytes because the progress verification format is
outside gameplay run cleanup. After all writers exited, the disposable cloned
JSON/HTML/lock and Rust helper executable were removed, reclaiming 9,102,790
measured apparent bytes (`recovery-real-registry-before-remove.txt`). The Rust
source, writer diagnostics, verification JSON and cleanup receipts remain.
Available filesystem bytes were 234,812,915,712; disk output and checked retained
digests are `recovery-real-registry-disk.txt` and
`recovery-real-registry-evidence.sha256`. No gameplay evidence, shared store or
implementer worktree was changed. Debug instrumentation is absent.

## Remaining gameplay frontier

The progress ticket and spec are resolved locally on `affected-family-assignment`.
Source hashes and current published artifacts are recorded in
`recovery-current-publication.sha256`; the coordinator owns the final shared
wiring commit. Independent trail/composed live and broad Ridley gates remain
gameplay integration work. Publish their additional stages as new observations
only when passing coordinator post-cleanup receipts justify them.

## Current-source integration publication

The coordinator's twelve current affected Offline families completed 20,943
actual-C fixture pairs and authenticated after ordinary cleanup, deduplication
and worktree retirement. Six reports are `progress-integrated-*`; six are the
coordinator's `recovery-integrated-*-post-cleanup-authentication.toml`. The exact
summary is `.scratch/decomp-port/recovery-integrated-final-authentication-summary.toml`.
The first-six full report hashes passed independent checks.

The retained Rust publisher
`.scratch/decomp-port/progress-publish-current-integrated-20261013.rs` used the
release CLI to publish eighteen achieved observations: translated, verified and
integrated for trails, Samus projectile frame, camera scrolling, room scrolling,
bounded Ridley attack instructions and native137 allocation/scheduling. Total
publication time was 7.75 wall / 4.67 user CPU seconds. Scope details and exact
installed commits are `.scratch/decomp-port/progress-current-integrated-stage-evidence-20261013.md`;
shared wiring remains uncommitted at its authenticated freeze. Synthetic
`EprojInstruction` is explicitly excluded from native C-definition coverage.

The shared registry now has 28 observations, including four historical records.
Independent current counts are translating 220 / 3,129 body lines, translated
247 / 3,785, verified 34 / 719 and integrated 34 / 719; live-exercised remains
zero pending independently authenticated live receipts. The broad 220-definition
Ridley/Ceres scope retains translating/translated stages and does not inherit
the bounded eight-definition attack verification.

After publication writers exited, both scoped progress cleanup passes completed
with zero supported disposable bytes. The identified Rust publisher executable
was removed, reclaiming 4,717,712 apparent bytes (size receipt
`recovery-publish-current-helper-size.txt`), retaining
its source, generated update files and CLI logs. Final cleanup/disk receipts are
`recovery-final-progress-cleanup.toml`, `recovery-final-progress-deduplicate.toml`
and `recovery-final-progress-disk.txt` (232,811,167,744 filesystem bytes available).
Final maintained progress status passes,
and Rust formatting plus owned diff checks pass. Ticket01 and the spec are
resolved; the pending gameplay live frontier is explicitly recorded above.
