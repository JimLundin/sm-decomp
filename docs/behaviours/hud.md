# The HUD, and the mini-map: behaviour spec

What the player sees in the status bar at the top of the screen while playing: the energy, its
tanks, the ammunition counts, the sign that says the reserve tanks refill on their own, which item
is selected and how Select moves the choice, and the small map of the cells around Samus, the ones
she has explored and her own cell blinking. Written behaviour first, from the oracle's records over
time. Vectors: [`scenarios/vectors/hud.toml`](../../scenarios/vectors/hud.toml).

## Header

**Scope.** One domain, because the player reads one thing: the status bar. It covers what each of
the HUD's cells shows from one frame to the next, the selected item and the one-shot item, how
Select and item cancel change them, and the cells of her map she has explored. It reads the
values it shows from the domains that own them (§1.3), and draws pictures the importer supplies
(§1.4): her energy and ammunition, her reserve
tanks, the items she has on, her room's place on her map and where she is, and the frame count.

**What it leaves to others** (each an input where it feeds us, §6 names who supplies each): what a
shot does to the selection (the weapons cancel it); the energy, ammunition and items themselves
(damage, pickups, the stations that refill her); the pictures of the HUD and of her map (the
importer); when a sound request is heard (the sound
domain's queue); the HUD's first contents when a game starts or the pause screen gives the HUD
back (the load flow, the pause screen); moving the HUD's cells to the screen (the video
encoder); the pause screen's own map, which reads the explored cells this domain keeps.

**Notation.** The HUD is three rows of 32 cells, counted from 0 at the top left. A cell shows a
picture in a colour set, perhaps flipped left to right or top to bottom; vectors write it `PPP:c`
with `h` and `v` for the flips (§1.4). The pictures are numbers the importer gives them; the
vectors' are invented. Her map, the one of the area she is in, is 64 columns by 32 rows of cells,
counted from 0 at the top left; no room lies on row 0. A place is written column first, `(column,
row)`, on the HUD and on her map alike, as the vectors write it. `div` and `mod` are whole-number
division and its remainder.

---

## 1. What the player can tell about the HUD

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| the HUD's cells | 3 × 32 cells, each a picture, a colour set and its flips | the status bar as drawn | `SelectionHighlight`, `OneShotFlash`, `IconsAppear`; output |
| the values last shown | energy, missiles, super missiles, power bombs: counts; an item or none | a value is drawn again only when it differs from the one the HUD shows | `EnergyGauge`, `AmmoCounts`, `SelectionHighlight` |
| the selected item | an item or none (§1.1) | its icon is highlighted; what she fires | `CyclingItems`, `SelectionHighlight`; output (the weapons) |
| the one-shot item | the selected item or none: whether the selection is the one-shot item | its icon flashes; the weapons drop the selection after one shot of it | `OneShotFlash`, `CyclingItems`; output (the weapons) |
| the explored cells | a set of her map's cells | the mini-map shows them, and so does the pause screen's map | `Exploring`, `MiniMapWindow`, `DiagonalPicture`; output (the pause screen, saves) |
| the mini-map is off | `bool`: from when a boss turns it off until it is turned on | the mini-map stops changing, her cell's blink with it | `MiniMapStill` |

Every cell keeps what it shows until a rule of this spec draws it. Nothing else carries from one
frame to the next.

### 1.1 Items

The items the HUD can select, in the HUD's order, left to right: missiles, super missiles, power
bombs, the grapple beam, the x-ray scope. "None" is no item selected; it comes before missiles
when Select cycles.

### 1.2 Where things sit

| Thing | Cells |
| --- | --- |
| energy tanks, in the order they fill | row 1 columns 1 to 7, then row 0 columns 1 to 7 |
| the energy's digits | row 2, columns 6 and 7 |
| the missiles' digits | row 2, columns 10 to 12 |
| the super missiles' digits | row 2, columns 14 and 15 |
| the power bombs' digits | row 2, columns 17 and 18 |
| the reserve sign (it reads AUTO) | rows 0 to 2, columns 8 and 9 |
| the missiles' icon | rows 0 and 1, columns 10 to 12 |
| the super missiles' icon | rows 0 and 1, columns 14 and 15 |
| the power bombs' icon | rows 0 and 1, columns 17 and 18 |
| the grapple beam's icon | rows 0 and 1, columns 20 and 21 |
| the x-ray scope's icon | rows 0 and 1, columns 23 and 24 |
| the mini-map's window | rows 0 to 2, columns 26 to 30; her cell is row 1, column 28 |

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| frame count | 0 to 255: the count of frames the game has finished, mod 256, as this frame begins (a frame the game didn't finish in time, a lag frame, isn't one) | `OneShotFlash`, `HerBlink` |
| energy | now and capacity, counts | `EnergyGauge` |
| reserve | the energy in the reserve tanks, a count; whether they refill on their own (`auto`) | `AutoReserveSign` |
| missiles, super missiles, power bombs | each a count and a capacity | `AmmoCounts`, `CyclingItems` |
| equipment | whether she has the grapple beam on, and the x-ray scope | `CyclingItems` |
| item select, item cancel | each: pressed this frame (it went down), or (cancel) held from before | `CyclingItems`, `CancellingTheItem`, `OneShotItem` |
| can switch | `bool`: the player controls her and she doesn't face the screen | `CyclingItems`, `CancellingTheItem` |
| the shot's cancel | `bool`: the weapons cancelled the selection after a shot this frame | `CancellingTheItem` |
| why no click | none, or she spin jumps, she wall jumps, the grapple beam is out, time is stopped | `SwitchClick` |
| collected | an item whose pickup this frame draws its icon, or none | `IconsAppear` |
| the mini-map runs | `bool`: false while she is held at a station (save, refill, map), drained, entering or leaving the gunship, or while the game's opening shows her (the title screen's demos still run it) | `MiniMapStill` |
| the mini-map turned on | `bool`, an event held until a frame reads it, not a state: true when something turned the mini-map on since the last frame of play and no boss was encountered after it, false on every other frame. A turn-on outside play (the next room, a load, a new game or the demos' load) reaches the frame of play that follows it; one during this frame before the mini-map's turn (Crocomire's fall into the acid) reaches this frame. Whether the mini-map is on is the State `the mini-map is off` (§1) | `MiniMapStill` |
| a boss encountered | the boss she encounters: Kraid, Draygon, Crocomire, Phantoon, or Ridley in Norfair or in Ceres, on entering its room while it lives; a step of its own, outside a frame of play (supplied by the door and the load flow) | `BossBlanksTheMap` |
| Mother Brain falls | `bool`: her first form falls this frame, her brain in the tank destroyed (supplied by her fight) | `BossBlanksTheMap` |
| her position | x, y: whole pixels in her room, after this frame's moves and before a pose change places her | `Exploring`, `MiniMapStill` |
| her room | its top-left cell on her map, and its size in 16 px blocks across and down | `Exploring`, `MiniMapStill` |
| her map | each cell's picture with its flips; the cells it charts; whether she has collected it; its beyond pictures (`LastColumnsShift`) | `MiniMapWindow`, `DiagonalPicture` |
| the HUD's pictures | content, by name (§1.4), from the importer | every entry that draws a cell |

### 1.4 The HUD's pictures

The HUD's pictures are content: the importer supplies them by name, in this format, and the
vectors use an invented set in the same format. A picture is a number with its flips, left to
right and top to bottom; her map's pictures are numbered the same way. A colour set is one of the
HUD's eight palettes. Every cell the HUD draws is drawn in front of the room.

Cells: picture, flips, colours

| Name | Pictures | Colours |
| --- | --- | --- |
| blank | one | the blank's |
| digit 0 to 9 | one per digit | the digits' |
| full tank, empty tank | one each | full, empty |
| reserve sign | three rows of two, each with its flips | lit, dim |
| the missiles' icon | two rows of three, each with its flips | plain or highlighted |
| each other item's icon | two rows of two, each with its flips | plain or highlighted |
| the map's blank | one | charted |
| the diagonal | one: a map cell drawn as a diagonal (`DiagonalPicture`) | |

Colour sets by role: the blank's and the digits' 3; full 2, empty 5; the sign lit 7, dim 3; the
icons plain 5, highlighted 4; on the mini-map explored 2, charted 3, her blink 7.

Every picture is drawn with its flips. The blank's flips and the diagonal's are read by no rule:
the blank cell a highlight tests is the blank's picture unflipped (`SelectionHighlight`), a
pickup tests the blank's picture alone (`IconsAppear`), and the diagonal's test compares numbers
(`DiagonalPicture`).

The pictures are always supplied: the spec defines the HUD only with a set the importer gives it,
as the game always has its own. A stand-in for pictures not yet imported is outside the spec: no
rule says what it shows, and no vector or record checks it.

### 1.5 The inputs' domain

The spec defines the HUD on these inputs; outside them the original reads past its tables, and
this spec chooses instead (stated simplifications, docs/LEGACY.md):

- missiles, count and capacity, at most 999; super missiles and power bombs at most 99. No vanilla
  capacity comes near (230, 50 and 50).
- A cell outside the map's 64 columns and 32 rows (a row before 0 or past 31; a column past 63,
  except the beyond pictures `LastColumnsShift` reads) is unexplored and uncharted, and its picture
  is the map's blank; exploring it changes nothing. The window reaches such a cell with her cell
  in the map's last row or below it, in its row 1 at columns 0 and 1 (`FirstColumnsShift`), or in
  its rows 30 and 31 at columns 62 and 63 (`LastColumnsShift`); a boss's screens can too. No
  vanilla room lies there.

---

## 2. The status bar

### EnergyGauge
- kind: mechanic.
- summary: her energy's last two digits, and one tank per 100 of her capacity, full while whole
  hundreds remain; drawn again only when her energy changes.
- params: at most 14 tanks (§1.2).
- switch: none.
- Her energy is not the energy last shown → the energy last shown becomes it, and the tanks and
  digits are drawn [C-R1, C-R3, C-R4, C-tanks-empty-row]
- Drawing, tank `i` (from 0, in fill order) for `i < min(capacity div 100, 14)`, with
  `i < energy div 100` → full [C-R3, C-R4, C-tanks-fourteen]
- Drawing, tank `i` for `i < min(capacity div 100, 14)`, with `i ≥ energy div 100` → empty [C-R3,
  C-R4, C-tanks-empty-row, C-flips-drawn]
- Drawing, the digits → `(energy mod 100) div 10` and `energy mod 10` [C-R1, C-R4, C-tanks-fourteen,
  C-flips-drawn]
- So a tank past her capacity, and every cell while her energy is the one last shown, keeps what
  it shows (C-R5, C-R6, C-energy-unchanged-stale).
- checked: oracle (every harness case); constructed (their twins, C-tanks-fourteen,
  C-tanks-empty-row, C-energy-unchanged-stale, C-flips-drawn).

### AmmoCounts
- kind: mechanic.
- summary: the missiles' count in three digits, the super missiles' and the power bombs' in two,
  each drawn again only when it changes, and only once she can carry some.
- params: the digits' cells (§1.2).
- switch: none.
- A count whose capacity isn't 0 and isn't the count last shown → the count last shown becomes
  it, and its digits are drawn [C-R2, C-R7, C-R8, C-icon-drawn, C-ammo-unchanged-stale]
- Missiles' digits → `n div 100`, `(n mod 100) div 10`, `n mod 10` [C-R2, C-R7]
- Super missiles' and power bombs' digits → `n div 10`, `n mod 10` [C-R2, C-R8]
- So with a capacity of 0 nothing is drawn and the count last shown stays (C-ammo-no-capacity),
  and a count that is the one last shown leaves its cells as they are (C-ammo-unchanged-stale).
- checked: oracle (the harness cases R2, R7, R8); constructed (C-R2, C-R7, C-R8,
  C-ammo-no-capacity, C-ammo-unchanged-stale).

### AutoReserveSign
- kind: mechanic.
- summary: while the reserve tanks refill on their own, the reserve sign shows, lit with energy in
  them and dim when they are empty.
- params: the sign's pictures (§1.4).
- switch: none.
- Reserve tanks on `auto`, with energy in them → the sign, lit [C-R2, C-R6, C-sign-lights]
- Reserve tanks on `auto`, empty → the sign, dim [C-R1, C-R3, C-sign-dims]
- So set to refill by hand, the sign's cells keep what they show (C-sign-manual, C-R5).
- checked: oracle (the harness cases R1, R2, R3, R6); constructed (C-R1, C-R2, C-R3, C-R6,
  C-sign-manual, C-sign-lights, C-sign-dims).

### SelectionHighlight
- kind: mechanic.
- summary: the selected item's icon shows in the highlighted colours, every other in the plain
  ones; when the selection changes the new icon lights and the old one goes plain.
- params: the icons' cells (§1.2), the colour sets (§1.4).
- switch: none.
- The selected item is not the one last shown → its icon takes the highlighted colours, the
  last shown's icon takes the plain ones, and the item last shown becomes it [C-R9, C-R11,
  C-R12, C-R13, C-R14, C-select-skip]
- An icon taking colours, a cell of it showing the blank cell (the blank's picture in the
  blank's colours, unflipped) → that cell keeps it; every other cell of the icon keeps its picture
  and flips [C-highlight-blank, C-highlight-tinted, C-highlight-blank-flipped, C-R11]
- So a cell showing the blank's picture in the blank's colours, but flipped, isn't the blank
  cell: it takes the colours, its flips kept (C-highlight-blank-flipped).
- So none selected has no icon to light (C-R13, C-R15, C-shot-cancels), and the flash
  (`OneShotFlash`) colours an icon the same way.
- So the highlight tests the whole cell and a pickup (`IconsAppear`) the picture alone, on
  purpose: the blank's picture in other colours takes the colours here (C-highlight-tinted) and
  counts as blank there (C-icon-drawn-tinted).
- checked: oracle (the harness cases R9 to R15); constructed (C-R9, C-R10, C-R11, C-R12,
  C-R13, C-R14, C-R15, C-highlight-blank, C-highlight-tinted,
  C-highlight-blank-flipped).

### SwitchClick
- kind: output.
- summary: a click sounds when the highlight moves, unless she spin jumps, wall jumps, the grapple
  beam is out or time is stopped.
- params: none.
- switch: none.
- The highlight moves (`SelectionHighlight`'s first rule), and she isn't spin jumping or wall
  jumping, the grapple beam isn't out and time isn't stopped → a `Click` sound request [C-R9, C-R11,
  C-R13, C-select-skip, C-R10, C-R15, C-click-wall-jump, C-click-grapple, C-click-spin,
  C-click-time-stopped]
- So the highlight moving while she spin jumps (C-R10), wall jumps (C-click-wall-jump), with the
  grapple beam out (C-click-grapple) or with time stopped (C-R15) makes no click.
- So a click is a request: the sound domain decides when, or whether, it is heard.
- checked: constructed (the events of every vector; a record can't show the request). The harness
  cases reach the spin jump (R2, R10), the grapple beam (R7) and time stopped (R15).

### OneShotFlash
- kind: mechanic.
- summary: the icon of the one-shot item flashes, 16 frames plain, then 16 highlighted.
- params: 16 frames each way.
- switch: **Parameter** (the frames each way; the reference rules hold 16).
- Every frame, after the highlight, the selection the one-shot item, `frame mod 32 ≥ 16` →
  its icon takes the highlighted colours [C-flash-on, C-one-shot-set]
- Every frame, after the highlight, the selection the one-shot item, `frame mod 32 < 16` →
  its icon takes the plain colours, though it is selected [C-flash-off]
- checked: constructed (no recorded row sets the one-shot item).

### IconsAppear
- kind: mechanic.
- summary: a pickup puts its icon on the HUD in the plain colours, unless the icon already
  shows.
- params: the icons' pictures (§1.4).
- switch: none.
- A pickup this frame, its icon's top-left cell's picture the blank's, whatever its colours
  and flips → its icon is drawn, plain [C-icon-drawn, C-icon-grapple, C-icon-drawn-tinted]
- A pickup, its icon's top-left cell any other picture → its cells keep what they show
  [C-icon-kept]
- checked: constructed (no recorded row has a pickup with an icon).

---

## 3. The selection

### CyclingItems
- kind: mechanic.
- summary: Select moves the selection to the next item she has, in the HUD's order, then back to
  none.
- params: the order (§1.1).
- switch: none.
- Item select pressed this frame, she can switch, item cancel not pressed this frame → the
  selection moves to the next item in the order after it, wrapping after the x-ray scope to none
  [C-R9, C-R11, C-R12, C-R15, C-select-nothing-to-have]
- The next item is missiles, super missiles or power bombs with a count of 0, the grapple beam
  she doesn't have on, or the x-ray scope she doesn't have on → it moves past it to the one
  after [C-select-skip, C-select-lacks, C-select-nothing-to-have]
- She can't switch → the selection and the one-shot item stay as they are [C-cannot-switch]
- So none is always there to land on (C-R15, C-select-nothing-to-have).
- checked: oracle (the harness cases R9 to R12, R15); constructed (C-R9, C-R10, C-R11, C-R12, C-R15,
  C-select-skip, C-select-lacks, C-cannot-switch).

### CancellingTheItem
- kind: mechanic.
- summary: item cancel puts the selection back to none; so does a shot the weapons cancel.
- params: none.
- switch: none.
- Item cancel pressed this frame, she can switch → the selection becomes none, and with it
  the one-shot item, whatever Select does [C-R13, C-R14, C-cancel-wins]
- The shot's cancel, after the cycling → the selection becomes none, and with it the one-shot
  item [C-shot-cancels, C-shot-cancels-cannot-switch]
- So the shot's cancel holds whether or not she can switch (C-shot-cancels-cannot-switch).
- checked: oracle (the harness cases R13, R14); constructed (C-R13, C-R14, C-cancel-wins,
  C-shot-cancels, C-shot-cancels-cannot-switch).

### OneShotItem
- kind: mechanic.
- summary: selecting an item with item cancel held makes it the one-shot item, the one the
  weapons drop after a shot; selecting without it clears that.
- params: none.
- switch: none.
- Select moves the selection with item cancel held → the new selection is the one-shot item
  (none stays none) [C-one-shot-set]
- Select moves the selection without item cancel held → the new selection isn't the one-shot
  item [C-one-shot-cleared, C-R9]
- checked: constructed (no recorded row holds item cancel while selecting).

### Switching
- kind: output.
- summary: each switch tells the weapons and her look which item she switched to.
- params: none.
- switch: none.
- Select or item cancel acting (`CyclingItems`' first rule, `CancellingTheItem`'s first) → a
  `Switched` event with the item it lands on, the same item or none included [C-R9, C-R13,
  C-select-nothing-to-have, C-cancel-wins]
- checked: constructed (the event isn't in a record; its effects are other domains').

---

## 4. The mini-map

### Exploring
- kind: mechanic.
- summary: the cell she is in becomes explored.
- params: none.
- switch: none.
- Her cell is column `cx = (rx + x div 256) mod 64` and row `cy = ry + y div 256` of her map:
  `rx`, `ry` her room's top-left cell, `x`, `y` where she is.
- The mini-map updates (`MiniMapStill`), her cell `(cx, cy)` on the map (§1.5) → it joins the
  explored cells [C-R16, C-R17, C-mm-diagonal-512-apart, C-mm-her-cell-outside,
  C-mm-column-wraps]
- So her room's cell in column 63 and she in its second screen across puts her cell in column 0
  (C-mm-column-wraps).
- checked: oracle (the harness cases R16, R17); constructed (C-R16, C-R17,
  C-mm-her-cell-outside, C-mm-column-wraps).

### MiniMapWindow
- kind: mechanic.
- summary: the mini-map shows the 5 × 3 cells around her: the explored ones with their pictures,
  the ones the map charts once she has it, the rest blank.
- params: the window (§1.2), the colour sets (§1.4).
- switch: none.
- The mini-map updates → the window's cell `k` columns and `j` rows from hers, for `k` from −2
  to 2 and `j` from −1 to 1, shows the map's cell `(cx + k, cy + j)`; the map's columns run on
  from 31 to 32 [C-R1, C-R8, C-R13, C-mm-halves]
- The cell it shows explored → its picture, flips kept, in the explored colours [C-R1, C-R16,
  C-mm-charted]
- Not explored, the map charts it and she has collected the map → its picture, flips kept, in
  the charted colours [C-mm-charted, C-mm-first-columns-0]
- Neither, or the cell it reads lies outside the map (§1.5) → the map's blank in the charted
  colours [C-R1, C-R16, C-mm-charted-uncollected, C-mm-last-row, C-mm-last-columns-row-30,
  C-mm-first-columns-row-1]
- checked: oracle (every harness case with a mini-map); constructed (their twins,
  C-mm-charted).

### HerBlink
- kind: mechanic.
- summary: her cell blinks, 8 frames in the blink colours and 8 as drawn.
- params: 8 frames each way.
- switch: **Parameter** (the frames each way; the reference rules hold 8).
- After the window is drawn, `frame mod 16 < 8` → her window cell takes the blink colours [C-R1,
  C-R2, C-mm-charted]
- So with `frame mod 16 ≥ 8` it shows as `MiniMapWindow` drew it (C-R3, C-R16).
- checked: oracle (every harness case with a mini-map); constructed (their twins).

### MiniMapStill
- kind: mechanic.
- summary: some frames the mini-map doesn't update: nothing is explored and its cells keep what
  they show, blink included.
- params: none.
- switch: none.
- The mini-map turned on → it isn't off, before this frame's update [C-mm-turned-on,
  C-mm-turned-on-not-running]
- The mini-map doesn't run, or is off, or `x div 16 ≥ w` or `y div 16 ≥ h` (`w`, `h` her room's
  blocks, `x` and `y` read as 0 to 65,535) → it doesn't update [C-R18, C-mm-off,
  C-mm-not-running, C-mm-boss, C-mm-outside, C-mm-outside-left, C-mm-outside-below]
- So a position left of or above the room (FFF8) reads as past it (C-mm-outside-left).
- So a turn-on holds while the mini-map doesn't run, though it doesn't update
  (C-mm-turned-on-not-running).
- checked: oracle (the harness case R18, a recorded frame with the mini-map off); constructed
  (the others, C-mm-turned-on, C-mm-turned-on-not-running, C-mm-outside-below).

### BossBlanksTheMap
- kind: mechanic.
- summary: on entering the room of Kraid, Draygon, Crocomire, Phantoon or Ridley (Ceres's or
  Norfair's) while it lives, and when Mother Brain's first form falls (her brain in the tank is
  destroyed), the mini-map goes blank and the cells the boss names count as explored. Entering
  the room is a step of its own, outside a frame of play; her fall comes during a frame.
- params: the cells each boss explores, as screens across and down from her room's top-left
  cell:

  | Boss | Screens |
  | --- | --- |
  | Kraid | (0, 0), (1, 0), (0, 1), (1, 1) |
  | Draygon | (0, 0), (1, 0), (0, 1), (1, 1) |
  | Crocomire | (3, 0), (4, 0), (5, 0) |
  | Phantoon | (0, 0) |
  | Mother Brain | (0, 0), (1, 0), (2, 0), (3, 0) |
  | Ridley, in Norfair | (0, 0), (0, 1) |
  | Ridley, in Ceres | none |

- switch: **Parameter** (each boss's screens; the reference rules hold the table).
- A boss encountered, or Mother Brain falls → every cell of the window shows the map's blank
  in the charted colours, and the mini-map is off [C-mm-boss, C-mm-boss-across, C-mm-boss-alone,
  C-mm-turned-on-and-fall, C-mm-boss-not-running, C-mm-boss-while-off]
- The same, a screen `(sx, sy)` the boss names → the map's cell `((rx + sx) mod 64, ry + sy)`
  joins the explored cells [C-mm-boss, C-mm-boss-across, C-mm-boss-alone, C-mm-boss-draygon,
  C-mm-boss-phantoon, C-mm-boss-ridley, C-mm-boss-alone-none]
- A boss encountered → the window, the explored cells and the mini-map's off as above; nothing
  else changes: the mini-map doesn't update or blink, the selection stays, and the status bar
  isn't drawn [C-mm-boss-alone, C-mm-boss-alone-none]
- So Mother Brain's fall comes before the mini-map's turn and leaves it off: that frame's update
  doesn't happen (`MiniMapStill`), and the frame goes on (C-mm-boss).
- So her fall blanks the map and explores her screens whether or not the mini-map runs
  (C-mm-boss-not-running), and when it is already off (C-mm-boss-while-off).
- checked: constructed (no recorded row reaches these bosses).

### DiagonalPicture
- kind: quirk.
- summary: where her cell shows the diagonal picture (cells drawn as a diagonal), the cell
  over hers becomes explored too.
- params: the diagonal (§1.4), a picture the importer names.
- switch: none.
- Her window cell shown explored, its picture's number `p` and the diagonal's `d` with
  `p mod 512 = d mod 512` → the cell over hers, `(cx, cy − 1)`, joins the explored cells, shown
  from the next frame [C-mm-diagonal, C-mm-diagonal-512-apart, C-mm-diagonal-shifted]
- So two numbers 512 apart are the same picture here, whichever is the larger: both are read mod
  512 (C-mm-diagonal, C-mm-diagonal-512-apart).
- So only her own window cell counts: the diagonal in the cell over hers
  (C-mm-diagonal-above-only), or on an explored cell beside hers or below it
  (C-mm-diagonal-beside), explores nothing.
- So where her window cell shows another cell's state (`FirstColumnsShift`, `LastColumnsShift`),
  that state decides: her window cell shown charted leaves the cell over hers unexplored, though
  she has explored her own (C-mm-diagonal-charted); shown explored, it joins
  (C-mm-diagonal-shifted).
- checked: constructed (no recorded row passes such a room).

### FirstColumnsShift
- kind: quirk.
- summary: with her cell in the map's columns 0 and 1, the mini-map shows the cells 32 columns
  along and a row higher.
- params: none.
- switch: none.
- `cx ≤ 1` → each window cell shows the map's cell `(cx + k + 32, cy + j − 1)` in place of
  `(cx + k, cy + j)`: its picture and whether it is explored or charted [C-mm-first-columns-0,
  C-mm-first-columns-1]
- So her own window cell shows that other cell too, and `DiagonalPicture` reads its picture; the
  cells she explores are still her own (C-mm-first-columns-0).
- checked: constructed (no recorded row reaches the map's edges). No cell a vanilla map draws or
  charts lies in its columns 0, 1, 62 or 63, so this is unreachable in vanilla; it is kept for
  randomised maps, which can place rooms there.

### LastColumnsShift
- kind: quirk.
- summary: with her cell in the map's columns 62 and 63, the mini-map's pictures are its own
  cells', but whether each is explored or charted is read 32 columns back and a row lower, and
  past column 63 the pictures are another map's.
- params: none.
- switch: none.
- `cx ≥ 62` → each window cell takes its explored and charted state from the map's cell
  `(cx + k − 32, cy + j + 1)`, its picture still from `(cx + k, cy + j)` [C-mm-last-columns-62,
  C-mm-last-columns-63, C-mm-diagonal-charted]
- `cx ≥ 62`, `cx + k ≥ 64` → the picture is the map's beyond picture `(cx + k − 64, cy + j)`
  [C-mm-last-columns-62, C-mm-last-columns-63]
- So past column 63 the state is that of columns 32 and 33, a row lower (C-mm-last-columns-63).
- checked: constructed (no recorded row reaches the map's edges). No cell a vanilla map draws or
  charts lies in its columns 0, 1, 62 or 63, so this is unreachable in vanilla; it is kept for
  randomised maps, which can place rooms there.

---

## 5. Order that matters

- The selection changes first (Select, item cancel), then the weapons may cancel it after a shot
  and Mother Brain may fall, then the mini-map updates (unless she fell), then the status bar is
  drawn from the selection as it then is.
- In the status bar: the icon a pickup draws comes before the highlight and the flash, so a
  selected item's new icon is lit; the flash has the last word on the colours of the one-shot
  item.
- In the mini-map: a turn-on comes before Mother Brain's fall, and both before the update. So in
  a frame where the mini-map is turned on and her first form falls, the fall has the last word:
  the mini-map is off after it and doesn't update (C-mm-turned-on-and-fall). No vanilla room
  holds both (the one turn-on during play is Crocomire's fall, in another area), so this order
  is the spec's choice (docs/LEGACY.md). Her cell is explored before the window reads the
  explored cells; the cell `DiagonalPicture` adds over hers is explored after, so it shows from
  the next frame; the blink comes last.
- Where she is: after this frame's moves and before a pose change places her. On a frame whose
  pose changes, a placement that takes her into another screen comes after the mini-map has read
  her.

## 6. Out of scope, and who supplies it

- **The weapons:** firing, and the shot's cancel: a shot of the one-shot item, or the
  last of the selected ammunition, puts the selection back to none. They read the selection, the
  one-shot item and `Switched` (a switch drops the charge).
- **Her look:** her gun opens or closes when the selection changes, and a switch reloads her
  suit's colours; they read the selection and `Switched`.
- **Energy, reserves and pickups:** the values this domain shows, and the pickup that calls
  `IconsAppear`.
- **Progress:** the area, the map collected, the explored cells of the other areas.
- **The load flow and the pause screen:** the HUD's first contents (the status bar's frame, the
  icons she has, the counts).
- **Doors:** the frame count's reset at a door.
- **Doors and the load flow:** `a boss encountered`, on entering the room of Kraid,
  Draygon, Crocomire, Phantoon or Ridley (Ceres's or Norfair's) while it lives, outside a frame
  of play; and `the mini-map turned on`, at each new room ahead of its boss, at a load, at a
  new game and at the demos' load.
- **The bosses' domain (Crocomire's fight):** `the mini-map turned on` when Crocomire falls into
  the acid, during a frame of play.
- **Her control (samus-motion, the stations, cinematics and the intro):** `the mini-map runs`,
  false while she is held at a station, drained, entering or leaving the gunship, or shown by the
  game's opening.
- **The bosses' domain (the fight with Mother Brain):** `Mother Brain falls`, when her first
  form falls, her brain in the tank destroyed, during a frame of play.
- **The importer:** the HUD's pictures (§1.4) and her map's.
- **The video encoder:** moving the HUD's cells to the screen every frame.
- **The sound domain:** when the `Click` request is heard, or dropped.

## 7. Open questions

1. Both of the mini-map's column rules, the diagonal picture, the bosses' blank, the one-shot item
   and a pickup's icon have constructed vectors only; a recording of a pickup, of a one-shot item
   and of Kraid's fight would give the harness cases for most of them.
