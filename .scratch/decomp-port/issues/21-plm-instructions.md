# Native PLM script control and draw-data engine
Status: resolved
Blocked by: 13
Triage: ready-for-agent

Translate bank84 control flow, timers, event/item/door conditions, scroll and
queue effects plus PLM draw-data decoding. Preserve byte-granular operands and
WRAM aliases. Bounded frame interpreter rejects unsupported preinstructions,
message coroutines and rendering rather than claiming whole-engine replacement.
Use actual CallPlmInstr and PlmHandler_Async C execution as oracle.

## Completion evidence

Implemented42 bank84 opcode handlers, byte-aligned control flow/call/return,
condition/door bit effects, scroll/BTS, VRAM records and native audio dependency
closure. Native draw-data decoder covers ROM/WRAM row/column runs and signed
origin offsets. Bounded real frame-control path explicitly rejects rendering,
coroutines and unknown preinstructions; whole live engine replacement excluded.
Actual pinned complete C dispatcher/tree matches360 cases (336 direct42 opcodes,
12 frame-loop/timer/sleep/delete,12 draw):
`/tmp/sm-plm-instructions-sealed/20261010T210919.008267Z/verdict.json`.
Five independent synthetic Rust tests pass. Exact final consumer seal and wrong
low-byte timer-store/admission controls are running; final pointers follow.

Final exact module/harness seal, authenticated current worktree:
`/tmp/sm-plm-instructions-admitted/20261010T211046.706854Z/verdict.json`,360 cases.
Actual-C negative-control test passes32.412s: intentionally wrong full-word timer
store differs, omitted stdout and tampered WRAM reject. Five Rust tests also
verify coroutine/budget/render gaps explicitly reject. `docs/DECOMP_PLM_INSTRUCTIONS.md`
records public APIs, bounds and excluded whole-engine live replacement.
