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
live admission. Native enemy-projectile scheduling remains offline verified;
later live evidence below covers specific rendering/contact entry points.

The maintained crate also exports `enemy_projectile_rendering`,
`enemy_projectile_contacts` and `enemy_projectile_sparks`. Their current-source
oracle evidence covers 3,916 rendering, 8,070 contact and 3,154 sparks cases,
including complete WRAM, raw returns and required host words. Exact verdicts
and cleanup authentication are recorded in tickets 133–135 under
`.scratch/decomp-port/issues/`. All 280 port tests and strict all-target Clippy
pass. New modules pass formatting checks; earlier projectile/PLM/room formatting
debt remains recorded in `contact-render-checks.json`.

Live run `out/port/projectile-contact-render-live/20261011T130027.565661Z`
matches two C and two mixed processes over 9,000 inputs. Each mixed process
executes 676 calls to each priority drawing pass and each contact detector,
plus 23 Rust decompressions and 8,783 Rust RNG calls, with zero measured CPU
fallback. All 190 observation artifacts per subject match and authenticate
after cleanup. Active captures prove low-priority Ceres pad/platform OAM
emission; the high-priority pass filters these low-priority slots. The detectors
exercise their entry gates, while actual collision hits remain a separate live
verification task. This establishes those exercised entry points, with other
gameplay still C-owned and original-ROM movie parity unproven.

The later 45,000-input run at
`out/port/projectile-contact-hit-live/20261011T130615.144077Z` also matches four
fresh processes through the existing soft-reset boundary. Each mixed process
executes 31,713 calls per rendering pass, 166 Rust decompressions and 43,276 Rust
RNG calls with zero measured CPU fallback. The C contact detectors remain
installed to measure native hit-handler wrappers, but both handlers execute
zero times. They remain offline-only. Native combat projectile properties
`5003` exclude the shot-detection gate; this trajectory does not hit Samus.
Further observed allocation/scheduler ownership is specified in ticket 137.
