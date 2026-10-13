# Compact causal controller diagnostic

Status: claimed
Blocked by: None
Triage: ready-for-agent
Type: task

Implement the compact diagnostic in the five exact files listed by the durable
assignment. Preserve the default full observer/report behavior. Initial actual
experiment is capped at 60 seconds as a whole, compares causal candidate and
changed-RNG control, and retains failures. Extend through frame 2240 where
feasible. Report exact matching frontier, material game state and next mismatch;
do not claim original-ROM timing, full WRAM or Ceres completion.

Required focused gates: driver unit tests, public prefix command tests and the
actual compact experiment. Cleanup and post-cleanup authentication are required.

## Comments

This advances ticket 78's existing causal seam; ticket 78 remains claimed until
its full requested controller/scheduler contract is established.

<!-- decomp-assignment:compact-controller -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `compact-controller`
Input revision: `48656a2-compact-v1`
Lifecycle: `submitted`

Implement explicit compact profile preserving full contract. Execute Rust focused tests and bounded60s frame2240 diagnostic, retain evidence and failures. Shared root /home/jim/sm-decomp/out/pipeline workers4. No live acceptance. Root integrates and cleans.

Writable ownership:

- `crates/pipeline/src/decomp_prefix.rs`
- `crates/pipeline/src/prefix_observer.lua.in`
- `crates/pipeline/src/prefix_compact_observer.lua.in`
- `crates/pipeline/tests/prefix.rs`
- `docs/DECOMP_PREFIX.md`

Claim owner: `ceres-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "submitted"
commit = "223f8a1b0c7382817b1290d13c7a41f4e722d5e1"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/ceres-continuation/controller-submission-evidence.toml"
sha256 = "8da107cdff9c14182a0ffae1d24ce1d3acbadfccd742fccf4e31c0896bd3634e"

```

<!-- decomp-assignment-record:end -->

## Checkpoint handoff, 2026-10-13

The worker and reviewer finished and are idle. Exact submitted source is
`223f8a1b0c7382817b1290d13c7a41f4e722d5e1`; the original claim is submitted
in `../ledger.toml`. Eight unit tests and two public tests pass. The 2,240-frame
diagnostic matches 1,736 updates and rejects the independent RNG control.
The longer diagnosis retains an extra poll near original frame 8,508.
Both completed-run cleanup passes reclaimed zero bytes; the successful short
verdict authenticated afterward. Source, verdict hashes and unresolved
integration are recorded in `../controller-handoff-20261013.md`. Ceres escape
and original-ROM parity remain open; this ticket stays claimed for integration.
