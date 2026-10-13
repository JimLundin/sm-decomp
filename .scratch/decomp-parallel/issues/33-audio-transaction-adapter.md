# Implement the ordered audio transaction verification adapter

Type: task
Status: open
Blocked by: 11, 21, 22
Triage: ready-for-agent

Use the finished [audio contract](../run-20261013/research-18/proposed-children/02-audio-transaction-and-original-ack.md)
to implement Rust adapter/model/host/ABI and meaningful integration controls.
Preserve complete canonical WRAM, APU/SPC/DSP/backend/pending queue state and
ordered reads/writes/uploads. Keep pinned-C acknowledgment behavior and
original-ROM timing as separate verification domains. Shared registration and
source ownership require the original integrator's checked scope.

Initial nonzero adapter tests are bounded to 60 seconds; merged family/live
controls to 3600 seconds. Current proposed targets are unimplemented. Record
exact files/inputs/argv before claiming, complete both cleanup passes and retain
source/store/reader dependencies. Deferred during the current coverage batch;
concrete offline audio work may use its own supported gate before live admission.

## Completion evidence

Pending adapter code and passing tests; the sealed source findings are input
facts and earn no gameplay coverage.
