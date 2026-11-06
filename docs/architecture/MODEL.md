# The behaviour model

This note sets the shape of the engine's game code. It amends ARCHITECTURE where they disagree,
and the amendments are listed in §7. It follows Jim's decisions of 2026-11-06, recorded in §1.

## 1. The goal, and what is exact

The goal is to model Super Metroid's behaviour, all of it, as named behaviours. Each one can then
be reproduced exactly, adjusted (the space jump timing), added (respin) or removed (lag). In the
end only this engine runs; the emulator is the oracle that proves the vanilla behaviours exact.

Jim's decisions, 2026-11-06:

1. **Exact means the compared cuts and the output bytes.** The engine may depart from the
   original's routine boundaries and from the order of work inside a frame. What must match is
   every compared cut (the NMI's return at the end of each frame) and every output byte. Order
   is kept only where it can be observed: lag, overruns, stale video memory.
2. **Reshape before features.** Feature work in a domain waits until that domain has its model
   shape.
3. **Uncompared leftovers.** Hardware leftovers that no game rule reads and no player sees, such
   as DMA registers after a transfer, are not compared. They leave the State and the decoder and
   are listed by name in §11.6 of ARCHITECTURE.

What this rules out: code whose structure is the original's routines. That means a function per
bank routine, branches in the 65816's order, flags re-derived from `wrapping_sub`, and module docs
that narrate call order. The original stays the authority for *what* happens, never for how our
code is organised.

## 2. Layers

| Layer | Holds | May name | Never names |
| --- | --- | --- | --- |
| `data` | content decoded from the ROM, and the **rule tables** (vanilla values of every behaviour parameter) | ROM formats at its edge | game State |
| `game::model` | behaviour, in domain terms: Samus, the camera, actors (PLMs, enemies), the HUD's values, progress, the message box | State, `data`, `Rules`, intents | `VideoMemory`, `NmiBatch`, `VramWord`, `DmaChannel`, OAM bytes, B-bus registers |
| `game::intent` | the per-Tick intents: `SpriteDraw`, `TileWrite`, `LayerExposure`, `HudValues`, `PaletteWrite`, `Effect`, `SoundRequest` | domain types | bytes |
| `game::exact` | the one encoder from intents to the original's bytes, with every encoding quirk as a named rule: the OAM format and its 128-entry wrap, the screen split of tilemap rows, the write queue's capacity, the HUD's tile words, the message box's video | intents, video State | behaviour decisions |
| `game::tick` | the Tick, strands, the NMI, the line program, the cost model (unchanged) | everything | |

- `model` decides; `exact` encodes; `tick` orders.
- `exact` runs where the original queued its uploads, so the batch order is unchanged.
- The intents are also ARCHITECTURE D16's view: the wide and HD renderers read them, and the
  exact path and the intent path come from one source by construction.
- `tools/check_crate_rules.sh` enforces the "never names" column. It starts as a warning and
  becomes an error after the reshape.

## 3. Behaviours are named, catalogued and switchable

`game::behaviours` is the catalogue. Each entry has:

```rust
pub struct Behaviour {
    pub id: BehaviourId,          // enum variant, e.g. BehaviourId::WordwiseSpeedCap
    pub summary: &'static str,    // what the player sees
    pub source: Source,           // one bank-log reference, e.g. Source::bank(0x90, 0x8F0C)
    pub kind: Kind,               // Mechanic | Quirk | Timing (lag, loading) | Encoding
    pub switch: Switch,           // Fixed | Toggle | Parameter(range)
}
```

- A **mechanic** is a deliberate game rule: running speed, the wall jump's reach, the space
  jump's window.
- A **quirk** is unintended behaviour that we reproduce: the word-by-word speed compare, a full
  write queue dropping a draw.
- **Timing** covers slowdown and loading time.
- **Encoding** covers the output's own quirks, which live in `exact`.

**Switches are Fixed unless there is a named use (review R7).** Every Toggle is a second code path
that needs its own tests. `SpeedCap` is a Toggle, as the proof of the thesis. Other entries stay
Fixed until a consumer asks for them: a Map Rando option, respin, the QoL list. A `Parameter`, a
value in `Rules`, is cheap and may stay.

`Rules` (the Session's options, ARCHITECTURE §10) carries one value for each switchable behaviour.
The Reference profile holds the vanilla values, and those are what the oracle checks. A different
value is an adjusted game. The engine runs it the same way; it just isn't compared.

A behaviour's code reads its parameter from `Rules` and its vanilla data from the `data` tables,
never from a constant buried in a routine. Adding a behaviour means adding a catalogue entry, a
rule value and its model code (a table row, a variant, an encoder rule). It never means editing a
port.

## 4. The reference domain: Samus on the ground

This domain is reshaped first, end to end, so every later domain has a working example to copy.

**Scope:**
- standing, running, turning on the ground, crouching;
- the morph ball on the ground;
- the speed rules they share: base speed, extra run speed, the cap;
- the step down onto lower ground;
- one collision sweep, horizontal and vertical.

The air (jumps, the fall, the wall jump) is the second step.

**Shape:**

```rust
// data::motion: the vanilla values, decoded from the speed tables or declared, catalogue-linked
pub struct MotionRules {
    pub ground: KindTable<SpeedEntry>,
    pub extra_run: ExtraRun,
    pub step_down_extra: Pixels,
    pub speed_cap: SpeedCap,  // SpeedCap::Wordwise (vanilla) | SpeedCap::Exact
}

// game::model::samus::motion: each way of moving is one row of data
enum Horizontal { Hold, Accelerate { decelerate: bool }, Coast }
enum Vertical { KeepToGround, None }
enum After { Nothing, StopAll, EndTurn }
struct Row { horizontal: Horizontal, extra_run: ExtraRunRule, vertical: Vertical, after: After }

const fn row(locomotion: Locomotion) -> Row { /* an exhaustive match, no `_` */ }

pub fn step(samus: &mut Samus, pad: Controls, rules: &MotionRules, world: &World) -> Contacts {
    let row = row(Locomotion::of(samus));      // the one place the pose is read
    let velocity = velocity(samus, row, pad, rules);
    let outcome = world.sweep(samus.body(), samus.position, velocity);
    apply(row.after, samus, outcome)
}
```

**What the reshape proves:**
- It is exact (review R1). Three checks:
  - the free-run gate's green pairs hold;
  - the free-run localiser's first differing frames are unchanged;
  - the lockstep ratchet holds **on the domain's declared fields**.

  The declared fields are the spec's quantities whose **Read by** is not "none" (§5). Lockstep
  compares State field by field, so comparing every decoded field would pull the State back to the
  RAM layout. A quantity that no later rule reads and that never reaches output is not State, not
  decoded and not compared.
- The thesis holds in code. `SpeedCap` is a switchable quirk:
  - `SpeedCap::Wordwise` is vanilla, the oracle-checked value. The original compares the pixel
    and subpixel words separately, so 2.1000 counts as past a cap of 2.C000.
  - `SpeedCap::Exact` is the corrected rule.
  - A unit test shows the two differ on exactly that case, and the Reference profile stays
    exact.
- It's smaller (review R8). The spec header holds the baseline (today's lines for the domain) and
  a budget, written before the build. The size report shows the domain against its budget, and
  going over is a review stop.

**Done means:**
- A movement kind is a row, a parameter is a table entry or a rule, and collision is one sweep.
- Nothing in `model::samus` names a video type or an address. Source lines live in the catalogue
  only (review R3).

**The pilot passes (review R10) when all of these hold:**
1. The three exactness checks above hold.
2. No asm address appears in `game::model::samus`.
3. The domain's RAM-shape list (§5) is short and every item on it is justified.
4. The vector tests pass, and the domain is under its budget.
5. At least one gap went from the implementer to the observer and back, and was fixed by amending
   the spec, not by patching the code.
6. The implementer never read the bank logs, `verify`'s address files or the old module.

If the other items hold but item 3 doesn't, the clean room failed, however well the code reads.
That is fixed before the second domain.

## 5. How the work is planned

**By mechanism, not by divergence.** A piece of work builds or extends one general mechanism
that covers a family of behaviours:
- a rule table;
- the instruction-list interpreter (`Program<F>`, which enemies have and PLMs need);
- the collision sweep;
- an encoder rule.

Lockstep and the free-run localiser are the acceptance check, and they tell *which behaviour* is
wrong. They don't decide *what code to write*.

**Two roles, always separate sessions: the observer and the implementer.** One agent never
switches between them. A domain goes in this order (review R2):
1. **The observer** reads the original (the bank logs, the ROM, the oracle's records) and writes
   the **behaviour spec**, `docs/behaviours/<domain>.md`, to the template below. It also commits
   the spec's test vectors as data in `scenarios/vectors/<domain>.toml`: before, pad and after, in
   domain terms, the constructed ones and the `Exact` variants included (review R5). The specs are
   the catalogue's source and the domain's design note, and Jim can read them as a description of
   what Super Metroid does.
2. **A spec reviewer** checks the spec before any implementer starts. That is Jim, or an agent that
   hasn't seen the asm. The question: could someone who never saw the asm implement this, and
   would the code be shaped by the domain? (review R4)
3. **The implementer** designs the State types and the model from the spec alone. It writes into
   an **empty module**, `game::model::<domain>`, never opens the old module, and deletes it in the
   same commit (review R3). A test in `game` runs the vector file, so it iterates locally and
   reaches the oracle only to confirm.
4. **The observer writes `verify`'s decode for the implementer's types.** Decode is an abstraction
   from RAM to the State, not a copy. If a State value can't be decoded, the observer either names
   the hidden quantity as a new spec entry, in domain terms, or shows that the value is derived,
   so the model computes it and doesn't store it. The implementer never adds a field because the
   decoder has one.
5. **Gaps go back to the observer** (review R6). The implementer doesn't fix a difference it can't
   explain. It writes an entry in `docs/behaviours/<domain>.gaps.md` with the frame, the field, the
   expected and actual values and the entry it suspects, then moves on. The observer answers from
   the original and amends the spec.

**What the implementer may read:** this note, its domain's spec and vectors, the localiser's
output, `data`'s public types, and the model code. **Not:** the bank logs or the asm,
`verify/src/{decode,source,strand}.rs` (they hold WRAM addresses), the old module it replaces, and
history before the tag `archive/old-engine`.

**No fitting to the oracle (review R6).**
- Every literal and every branch in `game::model` traces to a `BehaviourId`, and values come from
  `Rules` or `data`.
- Catalogue entries are added only in observer commits, with the trailer `Role: observer`.
- No conditions on frame numbers, positions or room IDs that no spec entry names.

**The spec template (review R4).**
- **Header:** the domain's scope, its baseline lines and its budget (R8).
- **Each quantity:** its units, a domain type (an enum, never ROM codes), and **Read by**: the
  behaviour IDs that read it on a later frame, or "output" if it reaches bytes, or "none". "None"
  means it isn't State (decision 3). ROM codes belong in the decode notes.
- **Each entry:** id, kind, summary, rule, params, switch, **checked** (`oracle (V3, V4)` or
  `original only`), and one `Source:` line.
- **Not allowed:** headings named after routines; RAM addresses outside the Source line; step lists
  that follow a routine. Order is stated only in "Order that matters", and only where it is
  visible at a cut.

**The RAM-shape report (review R2).** The size report lists, for each domain, the State fields whose
decode is one WRAM word copied as-is. Each needs a one-line reason in the spec.

**Address check (review R3).** `tools/check_crate_rules.sh` reports `$XX:XXXX` and `$XXXX` anywhere
in `game` outside `behaviours.rs`. It starts as a report, because 881 lines cite addresses today,
and becomes an error domain by domain as each one is reshaped.

**A brief names behaviours and the mechanism** by spec entry IDs (for example: "`BallAtRest`
coasts like standing; add its row"). It carries no bank-log excerpt and no address. A new free
function named after a routine is a review stop.

**Order** (each step is byte-identical unless it says otherwise):
1. **Groundwork:**
   - the catalogue type and the `Rules` hook;
   - the numeric quirks once, on the types (signed compare, the word-by-word cap, wrapping
     moves);
   - the layer check as a warning.
2. **Reference domain:** Samus on the ground (§4). Then the air.
3. **Intents and the encoder:**
   - `SpriteDraw` plus `exact::oam` (Samus, enemies);
   - `TileWrite` plus `exact::tilemap`, which absorbs `plm/draw.rs`, the mapping in
     `bg_graphics.rs` and `tilemap_dma.rs`;
   - the HUD;
   - the message box.
4. **Actors:**
   - `Program<F>` generalised from `enemy/instructions.rs`;
   - the PLM lists decoded from the ROM, replacing `ItemStep` and the crumble's hand interpreter.
5. **Leftovers:** remove the uncompared hardware leftovers (decision 3).
6. **Features resume** in reshaped domains, as mechanisms: sound (C11), the line program (B6),
   the remaining slice-0 differences.

## 6. How progress is measured

Measured first (review R9; a raw behaviour count is easy to inflate by splitting entries, so it
isn't a goal):
- the domains that meet §4's definition of done;
- for each domain, its oracle-checked entries out of its total;
- for each domain, the RAM-shape list (§5) and its size against budget.

Then, as checks that must not regress:
- the lockstep ratchet and the free-run gate's green pairs;
- the free-run localiser's first differing frames;
- Ticks/s.

The size report adds lines per layer, so the routine-port share is visible and falls.

## 7. Amendments to ARCHITECTURE

1. **§2 and SLICE0 §5.5:** phases run in the original's order (§6.3). Inside a domain, structure
   follows the model, and an order that can be observed at a cut is a named behaviour.
2. **§4.1 and §9.1:** add `game::exact`. Producers never write video State.
3. **D16:** intents are the model's primary per-Tick output. `describe` returns them rather than
   reconstructing them.
4. **D3, D5 and §11.6:** hardware leftovers are State only when an engine rule reads them.
   Otherwise they are not compared, and are listed by name.
5. **D13:** a family whose behaviour is a ROM instruction list runs it through the interpreter.
   Hand-coding a list as an enum of positions is a review stop.
6. **P20:** the catalogue (§3) is its artifact, and D24 reports it.
7. **K9:** a bank reference is one `Source:` line on an item; module docs describe the domain.

## 8. Lessons

This note answers these lessons from the three attempts so far. The "review R1" to "R10" labels
refer to an external review of this note (2026-11-06). Its text and the answers are in git history at
commit `c1b3ea3`, `docs/architecture/MODEL-REVIEW.md`.

**The earlier attempts.** Their rules are in ARCHITECTURE §16; these three apply here:
- **The transpile is reference only (rule 1).** Attempt 1 transpiled the C, and attempt 2 spent
  its effort decoupling that transpile. Attempt 3 (this one) kept the rule in name, but the bank
  logs became the module structure. §1 and §2 make the rule structural, and the two roles (§5)
  enforce it: the implementer never sees the original.
- **Nothing exists only for comparison (rule 5).** Broken by the DMA leftovers; fixed by
  decision 3.
- **One short rules file, and ask Jim only goal trade-offs (rule 23).** Broken by 44 docs
  (16,000 lines) and old skills next to the new code. The repository now holds only this
  engine's docs and code; the history is in git (tag `archive/old-engine`).

**From attempt 3 (2026-11-06):**

| Lesson | Answer |
| --- | --- |
| "Fill functions and never reorder" made every domain a routine port (41 % of `game`) | §1 decision 1: exact means cuts and bytes, and structure is the model's |
| With no intent type, every drawing lane wrote bytes | §2: intents and one encoder; `model` may not name video types |
| The thesis had no artifact in the code | §3: the catalogue and `Rules` switches |
| Lanes chose work by the next differing field; tick counts rose while the model didn't | §5: plan by mechanism, with the oracle as the acceptance check; §6: measure behaviours first |
| A ROM list was hand-coded as an enum (`ItemStep`) | §7.5: interpreter mandatory |
| Quirks were re-derived in each routine (16 local signed compares) | §4: once, on the type, named and switchable |
| Each lane rediscovered context from the bank logs; fixed overhead was about half of each lane | §5: the observer writes it once, and briefs carry the excerpts |
| Coordinator waves and hourly reviews measured the process, not the goal | the orchestrator holds the design; review happens once per finished mechanism, against §6 |
| Agents read old docs and old code first and copied their shapes | old code and docs removed from the repository |
