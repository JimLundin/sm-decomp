# Readable Rust gameplay translation

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

The public CLI regression compiles actual `NextRandom` and independently known
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
