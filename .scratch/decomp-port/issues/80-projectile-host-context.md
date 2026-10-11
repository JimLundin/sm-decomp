# Explicit enemy projectile host context
Status: resolved
Blocked by: None
Triage: ready-for-agent

Represent eproj_spawn_pt, eproj_spawn_rect (x/y/w/h), r22 and varE24 as typed
host state outside canonical WRAM. Add explicit host allocation entry preserving
absence refusal and full-slot parameter writes. Port947F stationary fireball
explosion, E468 dust/explosion and EF29 drop initialization. Verify actual C full
WRAM/rawreturns plus mandatory host sidecars; no guessed aliases or live claim.

## Implementation

ProjectileHost exposes exact point x/y, rectangle x/y/width/height, r22 and enemy
header corresponding to actual C globals (eight words, outside WRAM). Explicit
spawn_room_with_host preserves context; existing spawn APIs refuse required
context before mutation, except full slots still write only init parameter as C.
947Ffireball explosion, E468dust scripts (30entry table), and EF29drop initialization
now use this seam. EF29validates header/chances/script extents before allocation.

Opt-in projectile_host manifests carry eight host_words per case. Actual bank86
shim loads/saves C globals through explicit sidecar argv paths; Rust loads typed
context. Verification compares mandatory16byte hostsidecars independently of
full WRAM and raw return, and authenticates their sealed bytes. A regression
rejects host-only mismatch and missing inventory entry. Existing non-host ABI
unchanged. Validated hostmanifest requires ROM and allocator module.

Independent Rust tests prove absent-host refusal leaves all WRAM untouched,
stationary explosion consumes supplied coordinates/gfx/parameter while preserving
context, and fullslots alter only parameter with no ROM reads. Fullcrate
checks pass settled source. Initial host97 and expanded769runs passed; final
source-stable evidence follows. Diagnostic first run rejected relative sidecar
paths (execute cwdROOT); fixed explicitargv paths, no gameplay mismatch.

## Completion evidence

Final769 actual-C cases pass full WRAM, raw return and all eight external host
words at `out/port/projectile-host-context/20261011T063835.827427Z/verdict.json`.
Includes all three initializers, all projectile slot choices, full allocation,
coordinate sign/wrap edges,30dust scripts, four actual enemy-header drop tables.
Final evidence authenticated with verify_plm_instruction_evidence after cleanup.
Fullport tests/fmt/strictClippy pass;11PLM/family harness tests pass, including
host-onlytamper and missing sidecar rejection. Tests' temporary runs removed
with their temporarydirectories. Allfour finished explicit runs cleaned with
port_cleanup.py apply/deduplicate: unsealed0; snapshots21,102,592 +170,000,384 +
170,000,384 =361,103,360bytes (~344MiB).17GiBdisk free; no implementerworktree.
Historical diagnostic/superseded snapshots retained; final source-stable verdict
is current authenticated evidence. Explicit context is offline only; native
scheduler/caller context propagation and live admission remain open.
