# File select: the three files, the cursor and the choice

What the player sees from the dark frame that ends the title until the options menu begins:
black while file select loads, the menu (three files, copy, clear and exit) fading in, the
cursor moved with Up and Down, a file chosen with A or Start, its helmet turning, and the menu
fading out. Vectors: [screens-file-select.toml](../../scenarios/vectors/screens-file-select.toml).

## Header

**Scope.** File select's main menu, one frame end at a time, from the frame end after the
title's `FileSelectBegins` (the [title spec](screens-title.md)) until the frame that hands over
to the options menu: its stage, the screen, the cursor, the fade's pace, the file chosen, the
sounds it asks for, and where its load holds one frame of work across a frame end. This
introduces file select's owner. There is no old production owner to replace.

**Reads.**
- From the platform, at each frame end: whether the game was waiting for it, its work or a part
  of it done (borrowed until timing is modelled; MODEL decision 35). Such a frame end is a
  finished one.
- From the controller, for each finished frame: the buttons just pressed as it began. A, B,
  Start, Up and Down are acted on.
- From the title's owner, as file select begins: the screen.
- From save memory's owner (P2), on the load's fourth finished frame end: which of the three
  slots holds a file, and the slot chosen last, if save memory holds a valid one.

**Reports.** The screen at each frame end, the cursor, the file
chosen (to save memory's owner, which records the choice and loads the file there or starts a
new one), the sounds asked for in each frame (`MenuOpens`, `CursorMoved`, `ChoiceMade`), whether the
frame of work goes on through the frame end (read by the game's frame owner, which then begins no
new frame of work, and so takes no random number, at the next finished frame end), and which way out
file select took: the options menu, back to the title, copy or clear.

**Leaves to others.** The count of finished frames and the shared random number's draws, which
the game's frame owner carries (one count per finished frame end, one draw per frame of work
begun). The pictures: the menu's text and borders, each file's energy and time,
the helmets and their turning, the cursor's sprite and its animation, the background of the
planet and the stars, and the colours (imported content and their owners' rules). Save memory
and its files: checking each slot, recording the choice, loading a file and making a new one
(P2). The sounds themselves. Copying and clearing a file, and the title again, each a later
piece; the options menu, the next one.

**Notation.** A frame end and a frame of work are as in the title spec. A brightness level runs
from 0 (dark) to 15 (full); the screen is blank or lit at its level. The menu's items, top to
bottom, are slot A, slot B, slot C, copy, clear and exit. A slot holds a file or is empty.

## Memory

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| stage | `FileSelectStage`: `Loading`, `Brightening`, `Choosing`, `Confirming`, `Leaving`, and the ways out `OptionsBegin`, `TitleReturns`, `CopyBegins`, `ClearBegins` | every entry |
| load step | `LoadStep`, finished frame ends of the load so far, 0 to 4, only while `Loading` | `MenuLoads` |
| screen | `Screen` (the power-on spec's type); `Lit(0)` to `Lit(14)` while `Brightening`, `Lit(1)` to `Lit(15)` while `Confirming` and `Leaving`, `Lit(15)` while `Choosing` | `MenuLoads`, `MenuBrightens`, `MenuDims`, output |
| cursor | `MenuItem`: `SlotA`, `SlotB`, `SlotC`, `Copy`, `Clear`, `Exit`; from the load's fourth finished frame end | `CursorMoves`, `ItemPressed`, output |
| copy and clear offered | `bool`: some slot holds a file; from the load's fourth finished frame end | `CursorMoves` |
| fade wait | `FadeWait`, finished frames before the next level, 0 to the menu pace less one; from the load's fourth finished frame end | `MenuBrightens`, `MenuDims` |
| file chosen | `Slot`: `First`, `Second` or `Third`, the top three items, from `Confirming` on | output |
| turn left | `TurnLeft`, frames of work left of the chosen helmet's turn, 1 to the helmet turn, only while `Confirming` | `HelmetTurns` |
| sounds asked for | `MenuSound` list, oldest first, for the frame end: `MenuOpens`, `CursorMoved`, `ChoiceMade` | output |
| frame goes on | `bool`, for the frame end: the frame of work goes on through it | output |

All but the last two are the owner's complete State: a caller can set any combination the table
allows, read it back, carry it forward and restore it. As file select begins, the State is the
title's screen (`Blank(0)`) and `Loading` at step 0.

## Rules

### MenuLoads

- kind: mechanic
- summary: black while file select loads over five finished frame ends; the fourth unblanks the
  screen at level 0.
- params: none
- switch: fixed
- `Loading` at step `s`, a finished frame, `s + 1` below 5 → step `s + 1`; at step 1 the screen
  `Blank(0)` and the frame goes on [R-load-first]; at step 2 asks for `MenuOpens`
  [R-load-second]; at step 3 nothing more [R-load-third]; at step 4 `Lit(0)` with the fade wait
  at the menu pace less one, and the frame goes on [R-load-unblanks, A-pace-3-unblanks].
- `Loading` at step 4, a finished frame → `Brightening` [R-load-last].
- `Loading`, a frame end with no frame finished → no change [R-load-busy].
- checked: oracle (R-load-first, R-load-busy, R-load-second, R-load-third, R-load-unblanks,
  R-load-last).

### CursorStarts

- kind: mechanic
- summary: the cursor starts on the slot chosen last, or on slot A; copy and clear are offered
  when a slot holds a file.
- params: none
- switch: fixed
- The load's fourth finished frame end, save memory holding a valid slot chosen last → the
  cursor on that slot, copy and clear offered when some slot holds a file
  [C-cursor-last-slot]; holding none → `SlotA` [R-load-unblanks].
- Copy and clear are not offered when no slot holds a file [R-load-unblanks].
- checked: oracle (R-load-unblanks); original only for a slot chosen last.

### MenuBrightens

- kind: mechanic
- summary: the menu brightens one level every menu pace of finished frames, up to full.
- params: menu pace `p`, finished frames per level, 1 to 256 (`menu_pace`, reference 2)
- switch: parameter
- `Brightening` with a fade wait `w` above 0, a finished frame → `w - 1` [R-brighten-waits].
- `Brightening` with the fade wait at 0, a finished frame → one level up, the fade wait back to
  `p - 1`; at 15 → `Choosing` [R-brighten-step, R-brighten-full, A-pace-1-brightens].
- `Brightening`, a frame end with no frame finished → no change [C-brighten-busy].
- checked: oracle (R-brighten-waits, R-brighten-step, R-brighten-full); original only for other
  paces.

### ItemPressed

- kind: mechanic
- summary: A or Start acts on the item under the cursor.
- params: none
- switch: fixed
- `Choosing`, A or Start just pressed, the cursor on a slot → `Confirming` with that file
  chosen, reported to save memory's owner, the whole helmet turn left, and asks for `ChoiceMade`
  [R-start-new-file, C-press-beats-up, A-helmet-turn-3].
- The same on `Copy` → `CopyBegins`, on `Clear` → `ClearBegins`, each asking for
  `CursorMoved` [C-copy-begins, C-clear-begins]; on `Exit` → `TitleReturns`, asking for none
  [C-exit-returns].
- `Choosing`, a frame end with no frame finished → no change [C-choosing-busy].
- checked: oracle (R-start-new-file); original only for the other items.

### CursorMoves

- kind: mechanic
- summary: Up and Down move the cursor round the menu, skipping copy and clear when they are not
  offered; B goes back to the title.
- params: none
- switch: fixed
- `Choosing`, no A or Start, Up just pressed, copy and clear offered → the item above, from slot
  A round to exit, asking for `CursorMoved` [C-up-files, C-up-files-wraps].
- The same, not offered → the item above, from slot A round to exit, and from exit or copy to
  slot C [C-up-no-files-wraps, C-up-no-files-skips, C-up-beats-down, C-up-no-files-copy].
- The same, not offered, from clear → copy [C-up-no-files-clear].
- `Choosing`, no A, Start or Up, Down just pressed, offered → the item below, from exit round
  to slot A, asking for `CursorMoved` [C-down-files, C-down-files-wraps].
- The same, not offered → the item below, from slot C, copy or clear to exit, and from exit to
  slot A [C-down-no-files-skips, C-down-no-files-copy, C-down-no-files-clear,
  C-down-no-files-wraps].
- `Choosing`, B alone of these just pressed → `TitleReturns`, asking for `CursorMoved` twice
  [C-b-returns].
- `Choosing`, none of A, Start, Up, Down or B → no change [C-other-buttons].
- checked: original only (the movie moves no cursor).

### HelmetTurns

- kind: mechanic
- summary: once a file is chosen its helmet turns for a fixed number of frames of work; A or
  Start, or the turn's end, starts the fade-out.
- params: helmet turn `h`, frames of work, 1 to 255 (`helmet_turn`, reference 57)
- switch: parameter
- `Confirming` with `t` left, a finished frame → `t - 1` left; then A or Start just pressed, or
  `t - 1` at 0 → `Leaving` [R-confirm-press, C-helmet-turned, A-helmet-turn-2-ends]; otherwise
  no other change [C-helmet-turning].
- `Confirming`, a frame end with no frame finished → no change [C-confirming-busy].
- checked: oracle (R-confirm-press); original only for the turn's end.

### MenuDims

- kind: mechanic
- summary: leaving, the menu dims one level every menu pace of finished frames; at dark the
  options menu begins.
- params: the menu pace `p` of `MenuBrightens`
- switch: parameter
- `Leaving` with a fade wait `w` above 0, a finished frame → `w - 1` [R-leave-waits].
- `Leaving` with the fade wait at 0, a finished frame, level `l` above 1 → `Lit(l - 1)`, the
  fade wait back to `p - 1` [R-leave-step, A-pace-3-waits].
- The same at level 1 → `Blank(0)`, `OptionsBegin` [R-leave-dark, A-pace-3-dims].
- `Leaving`, a frame end with no frame finished → no change [C-leaving-busy].
- checked: oracle (R-leave-waits, R-leave-step, R-leave-dark); original only for other paces.

## Order that matters

In `Choosing`, A or Start is looked at first, then Up, then Down, then B; the first found acts
alone [C-press-beats-up, C-up-beats-down]. The frame that brings the menu to full makes it
`Choosing`, so a press acts from the next finished frame. Each way out (`OptionsBegin`,
`TitleReturns`, `CopyBegins`, `ClearBegins`) belongs to the piece that follows: a further frame
end there is Unsupported here [C-after-options, C-after-title-returns, C-after-copy-begins,
C-after-clear-begins].
