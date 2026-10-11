# Offline RAM decompression pilot

Run `just decomp-routine` (or `cargo run -p pipeline --locked --offline -- decomp-verify --output DIR`).
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
`ROM WRAM SOURCE DESTINATION OUTPUT` and emits TOML diagnostics.

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
raw mismatch and exclusion remains in `original-evidence.toml`; any other WRAM
mismatch fails. This establishes bounded algorithm evidence using an instruction
CPU from the same upstream project, not independent emulator timing or whole-game
parity. Full C/Rust comparisons have no exclusions.

A reproducible local run is:

```sh
cargo run -p pipeline --locked --offline -- decomp-verify --captures CAPTURE_DIR/manifest.toml --original-rom rom/sm.sfc --output out/decomp/routine
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
offline acceptance. The checked-in schema-2 acceptance contract pins the canonical TOML job digest
(all 40 synthetic fixtures and branch declarations), 23-call coverage and original
ROM digest. A fresh `decomp-c-captures-v2` manifest must validate source and routine
identity, bounded source operands, full ROM and WRAM extents, and all artifact hashes.
The accepting run seals its exact capture manifest digest; historical JSON capture
digests do not authorize current sources. User manifests that alter
fixtures or shrink coverage fail, even when their remaining expectations match.
An accepting run executes 11 real controls through the same public CLI: wrong
output, write-and-restore, disabled/unexecuted replacement, reduced executed branch,
build error, crash, runtime timeout, altered source identity, removed fixture and
reduced case declaration. Runtime controls must actually build and reach a fixture;
unrelated compiler failure cannot satisfy them. `--fault NAME` reproduces each
control, retaining generated faulty candidate sources and an error verdict.

Acceptance retains the contract, exact source/implementation/build and binary
identities, full fixture states and ordered writes, observed cases, all negative
verdicts, and predecessor lineage in `acceptance.toml`. The separate
`decomp-acceptance.toml` pointer references that immutable run and verdict digest;
`latest-attempt.toml` always records the current result. Failed/partial attempts
preserve earlier acceptance as predecessor evidence but exit with their own result
and cannot acquire its verdict. `verify_accepted(pointer_path)` validates current
source and contract identities plus the sealed artifacts before another consumer
uses an accepted routine. The `decomp-artifact-inventory-v2` seal hashes every
retained file before publishing the seal and final verdict: all fixture outputs,
repeat traces and diagnostics, original CPU source snapshot/shim/executable/results,
and complete negative-control sources, executables, logs and failing verdicts.
The final verdict is authenticated by the pointer and the seal by the verdict.
Consumers reject missing or changed artifacts and inconsistent verdict/seal status
or evidence. Historical seals lacking this inventory remain unchanged evidence;
they fail consumption with an explicit unsupported-seal error and require a fresh
accepting run using current sources. No decomp artifact changes recomp acceptance.
The live C/Rust ABI belongs to ticket06.
No offline partial result establishes whole-game Rust ownership or original-ROM
parity.

Validation: `cargo test -p pipeline --test decomp --locked --offline` and
`cargo test -p decomp --locked --offline`.

The live command is `just decomp-live --accepted PATH/TO/decomp-acceptance.toml`.
It checks the current offline seal before building, then links the Rust static
library into a dedicated single-threaded C host. The unchanged C scheduler,
canonical C-owned WRAM, ROM loading and hardware remain in charge. The ABI takes
explicit pointers and extents, validates overflow/overlap and bounds, borrows
only synchronously, retains no pointers and invokes no callbacks. Rust panics are
caught and errors exit replay immediately; no original C fallback follows errors.
The caller must provide live disjoint allocations and exclude concurrent access.

Live acceptance uses the frozen 9000-frame prefix, four independent C/mixed
processes and two complete replay epochs per process after the real C reset.
Each corresponding checkpoint includes serialized CPU, APU, DMA, cart/SRAM and
full WRAM state, both PPU states and rendered pixels. All before/after call states
and ordered PPU/APU writes must also match. The host reports successful Rust/C
routine calls, actual executed command modes and remaining C-to-CPU fallback.
Seven ABI controls exercise pointer/extent/overlap/source/destination guards and
repeated allocation reuse. A separate gameplay error control forces an invalid
source at the first real replacement call and requires exit with zero Rust/C
success hits and no completed checkpoint or after-call state.

Success is labeled `mixed-C-Rust-integration`; original-ROM movie parity remains
`not-tested`. The input conversion remains bit reversal without lag adaptation.
Run `cargo test -p pipeline --test routine_jobs --locked --offline` for native
admission controls. `just decomp-live --accepted PATH` runs the full replay.

The retained ticket06 run accepted all 63 offline fixtures and rejected all 11
acceptance controls with the new ABI source identities, then independently
matched all live checkpoints across eight replay epochs. Each mixed epoch
executed 23 Rust calls and zero C decompression calls; seven command modes were
actually reached (relative XOR remains synthetic-only). Gameplay CPU fallback
was measured as zero in this prefix. This measurement does not establish
coverage beyond the prefix. Hardware write streams are retained as gzip logs.

Historical review verification regenerated acceptance at
`out/decomp/review/offline/decomp-acceptance.json`: 63 fixtures, 38 observed cases,
11 rejected controls and 2,930 authenticated artifacts. That pointer retains its original JSON contract. Current consumers require fresh
TOML acceptance. For example:

```sh
cargo run -p pipeline --locked --offline -- decomp-jobs --count 1 --accepted out/decomp/current/offline/decomp-acceptance.toml --output out/decomp/review/jobs
```

The retained current-source job is `out/decomp/review/jobs/20261010T185638.603810Z`;
its verdict SHA-256 is `a2e58037755569015582d4d339e231bd9fb4c7f7c6c2863b7648dbfa2f2a81bf`.
It verifies every RNG seed, 16 captured states and 8,783 observed calls per epoch,
plus the independent 9,000-frame decompressor/RNG live comparison across four
processes and two epochs each. Every mixed epoch records 23 Rust decompressions,
8,783 Rust RNG calls, zero C replacements and zero measured gameplay CPU fallback.
The ABI error control still exits with no successful Rust/C calls. Original movie
parity and full Rust gameplay ownership remain unproven.

Public acceptance-consumer tests reject copied runs missing final outputs, repeat
traces, original CPU/shim/executable/results, negative-control sources/executables/logs,
altered original diagnostics, and inconsistent accepted status. Those historical checks used the retired Python suite. Maintained regression
tests now run in Rust through the commands above.
