# Activation validator recovery handoff

Status: source submitted; focused verification and scoped cleanup complete; actual drill and integration-owner worktree retirement pending
Blocked by: None
Owner: `/root/session_tools_finish`, real thread `01a13474-5fcc-7861-84f8-5c353938bdd2`
Branch: `decomp-preparation-activation-20261013`
Worktree: `/tmp/sm-decomp-preparation-activation-20261013`

## Source handoff

The recovered validator was checkpointed at `2db3318d1ae8755e8e247f9a44051f015e562646`. Final commits, in order:

- `c2fbc17`: validate sealed commands, disjoint reservations, exact completed integration claims and the highest observed integrator generation; add read-only `validate-activation` using the same full validator as activation.
- `2714b57`: fence submitted legacy claims against their historical integrator `parent_thread`, with `owner_thread` fallback; distinguish the submitted implementer from the integrator in the regression.
- `c88c7a1`: preserve mandatory sealed submission evidence in that regression.

Only `crates/pipeline/src/authority.rs`, `crates/pipeline/src/drill_evidence.rs`, and `crates/pipeline/tests/session_authority.rs` were changed. Owned Git paths are clean. Production authority and ledgers were not mutated. The existing production bridge remains sealed unchanged.

The typed execution schema agrees with the drill harness. Acceptance requires retained actual Codex launch JSONL, command/process/request/output seals, fenced transfer, complete integrated submission tuples, maintained gate/authentication/cleanup evidence, retired implementer worktrees, and native four-holder/fifth-waiter lease evidence. Boolean summaries and synthetic evidence alone are rejected. Validator test fixtures do not establish that actual independent Codex sessions ran.

## Completion evidence

Retained evidence: `activation-recovery-evidence/`, with three final source snapshots and `artifacts.sha256`. All retained hashes verified after relocation.

Locked/offline verification used `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`, `SM_PIPELINE_WORKERS=4`, private `CARGO_TARGET_DIR=/tmp/sm-decomp-preparation-activation-target-20261013`, and the local RustySNES patch at `/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core`.

- `cargo test -p pipeline --test session_authority --test decomp_sessions --locked --offline --config 'patch."https://github.com/doublegate/RustySNES".rustysnes-core.path="/home/jim/sm-decomp/deps/rustysnes/crates/rustysnes-core"'`: session controls 15 passed; authority controls initially 9 passed and 1 failed because the newly added fixture omitted mandatory sealed evidence. Diagnostic logs retained as `focused.stdout` and `focused.stderr`.
- Corrected final `cargo test -p pipeline --test session_authority --locked --offline` with the same environment and patch: 10 passed, 0 failed, 1 ignored subprocess helper; logs `authority-retry.stdout` and `authority-retry.stderr`.
- `git diff --check`: passed.

Cleanup ran after all private-target writers exited:

```sh
/tmp/sm-decomp-preparation-activation-target-20261013/debug/sm-pipeline cleanup --root out/activation-verification --apply
/tmp/sm-decomp-preparation-activation-target-20261013/debug/sm-pipeline cleanup --root out/activation-verification --deduplicate --apply
```

Both applied successfully and reclaimed 0 run bytes; receipts retained as `cleanup1.toml` and `cleanup2.toml`. The disposable private target measured 1,481,441,788 bytes and was removed with `find /tmp/sm-decomp-preparation-activation-target-20261013 -depth -delete`. Remaining filesystem space: 221,190,111,232 bytes. Retained log/source dependencies require no private binary. The cleanup utility binary SHA-256 was `30222861e109dc98514bb46555523ca2d819c9e37f57d6000da01233deb74c07`.

The integration owner must retire the registered activation worktree after integration, preserving any source path needed by later frozen drill evidence. Actual drill acceptance remains the preparation coordinator and drill harness owner's next step.
