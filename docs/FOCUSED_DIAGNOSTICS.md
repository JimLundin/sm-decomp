# Focused instruction diagnostics

`just diagnose <start-input> <instructions>` cold-replays the candidate and
reference to corresponding instruction boundaries, then compares a bounded
interval. `tools/readiness_diagnostics.py --start-instruction N` additionally
selects an instruction offset within the cold frame. The prefix must pass frame
parity; an earlier failure requests an earlier focus instead of attributing it
to the selected compiled entry. The command prints its immutable run and brief.

The subject's frame and instruction APIs share one dispatcher. Passive snapshots
decode all CPU registers, emulation/wait/stop/old-I state, architectural clocks,
interrupt counters and scanline/frame position. Pending hardware includes full
bus serialization plus ordinary-save omissions: DMA scratch and per-dot PPU
pixels, carry, cursors and CGRAM/overflow pending state. The private interpreter
per-step cycle accumulator is normalized for equality while all pending state
and total clocks remain exact. Snapshot blobs are diagnostic state, distinct
from restorable checkpoints.

The maintained core patch records existing CPU and internal DMA operations
while explicitly armed. It neither reads additional hardware nor ticks or polls
interrupts. Real opcode fetches, interrupt discarded fetches, operand reads,
writes, implied cycles, clock spans and DMA A/B transfers retain ordered values
and times. Snapshot equality catches effects before subsequent instructions
cancel them. Empty tables cannot claim generated execution.

On divergence, the report retains the first paired instruction, preceding
matching boundary, input index/pad/digest, PC/opcode/MX/generated key, decoded
before/after CPU and clock position, first bus/state difference, and up to 64
preceding paired instruction records. Paired before/after hardware/CPU blobs and
exact argument-array rerun commands remain beside the report. Symbols may help
interpret RAM offsets but are optional; absent symbols do not block diagnosis.

`Reference` and `Subject` expose `set_input`, `step_instruction`,
`step_instruction_observed` and `instruction_snapshot`. The focused comparator
consumes these public seams. Current replay is cold; checkpoint speedups require
separate restoration validation. Diagnostic results are scoped and never
advance readiness acceptance by themselves.

Candidate preparation is separate from validation: focused commands build exact
run-local artifacts without a one-frame parity claim. `--failure <JSON>` with a
selected `--predecessor <run>` invokes failure controls. The automatic Python
API `diagnose_failure(run, failure, predecessor, intervals=None)` discovers
actual changed-key execution boundaries, starts up to 64 preceding instructions,
and uses validated separate candidate, disabled-key and predecessor caches over
identical inputs and intervals. Disabled controls filter runtime table entries;
they preserve reviewed source/config. A predecessor is restored and rebuilt in
an isolated checkout from its verified source snapshot.

Passing controls support attribution to the increment; a failing or incomplete
control routes investigation to shared paths/input/cache/harness. Original-core
synchronization failures require independent observations/probes. Unexecuted
required keys yield uncovered with covered-interval or removal/deferral choices.
Experiment records retain hypotheses, predicted discriminators, results and
new evidence. Three successive experiments without new evidence produce a
concrete decision artifact while retaining both candidate and predecessor.
