# Native enemy projectiles

`enemy_projectiles` implements eighteen-slot allocation, room/enemy graphics
interfaces and ten room-effect initializers. `enemy_projectile_instructions`
implements45bank86 control/audio/palette/room opcodes, shared x/y/xy movement,
ten room-effect preinstructions and bounded descending frame scheduling.
Glass shards spawn native sparkles; lower newly allocated slots run in the same
frame. Shared movement updates subposition high bytes; room movement uses full
hilo arithmetic. Unsupported collision, AI and opcodes reject explicitly.

Actual-C frame evidence
`out/port/enemy-projectile-frames/20261010T231226.808149Z/verdict.json`
passes748full-WRAM/raw-return cases:360instructions,96movement,
160preinstructions and132frames. Six independent Rust examples, full port suite,
fmt/Clippy and harness integrity test pass. Evidence is authenticated after run
cleanup and integrated worktree removal.

Allocation predecessor evidence is recorded in ticket32 and DECOMP_PLM.md;
current-source consumer seals need regeneration after these additions before
live admission. Native enemy-projectile scheduling is offline verified, with
no live replacement or original-ROM parity claim.
