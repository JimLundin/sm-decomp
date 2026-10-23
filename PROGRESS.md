# Engine modernization progress

Updated: **2026-10-23 14:40 UTC**

**Player projectiles now run through one owned production subsystem.**
The broader engine modernization remains unfinished.

## How this reaches the engine structure

Production consolidation has advanced; net code reduction has not. Removing
unsafe calls alone does not establish the intended engine. The projectile change
retired 14 adapters but added 596 runtime/probe lines and 793 verification lines.
That is an ownership improvement with a real source-size cost.

The next concrete delivery is one room-mechanism subsystem replacing creation,
setup, scheduling, commands and drawing together. Its isolated implementation now
shares field/effect access across rule families and shares drawing tile emission
and destination calculations. Those are draft deduplications, not landed savings.
Caller integration, deletion of all 24 adapters and the old bank implementation,
runtime comparison and complete source accounting remain before adoption.

Each adoption must show the public operations, state owner, migrated callers,
deleted implementations and total size change. Merely moving files or adding
another safe adapter does not meet that milestone.

After responsibility consolidation, state must remain owned between operations:
external writers, reset/load and suspension must use those owners consistently.
Gameplay can then communicate with rendering and audio through explicit outputs
instead of shared emulated memory. Persistent ownership and replacement of the
emulated backend are still unfinished architectural steps. The accepted destination
is described in [the architecture](docs/target-architecture/README.md).

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
and the queries used by encounters and room loading. The baseline is now frozen;
replacement implementation is underway outside main. No replacement is adopted yet.

One owner will hold 40 mechanism slots, item graphics, drawing and execution
state during subsystem operations. Native calls must receive its published state;
execution then imports their returned state. This does not establish ownership
between frames. All drawing/clipping algorithms move with the subsystem; three
unrelated palette/sound routines relocate intact.

The candidate covers 24 RAM adapters, bank orchestration, spawning and setup
dispatch: **3,684 existing lines** before retaining required native behavior.
This is the scope to replace, not a savings claim. Estimated runtime change is
**-400 to +300 lines**, plus **450–800 validation lines**. These are planning
ranges; baseline verification has already exceeded the initial validation estimate,
as detailed below. Actual replacement code, tokens and bytes will be counted before adoption. Repeated
field/effect handling must be consolidated with the old adapters removed.

Verification will use current production entries and the existing DLL workflow,
comparing complete RAM plus return/yield values and relevant native effects.
Message resume, live selectors, partial failures and ROM allocation changes need
explicit cases. Exact old source and the verified DLL are preserved locally.
Unaligned raw reads in old code require qualified evidence; they cannot count as
defined Rust equivalence. Old experimental results are not new integration proof.

The **1,018-case actual-production baseline is frozen**: 1,015 normal captures,
two expected native diagnostics and one two-second nonreturn observation. It
includes RAM, returns, checked failures, native effects, message completion,
outer door continuation, external consumers and displacement aliases. All earlier
1,009 outputs match byte for byte, including the original 90-case references.
The release/frontend/verifier build passes. These are reference observations,
not a pass for the replacement or exhaustive branch coverage.

The expansion adds **609 main lines / 5,542 tokens / 22,304 LF bytes**, all for
verification. The complete mechanism baseline and exports now total 926 lines.
Current main has **109,836 Rust lines** and 2,778 unsafe declarations; nine added
declarations belong to test helpers calling old entries. No runtime reduction is
claimed for this preparation. Captured outputs and hardware traces remain local.

Nineteen replacement rule families now share one field interface, removing their
separate context traits and RAM adapters in the isolated draft. Remaining validation
includes qualified checks for eight old commands with undefined memory accesses.
The nonreturn capture observes the timeout, not the preceding RAM write dynamically.
Adoption still requires
the complete caller migration, old-code retirement and final combined verification.

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
Its main delta is +951 lexical tokens / +3,581 LF bytes; retained local diagnostics
add 10 lines / 282 tokens / 858 bytes.
Complete incremental authored cost is 99 lines / 1,233 tokens / 4,439 bytes;
staging copies count once. No new mechanism runtime is integrated yet.

Persistent ownership remains blocked by external state writers, whole-RAM state
replacement and a continuation byte shared with other game modes. Further
reduction, all-unsafe elimination, independent backends and boot-to-credits
verification remain unfinished.

See [PLAN](docs/PLAN.md), [HANDOFF](docs/HANDOFF.md), and
[the accepted architecture](docs/target-architecture/README.md).
