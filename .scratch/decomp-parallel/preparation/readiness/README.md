# Read-only production readiness validation

Status: helper compiled and current owners freshly inspected; final actual drill seal required before execution.
Owner: preparation recovery root `01a13460-4aa9-7fe0-865e-a8f54ffca310`.
Observed: 2026-10-13.

The maintained `decomp-authority validate-activation` shares the complete activation
validator and does not write authority state. It requires an already prepared
policy at the fixed path derived from the requested checkout's common Git root.
That root is `/home/jim/sm-decomp` even when `checkout` names the frozen integration
worktree. The command also takes the shared assignment mutation lock.

This helper creates the prepared fixture only in a private Linux mount namespace.
`bwrap` mounts the host filesystem read-only, overlays
`/home/jim/sm-decomp/.scratch/decomp-parallel` with private tmpfs, and mounts every
existing direct child read-only at its original path. Only the new receipt directory
and a private assignment-lock file are writable host binds. The outer helper holds
the real existing assignment lock without writing its bytes; the inner CLI sees a
distinct lock inode, so there is no recursive lock deadlock. Namespace identity and
the tmpfs mount are checked before `prepare` can run. The host canonical authority
and ledger remain absent at every instant; namespace teardown discards the fixture
policy even when evidence validation fails.

The helper runs the maintained `prepare` operation first. This checks every existing
ledger, capacity, persistent integration token, exact held claims and writable paths,
authenticated retained bridge/source/evidence, and generation-zero stopped-owner
transfer. It retains the generated policy in the receipt directory, runs
`validate-activation`, compares unchanged policy bytes, and checks `status` remains
`prepared`. No `activate`, claim mutation, gameplay gate or result admission runs.

## Frozen subject and validator version

- Executed drill engine: commit `c8799f326ffe198b652301f08866f1b7a7bd9ebc`,
  fingerprint `4d87e457e895a03f82f5a7ccdd891f7769b83d25d48caf2fd3a55432e5929338`,
  source `/tmp/sm-decomp-preparation-integration-20261013`, binary
  `.scratch/decomp-parallel/drill/bin/sm-pipeline`, SHA-256
  `89e3c46596ea556011cfcbb679cace5b96277bbf56abb8a7a2a61db7048268d9`.
- Validator authoring version: commit `f54fbed0f30f2c84e2c6aba171dcc54e96099c9a`,
  fingerprint `cde89f07dcf1a897abc81938581d893a3b037257bf23ec7939b5f9c11de0d352`,
  source `/tmp/sm-decomp-ticket-validator-20261013`, retained binary
  `preparation/ticket-validator-evidence/bin/sm-pipeline-validator`, SHA-256
  `becfd038af486b51e94f454eca84cfd05fcaf96ae77875a1185275ddfe9978d3`.
  Its `validator-build.toml` records both versions. The validator-only correction
  admits the actual duplicate-ticket rejection shape using sealed reservation,
  competing request, ticket and retained claim evidence. It changes evidence
  inspection, not the executed queue/session engine.

These are local source/build/hash provenance. The sealed platform/process
observations are a tool/human inspection trust boundary, not cryptographic platform
attestation. The helper checks exact validator build-field/binary agreement; the
maintained drill validator recomputes the frozen executed-engine source fingerprint.

## Before execution

Fresh observations at 10:17 UTC are sealed in `current-owners/`: historical integrator
failed, compact implementer idle/completed, host implementer notLoaded with completed
terminal turn, host reviewer idle/completed, reporting idle/completed and replacement
root active/inProgress. The read-only process scan found no Ceres claim worktree,
run, ledger or native-prefix process in cwd/argv/open descriptors. It retains exact
identity for the running report server PID420446, start ticks2249195. The new typed
generation-zero transfer/registration/prepare request references these inspections;
the 09:22 cutover package remains immutable. If owners change before execution,
retain these sealed observations and create another successor. The bridge remains
exact, with all claims and acceptance gates held.

The initial historical integrator is `parent_thread` when present on a bridge
holding, otherwise `owner_thread`. Thus both submitted Ceres implementers name
their actual worker thread as `owner_thread`, while their historical integrator is
`01a133eb-76e2-7073-9aef-d97c03a812e4`. The running integration holding has that same
thread as its owner. The generation-zero transfer accounts for all three claims
without completing or releasing any. Child proofs must keep actual worker
`owner_thread`; do not replace those with the parent identity.

Observed retained seals (unchanged):

- Legacy Ceres ledger: `77fdba066af4f4c045e8e8f419997563d569ac0bd8045cef0a68ac916a126b89`.
- Original bridge: `b4046304db39ed5d70501d0b7b29a24abe75b17c03fd0c20ead702857161b4c5`.
- Normalized successor bridge: `cb537f195b4f83529f2ea7a61bd65c6e77ad215366275e8d60a7a604c4eb0ba9`.
- Maintenance approval v2: `bcc953cb2d2a1459b62ef477cfaf49703652f3fdbf704a3828066dc2ed4b8264`.

Only `.scratch/ceres-continuation/ledger.toml` was present at this audit. The request
preserves its token and assignment capacity `3`, admits that effort plus the two
historical maintenance efforts, and keeps canonical `decomp-parallel` inactive.
`SM_PIPELINE_WORKERS=4` is a separate execution budget. The maintained preparation
check re-inspects all ledgers and rejects any newly unadmitted one.

The reporting source/evidence handoff is durable in
`.scratch/decomp-progress/state-audit/coordination.md`; final commit `9ed933d` records
the completed reporting save and releases main Git/index to root, after evidence
commit `b4eea5b`. The immutable old bridge/ownership observation still records its
historical reporting owner and must not be rewritten to reflect that later handoff.

## Exact execution

The actual drill owner's final outer manifest is intended at
`/home/jim/sm-decomp/.scratch/decomp-parallel/drill/concurrency-drill.toml`, linked to
its typed `execution.toml`. It must be complete and sealed before this run. Root
must preserve the isolated Git fixture and frozen subject checkout until the
validator consumes them. The helper does not launch, stop or edit drill sessions.

Compile the small dependency-free Rust helper (already syntax-checked):

```sh
rustc --edition 2024 /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness/main.rs -o /tmp/sm-production-readiness-helper-20261013
```

Run from the actual root session with its inherited `CODEX_THREAD_ID`. Do not
override that identity. The request below uses the sealed current-owner successor:

```sh
/tmp/sm-production-readiness-helper-20261013 \
  --validator /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/ticket-validator-evidence/bin/sm-pipeline-validator \
  --validator-source-checkout /tmp/sm-decomp-ticket-validator-20261013 \
  --validator-source-commit f54fbed0f30f2c84e2c6aba171dcc54e96099c9a \
  --validator-source-fingerprint cde89f07dcf1a897abc81938581d893a3b037257bf23ec7939b5f9c11de0d352 \
  --validator-build /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/ticket-validator-evidence/validator-build.toml \
  --prepare-request /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness/current-owners/prepare-request.toml \
  --drill /home/jim/sm-decomp/.scratch/decomp-parallel/drill/concurrency-drill.toml \
  --output /home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness/production-validation-1
```

Each output must be new. Rejected attempts retain diagnostic stdout/stderr, request,
version map and host preservation checks. A successful run seals all receipts in
`artifacts.sha256`, including unchanged before/after host ledger hashes. The inner
maintained request is generated with `operation = "validate-activation"`, exact
frozen `checkout`, Ceres token, root generation1 and the supplied drill's actual
SHA-256. Its output status is `prepared`; an absent drill field in that status
receipt is expected because validation does not attach the drill to durable policy.

After root's actual run, perform both measured cleanup utility passes on its small
output, retain all sealed receipts and reauthenticate `artifacts.sha256`. Remove
only the disposable helper binary when finished. This audit created no build tree
or verification fixture run; namespace smoke checks left no host marker or policy.

## Later launch prerequisite

Passing this package establishes read-only readiness for the exact executed-engine
subject and separately reviewed validator version. Production authority and queue
remain pending. The user's later gameplay launch instruction is still required,
then a real prepared policy and explicit `activate` command with current real-owner
observations under the same retained Ceres token. Every gameplay scope still needs
its own catalog, dependency, provenance and acceptance gates.

`checked_drill` binds `request.checkout` HEAD/fingerprint, not current main HEAD.
Retain the frozen subject worktree and branch for future activation while final
documentation/report/validator commits advance main. If relocating that checkout,
the outer manifest, frozen input and build source-path fields must be consistently
resealed, and source/receipt authentication repeated. Any new queue/session/admission
behavior requires a fresh actual drill on that new subject. A final integration
containing only the reviewed validator delta, tests and documentation/evidence
requires a recorded source-equivalence review; do not report that the old engine's
actual sessions exercised the new validator or other changed engine behavior.
