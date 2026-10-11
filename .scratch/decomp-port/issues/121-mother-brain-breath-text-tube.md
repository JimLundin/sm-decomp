# Mother Brain breath, time-bomb text and falling tubes
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate CA6A/CA83 purple breath, CAF6/CAFA time-bomb text and complete
CBC9/CBE7/CBEA/CC08 falling tube state machine, including both dust spawns.
Require actual-C full-WRAM/host evidence, port checks and finished-run cleanup.

## Completion evidence

Translated both breath initializers, text initializer/tracker, tube initializer
and complete two-state callback with initial dust plus signed-floor dust spawn.
All actual-C cases pass: allocation 64, tube motion 433, text callback 54.
out/port/mother-brain-breath-text-tube-allocation/20261011T103045.760426Z/verdict.json
out/port/mother-brain-tube-motion/20261011T103046.893180Z/verdict.json
out/port/time-bomb-text/20261011T103048.164943Z/verdict.json
Full port tests and strict Clippy pass. Ordinary cleanup zero bytes on each;
deduplication respectively 17,825,792, 89,391,104 and 13,762,560 bytes. All
three evidence inventories authenticate after cleanup. Retain source snapshots,
ROM, binaries and full-WRAM/host cases. Free disk 266,701,807,616 bytes.
Offline equivalence only; live ownership separate. Whole-decomp remains active.
