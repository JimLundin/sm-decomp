# Current-owner production readiness successor

Status: helper built and requests sealed; actual namespace validation pending final drill
Owner and required real caller: `01a13500-1ef8-73d3-9a0d-3446b6050eba`
Observer: `01a13501-0242-71b0-820d-fdea3808f4c9`
Thread inspection: 2026-10-13 11:26:10 UTC
Process inspection: 2026-10-13 11:29:17 UTC

This successor binds the recovered root to a prospective generation1 integrator
only inside the private readiness namespace. The predecessor
[readiness-final](../readiness-final/README.md) and its sealed artifacts remain
immutable. [predecessor.toml](predecessor.toml) records the exact predecessor seal.
Copied predecessor thread/process evidence and helper rejection diagnostics are
retained as historical files; current requests use only the new
`thread-inspection-current.json` and `process-inspection-current.toml`.
[helper-build.toml](helper-build.toml) identifies this successor executable;
copied old compiled-helper pointers do not authorize execution.

Direct thread inspection observed the current root active/inProgress; previous
readiness root idle with a failed terminal turn; older preparation root and Ceres
parent systemError/failed; Ceres controller idle/completed; host
notLoaded/completed; reporting idle/completed. Fresh process inspection observed
no retained Ceres claim path writer in cwd, argv or open descriptors. The original
Ceres three held claim tuples and worker identities remain unchanged:
compact-controller submitted, shared-game-host submitted and
integrate-shared-game-host running. A stopped submitter or parent releases none
of them. The exact original ledger and normalized legacy bridge seals remain
`77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89` and
`cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.

Fresh drill worker `01a134d2-0902-7c81-8ab6-4f98937db3ea` remains
active/inProgress in the prior collaboration tree. Its isolated children and
writers retain their claims and budget. The inspection retained independent
integration child `01a134fc-642b-7481-a4cb-d84c2d217cdd`, PID472259,
start ticks2687656, boot `7119fcce-5593-4526-aa9b-07127767c4ef`.
Root's cross-tree communication rejection does not establish stopping and does
not authorize another drill worker. The reporting server remains PID420446,
start ticks2249195 on that boot. Main Git/index belongs to the recovered root;
this package author owns only this successor directory.

The dependency-free Rust helper holds the existing host assignment lock without
changing its bytes and prepares a private policy through `bwrap`. Every host
path is read-only except the new output and a separate private assignment lock.
Namespace identity and private tmpfs are checked before preparation. Maintained
`validate-activation` verifies the real final drill against the exact frozen
source and returns state `prepared`. Neither host production authority nor
canonical ledger exists or is created. No gameplay/production claim operation runs.

The frozen subject and executable are unchanged:

- Source: `/tmp/sm-decomp-preparation-final-source-20261013`
- Full commit: `2d59536c059a1ca9c31d9278bff2d9222eafa56c`
- Fingerprint: `e5e412053f6dbe2cbadaba077192e885684fbb9f2699cafeaa6acd694879d78d`
- Engine: `/home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/bin/sm-pipeline`
- Engine SHA-256: `df6eb4dc3a9fd69e5d5923b5c22f34dea7b434cd7ee87f1b69085cd9e9431c84`
- Build receipt: [drill-final/engine-build.toml](../../drill-final/engine-build.toml)

Source/build/helper SHA-256 checks and predecessor authentication passed.
Current wrong-caller and direct-inner guards rejected before namespace launch;
no output directory or production state was created. Their diagnostics and
exit codes are retained in `helper-controls.toml`.
Both helpers were built with `cargo build --locked --offline` in the isolated
`/tmp/sm-readiness-recovery-20261013-target`; no runtime dependency was added.
The maintained engine/validator source and executable were not modified.
The helper's only behavioral change binds the new real caller and successor
output directory. The process inspector additionally records live isolated
drill processes. These package/build checks supply no actual readiness verdict.

Root runs the command below only after both sequential authenticated drill
integrations, 11 controls, cleanup, retirement, final manifest and recursive
retained authentication have finished. The final drill must be sealed at the
exact path below. Inherit root's actual `CODEX_THREAD_ID`; never override it.
Each output directory is new.

```sh
/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-recovery-20261013/bin/decomp-readiness \
  --engine /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/bin/sm-pipeline \
  --source-checkout /tmp/sm-decomp-preparation-final-source-20261013 \
  --source-commit 2d59536c059a1ca9c31d9278bff2d9222eafa56c \
  --source-fingerprint e5e412053f6dbe2cbadaba077192e885684fbb9f2699cafeaa6acd694879d78d \
  --build /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/engine-build.toml \
  --prepare-request /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-recovery-20261013/prepare-request.toml \
  --drill /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/concurrency-drill.toml \
  --output /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-recovery-20261013/production-validation-1
```

Successful validation retains private policy state `prepared`, preserves policy
bytes and seals unchanged host ledger hashes. Failed namespace runs retain
operation diagnostics and host-preservation receipts. Changed owners require a
new sealed successor. Later production activation still requires the user's
separate gameplay launch instruction and fresh exact real-owner observations.

No namespace validation or fixture was run by this package author. Root applies
both utility cleanup passes to its actual readiness output, reauthenticates all
retained manifests and records reclaimed bytes/free space before closure.
Both package utility cleanup passes returned zero bytes. `cargo clean` removed
233 private build files (37.7MiB reported; 27,092,699 logical directory bytes
before cleanup). Available filesystem space afterward: 221,877,424,128 bytes.
Source, build logs and executable remain retained; the successor executable is disposable
after root validation. Preserve predecessor evidence, frozen engine/source,
drill fixture Git history and source/store/readers/private input dependencies.

