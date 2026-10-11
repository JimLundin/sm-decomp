# Actual projectile execution inventory
Status: resolved
Blocked by: None
Triage: ready-for-agent

Run fresh normal C any% trajectory with bounded projectile ID, initializer,
preinstruction and instruction counters. Disable expensive diagnostic hardware
and decompression captures; preserve gameplay semantics, terminal state, CPU
fallback/progress and transformed-source/binary/artifact hashes. Diagnostic only,
no Rust ownership or parity claim. Select next coverage from observed execution.

## Completion evidence

Checked-in `tools/port_projectile_inventory.py` builds actual pinned C with
bounded counters at exact allocator initializer, EprojRunOne, preinstruction and
instruction dispatcher definitions. Gameplay untouched; no hardware log or
expensive decompression snapshots. Utility authenticates original pinned archive,
retains original source copy, transformed source and binary. Independent current
utility320-input control completed; full129712-input diagnostic completed with
zero CPU fallback and419 decompressions.

Full diagnostic: `out/port/projectile-inventory/20261011T072333.457801Z/verdict.json`.
Current-tool control: `out/port/projectile-inventory-control/20261011T072528.320032Z/verdict.json`.
All107/104 sealed artifacts authenticated after cleanup. First diagnostic was
started before cwd correction; exact new root rom-view.bin/projectiles.tsv moved
into run after completion. Tool reconstruction explicitly nonauthoritative;
actual transformed C and binary authoritative. Literal escaped TSV separators
are preserved, with separately sealed decoded TSV. Current utility retains same
format; consumers decode explicitly.

Preinstruction execution:8170=764,940E=553,950D=11,9522=13,A328=528,A364=524.
Script execution:8154=36,8161=16,816A=44,81AB=827,95BA=2,95ED=2,9620=20.
A364 Ceres elevator platform is a missing pure gameplay leaf exercised524 times;
selected next ticket90. A328 pad includes SamusCode_0E callback and requires
separate scope. Diagnostic counters establish C exercise, no Rust ownership/parity.

Both finished runs cleaned with port_cleanup.py --apply and --deduplicate --apply;
all four commands reclaimed0 bytes. No worktree; remaining disk2.3GiB.
`python3 -m py_compile tools/port_projectile_inventory.py` passes.
