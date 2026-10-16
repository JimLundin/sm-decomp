# SRAM reuse lookup after restart

Observed 2026-10-16. Source baseline: main
`193cad941713b3539d23e08238a79e7fb09dde61`. Bounded read-only investigation;
restart integrator already owns claim24 and its implementation.

**Verdict: no reusable Rust SRAM codec implementation or executable codec
fixture declaration was found on this main commit. The save/persistence file
leads are proposals, not completed source. Continue claim24 using the already
published restart persistence contract; do not wait for a main codec copy.**

Evidence:

- `git ls-tree -r --name-only 193cad941713b3539d23e08238a79e7fb09dde61`
  contains no save/SRAM/persistence-named implementation file. A `git grep`
  over all tracked Rust source at that commit found none of `SaveToSram`,
  `LoadFromSram`, `PackMapToSave`, `UnpackMapFromSave` or their snake-case names.
- Scoped `git log --all` returned no commits for `src/save.rs`, `tests/save.rs`,
  `crates/port/src/save.rs`, `crates/port/tests/save.rs`, `tools/port/save.toml`,
  `crates/port/src/persistence.rs`, `crates/port/tests/persistence.rs`, or
  `tools/port/persistence.toml`. This covers locally reachable refs and these
  paths only; it is not a claim about unreachable objects or remote branches.
- [Later-lanes catalog](../decomp-parallel/catalog/later-lanes.md:96) explicitly
  calls the proposed source/test/manifest paths absent. Its line112 proposes
  the `save.rs` triplet. Last change on main:
  `6b582fcd430828d2c52ad06f889ff411d352fc87`.
- [Research19 Answer](../decomp-parallel/run-20261013/research-19/Answer.md:243)
  explicitly states every proposed module/test/manifest is unimplemented and
  its example gates unrun. Lines181–191 identify the missing SRAM fixture
  adapter and merely proposed adapter files. Last change on main:
  `c2cae9ac145f9343cd345e6f978bff0774510b56`.
- [Issue42](../decomp-parallel/issues/42-complete-sram-codec.md:4) remains open;
  line22 says source and actual-C fixture submission are pending. Its proposal
  uses the newer `persistence.rs` triplet, not `save.rs`. Last change on main:
  `c2cae9ac145f9343cd345e6f978bff0774510b56`.

Useful existing inputs are research and control requirements, not executable
fixtures or translated source:

- [Codec proposal controls](../decomp-parallel/run-20261013/research-19/proposed-children/03-complete-codec.md:16)
  provide a compact checklist for independent controls; lines26–29 explicitly
  label test/gate targets unrun and unimplemented.
- [State observations](../decomp-parallel/run-20261013/research-19/state-contract.toml:1)
  are machine-readable static observations with `runtime_evidence = false`.
- [Routine observations](../decomp-parallel/run-20261013/research-19/routine-observations.toml:1)
  retain source locations, hashes and dependency names; line4 explicitly
  disclaims gameplay acceptance.
- [Retained inputs](../decomp-parallel/run-20261013/research-19/inputs.paths:16)
  locate the original C and shared module dependencies. Their semantics and
  ROM tables were deliberately not re-investigated: the restart already has
  `/home/jim/sm-decomp-restart/.scratch/restart-pipeline/persistence-contract.md`.

The cited catalog/research/issue files have no working-tree differences from
the pinned main commit. Reuse their control checklist only where it adds to
the existing restart contract; do not treat old planned commands or static
observations as test success, executable fixtures or current stage evidence.

## Completion evidence

Only repository reads, scoped Git queries and one Markdown write occurred.
No builds, tests, captures, network requests, run directories, temporary
binaries or worktrees were produced. Cleanup is complete with **0 bytes
reclaimed**; no existing evidence dependencies were removed. Available space
at inspection: **99,608,006,656 bytes**. This note is the sole authored file.
