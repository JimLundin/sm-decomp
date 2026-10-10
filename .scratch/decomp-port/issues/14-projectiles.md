# Samus projectile data and lifecycle
Status: resolved
Blocked by: 07
Triage: ready-for-agent

Own bank $93 initialization, damage/radii and complete delete/goto instruction
language, projectile clearing/explosion lifecycle, inherited initial velocities,
and shot origin/direction in bank $90. Actual pinned C and complete WRAM oracle.
Terrain-impact flight, projectile trails/Mode7 rendering and power-bomb HDMA
remain explicit follow-up systems; no hidden host fallback.

## Implementation scope

The Rust module owns projectile damage and directional instruction selection,
beam/missile/SBA initialization, the entire $93 delete/goto/data instruction
language, clear/kill/explosion state and audio, inherited packed velocities,
missile acceleration, super-missile link allocation, projectile reflection,
bomb fuse/explosion, screen lifetime, firing admission and Samus shot origins.
Native `audio` handles sound queues. All pointers and slot/table indices are
checked before unchecked C admission; malformed cycles reject with a 1,024-step
budget. Mode7 explosions, drawing/trails, terrain-impact flight pre-instructions,
power-bomb HDMA, and top-level firing/frame dispatch are untranslated dependencies.
None use a hidden C fallback in this module.

## Completion evidence

- Exact final native source: 34 routines / 3,656 actual pinned C fixtures agree
  on returns and every 128 KiB WRAM byte; acceptance consumer reverified.
- Verdict: `/tmp/sm-projectiles-final/20261010T202436.572621Z/verdict.json`.
- All real-ROM $93 goto/delete entries exercised, every valid beam/nonbeam data
  record, all ten directions, 253 poses, missile phases/link/reflection and bomb
  countdown/screen boundaries. Invalid absent direction records reject before C.
- Nine independent synthetic-ROM tests pass; targeted clippy `-D warnings` and
  rustfmt pass. Commercial data remains local/ignored.
- Scope is offline equivalence only; live exercised and original ROM parity are
  unproven. The broader decomp specification remains open.
