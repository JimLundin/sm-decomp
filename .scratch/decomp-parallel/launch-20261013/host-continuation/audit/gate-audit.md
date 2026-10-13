# Held Ceres integration gate audit

Status: complete read-only audit
Blocked by: None
Auditor thread: `01a1357f-a434-7802-ad4f-881646255fdd`
Integrator thread: `01a1356e-a8dd-7a80-88b0-1239a6a9b876`

This supports the running original `(integrate-shared-game-host,
48656a2-host-v1-1,1,ceres-20261013)` claim. It creates no replacement claim,
source authority or acceptance result. Source, Git and ledger are unchanged;
no verification, live replay or cleanup command was executed by this auditor.

## Findings

Both retained host verdict hashes match the reviewed bridge:

| Family | Retained verdict | SHA-256 |
| --- | --- | --- |
| game-host-projectile | `out/port/ceres-continuation/game-host-projectile/1791873394532541-258459-0/verdict.toml` | `18f355479a34ef10be6196e2c1507f6ded34d522693f068feb1313078492abad` |
| game-host-room-scroll | `out/port/ceres-continuation/game-host-room-scroll/1791873418836128-259326-0/verdict.toml` | `ddb49cbd79a7249b45745044f9f6c95a7e98671fa74e0b313a7f81bf8c869d1d` |

Every file listed under each verdict's artifacts table passes its stored SHA-256.
The records describe seven projectile pairs and three room pairs, five routines,
zero CPU fallback calls/instructions, five remaining C projectile callbacks and
51 remaining C room callbacks. Actual logs contain ordered bank88 HDMA requests,
PLM callback before/after WRAM and seventeen ordered VRAM flushes per room case.
These are retained Offline results in the declared fixture domain; Live replay
and original-ROM parity remain untested.

All five submitted host files still match `host-implementation.toml`. Both
verdicts' Rust dependency and harness maps match current source except
`crates/pipeline/src/main.rs`: retained hash
`1c2076de53340f6fd0af0489db696da299134f11d9a07bfd997fc9f10e47b3f4`,
current hash `cec9934fcc6f80a7038a9fa752e0754c28a25249822aba84baac0ec9501f673e`.
The host module, all its port dependencies, callback/family/build/input/cleanup
implementation and templates remain byte-identical to the retained seals.
`family::authenticate` checks both maps against current source unconditionally;
current main therefore cannot authenticate these old receipts as current
verification. Fresh current-source receipts are required. Keep old runs intact
as historical evidence.

The current launch host-focused stdout records 31 passing host, room-scroll and
Samus projectile tests. This auditor inspected that output and did not rerun it.
The shared host implementation contains transactional refusal before a request,
owned failure after a request starts, private canonical WRAM and synchronous
ordered host calls; recording controls cover first/later failure, response
mismatch, PLM mutation and all seventeen room flushes.

## Exact next commands for the integrator

Use cwd `/home/jim/sm-decomp` with
`SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline` and
`SM_PIPELINE_WORKERS=4`. Freeze the current source throughout these gates and
final admission. The original submitted host baseline is
`48656a284329d432291eca23a2ae0d1d7c0a3e48` (`base-head = "48656a2"` in
the original ledger). Its main/callback/family/catalog changes are shared inputs,
so every discovered Offline family remains affected. A newer baseline requires
reviewed input reconciliation; current HEAD is not a substitute for this baseline.

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- batch plan --profile offline-all --changed-since 48656a284329d432291eca23a2ae0d1d7c0a3e48
cargo test -p pipeline --locked --offline --bin sm-pipeline callback::tests
cargo test -p pipeline --locked --offline --bin sm-pipeline family::tests
cargo run -p pipeline --locked --offline --bin sm-pipeline -- batch verify --profile offline-all --changed-since 48656a284329d432291eca23a2ae0d1d7c0a3e48 --batch /home/jim/sm-decomp/out/port/ceres-continuation/host-integrated-20261013/batch.toml --jobs 3 --case-jobs 4 --fresh
```

Initial focused controls have a 60-second bound; declared merged integration
gates have at most 3600 seconds. The affected batch includes both new host
matrices, so separate matrix reruns are unnecessary after its success.
If a focused diagnosis is needed before the entire batch, the same batch command
can instead use `--families game-host-projectile game-host-room-scroll` with a
distinct batch path; that two-family result alone does not satisfy all-affected
completion.

For each admitted current receipt:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- family-authenticate --checkout /home/jim/sm-decomp --verdict CURRENT_RUN/verdict.toml
```

The original authenticated integrated result must name submission
`(shared-game-host,48656a2-host-v1,1,ceres-20261013)`, exact submitted commit
`73547105d870f8b05f8c6bcf55b768c223889fdb` and clean frozen merged source,
and retain passing batch evidence covering the selector's complete required set.
Original source was already merged by `e8c9c3e`; do not merge it again.
The coordinator completion contract rechecks required selection from the
submission baseline, authenticates selected batches, requires retired implementer
worktree, completes both cleanup passes and authenticates again. Stage publication
and durable legacy claim result admission remain the root integrator's work.

## Controller readiness after host completion

Read-only Git diff from the controller's original baseline to current HEAD is
empty for all five approved controller paths. Submitted commit
`223f8a1b0c7382817b1290d13c7a41f4e722d5e1` changes four of them and introduces
the compact observer; full `prefix_observer.lua.in` stays byte-identical. No
intervening source conflict was observed on the approved paths. No merge was run.
Reserve only the existing ready `integrate-compact-controller`, revision
`48656a2-compact-v1-1`, after completing the host claim, retaining its original
submitted worktree `/tmp/sm-ceres-continuation-compact-controller-1` and readers
until integration and post-removal authentication complete.

Required merged controller gates are the holding's focused unit/public prefix
tests and 2240-frame compact run with a 60-second whole-run cap. Expected compact
exit is 2, a nonaccepting diagnostic: 1736 updates / 1887 samples must match and
the independent changed-RNG control must reject. Full-profile legacy capture
compatibility remains a distinct gate; the added `observer_origin` handles old
sealed absolute observer output paths and regenerates every script byte. The
longer 8600-frame retained native run can be reused only with unchanged exact
compact observer identity; its parent subject timeout remains failure.

The known extra-poll divergence remains ordinal7741/frame8508/event63799,
C7960 versus native7959 polls, with matching prefix ending at ordinal7740/frame8317.
Repairing it requires reviewed writable scope for `tools/decomp/prefix_host.c`;
this controller submission does not authorize that change. Ticket78, ticket12,
Ceres escape, observed ordinary reset 32 -> 33 -> 34, whole Ridley, original-ROM
and credits acceptance remain open.

## Audit retention

Only this scratch report was created. No run or build output was created and
no cleanup was needed for this read-only inspection. Existing source/evidence
and readers were intentionally retained for the integrator.
