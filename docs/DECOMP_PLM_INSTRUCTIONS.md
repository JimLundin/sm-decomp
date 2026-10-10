# Native PLM bytecode and draw data

`plm_instructions::instruction` executes bank84 control, timer, conditions,
call/link/return, door counters, BTS, scroll data and audio/VRAM queue instructions.
Operands and continuations are byte addresses: SetTimer writes only the low byte,
subroutines use one original link register, decrement branches test the old value,
and door completion copies the original overlapping word from a byte bit array.
All state remains in canonical WRAM. VRAM instructions enqueue original records;
actual upload remains owned by the host.

`process_draw_data` decodes original horizontal/vertical tile runs and signed
origin-relative offsets from ROM or low WRAM. It writes full level tiles and
rejects invalid/unterminated input. It does not generate screen tilemaps or MMIO.
`process_control_frame` preserves descending slot order, no-op preinstructions,
instruction timer underflow and repeated control dispatch until Sleep/Delete.
Its instruction budget rejects loops. Unsupported preinstructions, rendering
phases and message coroutines return explicit errors. This bounded API must not
replace the complete live `PlmHandler_Async` engine.

The dedicated oracle adapter compiles the entire pinned C tree, including actual
`CallPlmInstr` and `PlmHandler_Async`; only pointer normalization and single-threaded
headless host definitions are added. It compares full 128 KiB WRAM and bank84
continuations. `PlmInstruction` and `PlmControlFrame` are oracle shims, not upstream
symbol counts. The synthetic ROM generator provides arbitrary byte-aligned
branches and scroll data without copying commercial ROM bytes.

```sh
python3 tools/port_plm_fixtures.py
python3 tools/port_plm_instructions.py --manifest tools/port/plm-instructions.json --source deps/snesrev-sm --output out/port/plm-instructions
```

The declared fixture domain covers 42 opcode handlers over 336 direct cases,
12 real-dispatcher frame-control scenarios and 12 ROM/WRAM draw-data cases.
Malformed input is tested against Rust errors separately, not unchecked C.
These are offline equivalence fixtures; live replacement and original-ROM
parity require separate evidence.
