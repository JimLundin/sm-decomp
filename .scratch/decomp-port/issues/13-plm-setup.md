# Native PLM setup and collision block families
Status: resolved
Blocked by: 11
Triage: ready-for-agent

Translate native SpawnPLM slot allocation/header reads and a substantial setup
dispatch family: quicksand/sandfalls, respawning shot/bomb/speed/crumble blocks,
extensions, scroll/touch, grapple and selected room block setup. Context:
/tmp/sm-port-plm-closure.md. Preserve full WRAM, PLM pointer/slot invariants,
mutable collision displacement and exact setup return flags. Unsupported setup
must be classified before mutation if the host chooses measured C execution;
never retry C after partial failed Rust execution. Own plm.rs/tests/plm.json.

## Completion evidence

Native `SpawnPLM` allocation and 44 named bank84 setup functions are implemented
in `crates/port/src/plm.rs`, with transactional WRAM/displacement commit,
immutable supported-header classification and explicit unsupported errors.
The real pinned C allocator and setup dispatcher matched native Rust for
673 header/state cases, complete WRAM, flags and synchronous displacement:
`/tmp/sm-plm-spawn-admission/20261010T202107.758829Z/verdict.json`.
All five compiled invalid ABI inputs reject before references are formed.
The prior direct setup evidence contains 44 routines/352 matching full-WRAM
cases: `/tmp/sm-plm-offline/20261010T201617.310520Z/verdict.json`.
These predecessor records remain evidence of their retained source versions;
current source seals are regenerated for root integration after registration.
Four independent Rust tests pass, including rollback on supported setup failure. The real-C wrong-displacement and retained
WRAM tamper controls pass; admission additionally rejects missing case artifacts
and changed pinned source. `docs/DECOMP_PLM.md` records domains and remaining
C setup ownership. Live exercise and original-ROM parity remain separate work.

Final formatted harness/manifests native seal:
`/tmp/sm-plm-spawn-sealed/20261010T202327.007047Z/verdict.json`, all 673 native
cases plus five ABI rejection controls, authenticated by `verify_plm_evidence`.
The complete actual-C negative-control test passed in 57.84 seconds, including
wrong displacement, retained WRAM tamper, omitted case artifact and changed pin.
Implementation commit: `18baa6a` (follow-up adds rollback test and final pointer).
