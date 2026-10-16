# Waver retained-control review checkpoint

Reviewer01a145c8,2026-10-16. Preliminary controls complete; final producer27
report awaits eight-call production composition and cleanup. No live claim.

Evidence root `/home/jim/sm-decomp-restart/restart/out/waver/`.
Native source5931d6cbb07cfd2971f405f5d8e4a2ed88ac48fc;
control sourcec5e437646bb59e436135a051207d5a4b80ddcbf0.
Both candidate source/executable seals pass. Library366561ea recipe SHA256
193f1be1aed69c68ac9b130b5d94707d553d24fcbd01cbc15052cb4da72c9911 passes.
Control executable SHA256674607fafc852f99d86480fab34bd7d8045dad9cdc911f79d0e080e6b86f3d89.
Result SHA256b3a9a87deea6fa0ef9e7fba48b96bd8a7adfb33613588dc2f023f696d56f9270.

All28 retained C/Rust full-WRAM/effects/observations and scalar outputs compare
equal; recorded WRAM/effect hashes authenticate. Counts:24shared body pairs,
two inherited Main chains, two ProcessEnemyInstructions pairs. Independent cmp
confirms chain inputs equal init-right/instruction-current compared outputs.
Rust completion deltas: Init4/Main14/Instruction4/helper6. Service request deltas:
horizontal14/sine11/vertical11. Three Main cases skip sine/vertical after collision.

Inspected comparator checks full observer snapshots, owner/service deltas,
horizontal-only or ordered horizontal/sine/vertical, root endpoints and same-jp
instruction result. Spike allocate/full-pool assertions check D094 effects and
level-word mutation/preservation. Invoke wrapper uses actual CallEnemyAi for
Init/Main and CallEnemyInstr for Instruction; helper is diagnostic only.

Generic interpreter uses original ProcessEnemyInstructions, verifies ROM event
86E3 followed by Sleep812F, and asserts same script pointer/event field. Both
interpreter cases reset k=current; interpreter-instruction-distinct-k is a seeded
variant, not a distinct-k interpreter guarantee. Separate direct Instruction
pairs cover current/explicit-k source semantics.

Fourteen admissions comprise12 native/immutable-Rust invalid-state controls,
one short-ROM immutable error and one unchanged callback refusal followed by
real C fallback. Fallback compares WRAM/effects and no Rust completion.
All13 pre-C result receipts report unchanged/refused_before_c. No malformed
state is executed as arbitrary original C.

All19 distinct fatal receipts authenticate stdout/stderr and report verified
exit2/no timeout; total1599ms. Modes cover refusal, postrequest error/panic,
buffer/length/generation/selector, changed state/argument/order, outside-Main
service, invalid output/continuation, root/observer recursion, both Waver↔player
directions and service observer failure. Harness owns20second kill/reap.

Reuse material's planning/native-admission-audit.md: current waver.c/hooks/prepare
hashes match that audit and control source matches frozen c5e4376. No duplicate
admission research. Native terrain/sine/scheduler/interpreter/contact/death/draw
remain dependencies. Main Init/Main/helper expressions are selective reuse;
the event instruction was missing in searched main source.

Retained logs:15semantic tests (0.67s compile/0.46s run),20runtime tests
(4.57s compile/0.05s run;3filtered), workspaceClippy9.24s, controlClippy2.62s,
shared build1.69s, control build3.98s. These are individual receipt timings,
not complete delivery time. No old player/item/persistence matrix repeated.

Next: authenticate production-composition candidate, ordinary installed calls,
typed owner deltas and state/effect/output equality; verify cleanup, retired
workers and retained source copies. Only Markdown/read-only checks authored;
no build, native, test, game or reference producer. Reviewer disposable bytes0.
