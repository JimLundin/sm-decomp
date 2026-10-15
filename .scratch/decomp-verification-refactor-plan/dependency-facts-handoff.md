# Ticket 03 dependency facts and identity handoff

Status: local implementation and required combined corpus controls passed;
finished-run cleanup and corpus retirement/readback completed. Root still owns
integration acceptance, implementer retirement and ticket resolution.

- Owner: `/root/dependency_facts_resume` (replacing stopped `/root/dependency_facts`),
  under root session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
- Worktree: `/home/jim/sm-decomp-performance-dependencies`.
- Branch: `performance/dependency-facts-20261015`.
- Original baseline: `94104cc224b6997c7eb1ad946be451af168287b5`.
- Final implementation checkpoint: `2b2712bc7f4b52db9edf7c9be6add6a221b32568`.
- Proof root: `/home/jim/sm-decomp-performance-dependencies-proof`.
- Exclusive Cargo target: `/home/jim/sm-decomp-performance-metrics-target`.
- Shared settings: `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`,
  `SM_PIPELINE_WORKERS=4`. Builds use locked/offline Cargo, one job, dev/test
  debug information disabled, and the proof root's `tmp` directory.
- No active local build/test/readback process or corpus mutation remains.

## Ownership and integration

This worker owns owned Rust source facts, canonical input discovery, affected
selection and recipe/computation admission checks with their scoped tests. The
bulk-context worker owns `InputView`, captured discovery/wrapper generation,
Native capture and catalog capture. Root owns main, shared ticket status,
combined acceptance and implementer retirement. Dependency links to main's
`deps/rustysnes` and `deps/snesrev-sm` are read-only. No gameplay change,
production policy change or new Native admission authority is included.

Bulk integrated the final guard and froze combined tickets 02/03/04/06 at
`b1f869102ac69897be674acfef682e3d867c6be6`. Its 48 owner, five catalog and four
public actual-C/Native controls passed. This worker used its matching retained
CLI for all required 177-family observations below. Ticket 08's older source is
not proof of these changes. Root has assigned the next ticket 07 in a new
`/home/jim/sm-decomp-performance-graph` worktree from this combined foundation;
the metrics target is released from ticket 03 for that explicitly assigned use.

## Source facts and identity changes

`source_facts.rs` retains an exact logical path, SHA-256, byte extent, syntax
identifiers, recognized input macros, supported literal module edges and
incompleteness. It discards AST/token structures. The facts are owned and
`Send + Sync`; cached facts and source bytes hold released-on-drop memory
reservations. Retained-buffer counters do not claim to bound parser transient
allocations or whole-process RSS.

The scanner has an 8 MiB source limit, 1 Mi token limit, delimiter depth 96,
grammar spine 512, 65,536 facts and traversal work of at most eight times the
token limit. It inherits the operation deadline. Resource failures stay errors;
unsupported syntax returns explicit incomplete facts. Comments and ordinary
literal prose do not create dependency edges. Real include/environment macros,
unmodeled macros/runtime capabilities and FFI remain unresolved inputs. Cfg
branches are unioned; rooted import aliases and literal top-level relative Rust
module paths are recognized.

The parent-glob proof (`5ae6970`, red `eeae31b`) handles the real Ridley pattern:
a literal child with top-level `super::*`, builtin derives and `vec!`. Raw child
facts remain incomplete outside context. A complete parent must explicitly name
that child and have neither a glob nor a potentially shadowing builtin import.
Only the particular builtin-shadow uncertainty is discharged. Every
`(file,parent)` placement is checked before source deduplication. This is not a
general Rust module resolver.

Recipe admission (`13e4d2b`, red `18c4501`) now compares the actual admitted
dependency/ABI contract with the required registered adapter. Caller
`FamilyEvidence.kind` is a hint, not computation proof. The root Cargo manifest
and lock and identity-checker source are required stamps. Existing broad source,
candidate, requirement, artifact, producer, comparator and Native attempt checks
remain required. There is no new result ledger or broad harness narrowing. See
`dependency-identities.md` for the four existing identity owners.

## Planner changes and controls

Complete transitive root closures are reused within one invocation under a
16 MiB optional closure-cache bound. Pending visits are deduplicated; each
family's reported memberships still count against the unchanged entry budget.
Reds `4e916d0` and `e4d5472` demonstrated repeated traversal and asset reads:

- `logs/planner-cycles-red.log`: 70 families sharing a 32-node cycle exhausted
  the 65,536-entry budget at entry 65,537, after only 50,326 bytes.
- `logs/planner-assets-red.log`: 70 declarations sharing a 4 MiB asset exhausted
  the 256 MiB read budget after 264,243,160 bytes.
- Fix `3aa65bd`: `logs/planner-public-green.log` has 14 passing public controls
  in 31.16 s; `logs/planner-owner-green.log` has three in 1.01 s.

The real corpus has about 214 MB of distinct external assets. Fix `1e3b488`
defers asset authentication until finalization, reads each distinct declared
asset once and checks every declaring family's expected digest. A mismatched or
missing asset keeps its families selected. Coherent before/after-read versions
are retained and rechecked at the end of the pass after final source/catalog
validation. Version identity includes device, inode, length, mtime/nanoseconds
and ctime/nanoseconds. Source/namespace checks and the original budgets remain.
Counters distinguish asset declarations from actual reads.

Red `6421800` showed why lexical normalization cannot prove filesystem identity:
`fixtures/alias/../input.bin` can name different bytes through a symlink. Final
guard `2b2712b` refuses parent traversal as unresolved before normalization.

Final local controls at `2b2712b`:

| Evidence under proof root | Result |
| --- | --- |
| `logs/asset-alias-green-build.log` | Completed final build in 22.23 s |
| `logs/asset-alias-owner-green.log` | 3/3 pass, 0.28 s |
| `logs/asset-alias-public-green.log` | 15/15 pass, 24.79 s |
| `logs/parent-facts-green.log` | 9/9 source-facts controls pass, 0.02 s |
| `logs/parent-inputs-green.log` | 6/6 canonical discovery controls pass, 0.60 s |
| `logs/asset-alias-red.log` | Wrong exclusion reproduced before guard |

The original cycle fixture still contains two valid Rust files with 8,192
repeated calls each and its original one-second deadline and byte cap. Neither
fixture nor deadline was relaxed. Initial deadline failures are retained in
`logs/discovery-{final,deadline-recheck,deadline-isolated}.log`. The unchanged
binary subsequently passed six isolated runs in 0.46–0.57 s and the final parent
input suite. Its SHA-256 was
`3efc067d30530fa11e7458138b24523d7d4021a866d8076e08923a0a55494462`.

## Actual-C identity evidence

`logs/identity-green.log` records the registered-admission control passing in
18.84 s at `13e4d2b`. It exercises one synthetic actual Rust/C Fixture pair,
not the production 16-pair batch. The actual Rust compiler dep-info includes
candidate → helper → literal `extra_impl` → `adjust`, and excludes a
`compile_error` source mentioned only in prose. Wrong recipe kind, claimed
manifest, candidate/transitive/comparator/Cargo.lock changes, requirements
changes and a missing source are refused. Inputs are restored and independent
admission then passes.

- Red verdict: `identity-red/batch/families/fixture/evidence/1792084387570022-122792-1/verdict.toml`.
- Green verdict: `identity-green/batch/families/fixture/evidence/1792084941132568-171040-1/verdict.toml`.
- Matching readers and test binaries: `tools/identity-{red,green}/`.
- Red CLI SHA-256: `72322f5d2260c1af3220685fe42e5c3a7cf48e155a73e2cc72f81ec6048517bb`.
- Red test SHA-256: `0fdac66238e4128a47bb1746ad3e4d41c485a4734418d7d51e964bf9dc2107e6`.
- Green CLI SHA-256: `989554c5685d9727afde9e21a5209893c0f7c18cacf585f8e015cccfd89ffb36`.
- Green test SHA-256: `153222adb48edcecdda6becde224964723c145c9e5ad199fde77f4436cb5f465`.

Both verdicts independently authenticated after their run cleanup; the receipts
are `identity-{red,green}-post-cleanup-authentication.toml` and corresponding
stderr logs are empty. These snapshots retain their original absolute source
paths. Reauthenticate with each matching copied reader after retirement.

## Required corpus observations

All four controls passed on the original 177-family corpus, with the original
256 MiB read budget, 65,536-entry limit and inherited deadline. They used clean
combined source `b1f8691` in the separate disposable corpus worktree, with one
explicit mutation per control. Gameplay bytes/names were restored before that
worktree was retired.

The exact debug CLI is retained at
`/home/jim/sm-decomp-performance-bulk-proof/b1f8691-20261015/tools/sm-pipeline`,
SHA-256 `ed88f55951b3ef8d853b4a283cef0205b4f675e3edb4f3d698dcac5958df0acc`.
The matching `sm-translation-sandbox` is in the same directory, SHA-256
`441daca1db663db0508d5977dfd4aaf005428d341e9ab0860d1050a24978d585`.
Both are bound in `corpus/reader-tools.sha256` and must remain available.

| Control | Selected / excluded | Wall / planner seconds | User / system seconds | Max RSS KiB |
| --- | --- | --- | --- | --- |
| Documentation only | 3 / 174 | 7.77 / 5.192 | 5.13 / 1.58 | 138,544 |
| Actual audio behavior change | 165 / 12 | 5.03 / 3.309 | 4.77 / 0.25 | 138,700 |
| Audio source rename | 165 / 12 | 2.78 / 1.079 | 2.63 / 0.14 | 130,916 |
| Audio source deletion | 165 / 12 | 2.74 / 1.060 | 2.58 / 0.15 | 132,044 |

The real transitive change was `queue_music`'s `delay.max(8)` → `delay.max(9)`.
Rename moved `audio.rs` to `audio_namespace_control.rs`; deletion removed
`audio.rs` without a replacement. The Rust report helper asserted that all three
selected exactly the baseline's three already uncertain families plus the 162
previously excluded families whose baseline closure contains `audio.rs`.
All 177 catalog members remained present. Full reports, all uncertainty, exact
mutation/status, timings and assertions are under proof `corpus/`; see its
`README.md` for commands and retention dependencies.

The documentation plan charged 257,399,575 bytes / 17,874 entries, parsed 45 Rust
sources, reused 437 root closures and read 1,360 distinct assets for 1,548
declarations. The three remaining uncertainty reasons are:

- `plm-instructions`: generated PLM fixture inputs require verification.
- `ridley-subsystem`: unknown Offline family adapter: ridley-subsystem.
- `ridley-subsystem-captured`: external declared dependency lacks changed-since history.

These are single debug-build observations with the existing filesystem cache,
not controlled cold-cache measurements or whole-batch verification speedups.
The reports explicitly admit no verification evidence or Live replay.
Earlier diagnostic observations are retained but are not successful final gains:

| Observation | Outcome |
| --- | --- |
| `corpus-documentation-c1f4c83.toml` | 20 excluded / 157 selected; 6.23 s, earlier alias uncertainty |
| `logs/corpus-documentation-13e4d2b.*` | Budget failure, 68.36 s |
| `logs/corpus-documentation-4e916d0.*` | Budget failure, 6.55 s; 268,351,464 bytes / 8,068 entries |
| `logs/corpus-documentation-green.*` | Intermediate budget failure, 9.06 s; 268,424,131 bytes / 12,153 entries |

`logs/corpus-inputs-open.trace` and `corpus-input-read-summary.txt` found 1,582
unique opened paths and 255,262,146 unique file extents, without hardlink
sharing. About 214 MB is assets and 40 MB declarations. This is an open/extent
diagnostic, not exact physical I/O. Combined raw-catalog capture and the final
asset pass allowed the unchanged corpus to complete within the same budgets.

All final controls used this command from the frozen corpus checkout:

```sh
sm-pipeline batch plan --profile offline-all --changed-since HEAD
```

The complete command was wrapped in `/usr/bin/time -v` and a 180-second outer
timeout; no internal planner limit was overridden. Matching raw reports and
`*-summary.toml` files preserve exact selection assertions rather than guessed
family counts.

## Cleanup, readback and remaining integration

Each actual-C root's `retention.toml` records both cleanup utility passes and
private-cache/.git removal: red 22,036,996 bytes, green 22,037,004 bytes. The last
utility passes found zero disposable bytes; each deduplication checked three
snapshots. Required sealed artifacts, exact source snapshots and readers remain.

After all final controls stopped, both whole-proof utility passes completed:
`cleanup-final.toml` found zero disposable bytes; `cleanup-deduplicate-final.toml`
checked six snapshots and reclaimed 262,144 bytes. The matching copied old/new
family readers then independently authenticated their actual-C verdicts again:
`identity-{red,green}-final-authentication.toml`, both `offline-verified`, one
case/routine, with empty stderr. Sealed artifacts, failure diagnostics, exact
source snapshots and matching readers are retained.

The corpus worktree was removed after `source-retention.toml` copied and checked
all 223 relative planning inputs (40,845,370 content bytes). That small snapshot
now occupies the same referenced path
`/home/jim/sm-decomp-performance-dependencies-corpus`, without Git metadata,
build outputs or dependency symlinks. Allocated bytes fell from 218,882,048 to
41,349,120: 177,532,928 bytes reclaimed. External assets remain at their original
read-only absolute paths; no shared store or main asset was removed.

The retained Rust `tools/planning-retention` verified every input hash in all
four reports after corpus retirement. Its source and hash are retained. The
transitive report uses only the explicit `corpus/audio.transitive.rs` override;
the wrong original audio bytes were shown to fail before successful readback.
`corpus/*-post-retirement-readback.toml` and their empty stderr preserve those
results. This authenticates advisory input hashes, not verification admission or
the completeness of uncertain dependency knowledge.

Final evidence hashes and disk measurements are retained in proof
`evidence.sha256`, `cleanup-after.bytes` and `cleanup-filesystem.bytes`. The
metrics target has no ticket 03 writer and its needed diagnostic binaries are
copied under `tools/final/`; root explicitly assigned its subsequent exclusive
use to ticket 07 rather than disposal now.

Root still owns retirement of the integrated implementer worktree
`/home/jim/sm-decomp-performance-dependencies`, final combined acceptance and
shared ticket resolution. All evidence's required source already lives outside
that implementer worktree. Reauthenticate the retained actual-C verdicts with
their matching copied readers after root removes it. Keep ticket 03 claimed
until that final integration/retirement is recorded.
