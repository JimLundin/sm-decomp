# Independent checkpoint and session review, 2026-10-13

Status: checkpoint and existing queue checks passed; real-session drill unproven
Blocked by: None for this review
Triage: ready-for-agent
Reviewer: `/root/checkpoint_review`

Recovery checkpoint **`d8a90451695408bbf72017ea04e7f4e915b06dd9`** is committed
on `affected-family-assignment`. This review records its completed source
checkpoint, supplementing the earlier
[checkpoint description](checkpoint-20261013.md). No gameplay run, live
acceptance, claim mutation, integration or Git/index operation was performed
by this reviewer. The separate report session's changing files were excluded
from source assertions.

## Passed: source checkpoint and retained dependencies

- `git show --stat d8a90451695408bbf72017ea04e7f4e915b06dd9` confirms the
  recovered source, ledger, handoffs and three backup archives are committed.
  All three archive hashes pass their committed manifest. All 1,441 originals
  pass their respective source/raw/generated manifests. Private extraction
  under `out/pipeline/session-review-20261013/archive-members` independently
  verifies all 537 source, 810 raw receipt and 94 generated receipt members.
  Source member names omit the leading `/`; their manifest was adjusted only
  for that prefix when checking the private extraction.
- `git worktree list --porcelain` lists only the main checkout and controller
  worktree. `/tmp/sm-ceres-continuation-compact-controller-1` is clean at
  `223f8a1b0c7382817b1290d13c7a41f4e722d5e1`. The commit is not an ancestor of
  the checkpoint. Its four changed files since baseline `48656a2` are within
  the five approved writable paths; integration remains pending.
- The ignored authored-source scan found no `.rs`, `.c`, `.h`, `.md`, `.py`
  or `.html.in` files outside the intentionally archived Ridley source
  contexts, after excluding their generated run trees. Untracked report
  audit/layout work belongs to the separately active report session and needs
  its own final scoped commit. This is an inventory of identified source, not
  a claim that Git contains private ROMs or every replay dependency.
- `ridley-live-final-verdict-reader.sha256`,
  `ridley-hardware-recovery/source-evidence.sha256` and
  `ridley-hardware-recovery/verdicts.sha256` all pass. Original frozen source
  contexts and readers remain at their authenticated paths.
- Controller/reviewer threads `01a133ec-4ef9-78b2-a8d7-248726023ce5` and
  `01a133f4-de2d-71e3-82ce-ead915ca3017` were read directly through
  `codex_tui.read_thread`: both are idle with completed turns. The worker
  reports that all children finished. Process inspection found no controller
  writer. The intentional report server remains PID `259921`, started
  `2026-10-13 08:37:24`, running `target/debug/sm-pipeline progress serve
  --root /home/jim/sm-decomp --bind 127.0.0.1:4185`.
- Read-only authentication of the completed 2,240-frame controller diagnostic
  succeeds through its retained reader:

  ```sh
  /tmp/sm-ceres-continuation-compact-controller-1/target/debug/sm-pipeline decomp-prefix-authenticate /home/jim/sm-decomp/out/port/ceres-continuation/controller/1791873256094852042-256891
  ```

  Output: `Prefix diagnostic authenticated; acceptance remains false`.
  The two longer interrupted/failing verdicts were preserved and hash checked;
  they were not promoted into successful acceptance.

Commands used for archive/original authentication:

```sh
sha256sum --status -c .scratch/decomp-parallel/checkpoint/archive-sha256.txt
sha256sum --status -c .scratch/decomp-parallel/checkpoint/frozen-sources-files.sha256
sha256sum --status -c .scratch/decomp-parallel/checkpoint/raw-receipts-files.sha256
sha256sum --status -c .scratch/decomp-parallel/checkpoint/generated-receipts-files.sha256
```

The retained ledger's SHA-256 is
`77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`.
Controller submission evidence matches ledger hash
`8da107cdff9c14182a0ffae1d24ce1d3acbadfccd742fccf4e31c0896bd3634e`;
host implementation evidence matches
`7b4f118b21b969bd02abc16192cbc3caccb1c118d5cef6a5d39e3751afeaffeb`.
The three controller verdict hashes match the checkpoint table exactly.

## Passed: existing queue and affected-selection tests

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 TMPDIR=/home/jim/sm-decomp/out/pipeline/session-review-20261013/test-tmp cargo test -p pipeline --locked --offline --test assignment_queue --test affected --target-dir out/pipeline/session-review-20261013/build
```

All **18 assignment queue tests and 9 affected-selection tests pass**. The
isolated test target and temporary directories were created before execution.
Existing tests exercise simultaneous coordinator calls across a worktree,
exclusive reservation, restart-persistent claims, checked owner-stop recovery,
new attempts, stale-result rejection, interrupted ticket projection,
submission path ownership, serialized integration and uncertain source inputs.
These tests use isolated temporary repositories. They do not launch two real
independent sessions or demonstrate a controller-crash handoff.

## Passed: practical read-only status and runbook commands

The built `sm-pipeline assign status` succeeds with:

```sh
--checkout /home/jim/sm-decomp --effort ceres-continuation --coordinator ceres-20261013 --capacity 3
```

Repeating with `--checkout /tmp/sm-ceres-continuation-compact-controller-1`
resolves to the same main-checkout ledger. Both reads preserve its bytes.
A read-only status request with capacity `4` correctly fails with
`assignment capacity differs from authoritative ledger`; this is an expected
negative check, not a failed requirement.

Current durable states are:

| Job | State | Practical meaning |
| --- | --- | --- |
| `compact-controller` | submitted | Source/evidence retained; writable ownership persists. |
| `integrate-compact-controller` | ready, deferred | Queue reports `integration owner is already running`. |
| `shared-game-host` | submitted | Source is merged; durable lifecycle is unfinished. |
| `integrate-shared-game-host` | running, agent `/root` | Interrupted reconciliation needs real owner inspection. |

The authoritative ledger retains capacity `3` and integration coordinator
`ceres-20261013`. Shared execution configuration reads `workers = 4` from
`out/pipeline/workers/capacity.toml`.

The existing Rust wrapper also successfully handles a stdin status request
with `operation`, `checkout`, `effort`, `coordinator` and `capacity`, using:

```sh
cargo run --manifest-path .scratch/ceres-continuation/Cargo.toml --target-dir out/pipeline/session-review-20261013/legacy-build --locked --offline --bin ceres-assignment-driver
```

Its build was private and removed afterward. No wrapper mutation was tested.
The stable [session runbook](../../docs/agents/decomp-sessions.md) and root
`AGENTS.md` pointer were reviewed against `main.rs`, `coordination.rs`,
`assignment.rs` and `translation.rs`. Supported flags, public response shapes,
claim/update schemas and the legacy-wrapper distinction agree with source.
The mutable `next`/`record` examples were inspected rather than executed.
The runbook correctly warns that `--no-translations` only controls discovery,
and existing integration jobs can still be selected.

## Limitations and required next checks

1. One global mutation lock serializes queue writes, but writable conflict
   checks iterate only the selected effort's ledger. Capacity and integration
   serialization are also per ledger. Separate efforts can overlap. All
   sessions need one authoritative queue plus reconciliation of historical
   ledgers; repository-wide path fencing remains implementation work.
2. The first submitter's coordinator becomes the retained integration owner.
   The current schema has no checked integration-owner transfer, and shared
   coordinator strings do not identify real sessions. A replacement session
   must establish the previous integrator's stop. Surviving workers retain
   ownership. Submitted work must continue through integration rather than
   being released because its implementer stopped.
3. Owner-stop validates artifact hash, exact stored agent, original coordinator,
   revision/attempt and an allowed stopped status. It does not inspect process
   or thread liveness. The caller must establish real liveness, account for
   surviving children and guard against PID reuse. Elapsed time grants no
   release.
4. Native `translation-scope-v1` admits 1–32 routines and exact `.rs`/`.toml`
   files under `crates/port/src`, `crates/port/tests` or `tools/port`.
   Infrastructure jobs use the legacy Rust engine wrapper and do not receive
   native gameplay result admission merely by being recorded there. Maintained
   infrastructure admission and automated handoff/restart support are future
   tooling work.
5. Source fingerprints cover shared pipeline, gameplay and tools namespaces.
   Disjoint gameplay ownership alone cannot guarantee unaffected verification.
   The integrator needs a frozen verification checkout and affected-family
   checks before admission.
6. The real two-session reservation/restart/integrator-recovery drill remains
   **unrun and unproven**. The planned tooling and owner reconciliation must
   precede that drill. Documentation and temporary-repository tests must not
   be presented as proof of independent-session operation.
7. At the initial review checkpoint, the plan/spec/map were not yet saved.
   The runbook and this initial review were committed independently at
   `2b53f68`. The final plan/ticket review below supersedes that temporary
   documentation gap.

## Passed: final plan, map and ten tickets

The stable spec, map, session runbook and all ten numbered tickets were read
and independently checked after delegated authors saved them. All **57 local
relative links resolve** across those 13 documents, covering 20 unique targets.
In particular, `../../../docs` from an issue resolves to the repository's
`docs`, and `../../decomp-strategy` resolves to the sibling scratch effort.

Every ticket has `Status: open`, `Triage: ready-for-agent`, a valid `Type:`,
`Blocked by:`, acceptance requirements and completion/cleanup evidence fields.
Research tickets 05 and 10 have `## Answer` sections. All map rows match their
ticket's type, status and blockers; every blocker refers to an earlier existing
ticket, with no missing reference or dependency cycle.

| Ticket | Blocked by | Reviewed sequencing |
| --- | --- | --- |
| 01 Legacy reconciliation | None | The sole currently ready task; existing claims stay authoritative. |
| 02 Global/session fencing | 01 | Extends existing Rust engine; retains bridged ownership. |
| 03 Maintained commands/handoffs | 01 | Disjoint interface work can parallelize with 02; final acceptance requires its validated authority contract. |
| 04 Two-session recovery drill | 02, 03 | Uses an isolated disposable checkout/effort before production activation. |
| 05 Subsystem catalog | 01 | Read-only whole-lane research can parallelize with tooling. |
| 06 Ordinary-reset Ceres escape | 04, 05 | Resumes existing controller/ticket 78 with approved child scopes. |
| 07 Vanilla progression | 06 | Adds route-required subsystem children as blockers; ready children can run independently. |
| 08 Save/load and credits | 07 | Persistence and route children retain exact state/input/evidence gates. |
| 09 Full Rust gameplay ownership | 08 | Requires supported behavior with zero gameplay C/CPU fallback. |
| 10 Map Rando research | 09 | Pins later seed/custom behavior and generates implementation children. |

The final spec, map and runbook agree: `.scratch/decomp-parallel/ledger.toml`
is **unactivated and absent**. The current Ceres ledger/token remains in force.
Ticket 04 requires two actual sessions, shared worker capacity, surviving-child
ownership, checked replacement, rejected stale/old authority writes and serial
fenced integration. Production activation additionally requires retained
passing drill receipts and the validated legacy ownership bridge. No production
claims are created to conduct its preactivation drill. Gameplay launches also
require approved catalog scopes and each child's actual dependencies.

Candidate modules are explicitly research starting points; they grant no
writable ownership or acceptance. Whole subsystem envelopes reuse existing Rust
and retain held broad gates, including tickets 78 and 141. Composed milestone
order does not serialize dependency-ready child implementation. Unknown inputs,
remaining C ownership and bounded evidence stay visible throughout.

Final startup/read-only commands and mutable claim/recovery examples agree with
the current schemas reviewed above. Existing capability limitations remain
explicit; this documentation review ran no new mutable queue command, gameplay
gate or real-session drill. New-file trailing-whitespace inspection and scoped
`git diff --check` pass. The separate active report's template/registry/index
changes were excluded from the plan's source and whitespace assertions.

Final documentation-only cleanup: no runs or build outputs were created, so
additional run cleanup is not applicable. Prior verification cleanup below
remains completed with its measured evidence. The plan deliverable is approved
for a scoped local commit; implementation, gameplay milestones and the actual
two-session drill remain open and **unproven**.

## Completion evidence: scoped cleanup

All verification/authentication processes completed before cleanup. Both project
utility passes on the private review root succeeded:

```sh
out/pipeline/session-review-20261013/build/debug/sm-pipeline cleanup --root /home/jim/sm-decomp/out/pipeline/session-review-20261013 --apply
out/pipeline/session-review-20261013/build/debug/sm-pipeline cleanup --root /home/jim/sm-decomp/out/pipeline/session-review-20261013 --deduplicate --apply
```

Ordinary output: `apply = true`, `bytes = 0`, `targets = []`.
Deduplication output additionally records `snapshots_checked = 0`.
Cargo test/extraction output has no finished gameplay verdict; its identified
unsealed build trees and copied archive members were removed under the
unsupported-format cleanup rule:

```sh
cargo clean -p pipeline --target-dir out/pipeline/session-review-20261013/build
cargo clean --target-dir out/pipeline/session-review-20261013/build
cargo clean --manifest-path .scratch/ceres-continuation/Cargo.toml --target-dir out/pipeline/session-review-20261013/legacy-build
```

Cargo reports 533 files/953.2 MiB, then 1,173 files/304.0 MiB, and
467 files/170.2 MiB removed respectively. Before removal, `du -sb` measured
1,133,536,208 apparent bytes for the test target and 152,782,186 for the
legacy-wrapper target. An exact-path Rust cleanup helper checked the private
root's allowed entries and rejected symlinks before removing 222,517,542
remaining apparent bytes, including extracted archive copies and empty private
directory metadata. The private review root is gone.

At the primary's explicit direction, the same helper removed only these four
disposable checkpoint helpers:

| File | Apparent bytes removed |
| --- | ---: |
| `/tmp/sm-decomp-checkpoint-archive` | 4,794,896 |
| `/tmp/sm-decomp-checkpoint-archive.rs` | 4,827 |
| `/tmp/sm-decomp-checkpoint-staged` | 4,748,592 |
| `/tmp/sm-decomp-checkpoint-staged.rs` | 3,683 |
| Total | 9,551,998 |

The disposable Rust cleanup helper removed itself and its source afterward
(4,560,964 apparent bytes). This completed cleanup after automatic command
review rejected direct `rm -f` style commands and requested a safer approach.
No shared target, report build, evidence store, source snapshot, retained reader
or submitted worktree was cleaned.

Final available space measured by `df -B1 --output=avail /home/jim/sm-decomp`:
**224,484,421,632 bytes**. Available space is a shared-filesystem snapshot, not
a before/after attribution to this review. Archive and whole-Ridley reader
manifests still pass after cleanup; the controller worktree remains clean and
the intentional report server remains running. Retained replay dependencies
include `out/evidence`, subject `evidence.toml` pointers, ROM/movie/captured
inputs, frozen source contexts, the controller reader and the Ridley readers.
Git source checkpoints do not replace those local dependencies.
