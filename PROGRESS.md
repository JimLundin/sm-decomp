# Engine modernization progress

Updated: **2026-10-23 14:52 UTC**

**Room mechanisms now use one production subsystem, with the superseded code
removed.** The complete engine modernization remains unfinished.

## What changed

[Room mechanisms](sm/src/game/room_mechanisms/mod.rs) now handles creation, setup,
script scheduling, commands, drawing, graphics, message suspension and lifecycle.
Room loading, player contact, encounters, enemy projectiles and Xray use its public
operations. **51 old files are deleted**, including 24 RAM adapters, their old rule
implementations, spawning, setup runtime and bank orchestration. Three unrelated
palette/sound routines relocate intact.

Rule families share one state/effect interface. Drawing shares tile emission and
destination calculations. These replace production paths; they are no longer an
isolated draft or an additional set of adapters.

## Verification

- Release library, frontend and verifier builds pass.
- **1,015 frozen RAM/return/native-trace outputs match byte for byte.** Two native
  diagnostics and one bounded nonreturn also match their expected outcomes.
- Ownership checks cover byte aliases, foreign-state adoption, callback unwind
  and partial versus atomic failures. Qualified-command checks cover six replaced
  algorithms and two player-call wrappers.
- **28,314 oracle frames pass with zero failures**, covering 12 rooms and nine
  enemy definitions. Required probes exercise all three spawn sources, terrain,
  drawing, gameplay and message completion, followed by 120 frames.
- All **106 projectile regression groups** pass. Independent static review passes.

This is not exhaustive branch or boot-to-credits coverage. Two retained native
pose intervals have undefined accesses in the old Rust implementation; helper
checks and static dispatch review do not establish defined native equivalence.
The nonreturn check observes a timeout, not the preceding RAM write dynamically.

## Complete cost

The current source has **108,682 Rust lines** and **2,732 unsafe declarations**.

| Room-mechanism work | Lines | Tokens | LF bytes |
| --- | ---: | ---: | ---: |
| Previously committed baseline preparation | +926 | +8,008 | +32,994 |
| Complete integration and additional checks | -1,154 | -12,222 | -39,458 |
| **Entire mechanism effort** | **-228** | **-4,214** | **-6,464** |

The full reduction is modest. Counting only the integration would hide the cost
of preparing its reference checks. No file moves or formatting compression are
claimed as savings. No new maintained native/tooling code or dependencies were
added. Reference captures remain local; no ROM-derived fixtures are committed.

The earlier projectile consolidation retired 14 adapters but added 1,389 main
lines: 596 runtime/probe lines and 793 verification lines. It established clearer
ownership at a real size cost. The separate room-loading overlap correction adds
89 lines and is excluded from the mechanism savings above.

## How this reaches the engine

Projectiles and room mechanisms now own their state **during operations**. Foreign
calls receive published state, and the subsystem imports their returned state.
The emulated runtime still owns memory between operations. This is an intermediate
architecture, not the finished owned engine.

The next step is to close external writers, reset/load, whole-memory replacement
and shared suspension interactions so state can remain owned between calls.
Gameplay can then communicate with rendering and audio through explicit outputs.
Independent rendering/audio, removal of all unsafe operations, the below-100,000
line goal and boot-to-credits fidelity remain unfinished.

Each delivery must identify its public operations, owner, migrated callers,
deleted implementations and complete cost. Adding safe accessors alone does not
meet that standard.

See [PLAN](docs/PLAN.md), [HANDOFF](docs/HANDOFF.md), and
[the accepted architecture](docs/target-architecture/README.md).
