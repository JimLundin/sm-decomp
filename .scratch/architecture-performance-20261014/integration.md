# Main workspace integration

Status: resolved
Owner: `/root`, session `01a13b35-079e-7151-8e5a-f759fee6bcf3`
Ticket: [06](issues/06-main-integration.md)
Verified source: `c0bd9061572ce56e4ef5218c8f60dd05aa25bd82`
Source baseline: `2942ba8e71b96aa7442e10cc8949db08a8bc6d80`
Evidence: `/home/jim/sm-decomp-architecture-runs-20261014/integration`

All five performance candidates are integrated into local `main`. Literal main
was an ancestor of the current work, so promotion preserved every existing
commit. The primary checkout is on main. The separate equipment implementation
and parked Original reuse were not part of this integration.

## Source and review

| Candidate | Main commits |
| --- | --- |
| 01 confinement accounting | Already present as `326c2aa`, `1198963`, `3c2e9e3` |
| 02 Fixture scheduling | `09cf01b` |
| 03 build preparation | `f0b47bc`, `16e708a` |
| 04 evidence restoration | `1cf27db` |
| 05 Live transactions | `9c5cfe4` |

The two conflict resolutions preserve Native's shared predecessor parser,
bounded waves, strict compilation and summed reservations. Ordinary execution
keeps indexed ready/dependent queues and acquires shared capacity per ready
Fixture pair. Native workers retain their shared bounded-input scope. All
`build_cache.rs` source registrations are present. Independent read-only review
by `/root/offline_performance` found no concrete defect in these merge seams.

## Verification

All **47 controls passed** with fresh build artifacts. Commands used locked,
offline Cargo, one compiler job, shared workers=4 and serial test execution.
`logs/validation-driver.log` records exact commands and `logs/*.log` results.

| Area | Passed |
| --- | ---: |
| Fixture worker/wave unit controls | 3 |
| Strict reservations and shared input budgets | 5 |
| Build preparation and process/cache controls | 11 |
| Actual-C capacity, independent chains and cancellation | 3 |
| Compiled Live adapters and Ridley frame controls | 8 |
| Evidence restoration/store controls | 15 |
| Fresh separate sandbox through public confinement run | 1 |
| Complete real Native proof | 1 |

The Native proof passed all 16 Fixture pairs across two routines, retained 34
durable producers, authenticated and reused every result on unchanged replay.
Missing coverage/inputs, undeclared compiler input, runtime parent/sibling access,
tampered invocation and aggregate prelaunch reservation controls also passed.
The full test took 71.21 seconds with the fresh debug helper; its positive proof,
authentication and replay portion took 30.389 seconds. This is correctness
evidence, not a controlled performance comparison with the prior optimized helper.

Two ordinary actual-C verdicts are retained under `fixture-capacity`; Native
positive and diagnostic proofs are under `native/combined-controls`. The Native
provenance still requires the independent source snapshot at
`/home/jim/sm-decomp-performance-20261014/native/provenance/source` and original
verdict named in `native/provenance/provenance.toml`.

Formatting and diff checks passed. Source hashes matched after all controls and
again after main promotion. Existing compiler warnings remain. The compiled
Live controls do not establish authenticated complete native Live replay.

## Tooling and scope

Frozen integration readers are retained in `bin/`; `bin/sha256.txt` binds the
controller, separate sandbox and Native test/authentication reader. The main
development controller and sandbox were rebuilt under `target/debug` in 2m47s;
`logs/main-tools.sha256` records their exact hashes. The installed status command
and rebuilt main command report identical production state.

The dirty assignment, published performance-review document, production registry
and deterministic fence match `logs/preserved-workspace.sha256`. Existing
untracked evidence was preserved. No installed production tool, policy, held
claim or gameplay stage was changed; Original production acceptance retains
the prerequisite stops reported by the installed status command.

## Cleanup and retention

The five implementer worktrees are retired; candidate branches preserve their
source history. Coordination documents are tracked files in main, replacing the
old symlink into 03-builds. Six small exact Live source files remain at the old
05-live path without Git metadata: `memory.rs` and five adapter templates.

The retained original Live generator expects Git at its old worktree path.
To repeat its measurement, compile the already-generated baseline/pooled probe
sources, or deliberately adapt a copy of the generator to read the retained
baseline template. The hashed original generator remains unchanged. Candidate04
before/after source snapshots are partial; Git history supplies the full workspace.

All writers completed before the two scoped utility passes:

```sh
bin/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/integration --apply
bin/sm-pipeline cleanup --root /home/jim/sm-decomp-architecture-runs-20261014/integration --deduplicate --apply
```

Normal cleanup found zero remaining eligible bytes; deduplication shared
10,485,760 bytes across 114 checked snapshots. Automatic Fixture cleanup had
already shared 1,835,008 bytes and removed 23,700,637 bytes of private Git/cache
data (`fixture-capacity/retention.toml`). Cargo clean removed 4,453 private target
files (Cargo reported 4.3 GiB). The five implementer worktrees and temporary
integration worktree are removed. Their branches and exact proof source remain.

| Identified disposable path | Apparent bytes before removal |
| --- | ---: |
| Five implementer worktrees | 2,197,280,306 |
| Private integration Cargo target | 4,073,405,058 |
| Temporary integration worktree | 439,718,727 |
| Empty private temporary directory | 4,096 |
| Restored tiny Live source directory (retained) | 37,318 |

Net removed path sizes total 6,710,370,869 bytes. Final observed available space
was 186,727,673,856 bytes, an increase of 5,041,516,544 bytes during retirement
and the main rebuild. This shared-filesystem delta includes other activity and
is not a physical reclamation measurement. Main's development target remains.

After retirement, all 11 candidate02 and both historical composition actual-C
verdicts authenticate, as do all three candidate04 stores. Candidate01 and
candidate03 retained manifests and all six Live source files pass their hashes.
Both new Fixture verdicts authenticate through the rebuilt main controller.
The retained Native reader authenticates four receipts: successful run,
compile-denial, runtime-denial and reservation-denial, covering 70 producers and
1,508 references. No producer executes during these checks. Results and exact
commands are in `logs/post-retirement`; all new source/tool/workspace hashes
pass in `logs/final-integrity.log`.

`completion.toml` records the verified source, main promotion, controls,
cleanup and remaining dependencies. Only the original unrelated assignment
edit and pre-existing untracked outputs remain outside this effort's Git scope.
