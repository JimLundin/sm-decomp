# Projectile sprite spawn instructions
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate native EprojInstr_ECE3 and EprojInstr_ED17 randomized sprite callers
into readable Rust using the existing RNG and sprite allocator. Preserve unsigned
position wrap, RNG-derived paired coordinates, ROM operand and pointer return.
Verify actual-C full-WRAM/return cases, independent semantics and scoped cleanup.
Report offline coverage separately from live ownership; do not infer allocator
movie execution from these fixtures.

## Completion evidence

Both native opcode paths and direct wrappers are Rust, using existing RNG and
sprite allocator. tools/port_projectile_sprite_fixtures.py finds all five real
byte-aligned ROM operands; 180 actual-C full-WRAM/return cases pass:
out/port/projectile-sprite-spawns/20261011T092633.650533Z/verdict.json.
Independent test confirms single RNG transition 1 -> 0x116, both radii, highest
slot allocation and unchanged projectile coordinates. Strict Clippy passes.
Offline coverage only; allocator live requirement in ticket 94 stays open.

Initial generator assumed word alignment and found no operands; corrected to
byte scanning. Empty-manifest diagnostic run 20261011T092618.710018Z retained;
both cleanup modes reclaimed zero bytes. Successful run ordinary cleanup zero;
dedup reclaimed 48758784 bytes; all 180 cases authenticate afterward. Free disk
269286813696 bytes. Retained sealed binaries, pinned source and ROM dependencies;
no implementer worktree created. Integration branch decomp-port-integration.

After room/fixed-sound additions, refreshed current dependency seal passes all
180 cases at out/port/projectile-sprite-spawns/20261011T093405.420038Z/verdict.json.
Cleanup ordinary zero; dedup 48758784 bytes; post-cleanup authentication passes.
