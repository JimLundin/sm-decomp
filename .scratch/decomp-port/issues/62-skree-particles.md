# Skree particles
Status: resolved
Blocked by: 61
Triage: ready-for-agent

Port four Skree particle initializers and 8B5D movement/gravity with exclusive
camera edges. Verify actual pinned C full WRAM and complete cleanup.

## Completion evidence

Implemented 8ACD/8AF1/8B15/8B39 enemy-relative initializers and 8B5D particle
movement/gravity. Shared exclusive screen predicate with existing E4FE dust
scheduling; inclusive lavaquake/horizontal projectile bounds stay separate.
Independent Rust test checks gravity update before deletion at X=256.

705 actual pinned C full-WRAM/raw-return cases pass: 192 allocations,
256 preinstructions, 256 scheduler frames and one sleep control, including
E4FE shared visibility regression. Evidence:
`out/port/skree-particles/20261011T042739.052165Z/verdict.json`.
Rebuild with `tools/port_skree_fixtures.py`. Full port tests, fmt and strict
Clippy pass. Offline coverage; live ownership remains open.

Cleanup applied/deduplicated `out/port/skree-particles`: zero ordinary bytes;
115,474,432 snapshot bytes reclaimed. Accepted verdict authenticated after
cleanup; immutable source/ROM snapshots retained. No implementer worktree
created. 32 GiB available disk space.
