# Admit cumulative gameplay families in live replay
Status: claimed
Blocked by: 16, 17, 18, 19, 20, 21
Triage: ready-for-agent

Run authenticated cumulative gameplay offline verification and independent
C/mixed live replay using tools/port_batch.py. Diagnose actual failures and keep
rejected evidence. Resolve ticket12 only with exercised collision/movement and
matching state, returns and effects. Record per-family hits and remaining C/CPU
ownership; zero-call families remain offline. Investigate PLM execution before
requiring its admission. No state repair or hidden C retry is acceptable.

Extend replay prefix where practical to expose substantive gameplay dependencies.


## Storage cleanup evidence

Reusable `tools/port_cleanup.py` previews by default and supports explicit
application. New live runs delete unsealed Cargo/archive intermediates; completed
batches deduplicate sealed immutable snapshots and replay effect logs by default.
Historical source seals remain unchanged. CLI tests cover preview/apply behavior,
sealed builds, unfinished runs, symlink/path escape, byte/hash preservation and
restart after interrupted temporary links.

Scoped accepted flight-fixed-live cleanup reclaimed1,317,372,945 bytes by
deduplicating identical authenticated observations (run20261010T204643.443282Z);
root additionally reclaimed355,637,476 bytes of unsealed historical build
intermediates. A broad snapshot-only cleanup was interrupted after preserving
already completed atomic deduplications; its unmeasured savings are not added
to these totals. No accepted verdict, pointer, hash or observation was rewritten.
