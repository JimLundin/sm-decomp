# Remaining tools and no-Python completion guard

Status: resolved
Blocked by: 02, 04, 05, 06

Port emulator setup/replay, synchronization utilities and storage studies. Remove
all remaining project-owned Python sources and tests; switch CI, recipes, active
docs and add a Rust check preventing Python source/runtime calls from returning.
Preserve immutable source snapshots referenced by historical evidence.

## Completion evidence

Maintained Python/runtime calls and internal JSON are removed; final guards,
workspace regression, utility checks and cleanup pass. See
[completion.md](../evidence/completion.md). Inventory/pending notes below are
historical; full gameplay readiness is outside this migration task.

## Historical inventory

62 maintained Python files remain before the HDMA/storage/diagnosis removals.
The project-wide spec remains open.

Three native migration modules now integrated: HDMA diagnostics, storage
studies and iteration diagnosis. HDMA has four native controls plus a real C
soft-reset control. Storage has five readback/corruption/containment controls
and a real retained-room trace format study. All native study/history/HDMA
metadata uses TOML/binary. User clarified that backwards compatibility is not
required; no new JSON import adapters are justified for historical formats.
Real probes, deletion of transferred Python sources, CI and cleanup pending.

## Comments

2026-10-11 resumed by user instruction. Finish the native migration in this
worktree, including the remaining timed-pattern, iteration demonstration,
original-prefix and source-inventory workflows. Then switch recipes/CI/docs,
remove transferred maintained scripts and run final-source controls and cleanup.
The main gameplay checkout and subsequent pipeline-upgrade plan stay separate.

2026-10-11 utilities section migrated movie corpus pins/reports, emulator replay
and checkpoint sidecars, paired/focused diagnostics and diff reports to typed
TOML. Paired state remains the explicit v2 binary envelope. Fuzz timed vectors,
prepared cases and ledgers now use bounded length-framed binary formats (`SMTC`
and `SMTL` version 1); only externally supplied SingleStepTests JSON is parsed.
Evidence completed runs now seal typed canonical TOML provenance under an
explicit v2 status tag, with typed TOML pointer/CLI contracts. Historical sealed
v1 readers and their snapshots remain unchanged. Packed capture/publication,
observation authentication, routine-live reset epochs and their tests use the
new contracts; C epoch producer output is a fixed text protocol.

Verification logs: `/tmp/sm-native-data-tests.log` (evidence 11, movie 6 and model
3), `/tmp/sm-native-binary-tests.log` (2 bounded-codec controls),
`/tmp/sm-native-snes-tests.log` (10 execution/cache controls),
`/tmp/sm-native-diff-tests.log` (6 CLI scope/error controls),
`/tmp/sm-native-clippy-verified.log` (all utility crates strict Clippy).
Pipeline integration/final cleanup and real final-source workflows remain with
the orchestrator; this ticket remains claimed.
