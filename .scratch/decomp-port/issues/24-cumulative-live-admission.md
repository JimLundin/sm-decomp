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
