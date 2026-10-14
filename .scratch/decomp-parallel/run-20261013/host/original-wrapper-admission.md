# Original Ceres host admission contract

Status: in progress
Blocked by: complete all-affected actual-C batch

The native `assign record` integration adapter requires `scope-file` and cannot
retroactively invent a native gameplay scope for the sealed legacy Ceres Jobs.
`decomp-session infrastructure-record` legacy envelopes replay only the original
requests explicitly sealed in the reviewed bridge; that bridge contains starts
and submissions, but no new host-completion request. The unchanged Rust
`.scratch/ceres-continuation/assignment_driver.rs` intentionally delegates result
evidence admission to its caller and keeps the exact original assignment
contract. This coordinator therefore verifies the original declared gates and
seals evidence before recording the exact integrated Update through that Rust
wrapper with current generation2 credentials. No bridge, Job, claim or mutation
source is edited to manufacture native admission.

Required host facts: submitted claim `(shared-game-host,48656a2-host-v1,1,
ceres-20261013)` at `73547105d870f8b05f8c6bcf55b768c223889fdb`; already merged
ancestor of the frozen current HEAD; five exact submitted file hashes; 31
passing focused tests already retained from setup; both callback and all three
family host controls; ten host pairs with material PLM/HDMA/17VRAM order;
passing and currently authenticatable batch covering the entire current
Offline catalog from original baseline
`48656a284329d432291eca23a2ae0d1d7c0a3e48`. Shared pipeline/tool changes keep
all catalog families affected. Neither a two-family batch nor newer baseline
can discharge this set.

The coordinator records original submission and frozen merged source identities
separately, complete selected family names/verdict hashes, authentication,
both cleanup passes, source/store/reader retention and absent registered host
implementer worktree. After cleanup it authenticates again, admits the original
running host tuple and publishes only independently evidenced stages. The
controller's original submitted tuple follows; Ceres escape and later gameplay
milestones retain separate gates.
