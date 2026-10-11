# Norfair lavaquake rock arc
Status: resolved
Blocked by: 53
Triage: ready-for-agent

Port BBDB allocation and BC0F two-state scheduling using actual A0 quadratic
speed data, full 16.16 position, scratch F writes, horizontal movement and
inclusive camera culling. Verify actual C and clean all runs. Live ownership
remains open.

## Completion evidence

Implemented BBDB initialization and BC0F rising/falling state dispatch. Native
Rust reads A0:CBC7 paired 16.16 speeds, applies two half-steps with final F
scratch value, preserves deferred horizontal motion at the apex, and culls
outside inclusive 256-pixel camera bounds. Unsupported state/table extents
return errors rather than performing unchecked memory access.

437 actual pinned C full-WRAM/raw-return cases pass: 182 arc preinstructions,
182 complete scheduler frames, 72 allocations and one sleep control. Evidence:
`out/port/lavaquake-rocks/20261011T041302.936755Z/verdict.json`.
`tools/port_lavaquake_fixtures.py` is the reproducible fixture entry point.
Independent Rust examples check the apex/inclusive boundary and both fractional
ROM half-steps. Full port tests, fmt and strict Clippy pass.
Offline verification only; live exercise and broad gameplay ownership stay open.

Cleanup: `port_cleanup.py --root out/port/lavaquake-rocks --apply` reclaimed
zero bytes; `--deduplicate --apply` reclaimed 68,943,872 snapshot bytes.
Both rejected manifest-generation attempts are retained and cleaned. Accepted
verdict authenticated after cleanup. No implementer worktree created; immutable
ROM/source snapshots retained. 35 GiB disk space remains.
