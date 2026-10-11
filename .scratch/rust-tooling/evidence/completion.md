# Python and internal JSON migration completion

Status: complete

The user narrowed completion on 2026-10-11 to replacing Python with Rust and
using another serialization format. Full gameplay readiness and new optimization
increments are outside this refactor. Earlier in-progress verification plans
are superseded by this scope; interrupted runs are not reported as acceptance.

## Result

All maintained Python sources/tests and runtime calls are removed. Rust commands
cover worker/build/cache ownership, cleanup, setup, movies/emulators, fixtures,
offline/batch verification, packed live, decomp/RNG, readiness/iteration,
diagnostics and storage utilities. CI, recipes and active documentation use them.
Readable configuration and small metadata use typed TOML. Material states,
baseline calls, timed cases and ledgers use explicitly versioned binary formats.
Internal JSON adapters/configs are removed. Only external SingleStepTests JSON
ingestion remains in pipeline patterns and fuzz; historical sealed evidence and
its original sources/readers are preserved.

## Verification

- Workspace regression passed: `/tmp/sm-root-workspace-final-regression.log`.
  Complete typed family config and all44 fixture domains were checked separately;
  real pattern corruption/substitution controls also passed.
- Final changed error path passed its real origin/publication refusal regression:
  `/tmp/sm-root-readiness-final-refusal.log`. Refusals durably record error/reason,
  clear eligibility and leave no accepted pointer.
- Final strict pipeline Clippy, build, fmt and diff whitespace passed:
  `/tmp/sm-root-migration-clippy-final.log`,
  `/tmp/sm-root-migration-build-final.log`. Prior complete workspace strict Clippy
  passed `/tmp/sm-root-clippy-auth-final.log`.
- Final maintained Python/runtime and JSON-config guards both passed:
  `/tmp/sm-root-migration-guards-final.log`.
- Bootstrap tests/Clippy/doctor, independent codec/evidence/movie/diagnostic
  controls, session recovery/lineage and selection refusal checks passed; see
  `utilities-final.md`, `readiness-final.md` and `metadata-integration.md`.
- Baseline9000 frames/23 captures, complete decomp acceptance63 fixtures/23
  captured calls/3 original-ROM controls/11 fault controls, RNG65536 seeds/16
  fixtures/8783 effects, offline system475/collision1752/PLM1184 cases and packed
  four9000-frame subjects passed. These are bounded workflow evidence, not full
  original-game readiness claims.
- Feedback9/9 authenticated after cleanup. Fresh RNG authenticated after cleanup.
  Packed live authenticated after cleanup:4subjects,148artifacts each, retained
  binary valid, `sm-evidence-v2`, verdict SHA
  `b37175079604dd3a027167844523f9e882e64e99bef6038e08b48bbd93cb04ab`.
  Receipt: `/tmp/sm-root-packed-auth-after-cleanup.toml`.

## Cleanup and retention

Native finished-run cleanup was used for all completed/failed/interrupted runs
in this continuation. Zero-byte receipts are retained. Fresh batch automatic
dedup reclaimed1345454080bytes; fresh RNG dedup176081290bytes; feedback build
121175687bytes; packed unsealed archives47304570bytes. Packed follow-up dedup
checked0snapshots/reclaimed0bytes. Authentication afterward passed.

Scoped Cargo clean removed disposable config converter100.5MiB, fixture
checker73.7MiB, migration Clippy132.6MiB and build1.4GiB targets; verifier149.2MiB,
bootstrap293.1MiB then recreated153.4MiB; stranded feedback115.6MiB; interrupted
pattern292.5MiB and final interrupted pattern372.3MiB. Exact tool receipts and
earlier cleanup are recorded in the section evidence and `/tmp/sm-root-cleanup-*`.
Verifier executable is retained at `/tmp/sm-retained-verifier/sm-retained-verifier`.

The final corpus run was cleanly stopped after70000/70011 cases passed with
zero failures/skips; remaining11 fixtures/full timing are not claimed. Its
incomplete verdict, binaries, corpus, cases, ledgers and logs remain. Readiness
was cleanly stopped with incomplete nonaccepting verdict and cleanup0bytes.
The redundant post-cleanup batch reuse check was terminated after the fresh
batch itself completed; no reuse success is claimed for that interrupted check.

No implementer worktrees were created. `/tmp/sm-decomp-pipeline` remains the
reviewable integration worktree on `decomp-pipeline-performance` and a source
dependency of retained evidence. Shared packed store objects, pointers, sealed
readers/source snapshots and diagnostic evidence are retained. One old
historical pattern build is retained because the available historical binary
expects a different format; this conservative retention is not migration work.
Main target remains the developer build/cache. Bootstrap/verifier disposable
targets are absent. Disk free217430736896bytes (about202GiB).

The verification above preceded the user's subsequent commit-and-merge request.
Local migration tickets are resolved by this implementation, verification and
cleanup evidence; earlier commit-pending notes are historical.

## Commit and integration, 2026-10-11

Migration commit `c89db9bcf824a3c4f973550d390a95c3f5d05f97` on
`decomp-pipeline-performance` was merged into `decomp-port-integration`, preserving
its newer gameplay changes from `0fe47e36698e8530655e46ceffb0066c227e0815`.
The merge record and focused verification are in `merge-integration.md`.

The now-integrated `/tmp/sm-decomp-pipeline` Git worktree was removed. Exact small
source snapshots, dependency/ROM symlinks, movies, sealed run artifacts, shared
stores and readers were restored at their original absolute paths. Its disposable
target tree (15566278634 logical bytes) and redundant fixture matrices
(157929412 logical bytes) were removed. Cargo clean initially refused the target
because its CACHEDIR.TAG was missing; removal used the clean, audited Git worktree
after preserving every evidence dependency. Other developer worktrees and the
main checkout's build/cache remain. Retained packed authentication after this
cleanup is recorded in `sm-merge-packed-auth-after-retention.toml`.
