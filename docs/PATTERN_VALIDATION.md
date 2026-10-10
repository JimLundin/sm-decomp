# Enabled timed-pattern validation

`just pattern-check` allocates an immutable scoped evidence run and executes all
available native SingleStepTests vectors for LDA immediate, LDX immediate,
REP/SEP, STA absolute, NOP and DEX, plus independent boundary fixtures. It then
runs independent generated-path SNES timing regressions. A scoped pass does
not certify readiness; the readiness command must combine all required gates.

Each batch compiles exactly the `recomp::step::emit_body` strings used by the
production table and executes those function pointers through the same
`runtime::step::execute_step` fetch-and-PC operation used by `Subject`. Only
identical complete body strings are deduplicated. Expectations remain runtime
JSON. The command preserves generated bodies, prepared inputs, binary hashes,
stdout/stderr, durations and ordered case-name/digest ledgers. Complete sparse
memory, all registers/status/emulation state, PBR/PC, ordered active bus accesses
and cycle count are compared. An inactive runner, missing/reordered ledger row,
unsupported case or compilation error cannot produce a pass. The ordinary
legacy fuzz output and tests remain intact.

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
without sharing generated binaries. The script verifies the pinned corpus
archive and consumed-file identities, compiler, source/config inventory and
unchanged inputs throughout execution. `--out`, `--opcodes` and `--batch-size`
support development runs; selected-opcode evidence declares its reduced scope.
The public `run(corpus, output, opcodes, batch_size, timing)` operation returns
0 for a scoped pass, 1 for divergence, and 2 for infrastructure/incomplete work.
