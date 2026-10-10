# Offline RAM decompression pilot

Run `just decomp-routine` (or `python3 tools/decomp_routine.py --output DIR`).
Each invocation creates fresh prepare/build/compare/verdict evidence. A successful
synthetic job reports **partial**, exits zero, and sets `routine_accepted: false`.
Errors exit nonzero and retain an error verdict. The 40 checked-in synthetic fixtures specify expected bytes independently and compare every byte
of canonical 128 KiB WRAM, including unchanged surrounding memory.

The versioned job manifest declares the selected $80:B119 routine, pinned
upstream revision and archive identity, memory/numeric contracts, dependencies,
notices, scope and exclusions. The runner checks the bank, all upstream headers
and complete license against that archive. It builds the unchanged whole
`sm_80.c` bank with function/data sections and discards unrelated sections at
link time. Its only retained external dependency is the checked `RomPtr` shim.
Every run retains source, shim, implementation, fixture and executable digests,
compiler versions, exact commands, stdout, final states and complete notices.
The original C algorithm is never retyped into the harness.

`crates/decomp` is independent of generated gameplay code. Its public seam is
`decompress_to_mem(&RomView, &mut WramView, source, destination)`. `RomView`
validates LoROM addresses and immutable extents; `WramView` provides checked
byte operations and explicit unaligned little-endian words against one full
WRAM allocation. Source reads preserve bank rollover. All eight RAM command modes execute through explicit byte operations. Short lengths
are 1–32 and extended lengths are 1–1024; relative XOR exists only in extended
form and stops at 768 because header FF is reserved for termination. Errors may leave stores already performed; a fresh process and
initial state are required for each comparison. The candidate CLI accepts
`ROM WRAM SOURCE DESTINATION OUTPUT` and emits JSON diagnostics.

The C fixture admission validates every source operand and every WRAM read/store
extent for all command modes, without predicting decompressed values. The shim validates ROM reads but cannot bound arbitrary stores
inside unchanged C; malformed or unsupported fixture streams are rejected
before it runs. Signed relative references may read initialized WRAM before the destination when
the absolute access remains in bounds. Synthetic streams can cross bank ends,
including multi-byte headers and operands. Rust's memory bounds remain checked.

The manifest declares the exercised semantic cases, and each fixture records the
Rust branches actually reached. Coverage includes every short/extended mode,
minimum/maximum lengths, odd word fill, byte wrap, sequential overlap and XOR,
preexisting relative reads, and header/operand/literal bank rollover. A successful
job sets `synthetic_verified: true` only after full-WRAM C/Rust/expected comparisons
and no missing declared cases. It still sets `routine_accepted: false`.

Malformed streams are a separate bounded-error contract. Truncated headers/data,
missing terminators, invalid ROM addresses and out-of-bounds references return
Rust errors; the fixture admission rejects these before the unchecked C routine
is built or invoked. No malformed-input C-equivalence claim is made.

Captured calls can now be supplied with `--captures MANIFEST`. The runner validates
source/routine identity, initial and per-call full 4 MiB ROM view identities,
source/destination bounds, and complete before/after WRAM identities before
building either implementation. Every captured stream receives operand/read/store
admission, and every declared call runs from a fresh initial state in separate C
and Rust processes twice. Both full WRAM and repeated Rust diagnostics must match.
The first mismatch retains routine/fixture/subject identity and byte offset in the
failing verdict. Coverage absent from captured game data is listed separately
from the synthetic coverage contract.

Add `--original-rom ROM` to independently execute three eligible captured calls
at original $80:B119. This instruction runner links the archive-verified upstream
65816 CPU, never the C decompressor. The ROM is unchanged and must match the
capture's original-ROM digest. Every captured compressed stream is checked byte
for byte against that ROM; the selected calls must have output below 64 KiB and
all destination/backreference accesses above harness scratch/stack memory. The
bus admits only WRAM and mapped ROM, rejects MMIO and ROM writes, and disables
gameplay hooks and interrupts. An instruction cap and an explicit RTL sentinel
validate termination; status, DB, D and stack restoration are checked.

The original ABI stages the source at direct-page $47–49 and long destination at
$4C–4E with D=0/native mode. Original execution changes routine scratch $47–48,
$4A–4B and $4F–50. Comparison excludes those bytes, staged pointer bytes and only
the observed stack-write interval plus the three harness RTL-frame bytes. Every
raw mismatch and exclusion remains in `original-evidence.json`; any other WRAM
mismatch fails. This establishes bounded algorithm evidence using an instruction
CPU from the same upstream project, not independent emulator timing or whole-game
parity. Full C/Rust comparisons have no exclusions.

A reproducible local run is:

```sh
python3 tools/decomp_routine.py --captures CAPTURE_DIR/manifest.json --original-rom rom/sm.sfc --output out/decomp/routine
```

Capture with `just decomp-baseline 9000`; use the resulting replay fixture manifest.
All ROM views, states and derived evidence remain beneath ignored local output.
The retained ticket04 run checked all 23 captured calls twice and three original
calls; it remained partial with `routine_accepted: false`.
Every admitted fixture also executes an explicitly instrumented copy of the pinned
C bank. Only its six destination store expressions change, with exact replacement
counts and original/transformed digests retained. Its whole WRAM must equal the
unchanged C oracle before its complete ordered `(offset,value)` store stream is
compared with the Rust safe WRAM observer. Repeated/transient stores are retained;
complete write sets and store counts appear in each fixture record.

Use `--accept` with the frozen capture manifest and original ROM to request bounded
offline acceptance. The independently checked-in `offline-acceptance-v1` contract
pins the canonical job digest (all 40 synthetic fixtures and branch declarations),
exact 23-call capture manifest and original ROM digest. User manifests that alter
fixtures or shrink coverage fail, even when their remaining expectations match.
An accepting run executes 11 real controls through the same public CLI: wrong
output, write-and-restore, disabled/unexecuted replacement, reduced executed branch,
build error, crash, runtime timeout, altered source identity, removed fixture and
reduced case declaration. Runtime controls must actually build and reach a fixture;
unrelated compiler failure cannot satisfy them. `--fault NAME` reproduces each
control, retaining generated faulty candidate sources and an error verdict.

Acceptance retains the contract, exact source/implementation/build and binary
identities, full fixture states and ordered writes, observed cases, all negative
verdicts, and predecessor lineage in `acceptance.json`. The separate
`decomp-acceptance.json` pointer references that immutable run and verdict digest;
`latest-attempt.json` always records the current result. Failed/partial attempts
preserve earlier acceptance as predecessor evidence but exit with their own result
and cannot acquire its verdict. `verify_accepted(pointer_path)` validates current
source and contract identities plus the sealed artifacts before another consumer
uses an accepted routine. No decomp artifact changes recomp acceptance.
The live C/Rust ABI belongs to ticket06.
No offline partial result establishes whole-game Rust ownership or original-ROM
parity.

Validation: `python3 -m unittest tools.test_decomp_routine` and
`cargo test -p decomp --locked --offline`.
