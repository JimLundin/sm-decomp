# Decomp pipeline throughput: source review

Research scope: current checked-in pipeline at commit
`16f18837023d1498f46aa555a1f0f6cc0c938092`, reviewed in the isolated worktree
`/tmp/sm-decomp-pipeline`. This report uses repository source and its retained
first-party experiment/ticket records. It does not claim new runtime benchmarks,
authenticate historical run directories, or change gameplay acceptance. The main
investigation owns new timings. No verification run was created by this review,
so there are no run outputs to clean up. Root glossary and ADR directories are
absent; the domain reading rule was followed.

Evidence labels below mean:

- **Source fact:** behavior directly present in the reviewed implementation.
- **Recorded observation:** measurements/results recorded by an earlier project
  experiment or ticket; historical observations, not a fresh reproduction.
- **Hypothesis:** expected benefit requiring a controlled measurement or prototype.

## Finding

The primary goal is heavy parallelization of both offline and live verification.
For live work, the central dependency is the single shared packed writer: separate
capture stores or packs are needed before concurrent subjects can improve throughput.
For offline work, the executor already exists, but the catalog and broad source
dependency boundary limit its usefulness. Reducing repeated compilation and evidence
reconstruction lowers each worker's cost. The existing batch catalog covers a small subset of
the newer manifests, making the repeatable orchestration less useful for the
current translation frontier. These are source facts, detailed below; this review
does not assign numerical speedups without new measurements.

## Existing behavior and evidence

### 1. Fresh builds discard useful compilation work

**Source facts.** Every family verification compiles its actual C bank/support and
an optimized standalone Rust wrapper. The Rust wrapper includes
`crates/port/src/lib.rs`, which declares every gameplay module. Callback,
power-bomb and PLM harnesses compile essentially the whole pinned C host, even
for focused fixtures. Each live invocation builds its selected Rust static
library, the decompressor into a new run-specific Cargo target, two RNG binaries,
and a C executable containing all game banks and SNES units. Packed storage also
builds the release evidence reader into a new run-specific target. Both Cargo
targets are removed after the run; the reader executable is retained. These
commands do not use a checked-in object cache or retained Cargo build target.
[Family build](../../tools/port_family.py#L140),
[gameplay module declarations](../../crates/port/src/lib.rs#L6),
[callback build](../../tools/port_collision.py#L137),
[power-bomb build](../../tools/port_power_bomb.py#L155),
[live builds](../../tools/port_live.py#L507),
[C host build](../../tools/port_live.py#L686),
[reader build](../../tools/port_evidence.py#L44),
[live target cleanup](../../tools/port_live.py#L800).

**Hypothesis.** A content-addressed build cache could retain stable C objects,
decompressor/reader binaries and RNG artifacts between candidate runs, while
rebuilding generated host/wrapper files and changed banks. Preserve the existing
exact source/compiler/command/binary identities in each run; build reuse is
different from reusing gameplay execution evidence. Measure cold and warm builds
separately before deciding whether C objects, Rust modules, or Cargo artifacts
offer the largest return.

### 2. Packed sealing reconstructs the same logical bytes repeatedly

**Source facts.** The capture command calls `put_run`, which traverses and restores
all observations through `verify_tree`. It commits, reopens the store, calls
`verify_run` for another full reconstruction, then calls `tree_inventory` for
another reconstruction and per-member hash computation. The latter two calls are
on the same reopened store. Python then hashes the original staged files before
publishing a pointer and removing staging. This source sequence performs three
logical member traversals plus the staged-file check; physical reads can benefit
from the store's small decompressed-block cache.
[Capture sequence](../../crates/evidence/src/main.rs#L28),
[put_run verification](../../crates/evidence/src/run.rs#L17),
[verify_run](../../crates/evidence/src/run.rs#L43),
[verify_tree reconstruction](../../crates/evidence/src/tree.rs#L229),
[inventory reconstruction](../../crates/evidence/src/tree.rs#L109),
[staged comparison](../../tools/port_evidence.py#L114),
[block cache](../../crates/evidence/src/pack.rs#L279).

Live authentication invokes `sm-evidence inventory` for each of the four subjects.
That command also calls both `verify_run` and `tree_inventory`, causing two full
logical traversals for each subject.
[Authentication](../../tools/port_authenticate_live.py#L21),
[subject authentication](../../tools/port_evidence.py#L153),
[inventory CLI](../../crates/evidence/src/main.rs#L63).

**Recorded observations.** The existing four-subject 1000-frame storage experiment
reports packed C producer times of 2.64–2.71 seconds and C sealing times of
4.41–4.48 seconds; mixed sealing takes 2.14–3.00 seconds. Packed captures occupy
about 3.20 MB for approximately 319 MB of logical bytes. These were sequential
short samples with uncontrolled filesystem caches/scheduling and should not be
extrapolated to full-movie rates.
[Experiment and caveats](../trace-storage-study/pipeline-results.md#L28).

**Hypothesis.** Add one verified-inventory traversal that returns member hashes,
counts and total bytes while checking the stored run counts. Use it after reopening
the committed store and in the inventory CLI. Retain the independent precommit
verification and Python staged-file equality. This removes redundant verification
work while retaining the important commit/reopen integrity boundary. Fresh
corruption, truncation, missing-member and changed-source controls should still
reject. The runtime benefit needs measurement, but the redundant traversal is
directly visible in source.

### 3. Unrelated module changes invalidate family evidence

**Source facts.** Family builders copy and seal every top-level Rust source file
under `crates/port/src`, plus declared external dependencies. Validators hash
every sealed dependency against its current path. The generated wrapper includes
`lib.rs` as its memory module, and that file declares all gameplay modules; the
wide dependency seal therefore reflects the actual wrapper compilation rather
than being safe to remove on its own. Equivalent broad sealing exists in the
callback, power-bomb, PLM and PLM-instruction harnesses.
[Family wrapper and seal](../../tools/port_family.py#L140),
[family invalidation](../../tools/port_family.py#L317),
[callback seal](../../tools/port_collision.py#L142),
[PLM required dependency set](../../tools/port_plm.py#L270),
[PLM-instruction seal](../../tools/port_plm_instructions.py#L224).

Batch verification already reuses authenticated family evidence within its selected
batch, and rebuilds rejected/stale evidence. However, this dependency granularity
means a change in one unrelated gameplay module can invalidate other families.
There is no global index that locates compatible evidence across batches.
[Batch reuse](../../tools/port_batch.py#L93).

**Recorded observation.** The cumulative admission ticket explicitly records
stale collision/lib.rs seals being rejected and rebuilt before replay.
[Historical invalidation](../decomp-port/issues/24-cumulative-live-admission.md#L72).

**Hypothesis.** Extract the canonical `Wram` implementation into a small module
without gameplay declarations. Generate a wrapper containing the selected family
and its real transitive gameplay dependencies, then seal that compiler dependency
closure. This is a module boundary change, not just deleting hashes. Changes in
an actual imported dependency must still invalidate the family. Projectile
allocator and instruction modules call each other and other subsystems, so some
closures will remain broad; simple system/audio families are better initial
candidates.
[Wram and declarations](../../crates/port/src/lib.rs#L4),
[projectile cross-module calls](../../crates/port/src/enemy_projectiles.rs#L146),
[instruction cross-module calls](../../crates/port/src/enemy_projectile_instructions.rs#L47).

### 4. Four live subjects are serial and hold the store lock during capture

**Source facts.** The live runner loops over C/mixed and two repetitions, completing
capture, sealing and counters for one subject before starting the next. It compares
all four exact observation inventories afterward. Packed replay takes an exclusive
`pipeline.lock` before starting its collector/producer and holds it through capture,
commit, validation and publication. The underlying writer additionally uses its
own crash-recovery lock. Batch `--jobs` controls family verification concurrency
(default three, allowed one to eight), not live-subject concurrency. The batch
runner itself takes a repository-local exclusive lock.
[Serial subjects and comparison](../../tools/port_live.py#L690),
[pipeline lock](../../tools/port_evidence.py#L54),
[lock scope](../../tools/port_evidence.py#L82),
[writer lock](../../crates/evidence/src/pack.rs#L69),
[verification executor](../../tools/port_batch.py#L125),
[jobs bounds](../../tools/port_batch.py#L203),
[batch lock](../../tools/port_batch.py#L235).

**Hypothesis.** Parallel capture needs separate writer stores or separate capture
packs followed by serialized authenticated import into the shared store. Simply
parallelizing `packed_replay` would serialize on the same lock. Measure two
subjects first, including peak staging bytes, collector CPU, sealing and import
cost. Keep all four independently launched subjects, their provenance and the
same inventory/ownership checks. This is a larger storage change than removing
duplicate verification traversals.

### 5. The catalog lags the translation frontier

**Observed source inventory.** At the reviewed commit, `tools/port` contains 137
family manifest JSON files excluding `batch.json` and `source-inventory.json`.
The batch catalog lists 20 families: 16 in `gameplay`, 20 in `all`. Thus 117
manifest names are absent from the catalog. Counted read-only from those JSON
files; this is a count of manifests, not distinct routines or accepted families.
[Catalog](../../tools/port/batch.json),
[example uncataloged manifest](../../tools/port/plant-spike-motion.json),
[example dispatcher manifest](../../tools/port/projectile-script-leaves.json).

The catalog's recognized kinds do not include dispatcher evidence and replay only
forwards `family` and `callback`; `port_live.py` separately supports
`--dispatcher-verdict`. The recent plant/spore tickets retain individual actual-C
offline runs while ownership remains separate. This supports the conclusion that
the current batch interface cannot express the entire newer workflow.
[Batch dispatch](../../tools/port_batch.py#L65),
[batch live kinds](../../tools/port_batch.py#L150),
[live dispatcher option](../../tools/port_live.py#L322),
[plant ticket](../decomp-port/issues/126-spike-shooting-plant.md),
[spore ticket](../decomp-port/issues/128-spore-spawner-state.md).

**Hypothesis.** Extend the catalog with manifest paths, fixture generator commands,
validator type, live adapter type and explicit live eligibility. Keep separate
offline-all and live-admitted profiles; do not automatically install every manifest.
Expose changed-family selection and batch compatible candidates at a source freeze.
This could reduce repeated manual orchestration and allow newer families to use
the existing executor/evidence reuse, even before deeper build optimizations.

### 6. Case execution pays process and full-memory I/O costs per fixture

**Source facts.** Each family case writes a 128 KiB initial memory file and starts
Rust and C subprocesses serially, with Rust admission first. Each process reads
memory (and for ROM families a whole ROM), executes one case and writes full WRAM.
The runner hashes, compares and finally seals those files. Callback harnesses
also start a process for each subject/case. Recent plant motion evidence includes
1,441 cases, illustrating the size of real fixture sets.
[Case loop](../../tools/port_family.py#L239),
[C process memory/ROM I/O](../../tools/port_family.py#L97),
[Rust process I/O](../../tools/port_family.py#L149),
[callback loop](../../tools/port_collision.py#L164),
[plant fixture count](../decomp-port/issues/126-spike-shooting-plant.md#L12).

**Hypothesis.** A batched fixture protocol or independent-case concurrency could
reduce process startup and repeated ROM loading. The safe first experiment is
parallel case pairs with isolated output directories, retaining Rust-before-C
ordering within each pair. A persistent process needs a proven reset contract for
all C globals, callbacks and host state; resetting WRAM alone is insufficient.
Do not claim equivalence by replacing full-memory comparison with a guessed write
set. Build costs may dominate small families, so profile first.

### 7. Choose exercised work and batch live admission

**Recorded observations.** Movement leaves had zero execution in both 9000- and
18000-frame prefixes, despite matching state/effects inventories. A lightweight
full-movie actual-C inventory, with expensive hardware/decompression captures
disabled, then identified first/last frames and call counts. It selected the
Ceres platform leaf, later admitted with 524 exclusive Rust calls per mixed
subject. Separate platform and script-leaf full-movie tickets report the same
4,967,833,433-byte hardware stream identity. These records illustrate both the
value of exercised work selection and the repeated capture volume of per-leaf
full-movie admission.
[Zero-hit prefixes](../decomp-port/issues/74-projectile-leaf-admission-evidence.md#L22),
[lightweight inventory](../decomp-port/issues/88-projectile-execution-inventory.md),
[inventory frame tracking](../../tools/port_projectile_inventory.py#L28),
[platform admission](../decomp-port/issues/90-ceres-platform-preinstruction.md#L35),
[script admission](../decomp-port/issues/91-native-projectile-script-leaves.md#L19).

**Hypothesis.** Generalize the lightweight execution inventory to other gameplay
subsystems, choose dependency-ready exercised work, and use first-hit frames to
pick bounded diagnostic horizons. Accumulate compatible offline-verified families
and run one cumulative full replay at a source freeze. A short smoke replay should
remain explicitly diagnostic unless it satisfies that family's exercised admission
contract. This follows the project's existing guidance to use coherent families,
exercised coverage, disjoint module ownership and one integration owner.
[Family selection guidance](../../docs/DECOMP_PORT.md),
[routine loop and ownership guidance](../../docs/DECOMP_PIVOT.md#L117).

## Ranked opportunities

The ranking below prioritizes enabling independent workers, following the user's
clarification that heavy parallelization is the goal. It is not a measured speedup
ordering; new controlled concurrency results should determine the first production
increment.

| Rank | Change | Expected payoff | Required boundary |
| --- | --- | --- | --- |
| 1 | Separate capture from shared-store publication, then parallel subjects | Enable the four live subjects and different candidate runs to execute concurrently | No shared writer collision; retain independent provenance, exact comparison and durable dependencies |
| 2 | Extend batch catalog to current manifest/dispatcher types and tune worker count | Apply the existing offline executor to the actual frontier | Separate offline coverage from explicitly admitted live adapters; benchmark resource contention |
| 3 | Isolate Wram and compile/seal actual family dependency closures | Allow disjoint source owners to verify without unrelated invalidation | Actual imported/transitive source edits must reject reuse |
| 4 | Retain a keyed build cache for stable C objects, Cargo artifacts, RNG and reader | Avoid every worker repeating stable cold compilation | Atomic keyed cache publication; seal exact inputs/compiler/options/binaries |
| 5 | Return verified inventory in one reconstruction pass | Reduce each worker's sealing/authentication CPU and I/O demand | Keep precommit plus reopened validation and staged-file equality |
| 6 | Parallel independent case pairs, or prove a persistent fixture protocol | Increase throughput inside large offline families | Cap total worker fan-out; Rust admission precedes C; prove full state reset for persistent processes |
| 7 | Generalize execution inventory; combine compatible live admission | Allocate independent implementation work by exercised dependencies and amortize integration replay | Shared module ownership and integration freeze remain explicit |
| 8 | Send typed events from C directly into the binary encoder | Remove event decimal formatting/parsing overhead | Preserve ordered values, duplicates, grammar, logical hashes and negative controls |

Typed events are grounded in the current `vfprintf` producer and the earlier
storage experiment's explicit follow-up; no new throughput gain is established
here. [Producer](../../tools/port_live.py#L80),
[prior experiment](../trace-storage-study/pipeline-results.md#L44).

## Acceptance constraints to preserve

The documented contract separates actual-C offline family equivalence, exercised
mixed C/Rust integration and original-ROM parity. Live acceptance currently requires
four fresh processes, equal exact observation inventories, equal progress/fallback,
exclusive ownership, and nonzero execution where the adapter requires it. Existing
snapshot deduplication and content sharing happen after producing independent
observations; authentication of historical bytes does not grant current-source
admission. Replacing the two C executions with historical baseline observations,
or starting acceptance replay from an injected checkpoint, would change that
contract and needs a separately specified/proven reuse boundary. No such shortcut
is recommended by this review.
[Four-process acceptance](../../tools/port_live.py#L762),
[exercised dispatcher acceptance](../../tools/port_live.py#L781),
[frozen observation authentication scope](../../tools/port_authenticate_live.py#L35),
[historical sharing scope](../../tools/port_live.py#L184),
[documented distinctions](../../docs/DECOMP_PORT.md),
[no state injection and coverage guidance](../../docs/DECOMP_PIVOT.md#L83).

## Useful next measurements

Record phase durations for source/archive authentication, each build, fixture
preparation/execution, artifact hashing, retained-evidence validation, replay
producer, packed sealing, final authentication and cleanup. Batch execution records
currently retain command/exit/status/log paths but no phase timings; packed live
subjects already expose producer/seal/pipeline seconds. Separate cold, warm unchanged,
and one-module-changed runs, with at least one small pure family and one large
callback family. For the verified-inventory experiment, benchmark against the same
retained packed subjects and compare identical inventory hashes and rejection
controls. Report wall time, logical bytes, peak staging space and CPU time; do not
infer producer speed from storage size alone.
[Batch records](../../tools/port_batch.py#L43),
[existing packed timing fields](../../tools/port_live.py#L710),
[packed measurement boundaries](../../tools/port_evidence.py#L78).
