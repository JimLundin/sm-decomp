# Native metadata integration

Status: complete

Final authoritative scope and results: [completion.md](completion.md).
The progress notes below retain historical observations; plans for full readiness
were superseded by the user's narrowed migration scope.

All production sections are assigned to the existing section agents. The root
integrates typed worker/build/cache/input APIs, native TOML CLI transport, batch
records, baseline capture manifests and binary decompressor call records.

Compilation and regression verification are still in progress. No completion or
readiness acceptance is claimed for the changing metadata source.

## Historical-run cleanup

No prior verification processes were observed writing these run paths before
cleanup. The preserved `/tmp/sm-pipeline-before-metadata` executable was used only
for the old sealed run format; no adapter is kept in the maintained implementation.

Cleanup --apply on `out/port/rust-final` reclaimed zero bytes. The representative
batch had already automatically deduplicated 1,008,861,184 bytes.

Cleanup --apply on each of these `out/runs/` owners:

- `1791741950131334976-2`: reclaimed 120,644,673 bytes from unsealed build.
- `1791741962600583063-2`: zero bytes.
- `1791741950121297178-2`: zero bytes; failed contaminated proof retained.
- `1791742064540177402-2`: zero bytes; failed contaminated synchronization retained.
- `1791742031188222330-2`: zero bytes; failed contaminated readiness retained.

Source snapshots, logs, sealed diagnostic artifacts, prior reader executables,
acceptance pointers and shared packed evidence are preserved. Reauthentication of
retained demonstrations and the extended prefix diagnostic passed after cleanup,
using their preserved original reader. These are historical-format checks and do
not establish final native-format acceptance.
Disk free space after these checks: approximately 211 GiB.

## Final native regression controls

The maintained Python/runtime and JSON configuration guards both pass:
`/tmp/sm-root-final-small-controls.log`. JSON dependencies remain solely at the
external SingleStepTests ingestion boundary. Typed coverage controls pass 2/2
and diagnosis controls pass 5/5 (`/tmp/sm-root-coverage-final.log`,
`/tmp/sm-root-diagnosis-rerun.log`). All 44 generated fixture domains and every
checked-in family configuration passed the complete typed audit in
`/tmp/sm-root-unit-controls.log`; the six earlier optional-field failures in that
log were independently corrected and rerun.

Strict workspace Clippy passed in `/tmp/sm-root-clippy-final.log`. Bootstrap test,
strict Clippy, formatting and doctor pass in `/tmp/sm-root-bootstrap-final.log`,
`/tmp/sm-root-bootstrap-clippy-final.log` and `/tmp/sm-root-doctor-final.log`.
Subsequent ledger changes are being checked again before completion.

The final baseline exposed a multi-call binary append defect: an `ab+` stream
starts its read position at zero, causing a repeated header. The producer now
seeks to EOF before testing whether the file is empty. The public regression
uses 300 frames and requires multiple calls. It passes in
`/tmp/sm-root-final-controls-retry.log`, alongside actual pointer refusal,
interruption retention and missing-selection controls.

The fresh 9000-frame C baseline completed with 23 captures at
`out/decomp/native-final-baseline-retry/1791744406352071262-2`. The first full
native acceptance attempt matched 63 fixtures and three original-ROM cases,
then failed serializing a root sequence as TOML. The negative-control ledger
now has a named root; its focused native roundtrip passes in
`/tmp/sm-root-ledger-final.log`. Fresh full acceptance is running again.

Earlier standalone pattern/feedback runs were contaminated by late test edits
and retained explicit failed verdicts. They were cleaned using native cleanup
--apply (zero bytes), as was the failed baseline and acceptance attempt.
They are not counted as acceptance. Final guarded runs are in progress with
all section sources frozen.

Disposable builds removed via Cargo clean: `/tmp/sm-native-config-convert/target`
100.5 MiB, `/tmp/sm-fixture-check/target` 73.7 MiB. No implementer worktrees were
created. Integration source worktree and all retained evidence dependencies
remain until verification and review are complete.

## Frozen-source continuation

The workspace regression passed in `/tmp/sm-root-workspace-final-regression.log`;
the separately executed complete typed config/44-domain audit and pattern
substitution controls also passed. Latest strict workspace Clippy after the
feedback and RNG authentication fixes passes in
`/tmp/sm-root-clippy-auth-final.log`. Native family, readiness lifecycle and RNG
artifact controls pass in `/tmp/sm-root-auth-integrated-controls.log` and
`/tmp/sm-root-rng-auth-control.log`.

Full final-format decomp acceptance passed at
`out/decomp/native-final-acceptance-retry/1791744699237190354-2`: 63 fixtures,
23 captured calls, three original-ROM controls and eleven rejection controls.
Authentication passed. The complete adversarial acceptance integration test ran
with the actual capture manifest and passed (`/tmp/sm-root-complete-acceptance-control.log`).
The 9000-frame baseline and eight-epoch routine replay passed. A retained native
verifier independently authenticated the routine inventory (472 artifacts) and
earlier RNG inventory (1069 artifacts) after cleanup; frozen implementation
verification does not imply admission of stale implementations.

Fresh RNG accepted at
`out/decomp/native-final-rng-authenticated/1791745975577862843-2` after the
native writer's full inventory self-authentication. CLI authentication passed
in `/tmp/sm-root-rng-native-auth.toml`: 65536 seeds, 16 captured fixtures and
8783 observed ordered RNG effects with four subjects/two reset epochs.
Native cleanup reclaimed zero intermediates; deduplication reclaimed
176081290 bytes from 737 snapshots. Post-cleanup authentication is in progress.

The older native batch passed system475/17, collision1752/63 and PLM1184/96
cases/routines. These differ from the separate PLM allocator fixture domain's
1444 cases. A late CLI authentication addition correctly made the ordinary
system verdict stale for current admission. Fresh frozen-source system
verification passed all475 cases/17 routines at
`out/port/native-frozen-system/1791746261570451-2-0`. Packed live, full70011-case
pattern timing, feedback9 and complete18000-frame readiness are running under
the source freeze. No optimization increment or reviewed config mutation is
needed for this migration.

Failed feedback retained a running manifest after an absent-key panic. Its
sealed generated/feedback-probe is byte-identical to the old build executable;
the origin reader authenticates the generated evidence path. After all writers
finished, scoped Cargo clean removed only its disposable build tree (115.6 MiB),
preserving the manifest, generated executable, source and failure diagnostics.
No terminal success was fabricated. The cleanly interrupted pattern run's nested
checks/patterns/build was unsealed and scoped Cargo clean reclaimed292.5 MiB;
all retained binaries, cases, ledgers and incomplete verdict remain.

The source integration worktree is the only migration worktree and remains an
evidence dependency. Gameplay checkout status is clean. Section audits are in
`utilities-final.md`, `family-final.md` and `readiness-final.md`. Tickets remain
claimed pending actual full workflow results and final cleanup.

Feedback final owner `1791745972678334997-2` completed all nine controls in
455.200736907 seconds, passed and remained nonaccepting as required. Independent
authentication passed before and after cleanup in
`/tmp/sm-root-feedback-native-auth.toml` and
`/tmp/sm-root-feedback-native-auth-after-cleanup.toml`. Native --apply removed
121175687 bytes from its unsealed build; deduplication reclaimed zero bytes.

Complete readiness owner `1791746216191267883-2` was cleanly interrupted before
changing source to repair the newly discovered final refusal branch. It returned
exit2 with an incomplete verdict, accepted=false and no accepted pointer; native
cleanup --apply returned zero bytes. Its logs and partial regression evidence
remain. `/tmp/sm-readiness-failure-reason-regression.patch` is prepared but not
applied while guarded pattern/live/batch runs remain active. It centralizes typed
durable refusal recording and tests actual missing material/publication failures.
Fresh complete readiness will run after that patch and focused checks.

Scoped Cargo clean removed bootstrap293.1 MiB and retained verifier149.2 MiB.
The retained verifier executable was copied to
`/tmp/sm-retained-verifier/sm-retained-verifier` before removing its target.
Readiness's full check recreated the bootstrap target, which remains for final
cleanup after the fresh verification. Historical pattern build cleanup is
deferred: `/tmp/sm-pipeline-before-metadata` expects declaration.toml whereas
the historical owner has JSON, so it cannot authenticate that old generation.
No retained artifacts or old format adapters were changed.
