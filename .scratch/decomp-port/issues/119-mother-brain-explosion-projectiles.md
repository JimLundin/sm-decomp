# Mother Brain explosion projectiles
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate C8F5/C92F initializers and C914/C94C tracking. Preserve native
three-entry death script table, host spawn offsets, wrapping coordinates and
unchanged host state. Require actual-C full-WRAM/host evidence and cleanup.

## Work evidence

C8F5/C92F initializers and C914/C94C callbacks translated. All 324 tracking
cases pass and authenticate after zero ordinary cleanup and 84,934,656 bytes
deduplication:
out/port/mother-brain-explosion-tracking/20261011T102400.372464Z/verdict.json.
Port suite and strict Clippy pass. Initial allocation manifest had wrong host
field name; rejected run preserved and cleaned (zero bytes). Corrected rerun
pending. Available disk 267,728,281,600 bytes; retain binaries, ROM, snapshots
and full-WRAM/host inventories required by evidence seals.

## Completion evidence

All 55 allocation/native instruction full-WRAM/return/host cases pass:
out/port/mother-brain-explosion-allocation/20261011T102607.501292Z/verdict.json.
Cleanup ordinary zero; deduplication 14,286,848 bytes. Evidence authenticates
after cleanup. Combined with 324 tracking cases, full port tests and strict
Clippy, translation verified offline; live ownership remains separate.
Available disk before final motion run cleanup: 267 GB.
