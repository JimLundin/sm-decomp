# The camera following Samus: behaviour spec

What the player sees the screen do as Samus moves through a room: how the view follows her across
and down, how far ahead of her it looks, how fast it moves, and how the room keeps it from showing
what it may not. Written behaviour first, from the oracle's records over time. Vectors:
[`scenarios/vectors/camera.toml`](../../scenarios/vectors/camera.toml).

## Header

**Scope.** One domain, because the player sees one thing: the view following her. It covers the
view's position in the room on both axes, the point it aims at (the lead across, the height down),
the speed it moves at, the room's edges, the room's closed screens and how it eases out of them,
and what it remembers of her between frames. It reads her motion's outputs: her position after her
moves, the y a placement put her at, if any, whether she skids, whether she rises, her facing and
whether she moonwalks.

**What it reads from samus-motion.** Its step's outputs: her position, the y a placement put her
at, if any (where pose fit or a change of shape placed her), the skid, her facing and `rising` (§1.3), which samus-motion's `RisingAndFalling` gives: she is rising from the
rise's start until it ends, a rise at 0 and the frame that spends the rise included. Her vertical
speed can't stand in for it: at a rise at 0 it is 0, and on the frame that spends the rise it
already points down, yet both aim the camera as a rise does (C-rise-at-0, C-rise-spent). While a knockback moves her, `rising` and going backwards are the
knockback's (§5).

**Not in this spec** (each an input where it feeds us, §1.3; §5 names who supplies each): a door's
passage, which places the view in the next room; the cells that open and close screens as she
touches them; layer 2 and the backgrounds' scroll; the grapple beam's own camera and any other
domain that moves where the camera saw her; a boss that changes the lead setting; the room rules
that move the view after it follows; what stops time.

**Notation.** A quantity written `P.S` is a pixel part and a subpixel part, each 16 bits, in hex,
as in samus-motion: `2.C000` is 2.75 px. x grows rightward, y grows downward. A room is a grid of
screens, each 256 px square; the view is 256 px across.

---

## 1. What the player can tell about the camera

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| camera position | x, y: `P.S`; the view's top-left corner in the room | where the room is drawn on the screen | every entry; output |
| where it last saw her | x, y: `P.S` | the view's next move: its speed and whether she moved are measured from here | `FollowingHer`, `Remembering` |

In §2's formulas, on one axis: `c` is the camera's position, `s` hers and `e` where it last saw
her, all `P.S`, and `sx`, `ex`, `sy`, `ey` the pixel parts of hers and the seen one on each axis.
§3 says what its `c` is.

### 1.2 What it remembers

Only the two quantities above. A vector's start gives both; nothing else carries from one frame to
the next.

### 1.3 What it reads (Inputs)

| Input | Type | Read by |
| --- | --- | --- |
| her position | x, y: `P.S`, after everything that moved her this frame | `FollowingHer`, `LeadingHer`, `AimingHeight`, `Remembering` |
| the y a placement put her at, if any | `P.S`, or none: where pose fit or a change of shape placed her this frame (her motion's output); none on a frame nothing placed her, and then the camera follows her from where it last saw her | `Remembering` |
| her facing | `Left`, `Right`, `Screen` (facing the player; leads as `Right`) | `LeadingHer` |
| going backwards | `bool`: she skids (her motion's output), moonwalks (her pose), or takes a knockback (the damage domain); the vectors name which | `LeadingHer` |
| rising | `bool`: she is rising, from the rise's start until it ends, a rise at 0 and the frame that spends the rise included: samus-motion's output (`RisingAndFalling`), or the knockback's while one moves her (§5) | `AimingHeight` |
| the room's screens | a grid of the shared `ScreenKind` (`clean_types`, the kind the blocks' scroll cells write), so many screens across (`w`) and down (`h`), and the run of screens the room's loading left after its last screen (`BelowTheRoom`) | `ClosedScreens`, `ShortScreens`, `Settling`, `RoomEdges`, `ScreensRunOn` |
| the aim heights | the room's two distances in px: while she isn't rising, and while she rises | `AimingHeight` |
| lead setting | `Ahead`, `KeepLeft`, `KeepRight`, `Near` (listed alphabetically; the order means nothing); `Ahead` unless a boss sets another | `LeadingHer` |
| time stopped | `bool` | `TimeStopped` |

**Screen kinds**, listed alphabetically (the order means nothing):
- `Closed`: the view may not show it; a view that would is pushed back to its edge.
- `Open`: the view may show all of it.
- `Short`: like `Open`, but where the view can go no lower it stops with its top on the screen's
  top edge, the margin higher than on an `Open` one, so less of the screen's bottom is shown
  (`ShortScreens`).

The camera's screen kind is the shared `ScreenKind` of `clean_types`, which the blocks' scroll
cells write: the camera has no kind of its own. The room's loading can leave a screen that is none
of the three: in the run past a room's own screens (`BelowTheRoom`), and in a room whose list of
screens is shorter than the room, whose last screens are then read from whatever follows the list
(docs/LEGACY.md). The camera tells only `Closed` and `Short` from the rest, at every screen it
reads, so such a screen is `Open` to it, and it reaches the camera as `Open`.

**Which screens it looks at** (`ClosedScreens`): across, the row that holds the point 128 px below
the view's top edge; down, the column that holds the point 128 px right of its left edge. On an
axis, the camera's screen is the one that holds its near edge (left or top), and the next screen is
the one after it (right or below): across, the one after a row's last column is `ScreensRunOn`'s;
below the room's last row, `BelowTheRoom`'s run. A position's screen is found from its pixel part
read as 0 to `FFFF`, 256 px a screen: so a top above the room (`FFF0`) is in the 256th row, below
the room (`BelowTheRoom`), not in its first row (C-edge-down-from-above-short), and a position right
of the room is in a column past the row's last (`ScreensRunOn`).

---

## 2. Following her

### FollowingHer
- kind: mechanic.
- summary: when she moves, the view moves towards its aim as fast as she moved plus an extra
  amount (a pixel in the reference profile), and never past the aim.
- params: the extra pixel, `1.0000`.
- switch: **Parameter**, the extra pixel (params).
- On each axis, she moved when her pixel part is not where it last saw her: `sx ≠ ex`.
- Positions compare in two ways, both on 16-bit pixel parts:
  - `before(a, b)`: bit 15 of `(a − b) mod 2^16` is set. So a value left of or above the room
    (`FFF0`) is before 0, and of two values `8000` apart each is before the other.
  - `exceeds(a, b)`: `a > b` as plain numbers from 0 to `FFFF`. So `FFF0` exceeds every
    position in the room.
  - A rule compares by `before`, and its `max` and `min` pick by `before`, unless it says
    `exceeds`. The two agree while both values are in the room; they differ only for a camera
    another domain left outside it (§5), or `8000` or more from what it is compared with
    (C-edge-right-from-left, C-edge-down-from-above, C-settle-reach-from-left, C-push-left-far).
- She moved on an axis, its aim's pixel part isn't the camera's, and the aim isn't `before` the
  camera → it moves right or down by its speed, carrying through the subpixels, `c + speed`
  [C-right-onto-edge, C-rise-into-corner, C-down-into-last-row-moving-left,
  C-down-short-closed-below, C-moonwalk-right-to-closed-edge, C-lead-near-right]
- She moved on an axis, and its aim's pixel part is `before` the camera's → it moves left or up by
  its speed, `c − speed` [C-left-past-edge, C-up-to-aim, C-skid-rising-down-to-bottom,
  C-rise-up-to-closed-edge, C-rise-up-to-closed-edge-moving-left, C-aim-half-way]
- So an aim exactly `8000` from the camera is `before` it, and the camera moves left or up
  (C-aim-half-way).
- Her pixel part is not `before` the one where it last saw her → its speed is `(s − e) + 1.0000`, on
  the whole `P.S` mod 2^32 [C-down-within-margin, C-right-onto-edge, C-down-to-bottom-31,
  C-rise-up-to-closed-edge, C-down-short-closed-below]
- Her pixel part is `before` it → its speed is `(e − s) + 1.0000` [C-aim-above-top,
  C-left-past-edge, C-down-into-last-row-moving-left, C-skid-rising-down-to-bottom,
  C-moonwalk-left-to-aim]
- The move would take its pixel part beyond the aim's (going right or down, the aim `before` it;
  going left or up, it `before` the aim) → it stops on the aim, its subpixel part 0
  [C-right-onto-edge, C-left-past-edge, C-aim-above-top-right-edge, C-up-to-aim,
  C-skid-rising-down-to-bottom, C-lead-near-right]
- So with the aim's pixel part the camera's, it stays as it is, subpixel included, and doesn't
  settle (C-at-aim-across-rising, C-at-aim-down, C-at-aim-down-held-across, C-at-aim-across). When
  she didn't move on an axis, it settles instead (`Settling`).
- So a creep left or up inside a pixel gives a speed under 1 px (`CreepReadsRightward`).
- Every other rule in this spec sets the camera's pixel part only: its subpixel part changes only
  by this move and this stop.
- checked: oracle (every harness case); constructed (every vector).

### LeadingHer
- kind: mechanic and parameter.
- summary: across, the view aims to keep her a set distance from its left edge, by the lead
  setting and the way she travels: she travels the way she faces, or the other way while she goes
  backwards.
- params: the lead, how far her x is from the view's left edge, by the lead setting: `Ahead` 96 when
  she faces right and 160 when she faces left (she is 96 px from the edge behind her); `KeepLeft`
  32 either way; `KeepRight` 224 either way; `Near` 64 when she faces right and 80 when she faces
  left. `lr` is the setting's lead when she faces right, `ll` when she faces left.
- switch: **Parameter**, the leads of each setting (params); the setting itself is an input.
- She faces right or the screen, going forwards, or she faces left, going backwards → the aim across
  is `sx − lr` [C-left-edge-keeps-subpixel, C-rise-up-to-closed-edge,
  C-moonwalk-right-to-closed-edge, C-knockback-rising-down-closed, C-moonwalk-left-to-aim,
  C-lead-near-right, C-lead-keep-right, C-facing-screen]
- She faces left, going forwards, or she faces right or the screen, going backwards → the aim across
  is `sx − ll` [C-right-onto-edge, C-down-into-last-row-moving-left, C-skid-rising-down-to-bottom,
  C-skid-left-to-closed-edge, C-knockback-left, C-lead-near-left, C-lead-keep-left]
- So a skid (C-skid-rising-down-to-bottom, C-skid-left-to-closed-edge), a moonwalk
  (C-moonwalk-right-to-closed-edge, C-moonwalk-left-to-aim) and a knockback (C-knockback-left,
  C-knockback-rising-down-closed) turn the lead round: the view looks the way she travels, not the
  way she faces.
- checked: oracle (`Ahead`, every harness case that moves across); constructed (`Near`,
  `KeepLeft`, `KeepRight`, facing the screen, and `Ahead` in the vectors that move across).

### AimingHeight
- kind: mechanic and parameter.
- summary: down, the view aims to keep her a set height below its top edge, and lower while she
  rises, so it shows more above her.
- params: the room's two heights, `hn` while she isn't rising and `hr` while she rises: 112 and
  160, or 144 and 160, in the game's recorded rooms; the vectors use heights of their own.
- switch: none (the heights are the room's, an input).
- Rising, a rise at 0 and the frame that spends it included → the aim down is `sy − hr`
  [C-rise-into-corner, C-skid-rising-down-to-bottom, C-rise-up-to-closed-edge,
  C-knockback-rising-down-closed, C-rise-up-to-closed-edge-moving-left, C-rise-at-0-to-aim,
  C-push-up-back, C-rise-at-0, C-rise-spent]
- Not rising → the aim down is `sy − hn` [C-aim-above-top, C-down-into-last-row,
  C-down-to-bottom-31, C-corner-other-heights, C-up-other-heights, C-unplaced, C-not-rising]
- So a rise at 0 aims as a rise (C-rise-at-0-to-aim, C-rise-at-0 against C-not-rising), and so does
  the frame that spends it, though her speed points down by then (C-rise-spent).
- checked: oracle (both heights of both kinds of room: the harness cases r05:1 and r08:656 at
  144; samus-shinespark-left:214 a rise at 0); constructed (both heights of two invented kinds of
  room, C-corner-other-heights and C-up-other-heights the second; C-rise-at-0-to-aim).

### Remembering
- kind: mechanic.
- summary: the camera remembers where it last saw her, which sets its next speed and whether she
  moved.
- params: none.
- switch: none.
- Each frame, after both axes → where it last saw her becomes her position, subpixels included
  [C-aim-above-top, C-hold-creep, C-skid-rising-down-to-bottom, C-stopped-hold-other-heights,
  C-placed]
- A placement put her at a y this frame, before it follows → the pixel part of where it last saw
  her down becomes that y's, its subpixel part kept [C-placed, C-placed-pixel]
- So with nothing placing her, where it last saw her stays as it was until after both axes, and
  the move pose fit or a change of shape made isn't followed: from the placed y she didn't move
  (C-placed); the same frame unplaced scrolls (C-unplaced). The kept subpixel can make her
  creep inside a pixel (C-placed-pixel eases 2 px).
- checked: oracle (every harness case's seen position); constructed (every vector's `after.seen`;
  C-placed, C-placed-pixel, C-unplaced).

### CreepReadsRightward
- kind: quirk.
- summary: a creep left or up inside a pixel reads as a move right or down of less than a pixel,
  so the view eases a pixel less that frame.
- params: none.
- switch: none.
- So her pixel part where it last saw her and her subpixel part smaller give `FollowingHer`'s
  first speed, under 1 px: its pixel part is 0, and `Settling` eases 2 px (C-creep against
  C-ease-right, the same frame with her still; C-placed-pixel).
- checked: constructed (C-creep, C-placed-pixel); no recorded frame settles out of a closed screen
  on a creep.

---

## 3. Keeping the view in the room

In this section `c` is the camera's pixel part on the axis, `cm` where its move took it before the
aim stopped it, and `v` its speed's pixel part. `margin` is how far under the top edge of its
screen the view's top may stop where it can go no lower, in px (`ShortScreens`); across it is 0.

### ClosedScreens
- kind: mechanic.
- summary: a closed screen is never shown: a view that goes into one is pushed back to its edge,
  backing off by the push each frame against her move (2 px in the reference profile).
- params: the push, 2 px beyond the speed.
- switch: **Parameter**, the push (params).
- `edge(p) = p − p mod 256`, the near edge of the screen holding `p`.
- The push forward from `p` is `ahead(p) = p + v + 2` (`PushCarries`), the push back
  `back(p) = p − v − 2`.
- So the push back has no carry (`PushCarries`): it would take 1 more when `p − v` goes below 0,
  but that needs `p` under the speed, at the room's first pixels, and every such push back lands
  before the edge it is held at.
- Across, the screens read are in the row holding `y + 128`, 128 px below the view's top edge
  (`y` the camera's pixel part down); down, in the column holding `x + 128`, 128 px right of its
  left edge.
- Going right or down, the next screen `Closed`, and, down, `c` `exceeds` `edge(c) + margin`
  (`ShortScreens`; across there is no such test and `margin` is 0) → `c = max(edge(c) + margin,
  back(cm))` [C-down-short-closed-below, C-moonwalk-right-to-closed-edge,
  C-right-to-closed-edge-down-to-bottom, C-knockback-rising-down-closed, C-push-right-back,
  C-push-down-back]
- Going left or up, the camera's screen `Closed` → `c = min(edge(c) + 256, ahead(cm))`, the `min` by
  `exceeds` [C-rise-up-to-closed-edge, C-skid-left-to-closed-edge,
  C-rise-up-to-closed-edge-moving-left, C-push-left-back, C-push-up-back, C-push-left-far]
- So in C-push-left-far the camera moves left from `80FE` onto its aim at `00FF`, and `ahead(cm)`
  is `8100`: `before` would keep `8100`, but it `exceeds` the edge, so the view stops on `0100`.
- The `exceeds` going down can't differ from `before`: `RoomEdges` lets only a `c` from 0 to `yb`
  reach it.
- So the view stops on the screen's edge (C-moonwalk-right-to-closed-edge, C-rise-up-to-closed-edge)
  or, short of it, backs 2 px from where it was before its move, or 1 when the move carried a
  subpixel (C-push-right-back): it moves against her until it is out.
- So going down, a top within `margin` of its screen's edge stays where the move put it
  (C-down-within-margin, C-left-edge-keeps-subpixel).
- checked: oracle (the stops on the edge: the harness cases r01:996, r02:7, r03:5, r03:210,
  r04:314, r05:66, item-morph-ball:709); constructed (their twins, the 2 px back-offs,
  C-push-left-far).

### RoomEdges
- kind: mechanic.
- summary: the view never shows past the room's left, top or right edge, nor below its bottom.
- params: none.
- switch: none.
- The camera's furthest right is the last screen's left edge, `xr = (w − 1) × 256`; its furthest
  down is `yb = (h − 1) × 256 + margin` (`ShortScreens`).
- Going right, `c` `exceeds` `xr` → it stops on `xr`, and no closed screen moves it that frame
  [C-right-onto-edge, C-rise-into-corner, C-down-short-closed-below, C-corner-other-heights,
  C-edge-right-from-left]
- Going left or up `before` 0 → it stops on 0 on that axis, and no closed screen moves it that frame
  [C-aim-above-top, C-left-past-edge, C-rise-into-corner, C-aim-above-top-right-edge,
  C-left-edge-keeps-subpixel]
- Going down, `c` `exceeds` `yb` → it backs up as from a closed screen, `max(yb, back(cm))`, and no
  closed screen moves it that frame [C-down-to-bottom-31, C-skid-rising-down-to-bottom,
  C-right-to-closed-edge-down-to-bottom, C-edge-down-from-above, C-edge-down-from-below]
- So a camera in the room stops on `yb`: its move ends at most `v + 1` px past it, so `back(cm)`
  is `before` `yb` (C-down-to-bottom-31).
- So does one above the room, since `back(cm)` is `before` `yb` there too
  (C-edge-down-from-above: from `FFF0`, `back(cm)` is `FFEE`).
- A camera another domain left below the room, far enough that `back(cm)` is after `yb`, backs up
  to `back(cm)`, which is `c − 2` plus the move's carry: 2 px from where it was, or 1 when the move
  carried a subpixel (C-edge-down-from-below: from `0150`, no carry, it ends on `014E`, not on
  `011F`).
- So a camera left of the room that moves right goes straight to the room's right edge
  (C-edge-right-from-left).
- checked: oracle; constructed (C-edge-right-from-left, C-edge-down-from-above,
  C-edge-down-from-below).

### ShortScreens
- kind: mechanic.
- summary: where the view can go no lower, it stops with its top on a short screen's top edge, or
  the margin lower on any other screen (31 px in the reference profile), which shows that much more
  of that screen's bottom.
- params: the margin on a screen that isn't short, 31 px.
- switch: **Parameter**, the margin (params).
- The screen it reads `Short` → `margin = 0` [C-skid-rising-down-to-bottom,
  C-down-short-closed-below, C-right-to-closed-edge-down-to-bottom, C-ease-up-short]
- Any other kind → `margin = 31` [C-down-to-bottom-31, C-push-down-back, C-ease-up]
- The screen it reads is the camera's screen down: going down, the one its move took its top to
  (`DownwardLooksAhead`); settling, the one its top is in before it settles
  (C-settle-up-from-below-short).
- checked: oracle (the harness case area-norfair:304 at 31; r01:994, r02:7 and r04:314 at 0);
  constructed (C-ease-up, C-ease-up-short, C-settle-up-from-below-short).

### Settling
- kind: mechanic.
- summary: on an axis she didn't move on, the view, brought into the room, eases out of a closed
  screen it partly shows a few pixels a frame (3 in the reference profile), towards the open screen
  beside it.
- params: the ease, `ClosedScreens`' push (the speed's pixel part is 1 here, or 0 after a creep,
  `CreepReadsRightward`).
- switch: none (the ease is `ClosedScreens`' push).
- `c0` is the camera's pixel part as the frame found it, and `a` its ease forward, `ahead(c0)`;
  the rules below read `c` after the first.
- Settling → it is first brought into the room: `c = min(max(c0, 0), cr)`, the `min` by
  `exceeds`, `cr` being `xr` across and `yb` down (`RoomEdges`) [C-settle-into-room-down]
- The camera's screen `Closed`, and `a` is the next screen's edge or `exceeds` it → it stops on
  that edge, `c = edge(c) + 256` [C-ease-right-snap, C-settle-reach-from-left]
- So from `c0` = `FFF0` the camera is brought to 0, its ease `a` is `FFF3`, which `exceeds` the
  next screen's edge `0100`, and it stops there (C-settle-reach-from-left).
- The camera's screen `Closed`, and the screen after the one `a` is in `Closed` too → it drops to
  the edge of the screen that ease is in, `c = edge(a)` [C-ease-both-closed]
- The camera's screen `Closed`, otherwise → it eases forward, `c = a` [C-ease-right, C-ease-down,
  C-creep, C-wrap]
- The camera's screen `Open` or `Short`, the next `Closed`, and, down, `c` `exceeds`
  `edge(c) + margin` (across there is no such test and `margin` is 0) → it eases back, `c =
  max(edge(c) + margin, back(c0))` [C-ease-left, C-ease-left-snap, C-ease-up, C-ease-up-short,
  C-settle-back-right-of-room]
- As that, and the `max` is `back(c0)`, which is in a `Closed` screen → it goes on to the next
  screen's edge from there, `c = edge(back(c0)) + 256` [C-settle-back-far-right]
- So with the next screen closed and the camera on its screen's edge, or down within `margin` of it,
  it holds (C-aim-above-top, C-down-within-margin, C-right-onto-edge, C-down-to-bottom-31,
  C-at-aim-down-held-across, C-at-aim-across).
- So from a camera right of the room, brought to its right edge, the ease back is measured from
  where the frame found it: from `0150` it eases to `014D` (C-settle-back-right-of-room); from
  `0250` the ease lands in the column after the last, which runs on to a closed screen, and it goes
  on to `0300` (C-settle-back-far-right). A camera below the room is brought to its bottom, its
  screen's edge plus `margin`, so it holds: from `0250`, below a `Short` last row, `margin` is the
  closed screen's 31 and it holds on `011F` (C-settle-up-from-below-short).
- So across, from a frame the camera began inside the room, the first rule changes nothing; down it
  can, when the screen its top is in became `Short` (C-settle-into-room-down).
- checked: oracle (the holds); constructed (every ease: no recorded frame starts with the view
  partly in a closed screen beside her still).

### TimeStopped
- kind: mechanic.
- summary: while time is stopped the view doesn't settle.
- params: none.
- switch: none.
- Time stopped, on an axis she didn't move on → settling is skipped: the camera keeps its position
  there [C-stopped, C-stopped-hold, C-stopped-hold-other-heights]
- checked: oracle (the harness cases area-maridia:419 and r08:714, where settling would have held
  anyway); constructed (C-stopped, against C-ease-right; C-stopped-hold).

### DownwardLooksAhead
- kind: quirk.
- summary: going down, the view checks the screens from where its move took it, before the aim
  stopped it, so a move that crosses into the next screen can back it up past the aim.
- params: none.
- switch: none.
- Going down → the next screen and `margin` are read from the screen `cm` is in, while `edge(c)`
  and the back-off measure from where the aim stopped it [C-ahead, C-below-closed,
  C-below-short-row]
- So in C-ahead she falls 6 px, the move crosses into the screen above a closed one, the aim stops
  it 2 px short of the crossing, and it backs up 2 px from where it was: the view rises as she
  falls.
- checked: constructed (C-ahead, C-below-closed, C-below-short-row); no recorded frame decides
  it: two harness cases cross into the room's last row (area-brinstar:414 and r01:655, their twins
  C-down-into-last-row and C-down-into-last-row-moving-left), but either read stops the view on its
  aim there.

### ScreensRunOn
- kind: quirk.
- summary: the screen after a row's last column is the first screen of the row below, so a view
  at the room's right edge can ease past it.
- params: none.
- switch: none.
- A screen `k` columns after a row's last → it is the `k`th screen of the row below, and so on
  row after row: the screens run on as one strip, after the room's last screen into the run the
  room's loading left (`BelowTheRoom`) [C-wrap, C-settle-back-far-right, C-wrap-into-below]
- So in C-wrap the view's own screen at the right edge is closed and the first of the row below
  is open: it eases 3 px past the room's right edge, and on from there each frame.
- So from the room's last row the screen after its last column is the run's first: in
  C-wrap-into-below it is open and the view eases 3 px; were it closed, the view would hold.
- checked: constructed (C-wrap, C-wrap-into-below); no recorded frame reads it with an effect.

### BelowTheRoom
- kind: quirk.
- summary: below the room's last row, the view reads the screens the room's loading left after
  the room's last screen.
- params: none.
- switch: none.
- A screen below the room's last row → its kind is the run's, `below`, which goes on from the room's
  last screen as one strip (`ScreensRunOn`): its first `w` screens are the row below the last, left
  to right, the next `w` the row below that, so only its end can fall short of a row; past its end,
  `Closed` [C-down-into-last-row, C-down-into-last-row-moving-left, C-below-closed, C-below-open,
  C-below-short-row, C-below-second-row]
- So in C-below-short-row the run holds one screen, under the left column: the screen under the
  right column is past its end, `Closed`, and the view backs up 2 px.
- So, from inside the room, only a move down that crosses into the last row or past it reads them
  to effect (`DownwardLooksAhead`); a view on the room's bottom reads the screen under it but holds
  whatever it is. C-down-into-last-row and C-down-into-last-row-moving-left would back up if the
  screen below were closed (C-below-closed against C-below-open).
- So a move whose top crosses into the row below the room reads the screen under it from the
  run's second `w`: in C-below-second-row it is open and the view stops on its aim; were it
  closed, the view would back up.
- checked: oracle (the harness cases area-brinstar:414 and r01:655); constructed (their twins,
  C-below-closed, C-below-open, C-below-short-row, C-below-second-row).

### PushCarries
- kind: quirk.
- summary: a push's sum carries once more.
- params: none.
- switch: none.
- `ahead(p)` whose `p + v` passes `FFFF` (`p` the position it pushes from) → it adds 1 more
  [C-push-carries, C-push-carries-move]
- So it shows only when the push sums from left of or above the room: from a camera another
  domain placed there (from `FFFF`, the first screen closed, it eases to `0003`, not `0002`:
  C-push-carries), or from where a move left or up took it before the aim stopped it in the
  room's first screen (from `0003` it moves 5 px to `FFFE`, stops on its aim `0002`, and the push
  from `FFFE` gives `0006`, not `0005`: C-push-carries-move).
- So only the push forward carries; `ClosedScreens` says why the push back doesn't.
- checked: constructed (C-push-carries, C-push-carries-move).

---

## 4. Order that matters

- Across first, then down: the row it reads across, 128 px below its top edge, is the camera's
  before it moves down, and the column it reads down, 128 px right of its left edge, is the
  camera's after it moved across.
- Where it last saw her becomes her position after both axes, so a frame's two speeds are both
  measured from the last frame's.
- A placement (the y it put her at) replaces where it last saw her down before the camera follows.
- On an axis, the aim's stop comes first; the room's edges and closed screens then measure from
  where it stopped (except the reads of `DownwardLooksAhead`).

## 5. Out of scope, and who supplies it

- **Doors and room loading:** the passage places the view in the next room and sets where it
  last saw her; the room's loading supplies the room's screens, its aim heights and the run of
  screens after the room's last screen (`BelowTheRoom`).
- **The cells:** a cell that opens or closes a screen changes the room's screens before the camera
  reads them.
- **Layer 2 and the backgrounds:** they follow the camera at the room's rates; the background's
  domain reads the camera's position.
- **Other placements:** the grapple's own slow camera; the grapple, the shinespark, a suit she
  collects, cutscenes, some bosses and the gunship changing where the camera saw her; their frames
  are masked in the harness until those domains come.
- **The knockback (the damage domain):** while a knockback moves her, it supplies going backwards
  and `rising`.
- **The enemies:** a boss changes the lead setting; Kraid's room keeps her on its first screen.
- **The room rules that move the view after it follows** (the room's own code): one room holds the
  view above a floor, one scrolls it right, one shakes it, and one waits for it.

## 6. Open questions

1. Past the run below the room the camera reads whatever the game holds there; we say `Closed`.
   The records do read there: in the four shinespark rows (left, right, up and up-left), a view on
   the room's bottom with her still down reads the two screens just past the run, 409 times, and
   the game holds `Closed` there each time, so the records agree (a view on the room's bottom holds
   whatever that screen is). Two ways would let the value change the view: a move down that crosses
   into the last row of one of the largest rooms, and a camera another domain left outside the
   room, above it (its top in the 256th row, C-edge-down-from-above-short) or right of it, past
   what `ScreensRunOn` runs on to. No record does either (docs/LEGACY.md, "Scroll areas past the
   room's table").
2. The lead settings other than `Ahead` have constructed vectors only: no recorded row meets the
   bosses that set them; a Crocomire or Kraid segment would promote them.
