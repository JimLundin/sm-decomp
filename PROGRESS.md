# Engine modernization progress

Updated: **2026-10-23 13:59 UTC**

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

The projectile milestone has **108,821 Rust lines in 226 files**. It adds 1,389 main lines:
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

## Current work: room-mechanism replacement

The projectile milestone is committed in `04dbc192`. One implementation worker
and an independent reviewer have agreed the complete room-mechanism boundary:
creation, setup, script scheduling, commands, drawing, message suspension, reset
and the queries used by encounters and room loading. Baseline preparation is active;
implementation follows capture of the old behavior. No replacement is adopted yet.

One owner will hold 40 mechanism slots, item graphics, drawing and execution
state during subsystem operations. Native calls must receive its published state;
execution then imports their returned state. This does not establish ownership
between frames. All drawing/clipping algorithms move with the subsystem; three
unrelated palette/sound routines relocate intact.

The candidate covers 24 RAM adapters, bank orchestration, spawning and setup
dispatch: **3,684 existing lines** before retaining required native behavior.
This is the scope to replace, not a savings claim. Estimated runtime change is
**-400 to +300 lines**, plus **450–800 validation lines**. These are planning
ranges; actual code, tokens and bytes will be counted before adoption. Repeated
field/effect handling must be consolidated with the old adapters removed.

Verification will use current production entries and the existing DLL workflow,
comparing complete RAM plus return/yield values and relevant native effects.
Message resume, live selectors, partial failures and ROM allocation changes need
explicit cases. Exact old source and the verified DLL are preserved locally.
Unaligned raw reads in old code require qualified evidence; they cannot count as
defined Rust equivalence. Old experimental results are not new integration proof.

The first **90 actual-production baseline cases are captured and independently
checked**, including RAM, return values and expected checked failures. The
release/frontend/verifier build passes. These are reference observations, not a
pass for the replacement; setup, script, suspension and caller coverage is expanding.
Baseline checks and their exports add 317 lines. Two unsafe declarations call old
entries and expose the test buffer; current count is 2,769. No runtime reduction
is claimed for this preparation.

The expanded **1,009-case baseline draft has passed source review** and awaits
runtime capture. It includes setup/script identities, actual native effects,
message suspension, external consumers and selected aliases. Coverage exclusions
remain explicit; this is not exhaustive branch coverage or replacement validation.

## Room-loading correction

The baseline investigation found and fixed a room-loading bug predating projectile
consolidation. Forward copies corrupted overlapping layer-two and BTS data in the
large landing-site room, causing 13,405 differing bytes at input 20,857. Valid
overlaps now preserve their source contents; non-overlap behavior and checked
failure prefixes remain intact. This matches the native copy result for the
normal room layout. Malformed odd layouts gain no new native-equivalence claim.

The regression failed before the fix and now passes, alongside boundary and
failure-prefix cases. The final build passes **27,684 consecutive input frames**
through a regenerated checkpoint: 20,857 prefix frames plus 6,827 resumed frames,
zero failures. Required probes confirm both corrected copies, completed loading,
gameplay and the message sequence, with 120 follow-up frames. No credits coverage.

The correction adds 89 main lines: 16 runtime/probe lines and 73 check/export lines.
Current main source is **109,227 Rust lines**. Main delta is +951 lexical tokens /
+3,581 LF bytes; retained local diagnostics add 10 lines / 282 tokens / 858 bytes.
Complete incremental authored cost is 99 lines / 1,233 tokens / 4,439 bytes;
staging copies count once. No new mechanism runtime is integrated yet.

Persistent ownership remains blocked by external state writers, whole-RAM state
replacement and a continuation byte shared with other game modes. Further
reduction, all-unsafe elimination, independent backends and boot-to-credits
verification remain unfinished.

See [PLAN](docs/PLAN.md), [HANDOFF](docs/HANDOFF.md), and
[the accepted architecture](docs/target-architecture/README.md).
