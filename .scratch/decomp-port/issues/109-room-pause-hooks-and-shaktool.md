# Room pause hooks and Shaktool setup
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate Draygon pause hook installation, pause/unpause IRQ callbacks and
Shaktool hardcoded PLM setup. Preserve 24-bit pointer writes and native coroutine
return. Verify actual-C full WRAM/return, tests, cleanup and authentication.

## Completion evidence

Four native callbacks translated. Oracle exposed extra primary-variable clear
in existing positioned PLM allocator: C preserves 1E17 while clearing DF0C.
Regression hardcoded_shaktool_allocation_preserves_native_primary_variable
failed with 0 versus A5A5 before removing that extra clear, then passed.
Native hardcoded and room allocation both preserve this field; shared allocator
fixed for both. No debug instrumentation added. Diagnosing-bugs skill applied.

All 48 current-source actual-C full-WRAM/return cases pass:
out/port/room-pause-hooks-shaktool/20261011T095718.836291Z/verdict.json.
Includes free/full PLM tables, exact 24-bit pointers, full-word HDMA comparisons
and zero coroutine return. Full port suite and strict Clippy pass.
Initial mismatch diagnostic 20261011T095457.887988Z retained and cleaned in both
modes, zero bytes reclaimed. Successful cleanup ordinary zero; dedup reclaimed
12713984 bytes; all cases authenticate afterward. Free disk 269622669312 bytes.
Retained sealed binary/source/pinned ROM dependencies; no implementer worktree
created. Offline scope only. Prior allocator-dependent seals are historical;
live ownership must be refreshed separately. Branch decomp-port-integration.
