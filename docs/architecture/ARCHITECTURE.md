# The engine's architecture

This document decides the engine's architecture. Its principles are in
[PRINCIPLES.md](PRINCIPLES.md) (P1-P20 for design, K1-K10 for code), and each decision cites
the ones it follows. [MODEL.md](MODEL.md) sets the shape of the game code (the layers `data`,
`game::model`, `game::intent`, `game::exact`, `game::tick`) and wins where the two disagree; its
amendments are applied here. The oracle and the lock are in [ORACLE.md](../ORACLE.md).

## 1. Summary

1. **Ordinary Rust over typed state:** no 65816 emulation, no bytecode VM, no RAM image.
2. **The Tick is one video frame in hardware order**, ending at the **commit cut**, the return
   from its NMI, a point every oracle has. Lag is the NMI's kind, and **NMI timing is a Tick
   input**: recorded from the oracle today, from a native cost model later.
3. **CPU time is modelled only where the frame count depends on it**, as named costs.
4. **The State is plain data with one authority.** Video memory is a SNES-layout image, because
   the game reads it back and shows stale contents.
5. **The model emits intents; one encoder (`game::exact`) makes the exact bytes** the oracle compares.
6. **One slot store, one instruction-list interpreter, one continuation stack.**
7. **Content is decoded from the ROM at load time**, with dense IDs inside and stable keys outside.
8. **Options are profiles resolved at open**, each field with one reader.
9. **The comparison is one-way:** raw oracle records, one decoder, one projection per stream; a row
   is green when every compared stream matches at the quiescent cuts and every fact holds.
10. **Verified speed is a requirement:** at least 1,000 Ticks/s with every stream on one CI core.

## 2. What the engine is

- **The game logic is ordinary Rust**: Samus, collision, enemy AI, bosses, doors, menus and the HUD
  are functions over typed structs. The gameplay **phases run in the original's order** (§6.3).
  Inside a domain, structure follows the model (MODEL §2), not the original's routines: an order
  that can be observed at a cut is a named behaviour; any other order is free.
- **The continuation stack is a bookmark, not a VM.** The original waits for the next frame from
  deep inside routines: an item's message box opens inside the PLM handler, inside the gameplay
  pass, which resumes about 370 frames later. The engine records where it stopped as typed values,
  `[MainLoop, Gameplay(Plms), Plms { slot: 3 }, MessageBox(Shown)]`, saved in the Checkpoint. `async`
  can't be saved, and re-running a pass to skip finished work would repeat side effects and RNG calls.
- **The interpreter walks data that is already data in the ROM:** the instruction lists of PLMs,
  enemies, sprite objects, HDMA objects, palette effects and animated tiles, decoded at load into
  typed steps and walked by one loop. Behaviour inside a step is Rust.
- **The emulator exists only in the oracle crate.**

## 3. Decisions at a glance

| # | Decision | Principles | Section |
| --- | --- | --- | --- |
| D1 | Crates with plain names | P3, P5, P15 | 4 |
| D2 | Two public engine traits (`Family`, `Canonical`) and concrete context types | K3, P16 | 4.3 |
| D3 | State is a public plain-data schema with private behaviour; one authority | P5, P6, P8 | 5.1 |
| D4 | `RoomScoped` is one struct, so the door's reset list is a type | P8, K2 | 5.1 |
| D5 | Video memory is a SNES-layout image, justified by readbacks and stale display | P1, P6 | 5.3 |
| D6 | The Tick is one video frame in hardware order, from cut to cut | P4, P9 | 6.1 |
| D7 | A named beam-sync point in the main strand places the scan-out | P4, P9 | 6.2 |
| D8 | NMI timing (`OnTime` or `Overrun`) is a Tick input from a timing source | P4, P17 | 6.4 |
| D9 | CPU time is a named cost model of the routines whose frame count depends on it | P2, P9 | 6.5 |
| D10 | Main strand: a bounded stack of typed continuation frames; interrupt strand: a typed command chain | P9, K2 | 6.6 |
| D11 | The line program places changes at (line, dot); the dots are oracle rules | P4 | 6.7 |
| D12 | Slot store with per-family policy | P7, P9 | 7.1 |
| D13 | One `Program<F>` interpreter for every ROM instruction list | P7, P16, K3 | 7.2 |
| D14 | AI concrete per species until two share a shape; bosses as data-driven phase tables | P16, P15 | 7.3 |
| D15 | Content decoded at load; dense IDs inside, stable keys outside; overlays | P1, P7, K1 | 8 |
| D16 | Intents are the model's primary output; one encoder makes the exact output | P18, K4 | 9 |
| D17 | A CPU line compositor writing PPU-domain pixels; `wgpu` presents | P12, P14 | 9.3 |
| D18 | Sound: protocol and driver model in the Session; synthesis in `runtime` | P18, P4 | 9.4 |
| D19 | Profiles: Reference, Gameplay (resolved `Rules`), Timing source, Presentation, Input | P17, P4 | 10 |
| D20 | The oracle contract: one record per video frame, cut at each NMI return | P5, P12 | 11.1 |
| D21 | One decoder, one projection per stream, compared at quiescent cuts | P6, P7 | 11.2 |
| D22 | Green = every compared stream matches, plus facts; locks keyed by (row, oracle) | P10, P11, P13 | 11.4 |
| D23 | Speed targets | P14 | 12 |
| D24 | Size, trait and behaviour-catalogue budgets, reported on every landing | P16, P20 | 13 |
| D25 | Each stream's lock comes from the most accurate oracle producing it | P10, P12 | 11.4 |
| D26 | Short trick movies from seeds; full TAS movies nightly | P10, P11 | 11.4 |

## 4. Packages and ownership

### 4.1 Crates

| Crate | Owns | Must not |
| --- | --- | --- |
| `data` | IDs and stable keys; the ROM decoder; typed content and world records; profiles and the rule tables (the vanilla value of every behaviour parameter); the portable records (display frame, video memory image, sound frame, intents); the address map; the digest and span fold | hold mutable state or game State |
| `game` | `Session`, `State`, the Tick, strands, slot store, interpreter, every gameplay domain, the behaviour catalogue, the sound protocol, the cost model, SRAM. Inside it MODEL §2's layers: `model` decides, `intent` carries the per-Tick intents, `exact` encodes them, `tick` orders. Producers never write video State; only `exact` does | touch pixels, samples, files, devices or addresses (K1) |
| `runtime` | the line compositor, `wgpu` presentation, the wide renderer, sound backends, storage | feed anything back into the Session |
| `app` | the run coordinator, Tick admission, the input profile, effect delivery; binary `super-metroid` | contain gameplay |
| `oracle` | emulator adapters, the `Oracle` trait, seeds, the lock recorder | depend on `game` or `verify` |
| `oracle_record` | `FrameRecord` and its codec (`oracle::record`), so `verify` reads records without an emulator | link an emulator |
| `verify` | the record → State decoder, projections, facts, digests, free-run and lockstep comparison | be depended on by an engine crate |
| `tools` | the command line and the importers | hold comparison logic of its own |
| `clean_types`, `clean_model/<domain>` | the clean room's crates (MODEL §5): the shared domain types and the `World` trait; each rebuilt domain's model and catalogue | depend on anything but each other, `std` and the registry crates of `tools/cleanroom/allowed_deps.txt`; hold an address, a bank reference or the original's names |
| `canonical`, `canonical_derive` | the `Canonical` trait (codec, diff, schema hash) and its derive over `proc_macro` alone (no `syn` or `quote`, whose build scripts Defender blocks); separate so `data` can derive it | hold State or rules |

**D1.** The names say what each crate is. `verify` is apart from `tools` because the decoder and
projections are the riskiest verification code: they get their own mutants, and the CLI can't grow a
second comparison path (P7). [crates/README.md](../../crates/README.md) lists the allowed workspace dependencies.

### 4.2 Dependency rules (checked in CI)

`tools/check_crate_rules.sh` checks, in Quality and `tools/lane/gate.sh`:

1. No engine crate (`data`, `game`, `runtime`, `app`) depends on `oracle`, `verify` or `tools`.
2. `game` and `runtime` don't depend on each other; `oracle` depends only on `data` and `oracle_record`.
3. Engine crates hold no `cfg(verify)`, probe or observer (P6), and no SNES address literal in
   `game`, `runtime` or `app` (K1).
4. No `#![feature]`, `allow` or `expect` attribute, and no routine-label names, in the engine (K8, K9).
5. MODEL §2's "never names" column: held by review until the reshape, then checked here.

### 4.3 The trait budget

**D2.** A new public trait is a design change to this table (K3, P16).

| Trait | Crate | Implementations | Why a trait |
| --- | --- | --- | --- |
| `Family` | `data` | `Plm`, `Enemy`, `SamusAnimation`; later projectiles, sprite objects, HDMA objects, palette FX, animated tiles, demo input, cinematics | the interpreter is generic over payload, op and pre-op |
| `Canonical` | `canonical` | every State type and the `data` types in it | one derived codec (K6) |

Outside the engine: `Oracle`, and `SoundBackend` and `MusicSource` in `runtime`. A family's ops take
a **concrete context** of disjoint borrows, e.g. `PlmContext<'a> { level: &'a mut Level, plms: &'a
mut PlmSlots, sound: &'a mut SoundCpu, .. }`, built by the pass that runs it: no host object.

## 5. State

### 5.1 The State schema

**D3.** The State is the whole Session as public plain data. It is the Checkpoint's schema, the
decoder's output and the projections' input; mutation and rules are private to `game`. Opening a
Session from a State validates its invariants.

| Domain | Key types | Across a door |
| --- | --- | --- |
| clock | `Tick`, `Counters { completed8, completed16, nmi, lag, max_lag }`, in-game time | carried |
| rng | `Rng(u16)` with a call counter | carried |
| controller | `Controller { auto_read, held, pressed, repeat, previous, repeat timers }` | carried |
| mode, strands, timing | `GameMode` (each state with its data); `MainStrand`, `IrqStrand`; the NMI timing and cost accumulator | carried |
| video | `Video { memory: VideoMemory, committed: Registers, batch: NmiBatch, hdma, nmi_requested }` | carried |
| sound | `SoundCpu { music_queue, libraries: [LibraryHandshake; 3], downtime }`, the driver model (D18) | carried |
| samus | locomotion, posture, equipment, weapon and charge, speed and blue state, damage, grapple, reserves, drawing state | carried |
| room | `RoomScoped`: room and state, blocks and BTS, scrolls, camera, FX, actors, visual effects, entered-by `DoorwayId` | reset |
| progress | items, events, bosses, explored map, opened locks by `LockId`, restart location | carried |
| menus, sram | pause, file select, game over, message box, HUD; the 8 KiB save image | per mode; persistent |

There is no phase field: Checkpoints and seeds sit at cuts, so no half-done Tick is stored.

- **Every field has an engine reader.** A field exists because a rule reads it, never because the
  oracle has a byte there. Hardware leftovers no rule reads, such as DMA registers after a transfer,
  are not State and not compared (§11.6). The decoder's coverage test runs both ways.
- **Values are typed** (AGENTS.md). The original's numbers appear only at the edges: the decoder,
  the projections, the SRAM codec and the encoders in `exact`.

**D4.** Everything a room transition resets is one struct, `RoomScoped`, so replacing it is the reset.
What a transition keeps on purpose (surviving projectiles, X-ray, G-mode leftovers) is a named field in
its keep table.

### 5.2 The slot store and leftover state

`SlotStore<F, C, B, const N>`: `N` slots of common fields `C` (position, radius, health, properties,
palette, layer, cursor) and a body in `Occupancy::{Free, Live(B), Deleting(B)}`. The `SlotPolicy` value
gives spawn order (ascending, descending, or pinned; a full table is `SpawnResult::Full`, gameplay),
run order (`LiveAscending`, `LiveDescending`, `ListedAtPassStart`; a live pass re-reads occupancy, so a
slot filled ahead of the cursor runs this pass) and deletion (`Immediate` or `NextPass`). Raw slot
references in the original are `SlotRef<F>` and follow reuse; `Handle<F>` is generation-checked where
behaviour must not. Multipart bosses are consecutive slots with typed part roles. The store never
clears common fields, so a field an init forgets keeps the previous occupant's value; a species that
reads an unset variable gets a named `Residue` field when the oracle shows it, listed in LEGACY.

### 5.3 Video memory

**D5.** `VideoMemory { vram: [u16; 0x8000], cgram: [u16; 0x100], oam: [u16; 0x100], oam_high: [u8;
0x20] }` is SNES-layout State, written only through a `VideoWriter` that exists at a commit point
(`CommitPoint::{Nmi, Loading, BlankLine}`) and only in `exact` and the NMI commit (K4). The reason is
behaviour: the game reads it back (the pause BG2 backup, the x-ray background), stale tiles and OAM
stay on screen, and uploads are already bytes. Only video memory a behaviour reads or shows is State
(MODEL decision 3); meaning lives in the intents. The main strand fills the **NMI batch** (OAM and
palette buffers, register mirrors, the VRAM write and read tables, Samus and animated tile transfers),
which the requested NMI applies in order; readbacks copy into their targets after the write table.

## 6. The Tick

### 6.1 What a Tick is

**D6.** A Tick is one video frame from one commit cut to the next; seeds, Checkpoints and records sit
at cuts. On the console: NMI k-1 returns and the main thread resumes; in vblank the HDMA object handler
runs (music queue, pending power bomb, HDMA objects, layer blending); the beam scans out picture k-1
(HDMA each line, the interrupt strand at its lines) while the rest of the iteration runs (RNG, OAM, the
game state dispatch, the sound handler, OAM finalisation) and yields; NMI k commits or counts lag,
auto-reads the pad and returns: cut k. The lines run concurrently with the iteration, so "the end of
the lines" is no instant; the NMI return is.

### 6.2 The beam-sync point

**D7.** Each continuation frame declares where in its step the scan-out runs: after the HDMA object
handler for the main loop, before the per-Tick work for the others unless measured otherwise. If the
strand is still waiting at the Tick's start, the scan-out runs first. So picture k-1 reads the tables
iteration k's handler wrote. Measured: the handler returns at lines 241-249 (ORACLE appendix A, (c)); a
frame where it finishes after line 0 tears, a LEGACY row until the cost model places it.

### 6.3 Order within the Tick

`Session::step(input)` runs Tick k:

1. **Resume the main strand** to its beam-sync point, unless it is lagging, overrunning or waiting on
   an `Until(c)` still false.
2. **Scan out picture k-1:** per line 0-224, apply the line's changes, run the enabled HDMA channels,
   and run the interrupt command due at its dot (D11).
3. **Finish the main strand** until it yields, charging costs (D9). One iteration: the HDMA object
   handler, one RNG call, clearing high OAM and the transfer flags, the game state dispatch, the sound
   handler, finalising OAM, the previous-input copy, `Yield(Wait::Nmi)`.
4. **NMI k** commits only if the strand yielded `Wait::Nmi`, the timing input is `OnTime` and the
   request is set: OAM and CGRAM, Samus tiles, animated tiles, register mirrors, HDMA bindings, Mode 7,
   the VRAM write table, the read table; then HDMA enable, the controller latch, `lag = 0` and the
   completed counters. Otherwise `lag += 1` and `max_lag`. Always `nmi += 1` and `auto_read = pad`.
5. **Cut k:** `TickOutput { tick, nmi, picture, sound, save: Option<SramWrite>, failure:
   Option<OriginalFailure> }`.

The input is needed only at step 4, so `begin()` runs 1-2 and returns the picture and `end(input)` runs
3-5; `begin`'s guard borrows the Session, so no Checkpoint can be taken mid-Tick (K4).

The gameplay pass's 19 phases follow `$82:8B44`: enemy selection, palette FX, Samus current state,
Samus and projectile interaction, enemies, Samus new state, enemy projectiles, PLMs, animated tiles,
collisions, power bomb, scrolling, drawing, HUD, BG scroll, room main code, health and time, room
shaking, hurt timers. In the enemy phase the `$A0` order holds: determine active, contacts, AI,
instructions and draw queue, solid-enemy contact, projectile and power bomb contact, draw and shake.
Inside a phase, the model decides the structure (MODEL §1).

### 6.4 NMI timing and lag

**D8.** **Structural lag** comes from the strands: a `Wait::LagFrame`, an `Until(c)` still false, or a
frame count in a cursor (`$85:8574`'s 2, `$85:8493`'s 360 or 10), never a constant elsewhere.
**Overrun lag** is an NMI arriving while the strand still works: the Tick's `NmiTiming::{OnTime,
Overrun}` input, from a timing source.

| Source | `NmiTiming` from | Used by |
| --- | --- | --- |
| `Recorded` | the lock row's `overruns` and `nmi_off` | the gate: the oracle's lag exactly |
| `Modeled` | the cost model (D9) against the frame budget | live play and anything with no oracle |
| `None` | always `OnTime` | the "lag off" option, which records that lag tricks break (P20) |

An overrun needs no special path: the strand yields in Tick k, NMI k lags because it is `Overrun`, the
request stays pending, the strand doesn't run in Tick k+1, and NMI k+1 commits: one iteration over two
frames, no second RNG call. At an overrun cut the oracle is mid-routine, so typed state isn't compared
(D21). The `timing` stream checks the model on its own, so a model error never hides or fakes a
gameplay divergence. Jim decided (2026-11-05) that vanilla play reproduces the console's load times
and busy-room lag through `Modeled`, so every TAS trick can be executed.

### 6.5 Named CPU costs

**D9.** On a real-beam oracle the lag count of some busy routines depends on CPU time, which varies
with the data: decompression in loading, the APU upload handshake, DMA under forced blank. So a door's
length depends on the rooms. The harness measures each busy span first (interrupted-PC histogram,
decompression sizes, DMA and upload bytes): if the count follows the named work, build a cost table;
if constant per transition kind, a count in `Rules`; if unpredictable, a recorded per-(room, state,
door) table for vanilla and the nearest estimate for new content, listed as a limit. Bounds: a routine
gets a cost only if NMIs land in it; a cost is per unit of input (command, byte), never per
instruction; about 10-30 rows, no CPU registers. More than that is a review, not a bigger model.

### 6.6 Strands and continuations

**D10.** `MainStrand { frames: Vec<Frame> (at most 8), waiting: Option<Wait> }` over a closed `Frame`
enum (`MainLoop`, `GameState`, `Gameplay(PassAt)`, `Plms`, `MessageBox`, `Door`, `Pause`, `Death`,
`Cinematic`, `Boot`). Each step returns `Next::{Continue, Call(Frame), Return, Yield(Wait)}` and each
variant declares its beam-sync point. A routine that can't span a Tick is an ordinary function.

| `Wait` | Original | Tick effect |
| --- | --- | --- |
| `Nmi` | `$80:8338` (sets `$05B4`) | requested NMI: the batch commits, the pad latches |
| `LagFrame` | `$85:8136` (waits on `$05B8`) | lag NMI |
| `Until(Condition)` | wait-then-test loops (`$80:D004`, door busy-waits, music queue) | test at the next resume |

`Condition`: `DoorScrollDone`, `DoorVramUpdated`, `MusicQueueEmpty`, `ApuReady`. The **interrupt
strand** is the `$AB` chain as `IrqCommand` (the HUD split and restore, the door-start HUD, Draygon's
HUD, the door commands); each handler returns the next command and line, and during doors it moves
Samus and the scrolls, runs the forced-blank VRAM update and grants the NMI request. The ROM is
unpatched, so the message box runs where the ROM calls it (`$85:8080`, inside the PLM instruction).

### 6.7 The line program

**D11.** The Session evaluates HDMA and the interrupt strand; the display frame carries typed changes
(scroll, mode, window, screens, colour math, forced blank, layer bases) at `LinePoint { line, dot }`.
HDMA writes land at dot 0. An IRQ handler's write lands at a dot set by its path: a table in
`OracleRules`, measured per line (the last third of the line). The compositor splits a line only where
a change has a dot. A lag Tick shows the latest HDMA tables over the previous registers and OAM.

### 6.8 Input, RNG and failures

`TickInput { pad, timing, commands }`, `pad` sampled at this Tick's vblank. A requested NMI latches it
(`held`, `pressed = pad & !previous`, repeats) for the next Tick; every NMI sets `auto_read`, which
direct reads use, so a press inside lag Ticks never reaches the game. Session commands are recorded
inputs; lag is not an input. `Rng(u16)` advances `r * 5 + 0x111` with the original's carry, once per
iteration and per consumer, never on a lag Tick. Original crashes are a typed `OriginalFailure`
(`SpringBallMenu`, `YappingMawSpark`, ...) returned at the point the oracle shows; engine faults are
`SessionFault`; original limits (a full table, a dropped sound) are gameplay.

## 7. Actors, slots and the interpreter

### 7.1 The slot store

**D12.** §5.2's store, generic over common fields and body because PLMs and enemies are two users with
different policies (P16). Its policy is a value, not a trait.

### 7.2 One instruction-list interpreter

**D13.** `Program<F> { steps }` with `Step::{Show { frames, payload }, Core(CoreOp), Family(F::Op)}`;
`Cursor<F> { at: Label<F>, delay, link, pre_op, timer }`; labels `(ProgramId<F>, StepIndex)` instead
of addresses, named through a per-family `LabelName` enum; a step bound validated at load. `CoreOp`:
delete, sleep, goto, link and return, set or clear the pre-op, set the timer, decrement and branch,
queue a sound. `run` does pre-op, timer, then steps until a `Show`, sleep or delete.

```rust
pub trait Family: Copy + 'static {
    type Payload: Copy;
    type Op: Copy;
    type PreOp: Copy + Default;
    type Context<'a>;
    fn pre_op(cx: &mut Self::Context<'_>, pre: Self::PreOp, cursor: &mut Cursor<Self>) -> Flow;
    fn show(cx: &mut Self::Context<'_>, payload: Self::Payload);
    fn op(cx: &mut Self::Context<'_>, op: Self::Op, cursor: &mut Cursor<Self>) -> Flow;
}
pub fn run<F: Family>(programs: &Programs<F>, cursor: &mut Cursor<F>, cx: &mut F::Context<'_>)
    -> Result<Lifetime, InterpretFault>;
```

Every family whose behaviour is a ROM instruction list runs it through this interpreter; hand-coding a
list as an enum of positions is a review stop.

### 7.3 AI, machines and bosses

**D14.** Species AI is an enum of states per species with handlers; a generic machine is extracted, as
an `[exact]` refactor, only when two families share its shape. Bosses are **data-driven phase tables**
(entry actions, a motion rule, exits on timer, contact, health or part death) run by a few combinators;
maths motion (swoops, neck chains) is a Rust function per variant. The table's shape comes from Spore
Spawn and Bomb Torizo designed together.

## 8. Content and IDs

**D15.** `data` decodes the ROM at load into validated typed records: species, drop tables,
`Programs<F>`, PLM and projectile headers, poses, animations, rooms and states, doorways, scroll data,
enemy sets, tilesets and media references, HDMA, palette FX and animated tile programs, N-SPC banks,
demos, cinematics. Each opcode is one table row; a raw value becomes an enum once (`TryFrom`), a
tolerated invalid one a named variant and a LEGACY line. Inside the engine IDs are dense (`RoomId`,
`DoorwayId`, `SpeciesId`); persisted data uses stable keys, `Vanilla(..)` or `Pack(pack, name)`.
Checkpoints carry the content fingerprint. Content is the ROM plus overlays (a world, custom rooms and
programs, media packs), validated after composition; new vocabulary is a new enum variant with its
handler. The `AddressMap` (ROM address to step, state, room, doorway) is for verification only (K1).

- `Room` is independent of where it sits; `Placement { room(s), map_area, map_x, map_y, .. }` puts it
  in a world. `Doorway { id, room, kind, arrival, exit side, lock }` owns its arrival data and typed
  `DoorAction`s. `Topology` connects doorways: "leave through A" is "arrive at partner(A)", and the
  area-change and CRE-reload flags come from the pair's placements.
- Items, events, bosses and locks have stable names that map to sm-json-data flags; pickup amounts are
  profile data. The vanilla world is one `World` value that must rebuild the vanilla headers exactly.
- Presentation is media references, replaceable without touching collision or actors. Loading cost
  comes from what is loaded.

## 9. Output and the presentation boundary

### 9.1 Exact output and intent output

**D16.** The model's primary per-Tick output is its **intents** (MODEL §2): `SpriteDraw`, `TileWrite`,
`LayerExposure`, `HudValues`, `PaletteWrite`, `Effect`, `SoundRequest`. **`game::exact`** is the one
encoder to the original's bytes, each encoding quirk a named rule (the OAM format and 128-entry wrap,
the tilemap row split, the write queue's capacity, the HUD's tile words, the message box). It runs
where the original queued uploads, so the batch order is unchanged. `game::view::describe` returns the
Tick's intents rather than reconstructing them, so both outputs come from one source.

| | Exact output | Intent output |
| --- | --- | --- |
| What | video memory, the display frame (uploads, registers at line 0, `LinePoint` changes), the port frame and log, SRAM writes | camera and layers in room coordinates, sprites at full `i32` precision (with margin draws on request), effect parameters, HUD values, sound requests |
| Read by | the vanilla renderer and sound backend; verification | wide and HD rendering, interpolation, replacement audio, accessibility |
| Checked | every stream | rendered at native width, it must equal the exact picture |

### 9.2 The presentation boundary

Presentation gets `&State`, the `TickOutput` and the `View`, nothing mutable (K4). It presents the
latest picture, never replays a sound for a repeated presentation, and never varies the Tick.

### 9.3 The renderer

**D17.** A CPU line compositor writes **PPU-domain pixels** (15-bit BGR after colour math, with each
line's brightness and forced blank), so `video` depends on no emulator's colour conversion
([ORACLE, "The picture compared"](../ORACLE.md#the-picture-compared)). It keeps its own video memory
mirror, applies every Tick's uploads even when not presenting (a scheduled check compares the mirror
with the State), and applies the 32-sprite and 34-tile line limits. `wgpu` scales and presents.

### 9.4 Sound

**D18.** The Session holds what the game can observe: the music queue (8 entries with delays), three
library handshakes (`LibraryState::{Idle, Writing(SoundId), Clearing { frames }, AwaitingZero, Reset}`,
16-deep queues, the original's caps, drops and lower-ID-wins), the downtime counter, and a **driver
model** of what the driver echoes on each port and when, fitted by measurement (it answers in about
half a frame). If echoes depend on sequencer state, the sequencer's control part joins the Session.
`SoundFrame { ports, log, intent }` goes out; a `SoundBackend` in `runtime` plays it (vanilla: the
N-SPC sequencer and DSP on the ROM's banks, checked sample-exact nightly).

### 9.5 Saves

SRAM is State; the save writes synchronously and the output carries an owned `SramWrite`. The corrupt
slot clear is vanilla behaviour; "never silently reset" applies to our versioned `PlayerSave` file.

## 10. Options and profiles

**D19.** A Session opens with a content set and five profiles; each field has one reader (P17).

| Profile | Owner | Fingerprint | Contents |
| --- | --- | --- | --- |
| `ReferenceProfile` | `data` | yes | NTSC cadence (236,250,000/11 Hz master, 357,366 cycles a frame); `OracleRules`: the oracle per stream, IRQ dots, beam-sync measurements, the cost table, the driver constants |
| `GameplayProfile` | `data` | yes | one value per switchable behaviour in the catalogue (MODEL §3), plus suits, door timing, movement, pickups, drops, bosses, glitch and failure policy, the `Viewport`; presets `VANILLA`, later `MAP_RANDO_BASE` |
| `TimingSource` | `app`, `tools` | in the replay | `Recorded`, `Modeled` or `None` (D8) |
| `PresentationProfile` | `runtime` | media revision | view width, palettes, Samus sheet, HD packs, shake and flash levels, sound backend |
| `InputProfile` | `app` | no | buttons and the bindings that emit Session commands |

`GameplayProfile` resolves once into `Rules`, and each module reads its own section: quick transitions
are a different `DoorTiming` table, not an `if`. Every corpus row runs on `VANILLA`; a test checks the
defaults give the vanilla digests, and a non-default option has its own scenarios with facts.

## 11. The engine ↔ oracle comparison contract

### 11.1 What the oracle provides

**D20.** One record per video frame, cut at every NMI return: the pad, the NMI's kind and interrupted
PC (an overrun when it isn't a wait site), CPU registers, raw WRAM, VRAM, CGRAM, OAM and SRAM, the
architectural registers, the APU port log, and the picture in PPU-domain BGR; tiers add per-line writes,
SPC state and audio. A seed is a record plus the emulator's state, and the engine opens only from the
record, through the decoder. Records are bit-identical on every host
([ORACLE §1](../ORACLE.md#the-frame-record-contract-v1)).

### 11.2 Decoder, projections and streams

**D21.** `verify::decode` turns a record into a `State` and a `Quiescence`, mapping the PC and stack to a
`MainStrand`: a known wait site is **quiescent**, an unknown one `DecodeError::UnknownYieldSite`, an
overrun `Quiescence::Busy`. One projection per stream reads either State, so the sides differ only
through behaviour or the decoder. Streams and folding are
[ORACLE, RELOCK §2](../ORACLE.md#streams-relock-2-streams). At a busy cut no engine state corresponds,
and comparing there would mean emulating (P9); later, each costed routine declares the domains it
writes, and the others are compared inside its span.

### 11.3 Facts

Every row has facts, typed predicates over the projected State that prove it reached its behaviour
("Morph Ball collected by Tick 900"), evaluated on the oracle when locked and on the engine every run.

### 11.4 Locks and green

**D22.** Locks are keyed by (row, oracle, oracle version) and recorded oracle-side, so they can't
follow the engine. A row is **green** when its free run under `Recorded` timing matches every compared
stream (all but `wram` and `audio`) and every fact holds. The gate: green (row, stream) pairs never
fall, and Quality passes. Reports keep reached, matches and deterministic (Checkpoint → restore →
advance) apart. Two oracles disagreeing is a dispute for a hardware capture, not an engine bug.

**D25.** The gate runs no oracle, so accuracy picks the oracle that records each stream: today lsnes
for every recorded stream, with LakeSnes as the fast oracle for lockstep. The `ReferenceProfile` names
the oracle per stream; changing one re-locks that stream.

**D26.** Short movies, one per trick or strat, start from a seed (a movie rung or another trick's end);
the full TAS movies run nightly. Strats come from [sm-json-data](https://github.com/vg-json-data/sm-json-data)
through Jim's typed translation, [sm-json-data-rs](https://github.com/JimLundin/sm-json-data-rs) (its
`sm-logic` crate at a pinned commit, loading a pack; nothing here reads the JSON). A `tools` step turns
a strat into a row skeleton with its end node and equipment as facts, door conditions first (G-mode,
shinecharged, runway, spark). A trick movie is accepted when the oracle reaches its end node with its
facts, then locked like any row.

### 11.5 Lockstep and mutants

Lockstep steps the engine one Tick from each decoded oracle cut and names the first differing field
(`Canonical::diff`); the free-run localiser names each group's first differing frame
([STATUS](../STATUS.md)). Every gate stream, and the decoder, has a typed mutant that must fail it (P11).

### 11.6 What isn't compared, by name

- **Hardware leftovers** no rule reads and no player sees, such as DMA channel registers after a
  transfer (MODEL decision 3): not State, not decoded, not compared, each named here as it leaves.
- **The five `ppu` fields lsnes lacks:** `nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel` (ORACLE Q4).
- **WRAM scratch** no decode rule reads, from the nightly unowned-RAM report: named in LEGACY, or a
  field once a behaviour reads it.
- **Overrun-frame tearing and multiplier clobber**, and **oracle-versus-hardware differences**: LEGACY.

## 12. Speed

**D23.** Verified speed is a requirement (P14).

| Mode | Target |
| --- | --- |
| free run against the lock, every stream including `video` | ≥ 1,000 Ticks/s on one CI core |
| free run, picture off | ≥ 3,000 Ticks/s |
| headless engine, no verification | ≥ 5,000 Ticks/s |
| empty Ticks, the Session's fixed cost (`cargo run --release -p game --example empty_ticks`) | ≥ 50,000 Ticks/s |
| compositor per frame on a busy screen (`cargo run --release -p runtime --example compose_bench`) | ≤ 0.3 ms, a third of the 1,000 Ticks/s budget |

Quality reports the last two. Digests fold at quiescent cuts only; VRAM's digest follows the Tick's
uploads instead of rehashing 64 KiB; the compositor's fast path takes lines with no mid-line change
and its slow path (Mode 1, BG3 at normal priority) is optimised; Checkpoints encode only on request.

## 13. Size, trait and behaviour budgets

**D24.** Every landing reports its size with `tools size` (weighted logical tokens, with the data
and the declarations reported beside them, MODEL §1 decision 5), per crate and per MODEL layer so
the routine-port share is visible and falls; the trait budget (§4.3); and the behaviour catalogue
(MODEL §3, §6): behaviours catalogued, proven exact and switchable (P20). A rebuilt domain's budget
comes from its spec (MODEL §1 decision 7). The budget is an indicator; a domain over the old code
it replaces stops for a design review.

## 14. Checkpoints and starts

`Checkpoint { schema: SchemaVersion, fingerprint: Fingerprint, state: State }`, encoded canonically
(declared field order, fixed-width little-endian integers, enums by index, no unordered maps) by the
derived `Canonical`, whose `diff` reports the first differing field path.

```rust
pub enum Start {
    Boot,
    Checkpoint(Box<Checkpoint>),               // ours, or a decoded oracle seed
    PlayerSave { save: PlayerSave, slot: u8 }, // the original's load flow
    Arrival { world: WorldId, doorway: DoorwayId, carried: Box<Carried> },
}
```

A seeded row opens with `Start::Checkpoint(decode(seed))`. A constructed arrival ("arrives shinecharged
with N frames") is a doorway plus the carried State, so strats between rooms that are never neighbours
can be tested.

## 15. Build order

MODEL §5 orders the work inside each step: by mechanism, with the oracle as the acceptance check.

1. Oracle side: rows recorded on the unmodified ROM with facts and schedules, from native seeds (ORACLE §5).
2. Measurements that set rules (P2): IRQ dots and handler finish lines, the driver's echo timing, load
   and upload costs, overrun frames per row.
3. The crates and their CI checks (§4.2).
4. `canonical` and the State skeleton for slice 0's domains.
5. The decoder for that skeleton, with its mutants.

**Slice 0, the Morph Ball Room:** `area-brinstar` then `item-morph-ball` (seed `100-input-phase:17`,
room 9E9F): the Tick and NMI, the HUD split, video memory and the batch, the compositor, a seeded start,
the slot store and interpreter, Samus on the ground and jumping, the message box's mid-pass suspension
and about 370 lag Ticks, the pickup sound, a Checkpoint restore mid-box. Exit: both rows green with
facts, determinism, a mutant per stream and in the decoder, intents equal to the exact picture, §12's
speeds, §13's sizes.

**Then clusters of rows that share rooms**, each adding only what its rows reach:

| Step | Rows | Adds |
| --- | --- | --- |
| 1 | Landing Site: `area-crateria`, `samus-walljump-*`, `samus-shinespark-*`, `samus-space-jump-chain`, `special-beam-*`, `xray-scan-91f8`, `samus-gunship-save` | movement depth, beams, x-ray HDMA, rain, saving |
| 2 | `door-transition`, `door-loading-lag-nmi`, `tas-ceres-*`, `elevator-*`, `power-bomb-*-entry` | the two-strand transition, elevators, power bomb; `Modeled` timing |
| 3 | `area-*`, `enemy-*`, `terrain-*`, `beam-reflect`, `xray-scan-*` | enemy families and projectiles, liquids, terrain; a Mesen spot check first |
| 4 | `item-*`, `pickup-*`, `samus-grapple-*`, `xray-activate-release` | equipment, grapple, reserves, message boxes |
| 5 | Ceres rows, `boss-ceres-ridley*` | Mode 7, the escape timer, cinematics |
| 6 | `pause-menu-tour`, `death`, `save-*`, `file-copy-clear` | pause and readback, death, loading |
| 7 | bosses, smallest first (Spore Spawn, Bomb Torizo, ... Mother Brain) | phase tables, multipart groups |
| 8 | `options-menu-*`, `demo-attract-setup`, `samus-scripted-intro` | boot, title, menus, demos |
| 9 | `credits*`, `music-fir-selector10` | ending, credits, music |
| 10 | room chains, then the any%, 100% and low% movies | completeness |

Map Rando starts after step 3 (the vanilla `World`, constructed arrivals, then topology import). From
slice 0 on, green rows also run continuously from their earliest green start, with facts.

## 16. Lessons

The rules from the earlier attempts, and where they are met: (1) fresh code to target types, the
original reference only: D1, MODEL §5's implementer; (2) no new code depends on old: the old code is
gone, tag `archive/old-engine`; (3) one authority, no codec back to a RAM layout: D3, P5; (4)
translation one way, in verification: D21; (5) nothing exists only for comparison: D3's reader rule,
§11.6; (6) no host traits, concrete contexts: D2, K3; (7) the spine runs end to end in the first
slice: §15; (8) the gate counts rows the engine executes: D22; (9) one path per job, a replacement
deletes the old: P7, D16; (10) no probes or verify cfgs in the engine: §4.2; (11) per-domain size
budgets: D24; (12) the reference shares no code and runs the unmodified ROM, with a second reference
for pixels: D20, D17, the Mesen spot check; (13) free run against the lock is the gate, lockstep
localises: D22, §11.5; (14) every exclusion is a named LEGACY row: §11.6; (15) a mutant per stream:
§11.5; (16) facts per row: §11.3; (17) verified speed: D23; (18) one canonical Checkpoint: §14; (19)
verification has its own budget and one runner: D24; (20) continuous runs: §15; (21) refactors are
`[exact]`: D14; (22) structural counts are advisory: P13; (23) one short rules file, Jim asked only
goal trade-offs: P19, AGENTS.md; (24) land one change at a time by fast-forward: `tools/lane/land.sh`;
(25) the spine is built by one lane: §15.
