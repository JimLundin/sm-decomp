# Enabled timed-pattern validation

`just pattern-check` allocates an immutable scoped evidence run and executes all
available native SingleStepTests vectors for LDA immediate, LDX immediate,
REP/SEP, STA absolute, NOP and DEX, plus independent boundary fixtures. It then
runs independent generated-path SNES timing regressions. A scoped pass does
not certify readiness; the readiness command must combine all required gates.

Each batch compiles exactly the `recomp::step::emit_body` strings used by the
production table and executes those function pointers through the same
`runtime::step::execute_step` fetch-and-PC operation used by `Subject`. Only
identical complete body strings are deduplicated. SingleStepTests JSON is read
at the external corpus boundary; prepared expectations and ordered case ledgers
use the native binary format. The command preserves generated bodies, prepared
inputs, binary hashes, stdout/stderr, durations and ordered case-name/digest ledgers. Complete sparse
memory, all registers/status/emulation state, PBR/PC, ordered active bus accesses
and cycle count are compared. An inactive runner, missing/reordered ledger row,
unsupported case or compilation error cannot produce a pass. Native fuzz output
uses binary case ledgers and TOML summaries.

The complete pinned native corpus contains 10,000 cases for each of seven
opcodes: 70,000 cases. All four M/X widths and D flag states are represented.
Eleven literal vectors additionally guarantee program-bank wrapping, 24-bit
wide-store wrapping, accumulator-high preservation, SEP index truncation,
REP/native/decimal preservation, and DEX zero/sign/wrap boundaries.

Corpus vectors specify ordered active memory cycles and inactive cycle counts;
the abstract bus does not expose inactive pin-level address/data outputs or
VDA/VPA/VPB signals. These are not claimed as validated. Decimal arithmetic,
branches, indexed/direct-page penalties and emulation execution are outside
the current seven-pattern compiled scope. ADC overflow failures remain explicit
unsupported debt in setup and iteration documentation.

SNES tests independently assert 6/8/12-clock access phases, slow/fast ROM
selection, STA-triggered pending DMA and 48/54-clock resume waits, 40-clock
refresh pauses, three old-I IRQ handler/stack oracles, eight NMI handler/RTI
phase oracles, and all 105 HDMA boundary rows. Expectations come from the
public Super Famicom timing documentation and original rr2-beta23 / bsnes085
Compatibility black-box observations, with source links and hashes in the
timing report. Frame/input rollover and resume tests retain matching final
state, WRAM and video. An inspected/plain generated replay also establishes
that instruction-boundary observation does not alter execution.

Build output is isolated under each evidence output, allowing concurrent runs
without sharing generated binaries. The Rust command verifies the pinned corpus
archive and consumed-file identities, compiler, source/config inventory and
unchanged inputs throughout execution. `--out`, `--opcodes` and `--batch-size`
support development runs; selected-opcode evidence declares its reduced scope.
The `sm-pipeline pattern-check` command returns 0 for a scoped pass, 1 for
divergence, and 2 for infrastructure/incomplete work.

Fresh instruction and timing reports use schema version 3 and readiness
declaration contract version 2. The source-owned timed
pattern declaration is also the production support registry: each opcode has
its addressing behavior, native M/X scope, required input conditions and timing
branches, independent fixture names, and explained exclusions. Changing a
report cannot shrink production support. A declaration override must exactly
match the candidate source. Extending support requires a declared complete
corpus, fixtures filling missing applicable conditions, and independently
specified hardware tests; existing unsupported patterns remain interpreted.

Required case counts are derived from the verified pinned opcode files and
named independent fixtures. Evidence retains a source-bound declaration,
complete consumed corpus files, independent expectations, ordered expected and
completed ledgers, and condition counts. Terminal verification reconstructs
the expected cases and conditions from those inputs, checks the actual emitted
binaries/processes, and requires every supported mode and timing branch to
pass. Development opcode subsets and custom corpus samples cannot establish
acceptance, even when their scoped execution passes. The command
`--declaration PATH` rejects a declaration differing from production source.

Readiness contract version 2 requires these extensible instruction and timing
reports. Historical accepted runs without the source-owned declaration retain
their original version 1 seven-opcode, 70,011-case contract and pointer lineage.
A candidate containing the declaration cannot downgrade its reports or
manifest to version 1. Pattern success alone still leaves fresh generated
parity, focused execution, independent validation and the other cold acceptance
gates mandatory.

`instruction.toml` and `timing.toml` summarize the checks, `expected-cases.bin`
preserves all typed expectations, and `cases.bin` preserves the ordered result
ledger. Each batch retains `input.bin`, `prepared.bin`, emitted `bodies.rs`, and
separate TOML generation/build/execution records.
