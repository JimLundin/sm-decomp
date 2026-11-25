# The options menu: settings, the special settings and starting the game

What the player sees from the dark frame that ends file select until the game starts: black
while the options menu loads, the menu fading in, the cursor moved with Up and Down, the
language switched, the screen breaking into growing blocks as it darkens and shrinking blocks as
it brightens on the way into the special settings and back, the special settings switched with
A, Left, Right or Start, and on start game the menu fading out. Vectors:
[screens-options.toml](../../scenarios/vectors/screens-options.toml).

## Header

**Scope.** The options menu and its special settings, one frame end at a time, from the frame
end after file select's `OptionsBegin` (the [file select spec](screens-file-select.md)) until
the dark frame after which the game starts: its stage, the screen, the cursor, the dissolve's
blocks, the language and the special settings, the sounds it asks for, and the way out it takes.
This introduces the options menu's owner. There is no old production owner to replace.

**Reads.**
- From the platform, at each frame end: whether the game had its work done by it (borrowed until
  timing is modelled; MODEL decision 35). Such a frame end is a finished one.
- From the controller, for each finished frame: the buttons just pressed as it began. A, B,
  Start, Up, Down, Left and Right are acted on.
- From save memory's owner (P2), as the menu begins: the chosen file's settings (the language,
  weapon-reset and moonwalk; a new file has all three off), and how the chosen file begins: with
  the intro (a new file), with the escape from the space station (a file saved after it), or in
  play.

**Reports.** The screen at each frame end, the cursor, the dissolve's block size, the settings,
the sounds asked for in each frame (`CursorMoved`, `Selected`), and the way out. The ways out
are three hand-offs of start game, back to file select, and the controller settings:
- `IntroBegins` (a new file; the any% movie's way): the intro reads the language, weapon-reset
  and moonwalk, the chosen file, and the controls unchanged. The frame that ends
  after the hand-off (568 in the movie) belongs to the intro and changes nothing on screen, so
  the intro's first piece starts with it;
- `EscapeBegins` (a file saved after the station's escape) and `PlayContinues` (a file that goes
  straight on in play), each with the same settings and file; the run refuses both as
  Unsupported until their pieces are owned;
- `FileSelectAgain` and `ControlsBegin`, later pieces.

**Leaves to others.** The count of finished frames and the shared random number's draws (the
game's frame owner). The pictures: the menu's text in each language, its borders, the cursor's
sprite, the highlights of the chosen settings, and the colours (imported content). The
controller settings screen and its bindings, a later piece; the controls are the chosen file's
and pass through unchanged here. What the settings do in play (their owners). The sounds
themselves.

**Notation.** A frame end and a finished frame are as in the title spec; levels and the screen
as in the power-on spec. The options' items, top to bottom, are start game, English text,
Japanese text, the controller settings and the special settings; the special settings' are
weapon-reset, moonwalk and end. A dissolve shows the screen in square blocks; a block size `n`
means blocks of `n + 1` pixels each side. Where a dissolve goes is the cursor's item: start game
(the options), the special settings or the controller settings.

## Memory

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| stage | `OptionsStage`: `Loading`, `Brightening`, `Choosing`, `Special`, `DissolvingOut`, `DissolvingIn`, `Leaving`, `Returning`, and the ways out `IntroBegins`, `EscapeBegins`, `PlayContinues`, `FileSelectAgain`, `ControlsBegin` | every entry |
| load step | `LoadStep`, 0 or 1, only while `Loading` | `OptionsLoad` |
| screen | `Screen` (the power-on spec's type); `Blank(0)` while `Loading`, `Lit(15)` while `Choosing` and `Special` | every entry, output |
| cursor | `OptionsItem` while `Brightening`, `Choosing`, `Leaving`, `Returning`; `SpecialItem` while `Special`; while dissolving, start game, the special settings or the controller settings, the dissolve's destination | `OptionsCursor`, `OptionsChoice`, `SpecialSettings`, `Dissolve`, output |
| block size | `BlockSize`, 0 to 15, only while dissolving | `Dissolve`, output |
| how the file begins | `FileBegins`: `Intro`, `Escape`; only while `Leaving` | `OptionsLeave` |
| settings | `Settings`: `japanese`, `weapon_reset`, `moonwalk`, each on or off | `OptionsChoice`, `SpecialSettings`, output |
| sounds asked for | `MenuSound` list, oldest first, for the frame end: `CursorMoved`, `Selected` | output |

All but the sounds are the owner's complete State: a caller can set any combination the table
allows, read it back, carry it forward and restore it. As the menu begins, the State is
`Loading` at step 0, the screen `Blank(0)` (file select always hands over dark) and the chosen
file's settings.

## Rules

### OptionsLoad

- kind: mechanic
- summary: black while the menu loads over two finished frame ends; the cursor starts on start
  game.
- params: none
- switch: fixed
- `Loading` at step 0, a finished frame → step 1 [R-load-first].
- `Loading` at step 1, a finished frame → `Brightening`, the cursor on start game [R-load-done].
- `Loading`, a frame end with no frame finished → no change [R-load-busy].
- checked: oracle (R-load-first, R-load-busy, R-load-done).

### OptionsBrighten

- kind: mechanic
- summary: the menu brightens one level per finished frame, up to full.
- params: none
- switch: fixed
- `Brightening` at level `l` (0 when blank), a finished frame → `Lit(l + 1)`; at 15 →
  `Choosing` [R-brighten-first, R-brighten-full].
- checked: oracle (R-brighten-first, R-brighten-full).

### OptionsCursor

- kind: mechanic
- summary: Up and Down move the cursor round the five items.
- params: none
- switch: fixed
- `Choosing`, a finished frame, Up just pressed → the item above, from start game round to the
  special settings, asking for `CursorMoved` [R-up-then-a, C-up-beats-down].
- The same, no Up, Down just pressed → the item below, from the special settings round to start
  game, asking for `CursorMoved` [C-down-wraps].
- checked: oracle (R-up-then-a); original only for Down.

### OptionsChoice

- kind: mechanic
- summary: after the cursor's move, B goes back to file select, or A or Start acts on the item
  under the cursor.
- params: none
- switch: fixed
- `Choosing`, a finished frame, B just pressed → `Returning` [C-b-returns].
- The same, no B, A or Start just pressed, asking for `Selected`: on start game, a file that
  begins with the intro or the escape → `Leaving`, keeping how it begins [R-start-game,
  C-escape-file]; a file that goes straight on in play → `PlayContinues` at once
  [C-resumes-play]; on English or Japanese text → the language switched, the cursor on start
  game [C-language, C-language-back]; on the controller or special settings → `DissolvingOut`
  with block size 0 [R-up-then-a, C-controls].
- `Choosing`, none of these → no change [C-other-buttons].
- checked: oracle (R-up-then-a, R-start-game); original only for the others.

### SpecialSettings

- kind: mechanic
- summary: Up and Down move round the three items; B or end goes back; A, Left, Right or Start
  switches the setting under the cursor.
- params: none
- switch: fixed
- `Special`, a finished frame, Up just pressed → the item above, round from weapon-reset to end;
  else Down → the item below, round from end to weapon-reset; each asking for `CursorMoved`
  [R-moonwalk-on, C-special-up-wraps].
- Then, B just pressed, or A, Left, Right or Start on end → `DissolvingOut` towards the options
  with block size 0, the cursor on start game, asking for `Selected` [R-special-back,
  C-special-end, C-special-b-over-toggle].
- Otherwise A, Left, Right or Start on weapon-reset or moonwalk → that setting switched, asking
  for `Selected` [R-moonwalk-on, C-weapon-reset, C-moonwalk-off].
- checked: oracle (R-moonwalk-on, R-special-back); original only for the others.

### Dissolve

- kind: mechanic
- summary: the screen darkens one level per finished frame as its blocks grow, then brightens one
  level per finished frame as they shrink, into the cursor's screen.
- params: none
- switch: fixed
- `DissolvingOut` at level `l`, block size `n`, a finished frame → level `l - 1` (`Blank(0)` at
  0), block size `min(15, n + 1)`; at dark → `DissolvingIn` [R-dissolve-out, R-dissolve-dark,
  C-controls-dark].
- `DissolvingIn` at level `l`, block size `n`, a finished frame → `Lit(l + 1)`, block size
  `max(0, n - 1)`; at 15 → the blocks gone and, by the cursor, `Choosing` on start game, `Special`
  on weapon-reset, or `ControlsBegin` [R-dissolve-in, R-dissolve-into-special,
  R-dissolve-into-options, C-controls-begin].
- A frame end with no frame finished → no change [C-busy-dissolve].
- checked: oracle (R-dissolve-out, R-dissolve-dark, R-dissolve-in, R-dissolve-into-special,
  R-dissolve-into-options); original only for the controller settings.

### OptionsLeave

- kind: mechanic
- summary: leaving for the game or for file select, the menu darkens one level per finished
  frame.
- params: none
- switch: fixed
- `Leaving` or `Returning` at level `l`, a finished frame → `Lit(l - 1)`; at 0 `Blank(0)` and
  `IntroBegins` or `EscapeBegins` by how the file begins, from `Leaving`, or `FileSelectAgain`
  from `Returning` [R-leaving, R-intro-begins, C-escape-begins, C-return-dark].
- checked: oracle (R-leaving, R-intro-begins); original only for the escape and the return.

## Order that matters

In `Choosing` and `Special` the cursor moves first, then B, then the choice acts on the item the
cursor is on after its move, so Up and A in one frame choose the item above [R-up-then-a] and
Down and Left switch the setting below [R-moonwalk-on]. B wins over A in the same frame
[C-b-returns, C-special-b-over-toggle]. The frame that chooses a dissolve or start game leaves
the screen as it was; the darkening starts on the next finished frame. A frame end in a way out
belongs to the piece that follows [C-after-intro-begins, C-after-escape-begins].
