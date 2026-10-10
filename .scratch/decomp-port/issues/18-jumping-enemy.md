# Sidehopper/Desgeega jump AI closure
Status: resolved
Blocked by: 15
Triage: ready-for-agent

Translate pinned A3 Sidehopper_Init/Main and Func2..22, coherent floor/ceiling jump state closure shared by Sidehopper and Desgeega variants. Reuse enemy_ai movement Host, preserve unusual unaligned quadratic speed integral and original overlapping extended WRAM. Validate all enemy definition and ROM table indices before unchecked C. Two animation instruction pointer handlers use a separate typed banked-pointer family, with actual sound queue and current-enemy animation signal.

## Completion evidence
Implemented 23 non-pointer routines (Sidehopper_Init/Main and Func2..22), plus two separately verified banked instruction pointer handlers. Enemy definition variants0..3 cover Sidehopper/Desgeega floor and ceiling jumps. All dispatcher states, random jump selection, configured acceleration, ascent apex, downward landing, ceiling reversal, wall reversal, bounded quadratic speed tables and animation signals retain actual C state behavior. Explicit enemy_ai::Host movement dependency; no fabricated callbacks.

State uses canonical base0xf78 and extended0x7800 + enemy byte offset. Definition pointers are allowed odd addresses, preserving actual vanilla definitions d93f..da3f. Init validates kind0..3 and all table reads before unchecked C. The quadratic table is bankA2 838f..8687, 95 rows; unaligned Func2 integration rejects zero/too-large step or any path leaving those rows. Ordinary descending physics clamps index64 exactly as C. Initial mistaken 65-row admission rejected the original large-jump integral at row68 before C; retained `/tmp/enemy-jump-final/20261010T204730.662579Z/verdict.json`.

Final stable Host full actual C comparison `/tmp/enemy-jump-stable-final/20261010T204924.118490Z/verdict.json`:23 routines/192 cases, full WRAM+return and ordered Host effects agree, zero CPU fallback. Prior `/tmp/enemy-jump-final/20261010T204825.383684Z/verdict.json` 23/192 pass before pointer helper addition.
Typed pointer actual C16-case seal: `/tmp/enemy-jump-instructions-final/20261010T204904.755650Z/verdict.json`, 2 handlers, full WRAM+normalized pointer equality. Manifests `tools/port/enemy-jump.json` (Host) and `tools/port/enemy-jump-instructions.json` (typed pointer). Neither claims live execution or original-ROM parity.

Five independent tests pass: ascent apex, descent speed cap and wall reversal, facing and animation order, invalid integral/unknown state rejection, and instruction signal uses current_enemy rather than supplied enemy argument. Tests compiled against actual library with rustc because worktree lacks ignored RustySNES dependency. rustfmt --check passes. Public registration and root Cargo/clippy/live integration required after merge.

Commit: this ticket's implementation commit in git history.
