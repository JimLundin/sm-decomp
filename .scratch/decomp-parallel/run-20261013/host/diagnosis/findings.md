# Decomp run accountability and compiler reproduction

Observed: 2026-10-13T16:14:23Z
Status: active; no new decomp coverage earned by this run

The coordinator directed worker capacity into subsystem research and ran the
existing all-affected Offline catalog instead of submitting new Rust routines
and fixture declarations. Source changes since the setup checkpoint are the
additive latest-submissions report. Research is useful scope input but is not
translation. The sole new gameplay observation reauthenticates five previously
verified roots over ten existing fixture pairs, adding zero unique routines.

The stopped integration batch records 130 of 176 required families: 106 passed,
11 failed compilation, 13 were interrupted, and 46 were not attempted. A Rust
audit of passing verdicts finds 99,566 fixture pairs and 566 distinct pinned-C
root symbols. These are existing tests, not new fixtures or decomp additions.
The exact audit is ../audit/stopped-batch.toml. Current-source authentication of
passing families runs separately and does not imply integration admission.

## Storage gap

crates/pipeline/src/family.rs writes before.wram, c-after.wram and
rust-after.wram as full 128 KiB files for every offline fixture pair. It also
copies the ROM and compiled/generated/source dependencies into each family run.
Its case/continuation/authentication paths read filesystem members directly;
there is no evidence::Store/put_run integration in the generic offline family
path. Packed live capture does use the evidence crate. Manually ingesting the
report/research package does not fix this generic capture gap.

cleanup.rs authenticates identical finished snapshots and replaces duplicates
with hard links after production. The stopped-batch cleanup scanned 300,412
eligible snapshots and reclaimed 21,900,558,336 logical file bytes. The byte
count includes all eligible snapshot kinds; it is equivalent to 167,088 full
128 KiB images, not an asserted count of files by extension. Original byte
hashes and paths remain intact. The exact receipt is ../batch-cleanup2.stdout.toml.

At the observation above, the Rust evidence store reports 983,785 objects,
27 packs, and 139,067,226 physical bytes. It works where used, but offline
verification bypasses it. This should have been detected before the large run.
Full offline storage admission must preserve actual-C comparisons, continuation
state, historical/current authentication and independently trusted run pointers;
packing after the run alone does not prevent peak duplication.

## Bounded reproductions already run

With SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline and
SM_PIPELINE_WORKERS=4, these actual maintained commands fail within the 60-second
bound:

```sh
sm-pipeline family-verify --manifest tools/port/floater-spike-allocation.toml --output out/port/decomp-run-20261013/host-diagnosis/floater-original --case-jobs 1
sm-pipeline family-verify --manifest tools/port/animated-tile-frames.toml --output out/port/decomp-run-20261013/host-diagnosis/animated-original --case-jobs 1
```

The exact executable is ../../../../decomp-progress/latest-submissions/bin/sm-pipeline.
The floater compiler reports six E0308 errors: family::ProjectileHost versus
memory::enemy_projectiles::ProjectileHost. The Rust compiler phase takes
1.54152997 seconds. The same failure persists with one routine and one fixture
(floater-one.toml and floater-one.stdout.toml), so case volume is irrelevant.
The animated C linker reports missing kSinCosTable8bit_Sext,
CalculatePlmBlockCoords, CalculateAngleFromXY and ConvertAngleToXy. No gameplay
fixture executes in either rejected family.

Ranked hypotheses, published before probe:

1. Rust wrapper declares the same gameplay file as both family and a discovered
   dependency; canonical aliasing removes the type mismatch without gameplay edits.
2. Import names shadow canonical modules; removing duplicate import would resolve it.
3. Dependency signatures changed incompatibly; canonical aliasing would still fail.
4. C manifests omit provider translation units; adding exact pinned providers resolves linking.
5. Compiler/cache flags or stale metadata lost valid symbols; provider additions alone fail.

Next: test the canonical alias in a copied generated compiler input and exact
missing C support in scratch manifests. These probes have no source admission.
Then retain passing evidence and integrate only reviewed repairs. Do not repeat
the entire fresh batch before these compiler blockers are resolved.

## Completed bounded checks and coverage delivery

2026-10-13T17:14Z: the canonical Rust alias probe passed compilation (5.662389888s),
confirming the duplicate-module type hypothesis. The copied animated manifest
with real `sm_84.c`, `sm_94.c` and `sm_a0.c` providers passed one actual-C pair;
its finished verdict authenticated after both cleanup passes. Neither probe
changed maintained gameplay/harness source or repairs the full host selection.

The stopped host batch's 106 passing families have all authenticated against
current source after the 21,900,558,336-byte duplicate cleanup. Those are reused
existing results, not newly decompiled routines. Diagnosis cleanup reclaimed
131,072 further logical snapshot bytes; ordinary pass removed 0 bytes. The audit
writer completed and its disposable build target has been removed.

The later coverage-first task committed eight new pause/equipment routines and
607 passing actual-C pairs to main at `fa6e9cd747acabcb4568ccd2c0fbf532a9525c18`.
Ten Rust tests passed; translated and verified stages are published. That task
adds actual coverage separately from this stalled host integration; see
[coverage delivery](../../../../decomp-coverage/renderer/coverage-result.toml).
Broad storage refactoring and speculative scope research are deferred.
