# Readable Rust gameplay translation

Use [the batch runner](DECOMP_BATCH.md) for repeatable verification, replay,
status and project checks: `python3 tools/port_batch.py run`.

Primary source: snesrev/sm revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, with the upstream MIT notice retained
in `crates/decomp/LICENSE.upstream.txt`. The immutable decomp foundation remains
predecessor evidence. New gameplay modules live in `crates/port`.

## Differential family CLI

Run a family against actual compiled upstream C:

```sh
python3 tools/port_family.py verify system --source deps/snesrev-sm --output /tmp/system-evidence
python3 tools/port_family.py verify audio --source deps/snesrev-sm --output /tmp/audio-evidence
```

Each family has `tools/port/<family>.json`:

```json
{
  "family": "example",
  "rust_module": "crates/port/src/example.rs",
  "bank": "sm_80.c",
  "routines": [{
    "symbol": "PrepareBitAccess",
    "args": 1,
    "return": "uint16",
    "cases": [{
      "name": "bit-zero",
      "a": 0,
      "b": 0,
      "fill": 165,
      "writes": [[1509, [0, 0]]],
      "expected_return": 0
    }]
  }]
}
```

`args` supports zero to two `uint16` parameters; `return` supports `void`,
`uint8`, `uint16`, and `uint32`. Case names must be unique across the family.
`fill` initializes **all** canonical WRAM, then `writes` applies byte patches.
Defaults are zero fill and arguments. `expected_return` is optional independent
worked-example evidence, checked against C as well as Rust. Each family must
document its admitted domain and provide branch/boundary fixtures within it;
fixture bounds alone do not establish safe indexed accesses inside unchecked C.

The Rust module exports:

```rust
pub fn dispatch(ram: &mut Wram, symbol: &str, a: u16, b: u16)
    -> Result<u32, &'static str>;
```

The harness includes the shared WRAM implementation and selected module in a
standalone wrapper. It compiles the unmodified actual bank with function/data
sections and linker garbage collection, so unresolved unrelated hardware paths
are excluded. Reaching a needed unresolved dependency fails the build; no fake
hardware stubs supply oracle behavior. Return values and every one of the
131,072 resulting WRAM bytes must agree. Void routines return zero in both
adapters. This seam checks state aliasing, preserved memory, wrapping arithmetic
and results; it does not check byte-store order, hardware effects or timing.

Every timestamped run retains the manifest, compiler versions, commands and
stdout/stderr, generated adapters, candidate and memory sources, executable
hashes, upstream license, source identities, before/after WRAM and verdict.
Failure evidence includes both outputs and first differing offset. All files in
the unpacked source must match the pinned hashed archive before compilation.
The CLI returns nonzero on mismatch/build/error/empty fixture sets. A successful
verdict is `offline-verified`, with `live_exercised: false` and original-ROM
parity explicitly `not-tested`. Live replacement evidence is a separate gate.

The public CLI regressions compile actual `NextRandom` and independently known
zero-seed result (273). Deliberately incorrect return and unrelated upper-WRAM
mutation both reject with retained mismatch artifacts:

```sh
python3 -m unittest discover -s tools -p test_port_family.py
```

## Inventory and next substantial gameplay frontier

Regenerate the definition index with:

```sh
python3 tools/port_family.py inventory --source deps/snesrev-sm --output tools/port/source-inventory.json
```

The checked-in inventory records **5,354 actual function definitions**, their
source file/line, original address where the pinned C supplies one, definition
hash, signature, body size, direct lexical calls, defined dependencies,
unresolved macro/external calls and indexed indirect calls. Every entry starts
`unverified`; an inventory entry is never acceptance evidence. Static helpers
without upstream address comments retain a null address. Lexical call discovery
is a planning aid: function-pointer tables and macros still need source review.

The largest gameplay-heavy banks include `sm_90.c` (439 definitions, about 6,044
body lines of Samus mechanics), `sm_a9.c` (481 definitions, 5,710 lines),
`sm_a6.c` (295 definitions, 4,043 lines), and `sm_86.c` (341 definitions, 3,996
lines). These counts describe source, not completed translation.

Next tickets should take ownership of coherent Samus movement/animation or
projectile subsystems with their dependencies, then enemy subsystems, rather
than accumulating short independent helpers. Concrete dependency examples:

- `Samus_Animate` at `$90:8000` calls `Samus_HandleAnimDelay` and dispatches through
  `kSamusFxHandlers`; its delay helper reaches ROM-backed speed-booster animation.
- `Samus_CalcBaseSpeed_X` at `$90:9A7E` depends on `IsGreaterThanQuirked`, `Mult8x8`,
  paired-word arithmetic and ROM speed-table entries. A ROM view and explicit
  table admission are needed before this movement family can be compared.
- `ProcessEnemyInstructions` at `$A0:C26A` reaches `CallEnemyInstr` dispatch and
  ROM instructions; translate the instruction engine with a handler registry.
- `EnemyMain` at `$A0:8FD4` includes collision, drawing, death and instruction
  execution. Its broad dependency surface is a target for progressively owned
  subsystems rather than an isolated routine fixture.

Family offline acceptance, mixed live execution and original-ROM parity remain
separate claims. Full large-scale gameplay ownership remains the active goal.

## Gameplay arguments, ROM and captured cases

For wider gameplay routines, replace `args` with explicit `arg_types`, each of
`uint8`, `uint16`, `uint32` or `int32`. Prototype declarations are checked against
actual upstream `funcs.h`/`sm_rtl.h`; return `int32` is compared as raw `u32` bits.
A typed family without ROM exports `dispatch_i32`; a ROM family exports
`dispatch_rom(ram: &mut Wram, rom: &[u8], symbol: &str, a: i32, b: i32)`.
Unsigned 32-bit arguments travel as decimal `i64` then cast to `i32`, retaining
all bits; the family casts back to `u32` for unsigned routines.

A manifest ROM asset is `"rom": {"file": "/absolute/game.sfc", "sha256": "..."}`.
The harness validates its hash and 1-byte to 4MiB extent, retains the exact ROM,
and passes identical bytes to both subjects. Each family's dispatch must reject
out-of-range ROM addresses/table indices before access. Add actual dependency
sources via `"c_support": ["sm_rtl.c", "sm_cpu_infra.c"]`; section garbage
collection keeps the needed real functions/storage. No hardware oracle stubs
are introduced.

**Rust runs before unchecked C on every case.** Its guarded dispatch must return
an error or panic on every state outside the declared valid C domain, without
unsafe memory access. A rejected candidate stops the case before C execution.
This is an executable admission check, including ROM table/state-dependent
indices, not merely trusting that a fixture's initial patches are in WRAM.
Candidate arithmetic alone is not a proof of C safety; family guards must match
the original indexed-access constraints and signed-overflow limits.

Captured fixtures can replace synthetic initialization with `before_file` and
mandatory `before_sha256`. Such captures cannot also specify `fill` or `writes`.
Optional `expected_after_file` plus mandatory `expected_after_sha256` authenticates
a live capture's full resulting WRAM. `expected_return` compares the captured
return value. Both subjects must agree with the capture; hashes and every byte
are retained, while original-ROM parity remains a distinct gate.

`verify_family_evidence(path_to_verdict)` authenticates evidence before live
consumption. It checks the complete retained artifact inventory, current module,
memory and harness hashes, binaries, actual full-memory/return equality, capture
expectations and nonempty complete manifest coverage. Older verdicts without
artifact inventories must be regenerated. This detects corruption against local
records; it is not an external cryptographic signature.

A family that calls existing Rust modules declares `rust_imports`, for example
`["audio", "samus"]`, to expose those shared modules at the wrapper's root.
`rust_dependencies` lists additional source paths outside `crates/port/src`.
The harness seals and retains every source in the shared port directory plus
these declared dependencies; changed imported implementations invalidate prior
verification. C diagnostics are retained verbatim; the final numeric stdout
line supplies the routine return. This supports actual upstream diagnostic
output without discarding evidence.

Live controls run at the generated family ABI, rejecting null/invalid extents,
unknown routine IDs and overlapping ROM/WRAM without mutation, and converting
candidate panics to an error. Retained offline output corruption rejects before
any live build. The opt-in compiled gameplay control intentionally supplies an
incomplete `ClearUnusedOam` candidate that passes an empty-loop fixture and then
must reject observed gameplay divergence:

```sh
SM_PORT_LIVE_CONTROLS=1 python3 -m unittest discover -s tools -p test_port_live.py
```

For continued gameplay translation, use actual-C family verification first, then
run live validation with `tools/port_live.py` or the batch runner. Future live
captures default to packed evidence in the shared `out/evidence` store. Omit
the historical `--compress-effects` option from ordinary replay commands: it
selects legacy gzip storage. Use `--storage packed` to make the current default
explicit. Family oracle runs keep their existing full-WRAM comparison format.

After live verification and scoped cleanup, authenticate the retained verdict
with `python3 tools/port_authenticate_live.py RUN/verdict.json`. Inspect packed
members and individual frames through `tools/port_evidence.py`; commands and
store retention requirements are in [DECOMP_BATCH.md](DECOMP_BATCH.md).
Packed storage changes retention and inspection, while gameplay admission still
requires its own authenticated source, coverage and ownership evidence.

The collision live adapter accepts `--collision-verdict` from the actual PLM
callback harness. Rust collision owns an independent WRAM transaction; its
synchronous callback transfers state and displacement to the canonical host and
back. `--plm-verdict` additionally admits native allocation evidence. With a real
collision amount reference, the host classifies the immutable PLM header before
execution and selects native Rust for supported setup functions. Other headers
and calls without collision context execute measured C. Native errors terminate
the replay without a C retry. Total, C and Rust PLM call counts are retained;
native admission requires actual setup execution and matching ownership in both mixed
processes. Ordered effects compare header, before/after amount and return flags.
Allocation counters include an exhausted slot pool. A separate Rust setup
counter requires a free slot at native entry; only successful calls with that
slot count as setup execution. Neither count proves every setup was reached.

Generic enemy instruction handlers can be installed individually with normalized
banked pointer arguments/returns. The bounded generic interpreter remains an
offline implementation: the live C dispatcher still owns bank-specific scripts.
