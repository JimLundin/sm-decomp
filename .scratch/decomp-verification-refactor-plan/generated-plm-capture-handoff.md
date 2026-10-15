# Generated PLM effective input capture

Status: ready for integration-owner handoff; all scoped controls pass.
Worktree retirement remains with the integration owner.
Owner: `/root/bulk_context_resume`, parent `/root`, session
`01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Worktree: `/home/jim/sm-decomp-performance-generated`.
Branch: `performance/generated-plm-capture-20261015`.
Baseline: `87fb368`.
Cargo target: `/home/jim/sm-decomp-offline-verifier/target`; explicitly returned
to `/root` after the completed gate. No further worker target use is planned.
Proof root: `/home/jim/sm-decomp-performance-generated-proof`.

Own `input_view.rs`, `ordinary.rs`, only `capture_batch_inputs` in `bulk.rs`
if necessary, focused generated-input controls and this handoff. The graph
owner retains bulk execution and graph tests. Preserve all other edits.

The public regression uses the complete real PLM registration and pinned
C/base-ROM inputs, with an isolated missing or stale obsolete ROM path.
No canonical manifest or global `/tmp` asset is changed. Preserve complete
generated coverage, source freshness, original per-file and aggregate limits,
the 1-GiB declaration resident ceiling, and shared workers 4. Root must review
any aggregate accounting redesign after measured charged-read evidence.

## Completion evidence

The public red is retained at `bfab2b5-red/`: the original combined control
failed in 4.17s on the missing obsolete ROM, and independently split controls
failed in 5.60s with ENOENT (missing) and `streamed input identity changed`
(stale). The initial test build took 1m 40s. Both cleanup passes on all three
failed controls reclaimed zero bytes. The original red reader is retained in
`bfab2b5-red/tools/generated-plm-tests`; source and real prerequisites are in
`frozen-source/`. An initial missing RustySNES worktree link caused a retained
Cargo setup failure before any test; the ignored link was then restored.

The production correction removes only GeneratedPlm raw declarations from
ordinary input selection. `InputView::capture` derives the complete generated
declaration from the checksum-pinned base ROM, captures every effective Rust
source and declared input, and leaves only the generated ROM to the existing
trusted output extension. The raw catalog remains captured and revalidated.
Standalone and bulk preparation share this same path. No generic skipped-asset
option, canonical manifest, Native contract, generator domain or limit changed.

The corrected two public controls passed in **7.57s**, with an incremental build
of 2.84s. Logs are `bfab2b5-red/logs/green-initial.{log,time}`; complete control
sources are retained under `bfab2b5-green/obsolete-{missing,stale}-*/source`.
Both missing and stale placeholders remained unchanged. Automatic cleanup
completed for both controls. The base ROM and C archive hashes match the pinned
identities; the retained cleanup binary's production source is unchanged from
`b1f8691`.

The corrected production capture was checkpointed at `8fcc32c` and integrated
by the root and graph owners. Full-catalog diagnostics then measured **6,967
distinct inputs / 477,164,947 bytes**, of which **7,104,059 bytes** are compiler
inputs. All materialization completed at **1,046,226,292 charged bytes**. The
immediate complete freshness pass failed after 140 files / 27,394,048 bytes,
at **1,073,751,412 charged bytes**. Peak resident reservation was **347,912,221
bytes**, below its unchanged 1-GiB ceiling. The exact production-path diagnostic
took 36.23s; logs are `bfab2b5-red/logs/full-capture-production-diagnostic.*` and
the retained source/statistics/cleanup are in
`8fcc32c-diagnostics/full-catalog-272929-0/`. An earlier diagnostic accidentally
composed catalog and capture under an extra outer scope; it is retained
separately as `full-capture-diagnostic.*` and is not the production measurement.

Root explicitly approved an **ordinary-only composition change**: materialize
owned inputs under one 1-GiB I/O phase, then authenticate every current input
and materialized compiler input under another 1-GiB phase. Both share the same
absolute deadline established before capture. Any existing caller scope still
aggregates both phases and remains stricter. With no enclosing scope the
combined maximum is **2 GiB**; this is not reduced I/O or a performance gain.
Per-file limits, the declaration resident ceiling, all input/namespace checks
and Native/predecessor/current-scope contracts remain unchanged. Root approval
and rationale are recorded in this session's handoff messages.

The three owner controls for per-phase ceilings, stricter outer aggregate
exhaustion spanning both phases and the original absolute deadline passed in
0.21s. The complete public capture plus missing/stale and source/base-ROM/
registration mutation controls passed in **67.11s**, exec 27139 exit 0, with logs
under `phase-controls/logs/public-controls.*`. The real production catalog had
all **177 families**; capture and subsequent explicit freshness validation both
passed. Each changed real input refused before any output/producer directory,
and restoring its bytes restored current validation. Both cleanup passes
completed for all six retained public control roots. All temporary diagnostic
instrumentation has been removed from the production source.

Deferred noncompiler materialization could avoid approximately 470 MB of eager
copy work, especially on unchanged-result reuse. It needs an explicit asset
lifetime and synchronized bounded materialization against the initial content
identity, plus at-use/current-source checks. It is a further optimization, not
part of this correction; no lazy asset lifetime or timestamp identity was added.

The ordinary composition change was checkpointed at `2b0e206`. The first
complete generated-family trial retained under `2b0e206/full-real/` failed after
21.42s during C dependency scanning because this focused test's prerequisite
copier omitted `tools/decomp/headless.c` and `headless_frame.h`. The production
full-catalog fixture already contained those files. Both cleanup passes on the
failed trial reclaimed zero bytes. The test-only copier now includes both files
and `rust-toolchain.toml`; its incremental build passed. No production source
changed in this repair.

The test-only prerequisite repair is checkpointed at **`2511abd`**. Its complete
generated ordinary composition passed in **80.01s** (80.02s external wall time),
exec 11481 exit 0, with all **1,444 actual-C Fixture cases / six routines**.
Unchanged ordinary reuse passed. Both cleanup passes then passed, and the test
spawned the retained reader in a separate process with
`PATH=/unavailable-generated-plm-compilers`; complete independent admission
passed in **8.16s**. No additional verification was launched after root's request
to finish the existing evidence only.

Final proof:
`/home/jim/sm-decomp-performance-generated-proof/2511abd`.
The complete control is `full-real/complete-ordinary-276456-0/`; its
`replay.toml` has SHA-256
`2dd56bde02754d2f823b326a1c4c78631e423b923bb68c2e983fa4195179c3a4`.
The verdict is
`bulk/evidence/1792094867831524-276456-3/verdict.toml`, SHA-256
`9f6b89c5c16dc12ed20cb147e12445e653cedcda9eae5009c4ca409b4007a9f3`.
The retained reader is `tools/generated-plm-tests`, SHA-256
`66569553053621c6be0e8f86156d637c77d0853f16308b987b10a8c2a0b6ce32`.
`completion-evidence.sha256` records these pointers, cleanup outputs, reader
output, logs, tool checksums and frozen-source/prerequisite checksum lists.
The exact tracked frozen source passed its checksum verification.

Cleanup reclaimed **235,274,240 bytes** across **4,332 snapshots**. The initial
pass and every earlier successful/failed control reclaimed zero bytes; all
recorded cleanup passes succeeded. Remaining filesystem space was
**96,054,546,432 bytes**. No worker Cargo/test/compiler/reader writer remains.
All four private `tmp/` roots contain no files.

## Retirement dependencies

No retained TOML/JSON/JSONL record references the worker checkout or Cargo
target, excluding copied source, frozen snapshots and build logs. No source
needs to remain at the worker checkout path. Preserve the existing proof root,
including historical failures and each control's source snapshot.

For the passing generated-family reader, the exact source dependency is
`2511abd/full-real/complete-ordinary-276456-0/source/` beneath the proof root.
It contains the pinned C source/archive, base ROM, registration and all captured
Rust/tool source. Retain the sibling `bulk/` tree (generated ROM/manifest and
complete evidence), `replay.toml`, cleanup records and the reader above.
The reader requires no worker build output or obsolete `/tmp` ROM asset.

Root already integrated production commits `8fcc32c`, `2b0e206` and test repair
`2511abd`. The remaining worker action is this documentation checkpoint;
integration-owner worktree retirement and final effort acceptance remain with
`/root`.
