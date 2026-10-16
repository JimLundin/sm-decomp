# Whole-batch verification: priorities from current evidence

Observed 2026-10-15 at main `5c011eb`. The user identified whole decomp batch /
integration verification as the slow path. This assessment changes no pipeline
source, gameplay, installed tool, production policy or existing claim.

## 1. Repair affected-family precision

The strongest finding is unnecessary selection. A current `batch plan --profile
offline-all --changed-since HEAD~1` selected **160 of 177 Offline families** and
excluded 17. The tracked diff contained only five architecture documents; the
retained probe also saw this assessment's untracked `.scratch/` files. There
were no changed verification or gameplay inputs.

| First refusal reported by selection | Families |
| --- | ---: |
| Unsupported Rust input in `plm.rs` | 151 |
| Unsupported Rust input in `projectile_flight.rs` | 7 |
| Generated PLM inputs | 1 |
| Unsupported `ridley-subsystem` discovery | 1 |

`Discovery::source` in `crates/pipeline/src/affected.rs` splits raw source into
words and rejects words such as `include`, `mod`, `fs` and `macro_rules`, even
inside comments. The only rejected word in each of the first two source files
is ordinary documentation:

```text
crates/port/src/plm.rs:769
/// ROM-aware preinstructions include native room projectile spawning.

crates/port/src/projectile_flight.rs:399
/// Trail coordinates include native Ceres Mode7 arithmetic and ROM trail frames.
```

Thus **158 first refusals arise from two documentation comments**. Discovery
also treats arbitrary words matching module names as dependencies, including
comment text. This can unnecessarily expand the traversal before refusal.

Implement syntax-aware source discovery backed by the compiler dependency
inventory where available. Resolve supported modules and declared file inputs;
retain the conservative fallback for genuinely unknown inputs. Do not merely
delete the keyword checks or rename comments. This should be the first
performance follow-up to the active Offline family extraction, whose approved
scope explicitly defers more precise selection.

Validation should cover documentation-only edits, real transitive code changes,
includes/macros/environment inputs, captures, missing inputs, renames, and
source mutation during selection. The final reduced family count is unmeasured:
fixing the first refusal can reveal a later uncertainty. Selection also does
not imply execution: an unchanged selected family may authenticate and reuse
its previous result, so this measurement is not 160 proven recompilations.

## 2. Reuse preparation and authentication work within a batch

The current `batch plan` loads the catalog at entry, again inside selection,
and again for its final stability comparison. Catalog discovery decodes the
complete manifests of discovered families, and selection decodes each examined
manifest again. Binary Fixtures already removed the old text-format cost;
the next opportunity is one authenticated catalog/input snapshot per operation,
with explicit final freshness checks and dependency/adapter metadata available
without repeatedly decoding all Fixture payloads.

Fresh family verification inventories the finished run; `batch::verify_one`
then calls independent family authentication, which inventories it again and
reopens the before/C-after/Rust-after WRAM snapshots to validate cases. Batch
cleanup traverses the retained batch root. These are real evidence boundaries,
not checks to omit. Profile their bytes and time, then share validated reads
inside a bounded immutable snapshot or use content-addressed storage to avoid
reprocessing identical bytes. Preserve fresh authentication at independent
admission boundaries, tamper controls and required post-cleanup authentication.

Keep build preparation shared across families with the same exact C inputs,
compiler, flags and environment. Retained Native measurements in
`../translation-scope-execution/ticket10/performance-results.md` show the
improved small-family Fixture span at 0.843 seconds while compiler producers
total 8.820 seconds. The latter is summed producer duration, not an exclusive
wall-time component. That result motivates examining build reuse before more
Fixture scheduling work; it does not quantify current whole-batch speedup.

## 3. Optimize and actually select the controller and sandbox

The measured development controller is 333,415,728 bytes. `main` hashes its
entire executable before most CLI dispatch, including planning and an invalid
command. A disposable copy stripped of debug sections is 106,780,368 bytes.
No production binary was stripped or replaced.

| Bounded observation | Original debug copy | Stripped copy |
| --- | ---: | ---: |
| Invalid command, startup only | 8.40 s | 1.84 s |
| Catalog | 5.52 s | 1.98 s |
| Whole-catalog plan | 14.30 s | Timed out at 45.02 s |

Catalog output bytes matched. The earlier in-place planning observation was
32.03 seconds. Concurrent compilation was active on the host. The failed
stripped-plan control and variable timings prevent an end-to-end speedup claim;
there is no successful plan-output equivalence result for that comparison.
Raw output, timeout diagnostics and timing files are retained.

Use current optimized controller and sandbox builds for sustained verification,
with debug symbols stored separately if needed. Consider initializing executable
identity only in commands that consume it; commands producing or authenticating
evidence must retain their required identity checks. Benchmark matched builds
in a quiet window before choosing a profile or asserting gains.

The installed production binaries date from October 14 at 10:58. The later
architecture performance integration explicitly left that installation
unchanged. Rebuilding main alone therefore does not deliver those changes to
the installed supervisor or its separately selected sandbox. Adoption requires
a new correctly frozen installation/policy; changing pinned binaries in place
would violate their evidence contract.

## 4. Make whole-batch cost visible

The latest retained batch pointer selects
`out/port/decomp-run-20261013/host-integrated/batch.toml`. Its metrics record has
`enabled = false`, no stages or processes, and an incomplete zero-duration
aggregate. Enable the existing mechanism on the next ordinary batch command:

```sh
SM_PIPELINE_METRICS=enabled <current-sm-pipeline> batch verify <the-required-selection-and-batch-options>
```

Add or report separate durations for planning, cache lookup/input hashing,
compiler execution, shared-capacity wait, Fixture execution/comparison,
authentication, evidence publication and cleanup. Report current reuse time:
`verify_one` currently clones the previous `FamilyRecord` on reuse, preserving
its old `seconds` value, and starts its fresh timer only after the reuse check.
Those saved durations cannot measure authentication cost on a warm retry.

The historical batch has 106 verified and 24 rejected entries. Its slowest
recorded families include `floater-spike-motion` (573.634 seconds),
`observed-projectile-initializers` (512.130 seconds), and
`mother-brain-blue-ring-lifecycle` (394.322 seconds). These predate current
format/performance changes and include overlapping family work. Do not treat
their summed seconds as batch elapsed time or current performance.

Ordinary batches already default to three scheduled families and four Fixture
jobs, sharing the four-process pool. Binary Fixtures, pristine fork children,
dependency-ready scheduling and authenticated build caching are already present.
Increasing concurrency should follow phase measurements and retain one shared
capacity across batches, compilers and fixtures.

## Evidence and reproduction

- `probe.rs`: Rust driver with a 45-second cap per read-only command.
- `probe-summary.txt`, `*.timing.txt`, `*.stdout.toml`, `*.stderr.txt`: raw probes.
- `plan-original.stdout.toml` and `selection-summary.json`: current selection.
- `historical-batch-summary.json`: historical family timing summary.
- `inputs.sha256`: measured controller/source identities before cleanup.
- `cleanup*.toml` and `retirement.txt`: scoped cleanup and probe retirement.

The probe stopped with a nonzero result on the stripped-plan timeout; that
control is not reported as passed. The successful original plan and matching
catalog reads provide the current findings. No actual-C or Live producer ran,
and no new gameplay verification stage is claimed. Disposable controller
copies and the probe executable are retired after the scoped utility passes;
their hashes remain as measurement provenance, not retained admission readers.
