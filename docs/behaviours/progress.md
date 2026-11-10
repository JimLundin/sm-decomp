# What the game remembers of a run: behaviour spec

What the game keeps of a run besides where Samus is: what she has (the items and beams that are
hers and those switched on, her tanks' capacities), what has happened in the world (the events,
the bosses defeated in each area, the places she has taken items from, the doors that stay open,
the maps she has, the save stations she has saved at), how the other domains tell it and ask it,
which version of a room loads given all that, and the item percentage at the end. Written
behaviour first, from the oracle's records. Vectors:
[`scenarios/vectors/progress.toml`](../../scenarios/vectors/progress.toml).

## Header

**Scope.** One domain, because the player sees one thing: the run's progress, which carries from
room to room and into the saved file. It is mostly state other domains read and set: a domain that
acts (an item, a boss, a door, a station) reports what happened, and progress records it; a
domain that depends on it (a room's loading, an enemy that stays dead, her motion's items)
asks. This spec states each quantity by what it means to a player, the rules that set it, and the
answers it gives. Nothing here decides when a report comes: that is the reporting domain's.

**The split with the items (W3), ruled by the root.** What taking an item gives her (`TakingItems`,
`TakingBeams`, `Tanks`) is this domain's; W3 keeps the item's place, its drawing and its message.
The maps' `OnArrival` is kept apart from `Downloaded`, though play reads them the same, because
P2 builds the saved file from progress, and the file tells them apart.

**How other domains reach it.**
- They report (§1.3): an event happens; a zebetite is destroyed; a boss is defeated; she takes
  an item; a door is opened; a map station downloads; she enters the room that gives a map; a
  door takes her into another area; she saves, or the gunship first lands; a room's loading
  places its items.
- They ask (§1.4): has an event happened; is a boss of any of these kinds defeated in this area;
  is this location emptied; is this door open; is the map known for this area; how many
  zebetites; which state a room loads; the item percentage.
- They read what she has as it is now (§1.1). Samus's motion reads the items switched on, never
  the ones merely hers; its table of the items she reads says which of them change her motion and
  which only her sprite ([samus-motion §1.3](samus-motion.md#13-what-she-reads-from-the-world-and-the-player-inputs)).

**Not in this spec** (§8 names who supplies each): saving and loading the file and its layout;
the explored map cells; the equipment screen's switches; the HUD's icons; the message boxes; the
music; energy and ammunition spent or refilled other than by taking a tank; play time; the
options.

**Notation.** Counts and capacities are whole numbers, 16 bits, and wrap at 65536. An area is one
of `Brinstar`, `Ceres`, `Crateria`, `Maridia`, `Norfair`, `Tourian`, `WreckedShip` (listed
alphabetically; the order means nothing). "The frame's area" is the area of the room she is in.

---

## 1. What the game remembers

### 1.1 What she has

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| her items | the `Item`s that are hers | the equipment screen lists them; the ending counts them | `ItemPercentage`, `RoomStates`; enemies that wake or hide once the morph ball is hers; output |
| items switched on | `Item`s | what she can do: jump higher, roll, spin through enemies | samus-motion; how she looks; her weapons; her damage; output |
| her beams | the `Beam`s that are hers | the equipment screen; the ending | `ItemPercentage`; output |
| beams switched on | `Beam`s | the beam she shoots | `TakingBeams`; her weapons; output |
| capacities | energy, reserve, missiles, super missiles, power bombs: counts | the HUD's tanks and ammunition maxima | `Tanks`; `RoomStates`; `ItemPercentage`; energy, reserves and weapons; the HUD; output |
| what she holds | energy, reserve energy, missiles, super missiles, power bombs: counts | the HUD's numbers | `Tanks` (which adds to them); energy, reserves and weapons, which spend and refill them; the HUD; output |
| reserve mode | `NoTank` (no reserve tank yet), `Automatic`, `Manual` | the HUD's AUTO sign; whether the reserves refill her on their own | `Tanks`; the reserves; the HUD; output |

**Items**, listed alphabetically: `Bombs`, `GrappleBeam`, `GravitySuit`, `HiJumpBoots`,
`MorphBall`, `ScrewAttack`, `SpaceJump`, `SpeedBooster`, `SpringBall`, `VariaSuit`, `XRayScope`.
**Beams**: `Charge`, `Ice`, `Plasma`, `Spazer`, `Wave`.

Others switch them too (§8): the equipment screen switches her items and beams on and off, and
the rainbow beam she is given late in the fight with Mother Brain switches on exactly `Charge`,
`Plasma` and `Wave`, whether or not they are hers. So a beam can be switched on without being
hers: nothing in this domain keeps the beams switched on within hers.

**The start of a run.** A new file (P2's) starts her with energy and its capacity at 99, every
other capacity and count at 0, the reserve mode `NoTank`, no items or beams, hers or switched on,
and no locations emptied, doors open, maps or stations, no events or bosses defeated, and no
items placed (C-new-file).

### 1.2 What has happened, and what the places remember

| Quantity | Units | Why the player can tell | Read by |
| --- | --- | --- | --- |
| events | the `Event`s that have happened | the world changes for good: rooms, creatures, statues | `RoomStates`; the blocks, enemies, bosses and room effects that change with them; output |
| zebetites destroyed | a count, 0 to 4 | the zebetites in Tourian's way stay broken | `ZebetitesCounted`; the zebetites; output |
| bosses defeated | for each area, the `BossKind`s defeated there | a defeated boss stays gone; the pause map greys its icon | `RoomStates`; the bosses; the pause map; output |
| emptied locations | location ids | a taken item never comes back | `EmptiedLocations`; the items; output |
| open doors | door ids | a door she shot open stays open | `DoorsStayOpen`; the doors; output |
| maps | for each area, `Unmapped`, `Downloaded` or `OnArrival` | the mini-map and the pause map show the rooms of the area | `MapsByArea`, `MapOnArrival`; the mini-map; the pause map; output |
| stations saved at | for each area, station ids | the pause map shows them | the pause map; the file select; output |
| items placed | a count | only in the saved file | output |

**Events**, by what the player sees, listed alphabetically (the order means nothing):
- `AcidReceded`: the chozo statue deep in Norfair has let the acid out of its room.
- `AnimalsFreed`: the animals she saved have escaped the planet.
- `DraygonStatueGrey`, `KraidStatueGrey`, `PhantoonStatueGrey`, `RidleyStatueGrey`: that boss's
  statue at Tourian's door has gone grey.
- `GlassTunnelBroken`: Maridia's glass tunnel is broken.
- `LavaQuakeOutrun`: she outran the rising lava in Norfair's speed booster hall.
- `MetroidRoom1Empty`, `MetroidRoom2Empty`, `MetroidRoom3Empty`, `MetroidRoom4Empty`: that one of
  Tourian's four metroid rooms, in the order she meets them, is cleared.
- `MotherBrainTankBroken`: the glass of Mother Brain's tank is broken.
- `PlanetAwake`: the planet is awake: Crateria's creatures and its rain are out.
- `ShaktoolPath`: Shaktool has dug its path through Maridia's sand.
- `TimeBombSet`: the planet's self-destruct is running (the escape).
- `TourianOpen`: the way into Tourian is open.
- `TourianRoomsUnset`: three of Tourian's rooms past the metroids have a version for it, which
  nothing in the game makes happen.

**Boss kinds** (`BossKind`, listed alphabetically): `MainBoss`, `MiniBoss`, `Torizo`. Which
boss is which, by area, for the reader (the bosses report their kind; this domain doesn't name
them):

| Area | `MainBoss` | `MiniBoss` | `Torizo` |
| --- | --- | --- | --- |
| Brinstar | Kraid | Spore Spawn | |
| Ceres | Ridley (Ceres) | | |
| Crateria | | | Bomb Torizo |
| Maridia | Draygon | Botwoon | |
| Norfair | Ridley | Crocomire | Golden Torizo |
| Tourian | | Mother Brain | |
| WreckedShip | Phantoon | | |

A location id and a door id come from the room's population, where each item and each door that
stays open is placed. A population may place one without an id: then nothing of it is remembered
(`EmptiedLocations`, `DoorsStayOpen`).

### 1.3 What other domains report (Inputs)

| Report | Carries | From |
| --- | --- | --- |
| an event happens | the `Event` | blocks, enemies, bosses, room effects |
| a zebetite is destroyed | | the zebetites |
| a boss is defeated | its `BossKind` | the bosses |
| she takes an item | an `Item`, a `Beam` or a tank and its amount; the location id, if it has one | the items |
| a door is opened | the door id, if it has one | the doors |
| a map station downloads | | the map stations |
| she enters the room that gives the map of its area | | the room's loading |
| a door takes her into another area | the area she leaves | doors and elevators |
| she saves, or the gunship first lands | the station's id in the area | the save stations, the gunship |
| a room's loading places its items | how many | the room's loading |

The frame's area is an input too, from the room's loading: in the frame a door takes her into
another area, it is the new room's, and her leaving carries the area she leaves
(C-door-from-tourian, C-door-into-tourian). So is, when a
room loads, the entrance she came in by (an `Entrance`, the way into the room she took), from the
doors. An entrance is not a door of `DoorsStayOpen`: its ids are a space of their own.

### 1.4 What other domains ask

Each answer reads what §1.1 and §1.2 hold, as the entries below say: whether an event has
happened (`EventsHappen`), the zebetites' count (`ZebetitesCounted`), whether a boss of any of
the kinds asked about is defeated in an area (`BossesDefeated`), whether a location is emptied
(`EmptiedLocations`), whether a door is open (`DoorsStayOpen`), whether the map is known for an
area (`MapsByArea`), which state a room loads (`RoomStates`), the item percentage
(`ItemPercentage`).

---

## 2. What has happened

### EventsHappen
- kind: mechanic.
- summary: an event, once it happens, has happened for the rest of the run.
- params: none.
- switch: none.
- An event reported → it has happened from then on: asked, yes [C-event, C-event-again,
  C-state-event]
- So an event never stops having happened; one that happens again changes nothing (C-event-again).
- checked: constructed (no recorded frame makes an event happen; the room states the harness
  checks at rows' first records read events, its cases samus-mockball:0 and area-maridia:0).

### ZebetitesCounted
- kind: mechanic.
- summary: the game counts the zebetites she has destroyed, so the ones she broke stay broken.
- params: none.
- switch: none.
- A zebetite destroyed → the count grows by one, `zebetites + 1` [C-zebetite-first,
  C-zebetite-fourth, C-zebetites-asked]
- So the zebetites, asked, get the count back, 0 to 4 (C-zebetites-asked).
- checked: constructed.

### BossesDefeated
- kind: mechanic.
- summary: a defeated boss is remembered by its kind in its area, so it stays gone.
- params: none.
- switch: none.
- A boss defeated → its kind is defeated in the frame's area [R-ceres-boss, C-boss-mini,
  C-boss-torizo]
- So kinds add up in an area (C-boss-mini), and one area's kind says nothing of another's
  (C-boss-torizo, C-state-boss-other-area).
- checked: oracle (R-ceres-boss); constructed.

---

## 3. What the places remember

### EmptiedLocations
- kind: mechanic.
- summary: an item she takes is gone from its place for good.
- params: none.
- switch: none.
- She takes an item with a location id → the location is emptied: asked, yes [C-item-first,
  C-item-second, C-missile-tank-stocked, C-item-beside-another, C-asked-no-location]
- She takes an item placed without a location id → asked about, it reads as still in its
  place [C-no-location, C-asked-no-location]
- So an item placed without an id appears each time its room loads, however often she takes it.
- checked: oracle (the harness cases item-morph-ball:817, samus-bomb-jump-vertical:55 and
  s01:37); constructed (no id).

### DoorsStayOpen
- kind: mechanic.
- summary: a door she opens for good stays open.
- params: none.
- switch: none.
- A door with an id opened → it is open: asked, yes [C-door-between, C-door-last,
  C-asked-doors]
- A door placed without an id opened → asked about, it reads as closed [C-door-no-id,
  C-asked-doors]
- checked: oracle (the harness case r04:280); constructed; r06:519 is stood in (a super
  missile opens it).

### MapsByArea
- kind: mechanic.
- summary: a map station gives her the map of its area.
- params: none.
- switch: none.
- The map station's download → the map of the frame's area is `Downloaded` [C-map-station]
- Asked whether the map is known for an area → yes when it is `Downloaded` or `OnArrival`
  [C-map-known]
- checked: constructed (no recorded row uses a map station).

### MapOnArrival
- kind: quirk.
- summary: Tourian, which has no map station, gives its map when she enters its first room; the
  map counts as downloaded only once she has left the area.
- params: none.
- switch: none.
- She enters the room that gives the map of its area, the map not `Downloaded` → it is
  `OnArrival` [C-map-on-arrival, C-map-on-arrival-had, C-door-from-tourian,
  C-door-into-tourian]
- When a door takes her into another area, the map of the area she leaves `OnArrival` → it
  becomes `Downloaded` [C-map-left, C-map-left-none, C-door-from-tourian, C-door-into-tourian]
- So the two read the same everywhere in play (`MapsByArea`); only the saved file tells them apart,
  when she saves before leaving.
- checked: constructed.

### StationsSavedAt
- kind: mechanic.
- summary: the save stations she has saved at are remembered by area, for the maps to show.
- params: none.
- switch: none.
- She saves at a station, or the gunship first lands (its area's first station) → it is one
  she has saved at, in the frame's area [R-gunship-save, C-save-station]
- checked: oracle (R-gunship-save, the gunship's landing); constructed.

### ItemsPlacedCount
- kind: quirk.
- summary: the game counts every item it places as rooms load, taken or not, over the whole run;
  nothing in play reads the count, but the file keeps it.
- params: none.
- switch: none.
- A room's loading places `n` items → the count grows by them, `count + n`
  [C-placed-several, C-placed-one, C-placed-taken, C-placed-wraps]
- So an item already taken still counts each time its room loads (C-placed-taken).
- checked: oracle (the harness cases area-norfair:200, r06:757 and r08:567); constructed.

---

## 4. What she has

### TakingItems
- kind: mechanic.
- summary: an item she takes is hers and switched on.
- params: none.
- switch: none.
- She takes an `Item` → it is hers and switched on, even one she had and switched off
  [C-item-first, C-item-second, C-item-beside-another, C-item-switched-off]
- checked: oracle (the harness cases item-morph-ball:817 and samus-bomb-jump-vertical:55);
  constructed.

### TakingBeams
- kind: mechanic.
- summary: a beam she takes is hers and switched on; spazer and plasma don't fire together, so
  taking one switches the other off.
- params: the beam each switches off, a table of the game's data: `Spazer` switches off
  `Plasma`; `Plasma` switches off `Spazer`; the rest none; its cells are beams.
- switch: **Parameter**, the beam each switches off (params).
- She takes a `Beam` → it is hers and switched on [C-beam-other, C-spazer-taken, C-plasma-taken]
- The beam taken switches another off → that one is switched off, hers still [C-spazer-taken,
  C-plasma-taken, C-plasma-taken-spazer-off]
- checked: constructed (no recorded frame takes a beam; the late files hold spazer as hers and
  switched off beside plasma, as the rule leaves them: samus-mockball's first record).

### Tanks
- kind: mechanic.
- summary: a tank raises a capacity by its amount: an energy tank fills her energy to the new
  capacity, a reserve tank leaves the reserves as they were and, the first time, makes them refill
  her automatically, and a missile, super missile or power bomb tank adds its amount to what she
  holds without filling her up.
- params: the tank's amount, carried by the report (100 for energy and reserve tanks, 5 for the
  others, in the game's items).
- switch: none.
- She takes an energy tank → the energy capacity grows by its amount and her energy becomes
  the new capacity, `capacity + amount` [C-energy-tank]
- She takes a reserve tank → the reserve capacity grows by its amount; the energy in them
  doesn't [C-reserve-first, C-reserve-manual]
- She takes a reserve tank, the reserve mode `NoTank` → it becomes `Automatic` [C-reserve-first]
- She takes a missile, super missile or power bomb tank → its capacity and her count of it
  both grow by its amount, `count + amount` [C-missile-tank-stocked, C-missile-not-full,
  C-super-tank, C-power-bomb-tank]
- So a reserve tank taken after she set them to `Manual` leaves them manual
  (C-reserve-manual).
- So after firing some, a missile tank leaves her short of the new capacity
  (C-missile-not-full: 3 of 20 becomes 7 of 24, by a tank of 4).
- checked: oracle (the harness case s01:37); constructed.

---

## 5. Which version of a room loads

### RoomStates
- kind: mechanic.
- summary: a room has versions, each with a condition on the run's progress; it loads the first
  whose condition holds, so it changes as the run goes on.
- params: none.
- switch: none.
- A room loads → it takes the first of its states, in its order, whose condition holds; the
  last always holds [C-state-earlier-wins, C-state-none-hold, C-state-event-not-yet,
  C-state-first-holds]
- `Happened(event)` → holds once the event has happened [C-state-event, C-state-first-holds,
  C-state-other-event]
- `Defeated(kinds)` → holds when any of the kinds is defeated in the room's own area
  [C-state-boss, C-state-boss-other-area, C-state-any-kind, C-state-boss-alive,
  C-state-boss-not-yet]
- `Has(item)` → holds when the item is hers, switched on or not [C-state-has]
- `MorphBallAndMissiles` → holds when the morph ball is hers and the missile capacity isn't 0
  [C-state-morph-missiles, C-state-missiles-only, C-state-morph-no-missiles]
- `PowerBombs` → holds when the power bomb capacity isn't 0 [C-state-earlier-wins,
  C-state-power-bombs-spent]
- `EnteredBy(entrance)` → holds when she came into the room by that entrance
  [C-state-entered-by, C-state-entered-other]
- So the missiles' and power bombs' conditions read capacity, not what she holds: a room changes
  for a tank taken, even with every one spent (C-state-morph-missiles,
  C-state-power-bombs-spent).
- So a state whose condition holds wins over every later one, whether or not theirs hold too
  (C-state-earlier-wins, C-state-first-holds).
- checked: oracle (the harness: 39 room loads in the 32 rows, every one agreeing, with the
  rooms' own state lists); constructed (events, bosses, items and entrances that no row's load
  reads).

---

## 6. The item percentage

### ItemPercentage
- kind: mechanic.
- summary: the ending shows how much of the game's equipment she found, as a percentage.
- params: the divisors, a table of the game's data: energy 100, reserve 100, missiles 5,
  super missiles 5, power bombs 5; its cells are numbers.
- switch: **Parameter**, the divisors (params).
- Asked → the sum over the divisors of each capacity ÷ its divisor, rounded down, plus the
  count of her items and beams, `Σ capacity ÷ divisor + items + beams` [C-percent-start,
  C-percent-many, C-percent-partial, C-percent-rainbow]
- So the 99 energy she starts with counts 0 (C-percent-start).
- checked: constructed (no recorded row reaches the ending); the harness, that every item, beam
  and tank the game places counts 100.

---

## 7. Order that matters

- A frame's reports apply in the order they come. Only taking a beam reads what another report
  writes (the beams switched on, `TakingBeams`), and no frame takes two of anything: each item,
  beam or tank she takes holds the game in its message box for seconds before the next can be
  taken. Her leaving names the area she leaves and every other report the new room's, so her
  leaving and a map given on arrival concern two areas; every other pair changes different
  quantities, or adds to the same count. So no order within a frame is visible.
- A room's state is chosen with the room's own area, after it is known, so `Defeated` reads the
  new room's area.

## 8. Out of scope, and who supplies it

- **Saving and loading** (P2): the file's layout, its checksums, where a loaded file resumes, the
  stations that refill energy and missiles. The file holds every quantity of §1 marked output; the
  encodings (which bit, which byte) are P2's.
- **The explored map cells** (the mini-map and the pause map): which cells she has seen.
- **The equipment screen** (F1): switching items and beams on and off, the reserve mode's switch.
- **The rainbow beam** (Mother Brain's domain): switching on `Charge`, `Plasma` and `Wave` late
  in her fight, whether they are hers or not (§1.1).
- **The HUD** (H2): the icons added when the first missiles, super missiles, power bombs, grapple
  or x-ray come, the item selection.
- **The message boxes** (H3) and **the music** (U1) that an item, a station or a map brings.
- **Energy and ammunition** spent, lost and refilled other than by taking a tank (her weapons, her
  damage, the reserves, the stations that refill her).
- **What each domain does with what it asks**: whether an item shows, a door is drawn open, a
  boss appears.
- **Play time** (game-time), **the options and controls** (F2).

## 9. Open questions

1. In none of the recorded rows does an event happen, is a boss other than Ceres's defeated,
   does a map station download, or is a beam or a tank other than a missile tank taken. Rows
   from the 100% route's chain (bosses, the statues, Tourian, the beams) would promote the
   constructed vectors.
