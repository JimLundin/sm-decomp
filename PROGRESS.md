# Engine modernization progress

Updated: **2026-10-23 13:18 UTC**

**Player projectiles now run through one owned production subsystem.**
The broader engine modernization remains unfinished.

## Latest consolidation

[Player projectiles](sm/src/game/player_projectiles/mod.rs) now owns shots, trails,
weapon/charge controls and launch history during each subsystem operation. Real
preparation, drawing, enemy, door, message and palette callers use it. Fourteen
whole adapters and superseded bank routines are removed. Existing shared algorithms
execute through one state-access boundary, including broad and overlapping indices.

External calls publish the owner and reacquire returned state. This is temporary
ownership during execution; the current game and emulated backend still own state
between operations. Native flare content and output boundaries retain explicit
unsafe code. Persistent engine state and independent rendering/audio remain open.

## Verified results

- Release library, frontend and verifier builds pass.
- **106 regression groups pass**, including all 391 owned words / 782 bytes.
- **104 complete RAM images match the old implementation byte for byte.** Synthetic
  cases cover actual weapon producers, motion/history tails, drawing, reflection,
  resets, both previously missing motions, flare ROM views, collision and expiry.
  Power Bomb expiry also checks real DMA channel configuration.
- **8,780 oracle frames pass with zero failures**, with seven required execution
  probes and 120 follow-up frames. This covers one gameplay room and no credits.
- Independent review approved implementation, callers, evidence and accounting.

The existing 60-suite result remains verified historical baseline evidence. It is
separate from the new integration checks. Neither the synthetic cases nor this
movie segment proves complete-game coverage.

## Code size and remaining work

Main source is **108,821 Rust lines in 226 files**. The change adds 1,389 main lines:
596 for runtime/probes and 793 for verification. Unsafe function declarations fall
by 22 to 2,767. The codebase grew; the structural gain is one production owner and
retirement of the superseded execution paths.

| Incremental source | Lines | Tokens | LF bytes |
| --- | ---: | ---: | ---: |
| Main implementation and verification | 1,389 | 11,644 | 43,988 |
| External old-reference fixtures | 136 | 4,316 | 11,355 |
| Initial ownership experiment | 478 | 5,029 | 20,068 |
| Complete change | 2,003 | 20,989 | 75,411 |

Staging copies are counted once. The rejected experimental Session is archived;
its removal contributes no claimed production savings. No ROM-derived fixtures
are committed.

Next: select another complete ownership boundary from the real caller graph,
with a public interface, actual caller migration and named deletions. Further
reduction, all-unsafe elimination, persistent state, independent backends and
boot-to-credits verification remain unfinished.

See [PLAN](docs/PLAN.md), [HANDOFF](docs/HANDOFF.md), and
[the accepted architecture](docs/target-architecture/README.md).
