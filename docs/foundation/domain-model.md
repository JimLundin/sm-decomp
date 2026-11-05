# Foundation domain model

Research lane `research/foundation-domains`, based on `origin/main` at `e3cf770`,
2026-11-05. This report changes no engine code. It answers the foundation step of
Jim's rebuild decision (REVIEW, "Rebuild on a new foundation", on `land/rebuild`):
what the game's content needs from the foundation's domain model, and how small the
rebuilt engine can be when that content is typed data run by a few interpreters.

Two inputs:

- **The current engine**, used as a map of what exists and how big it is. Lines are
  physical lines in `crates/sm_game/src` (157,123 non-blank by `sm_pipeline volume`,
  160,234 physical by `wc -l`), split into domains by file name. The host glue in
  `crates/sm_host/src/game` (14,345 lines) is frozen reference and is listed only
  where it adds to a domain.
- **The bank logs** (patrickjohnston.org/bank), for the original's design. I read
  banks $84, $86, $88, $8D, $8F, $91, $A0 and $B4. Facts marked *(confirm)* come from
  my knowledge of the game rather than a page I read, or a page was cut off before
  the routine body. Each one needs a bank-log or oracle check before the foundation
  depends on it.

## The main finding

The original game is mostly a set of **slot tables, each with a bytecode
interpreter**. Eight families run the same loop: a fixed array of slots, a spawn
routine that copies a header into a free slot, and a per-frame handler. The handler
runs an optional *pre-instruction* (a per-frame callback), counts down an
instruction timer, and at zero reads the next list entry. An entry with bit 15 clear
is a `(duration, payload)` pair, and an entry with bit 15 set is the address of an
assembly instruction handler with inline arguments.

| Family | Bank | Slots | Spawn search | Processing order | Payload of a timed entry |
| --- | --- | ---: | --- | --- | --- |
| Enemies | $A0 (+ the shared $xx:8000 block in each AI bank) | 32 | room population ascending from slot 0; runtime spawn *(confirm)* | active list built at frame start, in slot order | spritemap |
| Enemy projectiles | $86 | 18 | from slot 17 down; full returns failure | slot 17 down to 0, live | spritemap |
| PLMs (room objects) | $84 | 40 | from slot 39 down | slot 39 down *(confirm)* | block draw instruction |
| Sprite objects | $B4 | 32 *(confirm)* | *(confirm)* | *(confirm)* | spritemap |
| HDMA objects | $88 | 6 (channels 2-7, two slots pinned) | slot 0 up; full returns failure | slot 0 up | HDMA table |
| Palette FX objects | $8D | 8 *(confirm)* | *(confirm)* | *(confirm)* | colour run |
| Animated tile objects | $87 | 6 *(confirm)* | *(confirm)* | *(confirm)* | VRAM tile source |
| Cinematic objects (intro, Ceres, ending, credits) | $8B | several tables *(confirm)* | *(confirm)* | *(confirm)* | spritemap, mode 7 frame or BG3 text |

Four more consumers have the same shape on one cursor: Samus's animation delay
lists ($91:B010, with control bytes at $F6 and up), the demo input lists ($91, a
timer word then held and new-press words, with bit-15 commands), Samus projectile
lists ($93), and the cinematic and credits scripts in $8B.

Every family shares a common instruction core: delete, sleep, go to, call function,
set or clear the pre-instruction, set timer, decrement timer and branch if non-zero,
and queue sound. The rest are family- or species-specific: PLM `set event`, `draw
block`, `activate station`; projectile `aim toward Samus`, `random movement within
radius`; HDMA `set table bank`. The enemies' species-specific instructions live in
each AI bank next to the AI. Today's engine keeps this shape but reads the lists
from ROM bytes at run time (`crates/sm_game/src/enemy_script.rs`) and dispatches the
handlers by ROM address. `actors/species.rs` alone has 426 address arms, and
`sm_game` has 1,309 in all.

The rest is ordinary state-machine code. Samus's movement types (28, dispatched from
$91:8014), the game states in $82, the menus, and each enemy's AI are all
*function-pointer* state machines: the current routine's address is the state word.
Data tables sit beside them: Samus's pose definitions and transition table, enemy
headers, room and state headers, door lists, scroll data, enemy sets, and the
vulnerability and drop tables.

**So the foundation should have one generic program interpreter, a slot-store
abstraction with per-family policy, and typed content decoded from the ROM at load
time.** Behaviour then lives in typed opcode enums and state enums, each with a
small handler.

## Domain table

"Today" is physical lines in `crates/sm_game/src`, by domain. The estimate is for the
rebuilt `sm_game` code, excluding the content decoder (estimated separately below)
and the renderer and sound engine (in `sm_runtime`).

| Domain | Today | Original's shape | Target shape | Estimate |
| --- | ---: | --- | --- | ---: |
| **Samus**: poses, movement, block collision, beams and bombs, grapple, palettes, drawing, demo input | 16.4k, plus 2.6k of Samus scene binding in `scene/` and 1.0k of host runtimes | Pose definition table (8 bytes per pose) and an input → pose transition table ($91:9EE2: new-press, held, new pose). 28 movement types each with a handler. Pose-change commands ($91:8304). Animation delay bytecode. Block collision per block type ($94). Projectile definitions with instruction lists ($93). Grapple swing arithmetic | Pose definitions, transitions and animation delays decoded into typed tables (`Pose`, `MovementType` enum, `Transition { pressed, held, to }`). One `match` over `MovementType`. Block collision as ordinary Rust over a `BlockType` enum and slope table. Projectiles and demo input on the generic interpreter. Locomotion, posture, equipment and charge kept as separate fields (ARCHITECTURE) | 9-12k |
| **Enemy species** (about 90 ordinary species files) | 18.9k | Header per species: size, health, damage, palette, tiles, AI bank, and hook pointers for init, main, grapple, hurt, frozen, x-ray, touch, shot and power bomb. Vulnerability and drop tables. Per-species instruction lists with species ASM instructions. AI as function-pointer states over 0x40 bytes of slot RAM plus extra RAM | Species record decoded from the header (typed hooks, `Vulnerability`, `Drops`). Behaviour as `enum Behaviour` over shared families (crawlers, rippers, flies, pirates, hoppers, divers), each a state machine described as data: states with entry program, motion rule, transitions on timer, contact, Samus range or wall. Species with unusual motion keep one Rust function per state, named by a variant | 8-12k |
| **Actor infrastructure**: store, scheduling, contacts, movement probes, drawing queues, population | 10.3k | $A0: determine-active pass, main loop (contact, AI, instructions, draw queue), Samus/projectile/enemy-projectile contact passes, room population loader, enemy-set graphics | One slot store generic over family policy. One contact pass per pair kind in the original's order. Movement probes as collision functions shared with Samus | 3-4k |
| **Bosses**: Kraid, Phantoon, Draygon, Ridley (both), Crocomire, Botwoon, Spore Spawn, the Torizos, Mother Brain, the baby Metroid, plus statues, gunship, zebetites and corpses | 28.6k (Mother Brain fight 3.7k, Ridley 3.5k, Kraid 2.3k) | Function-pointer phase machines over several consecutive enemy slots (body and parts addressed by fixed slot offsets). Bespoke motion (swoops, neck chains, figure-eights from tables), many projectile and palette-FX spawns, scripted death sequences | Each boss is a typed phase machine described as data: `Phase { enter: [Action], motion: Motion, exits: [Exit] }`, plus a shared combinator set (wait, flash, shake, spawn volley, death explosions, drop items, set boss bit). Parts are ordered typed slot references inside the boss's group. Bespoke motion stays as Rust functions per motion variant. The structure shrinks; the motion maths does not | 10-15k |
| **Enemy projectiles** (about 150-200 headers *(confirm count)*, boss attacks, debris, drops) | 7.8k | Five-word header: init AI, pre-instruction, instruction list, radius, properties. Generic interpreter at $86:8125 | Typed header plus generic interpreter. A small motion vocabulary covers most pre-instructions: linear, gravity, sine, aimed, attached to parent, orbit. Boss-specific pre-instructions stay as named Rust variants | 3-4.5k |
| **PLMs / room objects**: doors, items, crumble, shot, bomb and speed blocks, scroll PLMs, stations, gates, boss walls | 4.7k, plus 0.8k host | Header: setup routine and instruction list. Interpreter with block-draw entries and many ASM instructions (set event, draw block, link and return, item GFX, station actions). Setup routines spawn by block BTS | Typed PLM definitions with `enum PlmOp`, block draws decoded into typed tile runs, setup routines as an enum. One interpreter (the generic one with link and return) | 2.5-3.5k |
| **Sprite objects** (about 62 lists: explosions, dust, smoke, particles) | 1.4k | Spawn, handle, draw; four instructions ($B4:BCF0-$BD12) | Pure data on the generic interpreter | 0.3-0.6k |
| **Visual effects**: HDMA objects, FX types (liquids, rain, fog, spores, haze, sky), power bomb, x-ray, crystal flash, palette FX (about 60 objects), animated tiles, layer blending (27 handlers) | 9.6k, plus 1.3k of `scene/hdma_sources.rs` | HDMA objects on the bytecode interpreter, with pre-instructions computing per-scanline tables. Palette FX and animated tiles on their own interpreters. Layer-blending configurations from a table | Effects produce typed Scene parameters (scanline curves, windows, colour-math settings), never register tables. HDMA, palette FX and animated tiles all run on the generic interpreter with family opcodes. The scanline maths (water wave, heat, power-bomb ellipse, x-ray cone) stays ordinary Rust. Layer blending becomes a table of typed records | 4-6k |
| **Rooms**: level data, scrolls, doors and the door transition, camera, room setup, main and door code, enemy population | 7.4k | Room header, state conditions (event, boss bit, item checks), state header (level data, tileset, music, FX, population, enemy set, scrolls, x-ray blocks, main and setup ASM, PLM population, library BG). Door definitions with door ASM, about 90 routines, mostly scroll changes. About 35 setup and 20 main routines: scroll, shake, sky, explosions, treadmill, elevatube | Typed `Room`, `RoomState`, `StateCondition` enum, `Door` with `DoorAction` (set scroll regions, start treadmill, ...) decoded by the importer. Most setup and door code becomes data. About 15 bespoke main routines stay as a `RoomMain` enum with Rust arms. The door transition is one explicit state machine | 3-4.5k |
| **Progression**: items, events, boss bits, map exploration, save file, HUD, escape timer, music requests | 2.6k | Bit arrays and counters in WRAM and SRAM. The HUD is drawn into BG3 | Typed flag sets (`EventSet`, `BossSet` per area, `ItemSet`), map exploration bitmap, a versioned Player save | 1.5-2k |
| **Menus, messages, cinematics, credits**: file select, options, pause map and equipment, game over, message boxes, intro, Ceres, title, ending, credits | 18.7k | Game-state machine in $82 (about 44 states). Menus as state machines in $81 and $82. Message boxes in $85. Intro, ending and credits in $8B on cinematic object interpreters, mode 7 objects and BG3 text, timed by instruction lists | Cinematics and credits as authored sequences on the generic interpreter (`ARCHITECTURE` "sequences"). Menus as ordinary typed state machines over a typed menu layout. Text and tilemaps as data | 7-10k |
| **Spine**: Session state, Tick phases, game mode, RNG, Scene extraction, checkpoint, actor store core, interpreter core | 33.7k, mostly compatibility: RAM encoding, six checkpoint codecs, leases, address dispatch, display registers | (Hardware NMI, DMA queues, the $82 main loop) | Session as typed domain state. One checkpoint codec derived from the state types. Scene extraction per domain. The generic interpreter and slot store live here | 6-10k |
| **Total `sm_game`** | **160.2k** | | | **57-84k** |

The **content decoder** (ROM → typed content, in `sm_data` or `sm_tools`) is extra.
Each opcode needs one row: its address, its argument layout and its typed variant.
I estimate about 400 distinct species instructions, about 60 projectile, 100 PLM, 40
HDMA, 20 palette-FX and 30 cinematic instructions, plus the header and room
decoders. That is **4-8k lines**, mostly tables. Under REVIEW's #231 ruling it
counts as a saving only where it replaces engine logic with data, which is the case
here.

### Why these ranges, and where they could be wrong

- **What shrinks is plumbing, not rules.** About 34k of today's lines are spine and
  compatibility (RAM encoding, leases, six checkpoint codecs, address dispatch) that
  a Session-first foundation does not need. Another large share is repeated
  per-species and per-family copies of interpreter steps, slot access (`X::of`,
  `edit_actor`, variable lanes) and `as`-cast arithmetic (#251 counts 4,660 casts
  and 1,800 inline bit tests). The game's rules, such as Samus's physics and boss
  motion, stay as code. That is why Samus and the bosses keep the largest shares.
- **The low end assumes** that about two thirds of ordinary species fit a dozen
  shared families on a small motion and transition vocabulary, and that the
  bosses' phase structure moves into data while their motion stays Rust. **The high
  end assumes** most species keep bespoke per-state functions, and that boss
  combinators save only the common set (death sequence, flashing, volleys, drops).
- **Data written as `.rs` counts.** Phase tables and transition lists are lines.
  Tables the ROM provides are decoded at load time (REVIEW decision 6) and cost
  nothing in source. Authored behaviour data (boss phases, species machine
  definitions) is source and is included in the estimates.
- **The estimate is not a floor.** It is what the structure below gives before any
  per-domain simplification found while writing the code.

## Cross-cutting decisions the foundation must make

### 1. One program interpreter, generic over a family

```rust
pub enum Step<F: Family> {
    Show { frames: Frames, payload: F::Payload },
    Core(CoreOp),        // delete, sleep, goto, call, set/clear pre-op, timer, branch, sound
    Family(F::Op),       // e.g. PlmOp::SetEvent(Event), ProjectileOp::AimAtSamus
}
pub struct Program<F: Family> { steps: Box<[Step<F>]>, labels: Box<[Label]> }
pub struct Cursor { program: ProgramId, at: StepIndex, timer: Frames }
```

- One `run(cursor, slot, family_ctx)` loop with the original's semantics: the
  pre-op first, then the timer countdown, then steps until a `Show`, `Sleep` or
  `Delete`. A family op returns `Continue`, `Yield` or `Delete`.
- Jumps are decoded to `StepIndex`, not addresses. **Every address any code
  compares against or jumps into becomes a `Label`**. AI often tests "is my
  instruction list X", and lists are entered mid-way. The decoder fails on an
  address that is not a step start unless that address is recorded as a quirk.
- The pre-op is part of the cursor's family state (`F::PreOp` enum), not a function
  pointer.
- Link and return (PLMs) is one extra cursor word, not a general call stack.
- Inline argument layout belongs to the decoder. The engine never reads bytes.
- Map Rando and custom content produce the same `Program<F>` values.

### 2. A slot store with per-family policy

The original's slot layout is observable, so the store models it rather than
hiding it:

- **Fixed capacity per family** (32, 18, 40, 32, 6, ...). A full table makes a
  spawn fail, which is a defined result (CONTRACTS distinguishes an original limit
  from an engine limit). An eighteenth enemy projectile not appearing is gameplay.
- **Spawn search order per family, as data**: ascending, descending, or a pinned
  slot (HDMA channels 6 and 7).
- **Processing order per family**: ascending (enemies, HDMA) or descending
  (projectiles, PLMs). Iteration must be a *live* slot loop, not a snapshot. A
  projectile spawned during the descending pass into a lower slot runs in the same
  frame, while one spawned into a higher slot waits a frame. Enemies are the
  exception: their active list is built once per frame ($A0:8EB6), so an enemy
  spawned mid-frame waits.
- **Deferred deletion where the original defers it.** An enemy's delete sets a
  property bit, and the slot is freed by the next frame's determine pass. A
  projectile's delete frees at once. The store needs a `Deleting` slot state and a
  per-family rule.
- **The slot index is part of behaviour.** It is the processing order and the
  contact-order tie-break, and bosses use slot arithmetic for their parts. So
  `SlotIndex<F>` is a newtype the content can see. It is separate from the handle
  generation the architecture requires, but the two must agree (CONTRACTS: "order
  keys may encode a reference slot priority").
- **Common fields plus a typed variant.** Position, radius, health, properties
  flags, palette, layer, cursor and timers are common to every enemy and are read
  by the shared contact and draw code. Species state is `enum SpeciesState`, with
  one variant per behaviour family. This replaces today's numbered variable lanes.
- **Groups for multipart actors.** A boss occupies consecutive slots with typed part
  roles (`KraidPart::{Body, Arm, TopLip, ...}`). Cross-part access resolves inside
  the group with disjoint borrows, matching ARCHITECTURE's "ordered part
  references".

### 3. Typed content, decoded once

- Decode at load time from the ROM (REVIEW decision 6) into validated `sm_data`
  records: species, programs per family, PLM and projectile headers, poses and
  transitions, rooms, states, doors, scroll data, enemy sets, vulnerabilities and
  drops.
- Raw values cross into enums once, with `TryFrom`. An invalid value the original
  tolerates (a bad function pointer, an out-of-range pose) becomes a named
  `Unknown(u16)` variant listed in docs/LEGACY.md. One the original treats as an
  error becomes `Fatal::unreachable`.
- **Verification needs the reverse map.** The oracle RAM → Session decoder must
  turn an instruction pointer at `$A3:C69C+n` into `(ProgramId, StepIndex)`, and
  an AI state address into a state enum. The decoder emits this address map as a
  side output for `sm_tools` verification. The engine never holds it. The map may
  be many-to-one (#251).

### 4. State machines as data, with Rust where the rule is maths

One pattern serves the species families, the bosses, the menus and the door
transition:

```rust
pub struct Machine<S, A, M, X> { states: EnumMap<S, State<A, M, X>> }
pub struct State<A, M, X> { enter: Box<[A]>, motion: M, exits: Box<[Exit<X, S>]> }
```

`A` (actions: set program, spawn, sound, flash, set flag), `M` (motion rules) and
`X` (exit conditions: timer, contact, Samus range, wall, health, part dead) are
per-domain enums with Rust handlers. The machine is a small interpreter. Bespoke
behaviour is a `Motion::Custom(fn)` or one named variant. It is never a second way
to write states.

### 5. Ordering guarantees the Tick must name

From $A0's frame notes, inside main gameplay: determine active enemies → Samus and
projectile vs enemy contact → enemy main loop (contact, AI, instructions, draw
queue) → Samus vs solid enemies → enemy projectile vs Samus → enemy projectile vs
Samus projectile → power bomb vs enemy → draw → shake. The places of Samus's update,
the PLM handler, the projectile handler, HDMA objects (with the music queue and
pending power-bomb spawn at their start), palette FX and room main code in the $82
main gameplay routine must be recorded *(confirm)* and become named Tick phases.
CONTRACTS' stage table is a starting design and must take this order exactly.

## Open questions for Jim

1. **Stale references.** ARCHITECTURE requires that no actor reference survive a
   despawn without a generation check. The original keeps raw slot indices: a
   projectile's parent enemy, a PLM's linked enemy, a boss's part slots. When the
   slot is reused, the reader sees the new occupant. In the faithful profile, may
   such references be unchecked `SlotRef<F>` with the original's semantics, with
   generation-checked handles used only where the original's behaviour does not
   depend on reuse?
2. **Leftover slot state.** If a species reads a variable its init does not clear,
   it sees the previous occupant's value *(confirm whether any species does)*. A
   typed variant cannot express that directly. Do we carry a per-slot raw residue
   for those cases, listed in LEGACY, or treat them as divergences from hardware to
   fix?
3. **Where the content decoder lives.** Is it `sm_data` at load time (decision 6
   says "read from the ROM at load time"), or `sm_tools` producing a prepared pack
   that `sm_data` validates? This choice decides whether the 4-8k of opcode tables
   counts as engine lines.
4. **Map Rando patches ASM.** Map Rando changes room setup, door and PLM code, not
   only data. Which of its patches must the typed vocabularies (`DoorAction`,
   `RoomMain`, `PlmOp`) cover at the first Map Rando gate? Every patched routine
   becomes either a vocabulary entry or a named gameplay option.
5. **How far the bosses go into data.** Is the target the phase machine plus shared
   combinators, with motion in Rust (10-15k)? Or should motion paths also become
   authored curves (smaller, but each path must be proven frame-exact against the
   oracle)?
6. **The scope of the generic interpreter.** Should Samus's animation delay lists
   and the demo inputs, whose encodings differ (byte control codes), also run on the
   shared `Program<F>`? The recommendation is yes: decode them into the same step
   form.

## Next checks before the foundation design is final

- Confirm from the bank logs the rows marked *(confirm)*: the runtime enemy spawn
  search ($A0:9275, $A0:92DB), the PLM processing direction ($84:85DA), and the
  sprite object, palette FX and animated tile slot counts and orders ($B4:BC26,
  $8D:C4E9, $87). Also confirm the main-gameplay phase order in $82.
- Count the headers from the ROM in CI: species, projectile headers, PLM headers,
  rooms, doors, and distinct instruction handler addresses per family. Those counts
  size the decoder tables and the opcode enums.
- Prove the slot store and interpreter on the representative encounter
  (ARCHITECTURE "Evidence before expansion"). A suitable case is a room with an
  enemy firing projectiles that fill the 18-slot table, a PLM door, and a
  multipart boss (Spore Spawn or Kraid), each compared with the oracle in Session
  space.
