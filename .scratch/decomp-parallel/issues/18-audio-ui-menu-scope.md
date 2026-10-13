# Research complete audio acknowledgment and UI/menu boundaries

Status: resolved
Blocked by: 05
Triage: ready-for-agent
Type: research

Consumers:07,08
Read-only work; future implementation waits required13APU/reset and player inputs

## Work

Read [later lanes](../catalog/later-lanes.md). Reuse resolved02system/03audio
bounded WRAM/queue routines. Research HandleMusicQueue and whole sound-effect
acknowledgment state machine first, then separate pause/file/options menu roots.
Pinned C modifies acknowledgment behavior; declare actual source choice and
original-ROM effect instead of treating faithful C parity as native semantics.
No complete Rust UI/menu or audio-frame module/receipt was found in preparation.

Exact candidate files: `crates/port/src/audio.rs`, `crates/port/src/system.rs`,
`crates/port/tests/audio.rs`, `crates/port/tests/system.rs`; proposed separate
`crates/port/src/audio_frame.rs`, `crates/port/tests/audio_frame.rs`,
`tools/port/audio-frame.toml`. UI exact new files are chosen only after root/
closure research, not a blanket module claim. Integrator owns state/SRAM/lib.rs,
APU/hardware callbacks and shared catalog/ABI registration.

## Acceptance / Answer

Record each coherent boundary's source/observed-root/dependency/input hashes,
current bounded stages and missing host interface. Current bank86-only native
exercise is a blocker; generate a reviewed Rust provenance extension task and
exact separate1–32 routine implementation children when facts exist. Require
real ordered audio port reads/writes/queue/ack states and controls, UI inventory/
menu resource state, actual-C parity plus original-ROM boundary comparisons,
independent material live ownership and visible fallback. Concrete60s initial/
3600s merged argv and required13dependencies; add resulting tasks to07/08.

## Completion evidence

The researched result is admitted in
[finish receipt](../run-20261013/receipts/finish-research-18.stdout.toml).
The evidence crate retains its25 files/345,203 bytes as run
`2156e965ec6cd98bd54b381c5d490784d34af141ee1a8ddb4cebc729583f3062`;
inventory, full verification and current-input comparison are in the same
receipts directory. Its no-run cleanup records0 bytes and no active writers.

Coordinator published implementation/verification tasks33–39, deferred remaining
scope40, and shared21 supplement. Eight renderer routines are already committed
and actual-C verified under [coverage task02](../../decomp-coverage/issues/02-pause-equipment-render.md),
with607 pairs and10 Rust tests. Research findings themselves earn no gameplay
stage. The original package/seals remain unchanged; later tasks replace its
proposed research labels for adapters and tests.

<!-- decomp-assignment:ticket-2966e528f4696d00eed786e3a895d9034fd217c5ad6db59153329ef23da35b6d -->

<!-- decomp-assignment-record:start -->

## Current assignment

Stable work: `ticket-2966e528f4696d00eed786e3a895d9034fd217c5ad6db59153329ef23da35b6d`
Input revision: `2b5be5ee42e86f922e679effe3968f60fc7d94afffa98e078f88a1c6107fa81d`
Lifecycle: `complete`

Honor existing local ticket .scratch/decomp-parallel/issues/18-audio-ui-menu-scope.md. It is open, unclaimed and dependency-ready. Read its requirements and establish a complete Translation scope before implementation: observed exercise provenance, required dependencies, exact writable paths and concrete verification argv. Include a bounded pre-integration test and an integration gate. Use native translation-scope-v1 TOML; limit each initial experiment to approximately 60 seconds. Research facts do not grant acceptance. Preserve task-origin ownership until its implementation is integrated, verified and cleaned.

Writable ownership:

No maintained source writes.

Claim owner: `decomp-run-20261013`; attempt: `1`.

## Completion evidence for this assignment

```toml
result = "researched"
followups = []

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-18/Answer.md"
sha256 = "45bd7e4152b6167a585ecd3805b1597c8ab6704a678aa8980f374668d463b366"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-18/children.toml"
sha256 = "18017dca2f6ea977fa25f298b0967c9179bdaf117cbc2454db420861a3c124f6"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-18/scope-holding.toml"
sha256 = "e7683c0fbfce9be67b2297e5028158debbe84c04c9888d51c50684dc4588353c"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-18/manual-handoff.toml"
sha256 = "d7e65579992f52c851aea212624df198e53867695a9babc8b4c54b484577f2b3"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/run-20261013/research-18/cleanup.toml"
sha256 = "8d2b99f0cd2314efcef478de9ae4d76e89b580b7211098522af8409cf4a9c4ec"

[[evidence]]
path = "/home/jim/sm-decomp/.scratch/decomp-parallel/receipts/ticket-2966e528f4696d00eed786e3a895d9034fd217c5ad6db59153329ef23da35b6d-1-request-b09ba461a44b7d1b7ab9609ef143a90a099c145801206d87fbed68d75d99f11f.toml"
sha256 = "6c47797fa39341b16d25d90970b543edb93ac4a1df75caf6010a0feb6c71ae93"

```

<!-- decomp-assignment-record:end -->


