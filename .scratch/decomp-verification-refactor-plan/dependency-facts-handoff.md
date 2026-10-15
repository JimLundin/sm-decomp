# Ticket 03 dependency facts and identity handoff

Status: implementation active; not accepted or integrated.

- Owner: `/root/dependency_facts_resume` (replacing stopped `/root/dependency_facts`), under root session
  `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
- Worktree: `/home/jim/sm-decomp-performance-dependencies`.
- Branch: `performance/dependency-facts-20261015`.
- Baseline: `94104cc224b6997c7eb1ad946be451af168287b5`.
- Red-capable test checkpoint: `3a58a86`.
- Proof root: `/home/jim/sm-decomp-performance-dependencies-proof`.
- Exclusive Cargo target: `/home/jim/sm-decomp-performance-metrics-target`.

## Ownership and current work

The worker owns a shared, owned Rust source-facts Module in
`offline-verification`, both input-discovery consumers, the `pipeline` affected
consumer, necessary implementation/dependency identity integration and relevant
tests. Root owns the main checkout, shared ticket status, frozen assembly and
combined acceptance. The bulk-context worker owns captured input preparation in
its separate checkout; final consumer integration waits for its next coherent
checkpoint. No production policy, gameplay source or Native operation admission
is edited here.

The public regression exercises actual `batch plan` selection and public wrapper
generation. It puts both observed `include` doc comments, nested comments and
ordinary/raw/byte strings into valid Rust, asserting that an unrelated module is
absent from selection and wrapper closure. Existing tests that used comments as
mock references now contain real Rust references.

## Verification in progress

The first no-run build stopped on missing local dependency paths before compiling
or running tests. `logs/red-build.log` retains that exact error. Only these ignored,
read-only links were added in the worktree:

- `deps/rustysnes` -> `/home/jim/sm-decomp/deps/rustysnes`
- `deps/snesrev-sm` -> `/home/jim/sm-decomp/deps/snesrev-sm`

The resumed bounded build is recorded in `logs/red-build-ready.log`:

```sh
env CARGO_TARGET_DIR=/home/jim/sm-decomp-performance-metrics-target \
    CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
    TMPDIR=/home/jim/sm-decomp-performance-dependencies-proof/tmp \
    SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
    timeout --signal=TERM --kill-after=5s 900s \
    cargo test --locked --offline -p pipeline -p offline-verification \
    --test affected --test source_facts --no-run
```

No passing tests, corpus selection counts, timing gains, accepted identity
contracts or completed cleanup are claimed yet. No verification producers have
run. Both scoped cleanup passes remain required after all outcomes. The root
integrator owns worktree retirement and post-retirement authentication.

## Resumed source-facts checkpoint

Resume baseline `320feebee257e55cc05d99ed543da9702353f1df` includes the bulk
ordinary captured-input checkpoint and the separate read-only metadata guard.
The interrupted scanner tests had been inserted inside a raw Rust fixture;
their scope is repaired before any claim about their results.

The owned `source_facts` Module is registered and bound into
`IMPLEMENTATION_SOURCES`, using exact locked `syn 3.0.6` and
`proc-macro2 1.0.107`. Its facts retain exact logical path/source hash, code
identifiers, recognized inputs and explicit incompleteness. Fixed byte/token,
nesting/work limits and inherited deadlines distinguish resource failure from
ordinary unsupported syntax. No parser AST crosses a thread boundary.

`logs/scanner-build.log`: locked/offline source-facts no-run build passed in
26.28 s. `logs/scanner-first.log`: five parser controls passed in 0.01 s,
covering comment/literal exclusion, exact byte hashes, cfg union/aliases and
macro arguments, actual include/environment inputs, unmodeled expansions and
runtime aliases, malformed source, limits and expired deadlines. The public
wrapper regression remains excluded until its consumer is converted. No
producer execution or accepted result reuse is claimed by these parser tests.

Next: convert both wrapper/source traversals, affected selection and captured
Rust guard; run their public green controls and the pending inherited-metrics
control; complete identity integration and corpus observations. Both scoped
cleanup passes remain required. Root owns final acceptance and retirement.
