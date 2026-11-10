# Blocks that break and come back, and the scroll cells: behaviour spec

What the player sees the room's blocks do when Samus or her weapons meet them: a crumble block
falling away under her feet, a speed block shattering as she dashes through it, shot, bomb, super
missile and power bomb blocks breaking, hidden blocks showing what they are, the blocks that grow
back a few seconds later, and the cells that open and close the screens the camera may show.
Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/blocks.toml`](../../scenarios/vectors/blocks.toml).

## Header

**Scope: the first of four parts.** The plan's W2, blocks that react, is too large for one spec
(about five rounds), so it is split into four parts, each one verifiable on its own:

1. **Blocks that break and come back, and the scroll cells (this spec).** The places reactions
   run in, the scripts they run and the steps a script is made of; crumble, speed, bomb, shot,
   super missile and power bomb blocks: what starts them, how they answer her motion, how they
   break, show themselves and grow back; the scroll cells; the stand-in cells a room places.
2. **Doors' caps and gates:** the coloured caps on doors and what opens each, the doors that
   close behind her, the caps that wait for a condition, the gates and what opens them. With the
   doors' passage (R1).
3. **Grapple blocks, spikes and hazards:** what her grapple holds, and what crumbles
   while it holds on; the spikes' hurt (with the energy domain); what enemies break; the plants.
4. **Cells the room scripts:** quicksand and falling sand, the treadmills, the stations, and the
   cells a boss or an escape changes.

Items are W3's. Each later part adds to this part's mechanism the families of reactions it
needs, the steps their scripts need, and what she meets inside its cells; its starts come to this
part as its requests, among this part's own, in the frame's order (§7).

**What samus-motion must add.** One output: each cell she is inside (`TouchedCells`) comes with
its point, her top edge, her centre or her bottom edge, a stand-in already resolved to the cell it
stands for. Every point is read: a scroll cell reacts only to her centre (`ScrollCells`), and a cell
that holds a place when she is inside it holds one for each point (`BlocksMarkedNothing`). Its
report today gives those cells in order but not their points, and the order alone can't tell them
apart: a cell in her bottom edge's row that her centre shares is her bottom edge's, and the next
cell may be her centre's or her top edge's (samus-motion's gaps, 36).
Its other outputs are read as it exports them: the cells her moves and looks reach, in order, with
their ways. Her motion asks this domain how each reacting cell meets it (`AnswersToHerMotion`); the
answer is per touch, in order, and may differ from one touch of a cell to the next.

**What her weapons must export** (the beams, missiles and bombs domain, W1): each frame, the cells
her weapons reach, in the order they reach them, each with its weapon: a beam, a missile, a super
missile, a bomb's blast or a power bomb's blast; a stand-in reached is reported as the cell it
stands for, as her motion reports her touches. Which cells a beam's flight or a blast reaches is
W1's; what a cell then does is this domain's.

**What her grapple must export** (the grapple beam, S6): each frame, the cells the grapple beam
reaches, in order, each as a hit by `Grapple`, a stand-in already resolved.

**What every other part must export:** each start of one of its reactions that takes a place, as
a request in the frame's order (§7): the plants and the quicksand she is inside, the stations and
the quicksand she touches, the blocks the enemies break.

**What the room's loading and the doors must export** (R1, R2): each frame, whether the run is
paused (`TheScriptsRun`), as it is while a room starts and while she passes a door.

**Not in this spec** (§8 names who supplies each): the other three parts; drawing a redrawn cell
on the screen, which the tile encoder does only for cells in view (E2); the sound queue and what
happens to a request it can't hold (audio, U1); the room's loading, which supplies its cells,
marks and placements (R2), and decides when the run is paused (R1, R2).

**Notation.** A cell is one 16 px square of the room, at `(x, y)` counted in cells from the room's
top-left, x rightward, y downward. A look is the picture a cell shows: one of the room's tileset's
pictures, by its index in hex (`310`), with `:x`, `:y` or `:xy` when it is flipped across, down or
both. Which picture a number is belongs to the room's tileset; this domain only sets the number
and the flips.

---

## 1. What the player can tell

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| each cell's kind | `CellKind` (below) | whether she and her shots pass it, and what reacts | every entry; samus-motion (her moves) |
| each cell's mark | `Mark` (below) | which reaction a reacting cell has, and whether it holds a place when she is inside it; how far a stand-in reaches | `PatternCells`, every block entry, `BlocksMarkedNothing`, `ScrollCells`; samus-motion (stand-ins) |
| each cell's look | `Look`: the picture's index, `000` to `3FF`, and whether it is flipped across, down, both or neither (Notation) | the picture it shows | output (`WhatTheScreenRedraws`; the backgrounds draw every cell's look as it scrolls into view) |
| the places | 40 `Place`s, numbered 0 to 39, each free or holding one of: a running reaction, a scroll reaction, another part's reaction, or a hold until the run | how many reactions can run at once; which runs first | `Places`, `TheScriptsRun`, `ScrollCells`, `RoomStart` |
| a running reaction's reaction | the name of an imported reaction (§2) | which pictures it shows | `TheScriptsRun`, `Regrowing` |
| its script | `broken`, `revealed` or `touched` (§2) | which pictures and sounds | `TheScriptsRun` |
| its cell | `(x, y)` in cells | where it shows | `PatternCells`, `Regrowing` |
| its next step | the index of a step in its script, from 0 | which picture comes next | `TheScriptsRun` |
| its wait | frames, 1 to 32,767: no frame ends with a wait of 0 | when the next picture comes | `TheScriptsRun` |
| what it grows back as | `Block` or `ReactingAir`, for a reaction whose script has a `Regrow` step: what its cell was when it started, kept for as long as it runs; none for a script without one | whether the regrown block stops her | `Regrowing` |
| a scroll reaction | its cell `(x, y)`; its changes: a list of screens `(x, y)`, each with the `ScreenKind` it takes; whether it is touched: a `bool` | which screens open or close when she touches the cell, and when | `ScrollCells` |
| a hold until the run | nothing: it only takes its place | which place a later start in the frame takes | `Places`, `TheScriptsRun` |
| the room's screens | the camera's grid of `ScreenKind` (`Closed`, `Open`, `Short`) | what the view may show | the camera; `ScrollCells` writes them |

**Cell kinds**, listed alphabetically (the order means nothing). Only these concern this domain;
the room's other kinds (slopes, spikes, doors, ...) are samus-motion's and the other parts'.
- `Air`: nothing stops her or her shots.
- `Block`: a reacting block, solid by its body; its mark says how it reacts.
- `ReactingAir`: a reacting cell with an air body: she and her shots pass it, and its mark says how
  it reacts.
- `Solid`: a plain solid block.
- `StandInAcross`, `StandInDown`: a cell that stands for another along its row or its column
  (samus-motion's `OtherBlockKinds`); its mark is how far, in cells.

**Marks.** The room gives each cell its mark. For a reacting kind it is one of:
- an imported reaction of §2;
- `Scroll`: a scroll cell (`ScrollCells`);
- `Nothing`: a cell that reacts to nothing but holds a place, with the family whose blocks it is
  met as, `Bomb`, `Crumble` or `Shot` (`BlocksMarkedNothing`);
- `Inert`: a cell that reacts to nothing and takes no place (`BlocksMarkedNothing`);
- `Other`: a cell of another part (a door's cap, an item, a station, quicksand). This part keeps
  its mark; its part answers it, and its starts come here as that part's requests. A block marked
  `Other` may also be met as crumble blocks by a blast, as a station's access cells and a save
  station are (`BlocksMarkedNothing`).

A mark that names a reaction always names one of the room's reactions: the room's loading
refuses a room with a mark that names none (R2), so this domain never meets one. A reacting cell's
mark may also say that she holds a place when she is inside it; it holds one only while the cell
is reacting air (`BlocksMarkedNothing`). For a stand-in, the mark is its reach, a signed number of
cells. A step changes a cell's kind and look, never its mark, so a block that has crumbled into
air still carries its mark and grows back as the same block.

### 1.2 What it remembers: state a caller can hold

Everything in §1's table, and nothing else: the cells' kinds, marks and looks; each place, free or
with what it holds and every field of it; and the screens. This is state another part of the game
carries from one frame to the next, saves and restores, and sets when play starts part-way, so a
caller can build each part at any value this spec allows and read each part back: a vector's
`before` builds it and its `after` reads it. A scroll reaction's touch and a hold until the run
both last to the next run that isn't paused, so a paused frame carries them to the next frame
(`TheScriptsRun`, `ScrollCells`).

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| the frame's requests | one list, in the frame's order (§7); each is one of: **inside**, a cell she is inside and its point, `Top`, `Centre` or `Bottom` (her top edge, her centre, her bottom edge, listed top to bottom; the point samus-motion must add); **touch**, a cell her move or look reaches and its way, `Down`, `Left`, `Look`, `Right` or `Up` (samus-motion's `TouchedCells`); **hit**, a cell and what reaches it, `Beam`, `Bomb` (a bomb's blast), `Grapple` (the grapple beam), `Missile`, `PowerBomb` (a power bomb's blast) or `SuperMissile`; **another part's start**. A cell is the one a stand-in stands for, already resolved, and always a cell of the room: what her motion and her weapons meet past its edge is theirs (§8) | `Places`, every block entry, `ScrollCells`, `AnswersToHerMotion`, `BlocksMarkedNothing` |
| how she moves | three `bool`s: dashing (at the speed booster's full speed), shinesparking, and screw attacking, which is spinning in a screw attack, not every pose in which the screw attack hurts (her other domains) | `SpeedBlocks`, `BombBlocks` |
| the run paused | `bool`, from the room's loading and the doors (R1, R2) | `TheScriptsRun` |
| the room's placements | when the room starts, in the room's order: scroll cells with their changes, stand-ins with their way and reach, and the other parts' placements | `RoomStart` |
| the reactions | the imported reactions (§2), one per mark that names a reaction | every block entry, `TheScriptsRun` |

### 1.4 What it reports (Outputs)

| Output | Type | Read by |
| --- | --- | --- |
| each touch's answer | in order, one per touch: `Air`, `Solid`, or `Other` for a cell marked `Other`, which its part answers (`AnswersToHerMotion`, `BlocksMarkedNothing`) | samus-motion, in the same frame |
| the cells redrawn | in order, each a cell `(x, y)` (`WhatTheScreenRedraws`) | output: the tile encoder (E2) |
| the sounds asked for | in order, each a sound (the script's) and its queue limit, a count (`Sounds`) | output: audio (U1) |
| the screens | as in §1's table, after `ScrollCells` | the camera, in the same frame |

---

## 2. Reactions and their scripts: the imported assets

A **reaction** is what a reacting cell does, as the game's content gives it. The reactions are
imported from the player's ROM by an importer outside the clean room, into this format; they are
never written into the model. A reaction holds:

| Field | Holds |
| --- | --- |
| family | `Bomb`, `Crumble`, `PowerBomb`, `Shot`, `Speed` or `SuperMissile` (alphabetical; the order means nothing): which rules of §5 start it |
| whole look | the picture the block has when whole, by its index (Notation), never flipped: its start gives the cell that picture and growing back restores it; none for the shot blocks that keep their own look |
| first wait | the frames from its start to its first step, 1 to 32,767, for a start of its `broken` or `touched` script; a `revealed` script's start always waits 1 |
| scripts | `broken`, what it does when it breaks; `touched`, for a bomb block broken by her touch; `revealed`, what a bomb that doesn't break it does |

A **script** is a list of steps, run in order (`TheScriptsRun`):

| Step | Holds | Does |
| --- | --- | --- |
| `Sound` | a sound and its queue limit | asks for the sound and goes straight on to the next step |
| `Show` | a pattern and a number of frames, 1 to 32,767 | the pattern's cells take their kinds and looks; the reaction waits that many frames |
| `Regrow` | nothing | the cell grows back as it was, with the whole look (`Regrowing`); the reaction waits one frame |
| `End` | nothing | the reaction ends and its place is free |

A **pattern** is a list of cells, each an offset from the reaction's cell (0, 0 is the cell
itself), what it becomes, and its look:

| Becomes | The cell becomes |
| --- | --- |
| `Air` | air |
| `Itself` | its reacting kind as a block: `Block`, its mark unchanged |
| `StandInAcross` | a stand-in along its row, reaching as its mark says |
| `StandInDown` | a stand-in along its column, reaching as its mark says |

- Cells: Air, Itself, StandInAcross, StandInDown.

So a crumble block's `broken` script may be: a sound; air showing one picture for some frames,
then another; grow back; end. A 2x1 block's patterns name both its cells, and its last show may
put back the block and the stand-in beside it rather than grow back. Which scripts the game's
blocks have is the importer's output; the vectors use invented reactions in this format. A
room never places a reacting cell whose patterns reach past the room's edge: the room's loading
refuses such a room (R2), so this domain never meets one.

**The boundary between assets and the spec.** What a reaction's script shows, for how long, and
with which sound is game content: the importer writes it, and changing it changes the game's
content, not its rules. What makes a reaction start, how its start changes the cell, how a block
answers her motion and how the steps run are rules: they are the arms below and the model's code.
The family names the rules a reaction follows; it is the only part of a reaction the code
decides on.

---

## 3. Running reactions

### Places
- kind: mechanic.
- summary: at most 40 reactions run at once; each one started takes the highest free place.
- params: 40 places.
- switch: none.
- A reaction starts → it takes the highest-numbered free place [C-place-highest,
  C-place-two]
- A reaction would start with every place taken → it doesn't start, and the cell is left as it was
  [C-place-full, C-place-full-shot, C-place-last]
- A touch or a hit that starts no reaction → it takes no place [C-place-refused-frees,
  C-crumble-twice]
- Another part's start → it takes the highest free place, which its part holds [C-order-other]
- So two blocks started in one frame take places in the order they started (C-place-two,
  C-order-hit-then-touch); a block started while another of the same cell still runs takes a
  place of its own (C-regrow-touch-again).
- checked: oracle (the places of every start on record, 111 starts over 44 rows, each seen to
  start, as the harness cases r04:110, r06:39, r11:120, item-charge-beam:53 and 122 show; the
  refused starts the oracle saw take none, r03 and r11);
  constructed (the full places, the order).

### TheScriptsRun
- kind: mechanic.
- summary: every frame, after her weapons and her moves, each running reaction counts down its
  wait; when the wait runs out its next steps run; a paused frame runs none of it.
- params: none.
- switch: none.
- Each frame, for each running reaction from the highest place down → its wait drops by one frame
  [C-run-wait, C-run-order]
- A reaction's wait reaches 0 → its steps run from its next step, until a step that waits or ends
  it [C-run-sound-then-show, C-run-order]
- A `Sound` step → its sound is asked for and the next step runs at once [C-run-sound-then-show,
  C-bomb-by-power-bomb]
- A `Show` step → its pattern's cells change, and the reaction waits the step's frames
  [C-run-sound-then-show, C-pattern-1x2-air, C-pattern-2x1-regrow]
- A `Regrow` step → its cell grows back (`Regrowing`), and the reaction waits one frame
  [C-regrow-air, C-regrow-hold]
- An `End` step → the reaction ends and its place is free [C-run-end, C-regrow-hold, C-reveal-ends]
- A reaction started this frame → its first wait counts down in this frame's run, so a first wait
  of 1 runs its first step in the frame it starts and a first wait of 3 two frames later
  [C-place-highest, C-shot-regrows, C-shot-blast, C-pb-first-wait]
- The run paused → every running reaction keeps its wait and its next step, and each place held
  until the run, and each scroll reaction touched, stays so until the next run that isn't paused;
  the requests start reactions and take places as on any frame [C-run-paused, C-scroll-paused,
  C-scroll-after-pause]
- So a crumble block she lands on stays solid for those frames, then shows the crumble pictures as
  air (C-place-highest, C-run-sound-then-show), and two reactions that show in one frame redraw
  their cells highest place first (C-run-order).
- checked: oracle (every running reaction of the 44 recorded rows, 603 frames, with the game's
  reactions: the harness cases r04:111, 113, 115, 159 and 160, r06:42 and 55, area-norfair:26,
  29, 33 and 38); constructed (the paused run: the records pause it only while no reaction of this
  part runs).

### PatternCells
- kind: mechanic.
- summary: a show changes each of its pattern's cells to what the pattern says, keeping its mark.
- params: none.
- switch: none.
- A pattern cell → the cell at the reaction's cell plus its offset takes the pattern's kind and look,
  and keeps its mark [C-pattern-1x2-air, C-pattern-2x1-regrow]
- A pattern cell that becomes `Itself` → the cell becomes `Block`, its reacting kind, so a broken
  block's own cell comes back as the block its mark names [C-pattern-2x1-regrow, C-pb-by-bomb,
  C-reveal-crumble-2x1]
- A pattern cell that becomes `StandInAcross` or `StandInDown` → the cell becomes that stand-in,
  reaching as its mark says [C-pattern-2x1-regrow, C-pb-by-bomb]
- checked: oracle (a 1x2 shot block breaking: the harness cases area-norfair:25, 29 and 33);
  constructed (the patterns that put blocks back).

### Regrowing
- kind: quirk.
- summary: a block that grows back takes its kind's whole look, not the look it had, so a hidden
  block shows itself once it has grown back.
- params: none.
- switch: none.
- A `Regrow` step → the cell becomes the reacting kind the reaction remembers, the one it was when
  the reaction started (`Block`, or `ReactingAir` for one that was air to her), with its
  reaction's whole look [C-regrow-air]
- So a crumble block that showed another picture, or a flipped one, before its start grows back
  showing its reaction's whole look (C-place-highest, C-regrow-air); and the reaction remembers
  what it grows back as until it ends, past its `Regrow` step (C-regrow-air, C-regrow-hold).
- checked: oracle (one crumble block's start, growing back and end: the harness cases r04:110,
  159 and 160).

### RoomStart
- kind: mechanic.
- summary: when a room starts, its placements take places in the room's order: the scroll cells
  keep theirs, the stand-ins leave theirs at once.
- params: none.
- switch: none.
- Each placement takes a place as any start does (`Places`).
- A scroll cell placed → its cell becomes `ReactingAir` marked `Scroll`, keeping its look, and its
  reaction holds its place with its changes [C-room-scroll, C-room-stand-in]
- A stand-in placed → its cell becomes that stand-in, marked with its reach, and its place is free
  again at once [C-room-stand-in, C-room-down]
- Another part's placement → it holds its place, as its part says [C-room-others]
- checked: constructed (no recorded row starts a room inside the frames the harness runs; the
  room's loading is R2's).

---

## 4. Scroll cells

### ScrollCells
- kind: mechanic.
- summary: touching a scroll cell changes screens the camera may show, in the same frame.
- params: none.
- switch: none.
- A scroll cell reached by any move or look, or she is inside it at her centre → its reaction is
  touched [C-scroll-move, C-scroll-look, C-scroll-centre]
- She is inside a scroll cell only at her top or her bottom edge → it isn't touched
  [C-scroll-bottom]
- A scroll cell touched → it takes the highest free place until the next run, so a reaction
  started after it in the frame takes the next one down [C-scroll-holds-place, C-scroll-paused]
- A scroll cell touched with every place taken → it isn't touched [C-scroll-full]
- A touched scroll reaction, in the next run that isn't paused → each of its changes sets its
  screen's kind, in the order the changes are listed, and it is no longer touched [C-scroll-move,
  C-scroll-after-pause]
- So a scroll cell touched twice before its run changes its screens once (C-scroll-twice,
  C-scroll-paused); one touched on every frame she stays in it sets screens that already have
  their kinds, which shows nothing; and of two scroll cells touched in one frame whose changes name
  the same screen, the lower place's change stays (C-scroll-order).
- A scroll cell is air to her (`ReactingAir`), so she passes through it as she touches it.
- A room never has a scroll cell without a scroll reaction on it: the room's loading refuses such a
  room (R2), so this domain never meets one.
- checked: oracle (197 scroll touches over 44 rows, as the harness cases show: the Morph Ball
  Room's shaft opening and closing a screen as she climbs and falls, area-brinstar:92 and 391 and
  item-morph-ball:92 and 391; touches every frame, area-brinstar:93 and r02:47; at her centre,
  item-charge-beam:139); constructed (the points, the looks, the full places, the paused run).

---

## 5. Blocks that break

In this section a block's start is its reaction starting (`Places`), with the reaction its mark
names; "the whole look" is that reaction's (§2).

### CrumbleBlocks
- kind: mechanic.
- summary: a crumble block falls away a few frames after she lands on it.
- params: the first wait (the reaction's).
- switch: none.
- A crumble block reached by a move down → it starts: it becomes `Solid` (`Air` if it was air to
  her) with the whole look, remembering what it was, and its `broken` script waits its first wait
  [C-place-highest, C-crumble-air, C-crumble-2x1]
- A crumble block reached by a move left, right or up, or by a look → it doesn't start
  [C-crumble-side, C-crumble-up, C-crumble-look]
- So a second touch of the same block in the frame meets a plain solid block (C-crumble-twice), and
  a 2x1 block's stand-in, which stands for its first cell, starts the block from either cell.
- checked: oracle (two rows' crumble bridges: the harness cases r04:110 and r06:39); constructed
  (the other ways).

### SpeedBlocks
- kind: mechanic.
- summary: a speed block shatters as she dashes or shinesparks into it, and stops her otherwise.
- params: none.
- switch: none.
- A speed block reached while she is dashing or shinesparking, by any way → it starts: it
  becomes `Air` with the whole look, remembering what it was [C-speed-dash, C-speed-spark,
  C-speed-look]
- A speed block reached otherwise → it doesn't start [C-speed-walk, C-speed-screw]
- checked: constructed (no cached row dashes through one).

### BombBlocks
- kind: mechanic.
- summary: a bomb block breaks under a bomb's or a power bomb's blast, or as she dashes,
  shinesparks or screw attacks into it.
- params: none.
- switch: none.
- A bomb block reached while she is dashing, shinesparking or screw attacking, by any way →
  its `touched` script starts: it becomes `Air` with the whole look, remembering what it was
  [C-bomb-screw, C-bomb-dash, C-bomb-spark]
- A bomb block reached otherwise → it doesn't start [C-bomb-walk, C-bomb-air-walk]
- A bomb's blast reaches a bomb block → its `broken` script starts without its leading `Sound`
  step, if it has one: it becomes `Solid` (`Air` if it was air to her) with the whole look,
  remembering what it was [C-bomb-by-bomb, C-bomb-quiet]
- A power bomb's blast reaches a bomb block → its `broken` script starts from its first step,
  the same way [C-bomb-by-power-bomb]
- A beam, a missile of either kind or the grapple reaches it → it doesn't start
  [C-bomb-by-missile, C-grapple-others]
- So a bomb breaks a bomb block without the sound its `broken` script begins with (the bomb's own
  blast is heard), and her touch breaks it with the `touched` script's sound (C-bomb-by-bomb,
  C-bomb-screw).
- checked: oracle (the harness cases item-reserve-tank:572, a bomb's blast, and
  item-charge-beam:122, a power bomb's; and r03's frames 185, 458, 460 and 461, where the oracle
  saw a shot and her touches meet one and start nothing); constructed (her touches that break
  one, the refusals).

### ShotBlocks
- kind: mechanic.
- summary: a shot block breaks under any of her weapons and under the grapple.
- params: none.
- switch: none.
- Any weapon or the grapple reaches a shot block → it starts: it becomes `Solid` (`Air` if it
  was air to her), with the whole look if its reaction has one and its own look otherwise, and one
  that grows back remembers what it was [C-shot-regrows, C-shot-blast, C-shot-air, C-shot-grapple]
- So a shot block whose reaction has no whole look keeps its own look as it turns solid, until
  its first show replaces it (C-shot-blast).
- checked: oracle (area-norfair's and the Morph Ball Room's shot blocks: the harness cases
  area-norfair:25 and item-morph-ball:1236); constructed.

### SuperMissileBlocks
- kind: mechanic.
- summary: a super missile breaks a super missile block; a bomb shows it for what it is.
- params: none.
- switch: none.
- A super missile reaches a super missile block → it starts: it becomes `Solid` (`Air` if it was
  air to her) with the whole look, remembering what it was [C-super-by-super, C-super-regrows]
- A bomb's blast reaches it, a block or air to her → its `revealed` script starts, leaving the cell
  as it is until the script's show [C-super-by-bomb, C-super-air-by-bomb]
- Any other weapon or the grapple reaches it → it doesn't start [C-super-by-missile,
  C-grapple-others]
- checked: oracle (one broken on item-charge-beam's way: the harness case item-charge-beam:53);
  constructed (the bomb, the other
  weapons).

### PowerBombBlocks
- kind: mechanic.
- summary: a power bomb's blast breaks a power bomb block; a bomb shows it for what it is.
- params: none.
- switch: none.
- A power bomb's blast reaches a power bomb block → it starts: it becomes `Solid` (`Air` if it was
  air to her) with the whole look, remembering what it was [C-pb-first-wait]
- A bomb's blast reaches it, a block or air to her → its `revealed` script starts, leaving the cell
  as it is until the script's show [C-pb-by-bomb, C-pb-air-by-bomb]
- Any other weapon or the grapple reaches it → it doesn't start [C-pb-by-super,
  C-grapple-others]
- So a super missile or power bomb block that is air to her is a block once its revealed show puts
  it back as `Itself` (C-super-air-by-bomb, C-pb-air-by-bomb).
- checked: oracle (two power bomb blocks broken by one blast: the harness case r11:120);
  constructed.

### BombsReveal
- kind: mechanic.
- summary: a bomb's blast shows a hidden crumble or speed block for what it is.
- params: none.
- switch: none.
- A bomb's blast reaches a crumble or speed block that is a block to her → its `revealed` script
  starts, leaving the cell as it is until the script's show [C-reveal-crumble, C-reveal-crumble-2x1,
  C-reveal-speed]
- A power bomb's blast, a beam, a missile or the grapple reaches one, or a bomb reaches one that
  is air to her → its reaction doesn't start [C-reveal-power-bomb, C-reveal-air,
  C-reveal-speed-air, C-grapple-others]
- So the revealed script shows the block's whole look as `Itself` for a frame and ends
  (C-reveal-ends): the block is unchanged but for its look.
- checked: constructed; on record with no vector, r11's frames 140 to 158, where the oracle saw a
  power bomb's blast meet crumble blocks ten times and start none.

### AnswersToHerMotion
- kind: mechanic.
- summary: how each reacting cell meets her motion, touch by touch.
- params: none.
- switch: none.
- A touch of a `ReactingAir` cell → `Air` [C-scroll-move, C-crumble-air, C-bomb-air-walk]
- A touch that starts a speed or a bomb block → `Air`; one that starts a crumble block → `Solid`
  [C-speed-dash, C-bomb-screw, C-place-highest]
- A touch of a crumble, speed or bomb block that doesn't start → `Solid` [C-crumble-side,
  C-speed-walk, C-bomb-walk]
- A touch of a shot, super missile or power bomb block, by any way and however she moves, with
  places free or none → `Solid` [C-shot-touch, C-shot-touch-look, C-shot-touch-full]
- A touch of a cell marked `Nothing` or `Other` is answered as `BlocksMarkedNothing` says
  (C-nothing-touch, C-nothing-shot-touch, C-other).
- The answer is for that touch: a later touch in the frame meets the cell as the earlier ones left
  it (C-crumble-twice).
- checked: constructed (her motion's records show the answers only through her moves).

### BlocksMarkedNothing
- kind: quirk.
- summary: a cell marked `Nothing` reacts to nothing, but what would ask a block of its family to
  react still takes a place until the run, and a touch of one passes through; a cell marked
  `Inert` takes none; a cell marked `Other` is its part's, but for a blast on one met as crumble
  blocks.
- params: none.
- switch: none.
- A touch of a cell marked `Nothing`, met as crumble or bomb blocks, by any way → `Air`, and the
  touch takes the highest free place until this frame's run [C-nothing-touch,
  C-nothing-bomb-touch]
- A touch of one met as shot blocks → it meets her by its body, as a shot block does, and takes no
  place [C-nothing-shot-touch]
- Any weapon or the grapple reaching one met as shot blocks or as bomb blocks (a block or
  reacting air), or a bomb's or a power bomb's blast reaching a block met as crumble blocks, marked
  `Nothing` or `Other` → it takes the highest free place until this frame's run [C-nothing-hit,
  C-nothing-hit-air, C-nothing-blast, C-nothing-grapple, C-other-blast]
- A beam, a missile or the grapple reaching one met as crumble blocks, or any blast reaching
  reacting air met as them → it takes no place [C-nothing-beam-crumble, C-nothing-blast-air,
  C-nothing-grapple]
- She is inside reacting air whose mark holds a place when she is inside it → at each point it
  takes the highest free place until this frame's run [C-nothing-inside]
- A block whose mark holds a place when she is inside it, with her in it → it takes no place
  [C-nothing-block-holds-none]
- With every place taken → it takes none, and nothing else changes [C-nothing-full,
  C-nothing-inside-full, C-other-blast-full]
- A cell marked `Inert`, touched or hit → it takes no place, and she meets it by its body
  [C-inert]
- A touch, an inside or any other hit of a cell marked `Other` → the touch is answered `Other`,
  for its part answers it, and the cell and the places stay as they were [C-other, C-other-blast]
- So a later start in the same frame takes the place below the one it holds (C-nothing-touch,
  C-nothing-hit, C-other-blast); a power bomb set off in a room with a station or a save station
  holds a place for each of their cells it reaches.
- checked: constructed (no recorded frame meets one; no room of the game holds a place when she
  is inside it, or is marked `Inert`; no harness case shows a blast on a station's or a save
  station's cells, whose place is free again by the frame's end).

### FullPlacesLetHerThrough
- kind: quirk.
- summary: with every place taken, a reacting block lets her through when she touches it.
- params: none.
- switch: none.
- A touch of a crumble, speed or bomb block, or of a cell marked `Nothing` met as crumble or bomb
  blocks, with every place taken → `Air` [C-place-full, C-place-full-side, C-place-full-speed,
  C-place-last]
- So with 40 reactions running (a room full of crumbling bridges and doors), she falls through a
  crumble block or walks through a speed block without breaking it, while a shot, super missile or
  power bomb block still stops her (C-shot-touch-full).
- checked: constructed.

---

## 6. What it reports

### WhatTheScreenRedraws
- kind: output.
- summary: the cells a step writes, in order, for the screen.
- params: none.
- switch: none.
- A `Show` or a `Regrow` step → each cell it writes is reported redrawn, in the pattern's order,
  whether or not its look changed [C-run-order, C-pattern-2x1-regrow, C-speed-dash]
- So a start's own change is not redrawn: the screen shows the block's old look until the first
  show (C-place-two), unless the cell scrolls into view first.
- checked: constructed (the redraws reach the video only through the tile encoder, E2).

### Sounds
- kind: output.
- summary: the sounds a script asks for, in order, each with its queue limit.
- params: none (the sounds and limits are the scripts').
- switch: none.
- A `Sound` step → its sound is reported with its queue limit [C-run-sound-then-show,
  C-bomb-screw, C-bomb-by-power-bomb]
- checked: constructed (the audio domain checks the queue).

---

## 7. Order that matters

- A frame's requests are answered one at a time, in the order the frame gives them; then the
  run of every reaction from the highest place down (C-order-as-given, C-order-other,
  C-scroll-holds-place).
- In the game, the order is the order in which the game meets the cells: the cells she is inside,
  her top edge last; the grapple's hits, then her weapons' hits, in their order (C-nothing-grapple);
  the enemies'; her touches, in her motion's order; the enemies' shots. Another part's start comes
  where its asker meets the room: a plant or quicksand she is inside among the cells she is inside,
  a station she touches among her touches (C-order-hit-then-touch).
- Some parts start after the run: the enemies' shots meeting her or her shots, the enemies a power
  bomb's blast meets, and the room's own script. Such a start comes as a request at the head of
  the next frame's list, in the order they started. It takes the place it took in the game, for
  nothing frees a place between the run and that frame's first request.
- A start changes its cell at once, so a later request in the frame meets the changed cell
  (C-crumble-twice).
- The run comes before the camera's turn, so a scroll cell's screens change before the camera reads
  them in the same frame.

## 8. Out of scope, and who supplies it

- **The other three parts of W2** (Header) and **items** (W3): their reactions take places like
  these; in this spec they hold their place and do nothing else. Which place another part's start
  takes, and how its part frees it, are added to this part's outputs and requests by each part
  that starts reactions: part 2 first, with the doors' caps, then parts 3 and 4, and W3's items.
  Until then nothing here frees another part's place.
- **Her motion:** the touches, their ways, the cells she is inside and their points; how a stand-in
  is followed to the cell it stands for; what she meets past the room's edge, since every request
  names a cell of the room.
- **Her weapons (W1):** which cells of the room a beam, a missile or a blast reaches; how a shot
  block stops a shot (by its body: `Block` stops it, `ReactingAir` doesn't).
- **The grapple beam (S6):** which cells it reaches, and what holds it.
- **The other parts:** their reactions, what she meets inside their cells, and their requests in
  the frame's order.
- **The tile encoder (E2):** drawing redrawn cells that are in view, the queue that carries them,
  and drawing every cell's look as the view scrolls.
- **Audio (U1):** the sound queue and its limits.
- **Room loading (R2):** the cells, their marks and looks (the cells other parts' placements write
  when the room starts, a station's or a save station's, included), the placements and their
  order; a room with a scroll cell no scroll placement covers, a reacting cell whose patterns reach
  past its edge, a reacting cell that isn't one mark to every asker, or a mark that names no
  reaction of the room, is refused there.
- **Room loading and the doors (R1, R2):** when the run is paused.

## 9. Open questions

1. The speed blocks, the bomb blocks broken as she dashes or screw attacks into them, the reveals
   that start, the cells marked `Nothing`, the full places, the paused run and the room's start
   have constructed vectors only: no cached row meets them. Chain rows of the 100% route that
   dash through speed blocks, screw attack through bomb blocks and bomb hidden crumble blocks
   would check them against the game's reactions in the harness; they need caching.
