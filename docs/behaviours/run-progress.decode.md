# What the game remembers of a run: decode notes

For the observer, the adversary and the domain harness's decoder only. Implementers never see this
file (MODEL §5). It holds every source the spec draws on: the original's routines, its RAM and its
codes. Addresses are the bank logs' (`$TEMP/banklogs/bank_*.txt`,
https://patrickjohnston.org/bank/).

## How the spec was made

Behaviour first: the 32 rows cached in `$TEMP/collab/more-caches/cache` were dumped by
`$TEMP/collab/progress-spec/dump` (a cdylib over `oracle_record`, run by `dumprun.py`; output in
`out/`): each row's progress RAM at its first record, then every byte that changes, with the PLM
table of the record before. The bank logs were then read to name the cases and pin the rules.

- `allframes.py` decodes the progress of every record and compares neighbours: of the 18,299
  frame pairs, 10 change what progress holds (the fields this domain owns: not energy and
  ammunition spent or refilled by other domains). The null model (nothing changes) matches the
  other 18,289. Each of the 10 was a recorded vector, and the reference model reproduces it from
  the report the acting domain made. Eight of them carry the game's content (a location or door
  id, a tank's amount, the items a room places) and are harness cases now (§The harness cases);
  the other two stay vectors.
- `analyse.py` checks the room state choice at every room load in the rows (13) and at every row's
  first record (26 more, the state the room holds there against the progress then): 39 of 39
  agree with the rule. `rooms.py` walks the ROM's rooms by their doors from the Landing Site and
  Ceres (261 rooms; `rooms.txt`) and lists every state condition the game uses.
- `model.py` is the reference model, in the spec's terms only; `gen.py` decodes the recorded
  vectors (the mapping below), writes the constructed ones from the model, and writes
  `scenarios/vectors/run-progress.toml` with `header.toml`; `vecrun.py` runs the file through the
  model. The two door vectors (`C-door-from-tourian`, `C-door-into-tourian`) and a report's own
  `area` key were added by hand after; `$TEMP/collab/progress-spec-r5/model.py` is the model
  that reads that key (a report's `area`, else the vector's): 65 of 65 agree. The model without
  it fails `C-door-from-tourian` (Tourian's map stays `OnArrival`). The two vectors check the
  area each report takes, not an order: the reports touch different areas, so a model applying
  them in reverse passes all 65. No recorded row leaves an area whose map is `OnArrival`, so
  `LeftArea`'s change is checked by constructed vectors only.
- `$TEMP/collab/progress-spec-r6/` holds the conversion to invented content (MODEL §1 decision
  13; §The harness cases): `edit_ids.py` gives the constructed vectors invented location and
  door ids and invented tank amounts, `tail.toml` holds the rewritten room state, percentage and
  twin vectors, and `real_lists.py` checks every vector's state list against the lists the ROM's
  rooms use (`rooms.txt`), and their shapes: 13 vectors carried a real list and 6 a real shape
  before, 0 after. `edit74.py` adds C-new-file, which spells out a new file's start in its
  `after`, so a start other than the vector header's fails (a mutant model starting at 0 energy
  fails it, and no other vector). r5's `model.py` agrees with 66 of 66 through `vecrun.py`.
- `$TEMP/collab/progress-r8/` holds round 8's pass: the order within a frame, questions by id,
  the switch-off table's direction, and a vector at a value other than the game's for each
  `Parameter` entry. `dump/` is the dump cdylib rebuilt over this tree's `oracle_record`, run
  on the 44 rows of `$TEMP/collab/scenes/cache-v2` (`out/`). `model.py` is r6's model with
  per-vector rules (a `switches_off` table of sets, the divisors) and questions by id;
  `vecrun.py` runs it, or a mutant, on the vector file: 76 of 76. `frames.py` runs it per frame
  over the cache: the reports are heard from each record's hits (`hits.txt`), the ids and kinds
  that only an argument names stood in from what changed (10 frames), and the progress after
  them compared with record N's on the fields this domain owns (what she holds on a tank's frame
  only; the beams switched on from the hyper beam's frame on, none in these rows): 25,426 of
  25,426 frame pairs agree, the null model 25,399. `analyse.py` over the same rows: 52 of 52
  room states (44 rows' first records, 8 loads) agree, the null model (the default) 42.
  `make_mutants.py` writes ten one-line mutants; each fails the vectors named:
  - the table read backwards (taking a beam switches off the beams whose rows hold it):
    C-switches-off-adjusted, C-switches-off-adjusted-other;
  - the game's table whatever the rules: C-switches-off-adjusted, C-switches-off-adjusted-pair;
  - one beam at most per row: C-switches-off-adjusted;
  - only a beam that is hers switched off: C-plasma-taken-spazer-not-hers;
  - a frame's reports applied last first: C-beams-in-order, C-beams-in-order-reversed,
    C-arrival-then-leaving, C-leaving-then-arrival;
  - the game's divisors whatever the rules: C-percent-adjusted;
  - energy's and reserve's divisors swapped: C-percent-adjusted;
  - her leaving taking the frame's area: C-door-from-tourian, C-door-into-tourian;
  - an event reported again un-happening: C-event-again, R-planet-awake;
  - a location asked by id reading emptied when any is: C-asked-locations.

  Over the cache every mutant agrees on every frame, as the model does (the last asks a
  question, which `frames.py` doesn't): the game's table is symmetric, no frame has two reports
  whose order shows, no row leaves an area, no row asks the percentage, and the stand-in hears
  event-zebes-awake 152's event once. That is why the vectors carry those rules.

**Row ids.** As in the camera's notes, the chain rows go by opaque ids, because a chain row's name
carries a room header's address; s01 and e01 are rows whose names match the lexicon:

| Id | Row |
| --- | --- |
| r01 | chain-0012-91F8 (Landing Site) |
| r02 | chain-0013-92FD |
| r03 | chain-0014-96BA |
| r04 | chain-0057-9FBA |
| r05 | chain-0059-A293 |
| r06 | chain-0097-ACF0 |
| r07 | chain-0100-B07A |
| r08 | chain-0106-AD5E |
| r09 | chain-0220-D017 |
| r10 | chain-0224-D0B9 |
| r11 | chain-0232-D5A7 |
| s01 | samus-square-slope-reserve |
| e01 | event-zebes-awake |

## State

All in the save area: `$09A2..0A01` (Samus's data and options, copied to `$7E:D7C0..D81F` on a
save, `$81:8019`) and `$7E:D820..DE1B` (the rest of the save data, `$81:8000` and `$81:8085`).

| Quantity | RAM | Decode |
| --- | --- | --- |
| items switched on / hers | `09A2` / `09A4` (equipped / collected items) | bits: 1 `VariaSuit`, 2 `SpringBall`, 4 `MorphBall`, 8 `ScrewAttack`, 20h `GravitySuit`, 100h `HiJumpBoots`, 200h `SpaceJump`, 1000h `Bombs`, 2000h `SpeedBooster`, 4000h `GrappleBeam`, 8000h `XRayScope`; other bits never set |
| beams switched on / hers | `09A6` / `09A8` | bits: 1 `Wave`, 2 `Ice`, 4 `Spazer`, 8 `Plasma`, 1000h `Charge` |
| capacities | `09C4` energy, `09D4` reserve, `09C8` missiles, `09CC` super missiles, `09D0` power bombs | as is |
| what she holds | `09C2` energy, `09D6` reserve energy, `09C6` missiles, `09CA` super missiles, `09CE` power bombs | as is |
| reserve mode | `09C0` | 0 `NoTank`, 1 `Automatic`, 2 `Manual` |
| events | `$7E:D820..D827`, bit n of byte n >> 3 | the event numbers of `$80:81FA`'s list: 0 `PlanetAwake`, 2 `MotherBrainTankBroken`, 6 `PhantoonStatueGrey`, 7 `RidleyStatueGrey`, 8 `DraygonStatueGrey`, 9 `KraidStatueGrey`, Ah `TourianOpen`, Bh `GlassTunnelBroken`, Ch `AcidReceded`, Dh `ShaktoolPath`, Eh `TimeBombSet`, Fh `AnimalsFreed`, 10h-13h `MetroidRoom1Empty`..`MetroidRoom4Empty`, 14h `TourianRoomsUnset`, 15h `LavaQuakeOutrun`; 3-5 the zebetites (below); 1 and 16h-3Fh no meaning |
| zebetites destroyed | events 3, 4, 5 | the count in binary, event 3 its lowest bit (`$A6:FBA5`, `$A6:FCAA`) |
| bosses defeated | `$7E:D828 + area`, one byte an area | bits: 1 `MainBoss`, 2 `MiniBoss`, 4 `Torizo` (`$80:81A6`'s list) |
| emptied locations | `$7E:D870..D8AF`, bit n of byte n >> 3 | location n (the item PLM's room argument) |
| open doors | `$7E:D8B0..D8EF`, the same | door n (the door PLM's room argument) |
| maps | `$7E:D908 + area`, one byte an area | 0 `Unmapped`; FFh `Downloaded`; any other value (01h) `OnArrival` |
| stations saved at | `$7E:D8F8 + area × 2`, low byte | bit n: station n (the save station PLM's room argument & 7) |
| items placed | `$7E:D91A`, a word | as is |
| the frame's area | `079F` | 0 Crateria, 1 Brinstar, 2 Norfair, 3 Wrecked Ship, 4 Maridia, 5 Tourian, 6 Ceres |
| the entrance she came in by (`EnteredBy`) | `078D` (the door pointer of the transition into the room) | the door entry's address in bank 83, opaque; the `$8F:E5EB` condition compares it with its own word. Not a door PLM argument (`$7E:D8B0`'s ids), so the spec gives it its own type |

The spec's enum orders are alphabetical (events, items, beams, areas, boss kinds, reserve
modes), none in the RAM's order.

**RAM shape** (fields decoded as one word copied as-is): the capacities, what she holds and the
items placed count. They are counts the player reads as numbers (the HUD's, the file's), and the
rules add to them; nothing narrower holds them. Every set above is decoded bit by bit into a set
of named values, and the maps' byte into an enum.

**Not State, or not this domain's:**
- `0789`, the current area's map flag: derived. The map station sets it with the area's byte
  (`$84:8C8F`), the Tourian elevator room's setup too (`$8F:C90A`), and every load of an area's
  explored map copies it from the byte (`$80:858C`, on a door with the area change bit 40h and on
  loading a file); a new file clears both (`$81:A256`, `$81:B2CB`). So it is always "the map is
  known for the frame's area", which `MapsByArea` answers.
- `$7E:D830..D86F` (chozo blocks destroyed): read and set only by the unused chozo block PLMs
  `$D700`/`$D708` (`$84:8848`, `$84:8865`); always 0, so nothing of it is modelled. The file keeps
  zeros there (P2's encoder).
- `D8F8`'s high bytes: the debug elevators' bits, set only by `$80:CD07`, which no door reaches
  (elevator properties & Fh is always 0, `$82:DF99`).
- `D914` (where a loaded file resumes), `D916` and `D918` (the save station and area the file
  resumes at), `D91C..` (the explored maps, compressed): P2 and the mini-map's.
- `$70:1FE0` ('supermetroid', the game completed): written by the ending (`$80:824F`), read for the
  number of demo sets (`$80:8261`): the front end's and P2's.

## Inputs (the reports, and where the original makes them)

| Report | The original | Effect in RAM |
| --- | --- | --- |
| `Happened(event)` | `$80:81FA` (mark event), called by PLM instruction `$84:883E`, animated tiles instruction `$87:8150` (the statues), and code: `$84:B86E` (15h), `$84:B8CB` (Dh), `$84:B9B9` (Fh), `$84:BE0B` (0), `$84:D1B4` (Ch), `$84:DAE6`..`$84:DB16` (10h-13h), `$88:DCA3` (Ah), `$8F:C91F` (Eh, Escape room 1), `$A9:B313` (Eh, Mother Brain) | ORA of the bit |
| `ZebetiteDestroyed` | `$A6:FCAA` (the counter + 1, written back through `$80:81FA` and `$80:8212`, unmark: the only caller) | the three bits = the count |
| `Defeated(kind)` | `$80:81A6` with the kind's bit, on `079F`'s byte: Ceres Ridley `$A6:C131`, Ridley `$A6:C5E2`, Mother Brain `$A9:B30F` (mini-boss), torizos `$AA:B250`, Botwoon `$B3:9B16`; direct writes of `$7E:D828,x`: Crocomire `$A4:9B9A`, Draygon `$A5:92DE`, Spore Spawn `$A5:EE3A`, Kraid `$A7:C831`, Phantoon `$A7:DB85`; PLM `$84:8821` and `$87:8139` (unused/animated tiles) | ORA |
| `Taken(item, location)` | the item PLM's list: `$84:8899` (set the location's bit, skipped for a room argument with bit 15), then its pickup instruction | |
| — an `Item` | `$84:88F3`, `$84:891A` (grapple, + HUD), `$84:8941` (x-ray, + HUD) | `09A2 |= bits`, `09A4 |= bits` |
| — a `Beam` | `$84:88B0` | `09A8 |=`, `09A6 |=`, then `TRB` of plasma when the bits hold spazer (`ASL; AND #8`) and of spazer when they hold plasma (`LSR; AND #4`) |
| — an energy tank | `$84:8968` | `09C4 += amount`, `09C2 = 09C4` |
| — a reserve tank | `$84:8986` | `09D4 += amount`; `09C0` 0 → 1 |
| — a missile / super / power bomb tank | `$84:89A9` / `$84:89D2` / `$84:89FB` | max and count `+= amount` (no cap) |
| `DoorOpened(door)` | `$84:8A91` once the door's hit counter reaches its count (skipped for bit 15) | ORA of the bit in `$7E:D8B0` |
| `MapStation` | `$84:8C8F` | `$7E:D908 + area` (a word) `|= 00FFh`; `0789 = 1` |
| `MapOnArrival` | `$8F:C90A`, the setup code of room `$DAAE` (the Tourian to Crateria elevator room) | `$7E:D908 + area |= 1`; `0789` = that word |
| `LeftArea` | `$82:DF99` → `$80:85C6` on a door with bit 40h (area change) | if `0789` ≠ 0: the byte `|= FFh` |
| `Saved(station)` | `$84:8CF1` (a save station, `$84:8D1B`); the gunship (station 0): `$A2:A9A7` on its first landing (the spec's "the gunship first lands", r01:610), `$A2:AB2B` when she saves in it | `$7E:D8F8 + area × 2 |=` the station's bit |
| `ItemsPlaced(n)` | the item PLMs' setups `$84:EE64` and `$84:EE8E` (every item PLM, `$EED7`..`$EFCF`, collected or not: the setup runs before the list tests the bit) | `$7E:D91A += 1` per item |

Not reports of this domain:
- `$80:81C0` (clear boss bits) has no caller; `$80:8212` (unmark) has one, the zebetites.
- `$91:E5F0` (Mother Brain's drain, step 3, the hyper beam): `09A6 = 1009h` (Charge, Plasma,
  Wave) whatever `09A8` holds. A writer of the beams switched on from Mother Brain's domain (spec
  §1.1, §8); a row recorded through her fight shows it. It is why the beams switched on need not
  be within hers.
- `$88:E320`, `$88:E361` (the varia and gravity suit pickup animation, stage 3): `09A2 |=` the
  suit's bit again, after the item PLM already set it: redundant, no change.
- `$91:DF12` (restore health): an overflow into the reserves sets `09C0` 0 → 1 when the reserve
  energy is above 0. Unreachable as a change: reserve energy above 0 needs a reserve tank, which
  already set the mode.
- `$91:87A8` (the demos' equipment: `09A2 = 09A4 =` the demo's items) and `$82:8932` (the
  Tourian entrance demo sets Brinstar's boss byte to 1), with `$82:86FC` (a demo's bits: every
  item, map and station set, doors, events and bosses clear): demos only, the front end's.
- `$91:E390` (debug: every item, `F33Fh`): no input reaches it in play.

**The start of a run** (spec §1). `$81:B2CB` (new save file, from `$81:A256` when the slot
won't load and `$81:9CDE` after erasing one): `09C4 = 09C2 = 99`; the other capacities and counts,
`09A2..09A8`, `09C0`, `09D4`, `09D6` to 0; `$7E:D830..D8EF` (chozo blocks, locations, doors),
`D8F0..D8FF` (stations, debug elevators) and `D908..D90F` (maps) to 0. It leaves `$7E:D820..D82F`
(events, bosses) and `D91A` (items placed) alone. `$81:8085` copies the slot's SRAM into
`$7E:D7C0..DE1B` before testing its checksums, so on a slot that won't load these hold the slot's
bytes, and those bytes are always 0. Every way into the file select's main menu runs "reload
main" (`$81:9EF3`; the initialise step `$81:9ED6` falls through into it), which loads each slot
in turn (`$81:9F1F`, `9F55`, `9F8B`, each through `$81:A053` to `$81:8085`). A slot that fails
its checksums has its 65Ch bytes of SRAM cleared there (`$81:8085`'s corrupt branch,
`$81:80FC..811B`), so when it is selected the load at `$81:A24A` copies zeros into `D820..D82F`
and `D91A` before `B2CB` runs. An erased slot (`$81:9CBA`, 65Ch bytes cleared) gives 0 the same
way. So a new file starts with no events, no bosses defeated and 0 items placed (spec §1.1). The
one exception is not a new file: the corrupt branch leaves the slot's checksum words
(`$70:0000/0008/1FF0/1FF8` + slot × 2) as they were, so stale words that happen to be 0 and
FFFFh would make the cleared slot load as a file. The spec names this P2's.

## Questions (where the original asks)

- Has an event happened: `$80:8233`, from PLM instruction `$84:882D`, the state condition
  `$8F:E612`, the Tourian statue FX `$88:DCCB`, enemies and bosses (`$A6:FBA5`, the zebetites).
- A boss kind defeated in the current area: `$80:81DC`, from PLM instruction `$84:880E`, the
  state conditions `$8F:E5FF` (with 1) and `$8F:E629`, and the bosses' own reads of `$7E:D828,x`.
- A location emptied: `$84:887C` (bit 15 of the room argument: never).
- A door open: `$84:8A72` (bit 15: never).
- Both questions are asked only with an id. An item or door PLM whose room argument has bit 15
  (placed without an id) skips the question itself and shows its item, or stays shut, and
  setting the bit is skipped the same way (`$84:8899`, `$84:8A91`). So the spec has progress
  asked by id only (§1.2), and the no-id case is the asker's (§8); no vanilla population sets
  bit 15.
- The map known: `0789` or the area's byte, nonzero (`$82:9478`, `$82:9565`, `$82:BB3D`,
  `$82:9EC6`, `$82:B8C6`, `$90:AAA0`): every reader tests nonzero, so `OnArrival` and
  `Downloaded` read the same.
- The room state: `$8F:E5D2` walks the list from the header + Bh; `$82:DED3` calls it after
  `$82:DE86` set `079F` from the new room's header.
- The percentage: `$8B:E627`, with the table `$8B:E70D`/`$8B:E717` (capacities and divisors),
  `$8B:E721` (11 item bits), `$8B:E737` (5 beam bits), the hardware divider (16 by 8 bits,
  exact for every value the game reaches).

## Sources per entry

| Entry | Source |
| --- | --- |
| EventsHappen | `$80:81FA`, `$80:8233`, `$80:818E` (bit and byte), the callers above |
| ZebetitesCounted | `$A6:FCAA`, `$A6:FCCB`, `$A6:FBA5` |
| BossesDefeated | `$80:81A6`, `$80:81DC`, the bosses' writes above |
| EmptiedLocations | `$84:8899`, `$84:887C` |
| DoorsStayOpen | `$84:8A91`, `$84:8A72` |
| MapsByArea | `$84:8C8F`; the answer from its readers `$82:9478`, `$82:9565`, `$82:BB3D`, `$82:9EC6`, `$90:AAA0` |
| MapOnArrival | `$8F:C90A`, `$80:85C6` via `$82:DF99` |
| StationsSavedAt | `$84:8D12`..`$84:8D22`, `$A2:A9A7`, `$A2:AB2B` |
| ItemsPlacedCount | `$84:EE6D`, `$84:EEA1` |
| TakingItems | `$84:88F3`, `$84:891A`, `$84:8941` |
| TakingBeams* | `$84:88B0` |
| Tanks | `$84:8968`, `$84:8986`, `$84:89A9`, `$84:89D2`, `$84:89FB` |
| RoomStates | `$8F:E5D2`..`$8F:E689` (each condition its own routine), `$82:DED3` |
| ItemPercentage* | `$8B:E627` |

Entries marked * have a single routine as their source: `scan.sh --mirror` finds 2 of 14
(14 %). The spec's grouping is by what the player sees (an item, a beam, a tank, a map), not by
routine: the item instructions, the tanks and the condition routines are merged or split by
behaviour.

## Findings behind the spec's choices

- **Events past the list.** Event 1 has no setter and no reader in the rooms' conditions; event
  14h (`TourianRoomsUnset`) is read by the states of rooms `$DC19`, `$DC65`, `$DCB1` and set by
  nothing. Events 16h-3Fh have no meaning. The importer refuses an event number with no name
  (a PLM list or a room naming one would be a ROM hack).
- **The state conditions in the ROM** (`rooms.txt`): event 0 (9 rooms), Eh (5), 14h (3), 2, Bh, Dh,
  10h-13h (1 each); boss bit 1 in Brinstar (2), Norfair (1), Wrecked Ship (16), Maridia (2), Ceres
  (6); bit 2 in Brinstar, Norfair, Maridia; bit 4 in Crateria (2) and Norfair; the main-boss
  condition once (Tourian, `$DD58`); morph ball and missiles (2); power bombs (1). The door
  (`$E5EB`), morph ball (`$E640`) and speed booster (`$E678`) conditions are used by no room: the
  spec keeps `EnteredBy` and `Has` because the condition list is an imported asset and those are
  its words (constructed vectors).
- **The main-boss condition** (`$E5FF`) is `$E629` with bit 1: one condition, `Defeated` with
  `MainBoss`.
- **Location and door ids past 511.** `$80:818E` takes any id below 8000h; an id of 512 or more
  reaches past its table (location ids into the doors' bits, door ids into `D8F0`). The game's
  populations use ids below 100h. The importer refuses larger ones; not modelled (a LEGACY entry
  if a hack needs it).
- **The tanks don't cap.** A missile tank adds its amount to the count whatever the count is; the
  count never exceeds the capacity in play only because the capacity grows by the same amount.
- **A reserve tank doesn't fill the reserves** (`09D6` untouched), and an energy tank fills her
  energy to the new capacity.
- **The map byte.** The station writes FFh (with a word `ORA #00FFh`, keeping the next area's
  byte); the Tourian elevator room ORs 1. Leaving the area by an area-change door ORs FFh while
  `0789` is nonzero, and it is nonzero whenever the area's byte is (the Tourian room sets it, and
  every arrival copies the byte). So `OnArrival` turns into `Downloaded` on leaving; a save in
  Tourian before leaving (Mother Brain's save station, room `$DE23`) writes 01h to the file.
- **The order within a frame** (spec §7). The model applies a frame's reports in the order
  they come, and two pairs give a different progress in the other order: two beams taken, one
  switching the other off, and an area's map given on arrival with her leaving that area. Three
  blind reads of the model doubted the old claim that no order is visible; tested against the
  original, neither pair ever falls in one frame, so in the game none is seen:
  - **No frame takes two pickups.** Every pickup instruction (`$84:88B0` beams; `$84:88F3`,
    `$891A`, `$8941` items; `$8968`, `$8986` energy and reserve tanks; `$89A9`, `$89D2`,
    `$89FB` ammunition tanks) ends in `JSL $85:8080`, which doesn't return until the message box
    has gone (about 6 s; it spins on `$85:8136`, wait for lag frame). A second item PLM touched
    in the same frame runs its pickup in the same PLM pass, but only after the box closes, and
    the oracle records one record per video frame, lag or not (ORACLE §2), so the two reports
    land hundreds of records apart, whatever the content. Over the 44 cached rows, 14 frames
    take a pickup (`frames.py -v`), each alone in its frame.
  - **Leaving an area and the map on arrival are frames apart.** `$82:DF99` (leaving, on a door
    with bit 40h) runs in the door transition step `$82:E36E`, which sets the next step and
    returns; the room's setup code (`$8F:C90A` for Tourian's map) runs from `$82:EB6C` in the
    later step `$82:E4A9`, with the room's PLMs. Each step runs in its own pass of the main
    loop. The records show the gap with the PLMs' setup, which runs in the same call as the
    setup code: `$82:DF99`'s hit and the room's first item PLM setup (`$84:EE64`) are 37 records
    apart in area-norfair (163, 200), 31 in chain-0097-ACF0 (726, 757) and 39 in
    chain-0106-AD5E (528, 567). And the two name different areas: her leaving the one `079F`
    held before the room header's load, the arrival the new room's.
  - **Every other pair commutes**: different quantities, one count added to, or a set member set
    again. event-zebes-awake 152 is a recorded frame with two reports, `$84:BE0B` from two PLMs,
    both event 0 (R-planet-awake).

  So the vectors that put either pair in one frame (C-beams-in-order, C-beams-in-order-reversed,
  C-arrival-then-leaving, C-leaving-then-arrival) are constructed: they pin the model's rule,
  which is what the same reports give one per frame, as an adjusted game that sends both in one
  frame would see. C-door-from-tourian and C-door-into-tourian, which put her leaving and an
  arrival in one frame, now hold the door's frame alone, as the game gives it.
- **The gunship** marks Crateria's station 0 itself, not through the save station PLM: on its
  first landing (`$A2:A9A7`, then `$81:8000` saves; r01:610 is that landing, so R-gunship-save
  is not a save she chose) and when she saves in it (`$A2:AB2B`). To progress both are the same
  report.
- **Items placed.** The count is incremented by the item PLMs' setups; nothing reads it but the
  save (`$81:8000` copies it to the file). In r08:567 the room's only item was already taken
  (location 64), and the count still rose.

## The harness (for the integrator)

Progress is a store: other domains report, and nothing in this spec reads another domain's output
of the same frame, so it is verified on its own with the reports as inputs.

- **Frames:** every record pair N-1, N of every row. A frame with no report must leave progress
  as it was (the null model is the model there); the domain's score is over all frames, beside the
  null model's (18,289 of 18,299 on the 32 rows). Of the 10 frames that tell them apart, 7 are
  the stand-in's reports and so unverified (below), and 3 are ItemsPlaced: the model is matched
  and verified on 18,292 frames, 3 more than the null model.
- **Decoded from record N-1:** the State (§State).
- **Reports:** taken from the record's watched instruction hits, not from the RAM's change: watch
  the entry points of the Inputs table (`oracle corpus --watch-words`, the record's `hits`) with
  their arguments (the PLM's room argument at `$1DC7,x`, its instruction argument at `[Y]`, the
  kind in A at `$80:81A6`, the event in A at `$80:81FA`). The cached rows carry these watches'
  hits, but not their arguments. So a report its instruction names alone (an event marker, a
  zebetite, a map station, the room that gives its map, leaving the area, the gunship's station)
  and a pickup (the one item PLM at N-1 whose list runs the pickup instruction hit) are heard
  from the hits. What only an argument names (the kind of boss, the door opened, the station
  saved at, an event a PLM or the statues mark) is decoded from what changed and the item and
  door PLMs' state at N-1 (`gen.py` does: the PLM whose room argument the frame's bit is), which
  is a stand-in, so those frames are reported as unverified, not matched. A peek of A at
  `$80:81A6` and `$80:81FA`, and of `$1DC7,x`, would hear them too. The frame's area is `079F`
  at N, but for `LeftArea`, whose area is `079F` at N-1: `$82:DF99` runs at `$82:E373`, just
  before the room header's load (`$82:E376` → `$82:DE6F`, `STA $079F` at `$82:DE86`) in the
  same frame, so `079F` at N already holds the new room's area. `MapOnArrival` (the room's setup
  code) runs after the header, so it takes `079F` at N. No recorded row leaves an area whose map
  is `OnArrival`, so no recorded frame tells N-1's area from N's for `LeftArea`. Every door's
  transition runs `$82:DF99`, so `LeftArea` is heard only on a door whose header has bit 40h
  (`$83:0002,x`, x the door pointer `078D` at N-1). On the cached rows no frame runs `$8F:C90A`
  and no `$82:DF99` hit is on such a door: `MapOnArrival` is checked by the vectors only.
- **Compared at N:** every quantity of §State, masked only where another domain changed it. What
  she holds (energy, ammunition) is masked by name (the energy and weapons domains) on the frames
  it changed with no tank report (549 of 18,299); on every other frame it is compared, so a report
  that wrongly touches it is caught. The items and beams switched on are compared only up to the
  other writers: from the frame `$91:E5F0` runs (the hyper beam), `09A6` is Mother Brain's
  domain's to set, and the equipment screen's switches (F1) are masked by name on the frames its
  menu is open and the item, beam or reserve mode switched (none on the 32 rows).
- **ItemsPlaced:** the count moves when the room's PLMs spawn (an item PLM in record N that
  record N-1 doesn't hold), about 37 frames after the load, by the item PLMs of the state the room
  loaded in (from the ROM). 3 frames on the 32 rows.
- **Room states:** a room loads on the frame its door list `07B5` changes: loading the room header
  sets it (`$82:DED0`) just before the state is chosen (`$82:DED3`). Not `07BB`, and not the room
  pointer `079B`, which changes at the door header's load about 40 frames earlier. At each load,
  the state chosen from the new room's list (decoded from the ROM: `$8F:E5D2`'s format) against
  the progress at the load, compared with the state pointer. The 32 rows hold 7 loads; each row's
  first record also holds a standing state (32), chosen by a load before the row, compared against
  the same choice over that record's progress and door.
- **Null model:** nothing changes (18,289 of 18,299 frames); for room states, the default state
  (the last in the list): 7 of the 7 loads, and 23 of the 32 standing states (the 9 others hold a
  conditional state).
- **Free run:** carry progress forward from a row's first record with only the reports; it should
  equal the record at every frame. Re-seeding is never needed within a run: progress is never
  reset except by loading a file.

## The harness cases

The vectors hold RAM observations and invented content only (MODEL §1 decision 13). Fifteen
recorded vectors carried the game's content, so they left the vector file and are **harness
cases**: the harness, which decodes progress from the records and loads the ROM's content
through its importer, must match each named frame. Each case keeps its old vector's id, so its
history reads. The constructed vector beside it carries the same rule over invented content,
worked by hand from the spec.

**A case counts only when its report is heard**: from the record's watched hits, record N-1's
PLM slots and the ROM, as §The harness says, never stood in from what changed at N. A stood-in
report matches by construction (the stand-in copies the id from the bit newly set at N), so
such a frame is reported as unverified and its case is not counted.

**Reports.** Each case is the frame pair record N-1, N; the harness decodes before and after
from the records (§State) and hears the report (§The harness). The content is what the report
carries from the room's population: the item PLM's room argument (the location id), its
instruction's amount, the door PLM's room argument, the item PLMs the room's state spawns.

| Case | Row and record N | Report, and the content the importer loads | Compared at N | Constructed vector |
| --- | --- | --- | --- | --- |
| R-morph-ball | item-morph-ball 817 | `Taken(MorphBall)`, location 26 (the Morph Ball Room's item PLM) | `09A2`, `09A4`, `D873` bit 2; the rest unchanged | C-item-first |
| R-bombs | samus-bomb-jump-vertical 55 | `Taken(Bombs)`, location 7 (Bomb Torizo's room's item PLM) | `09A3`, `09A5`, `D870` bit 7 | C-item-second |
| R-missile-tank | s01 37 | `Taken(MissileTank)`, location 63, amount 5 (the room's item PLM and its instruction) | `09C8` 32h → 37h, `09C6` + 5, `D877` bit 7 | C-missile-tank-stocked, C-missile-not-full |
| R-door-a | r04 280 | `DoorOpened`, door 51 (the room's door PLM); heard | `D8B6` bit 3 | C-door-between |
| R-door-b | r06 519 | `DoorOpened`, door 85; stood in (a super missile opens it) | `D8BA` bit 5 | C-door-last |
| R-placed-two | area-norfair 200 | `ItemsPlaced(2)`: the room's state spawns two item PLMs | `D91A` 2Eh → 30h | C-placed-several |
| R-placed-a | r06 757 | `ItemsPlaced(1)` | `D91A` 38h → 39h | C-placed-one |
| R-placed-b | r08 567 | `ItemsPlaced(1)`, the room's one item (location 64) already taken | `D91A` 3Eh → 3Fh; `D878` unchanged | C-placed-taken |
| R-plasma-beam | item-plasma-beam 146 | `Taken(Plasma)`, location 143, with spazer switched on | `09A6` 1007h → 100Bh (spazer off), `09A8` 1007h → 100Fh, `D881` bit 7 | C-plasma-taken |
| R-ice-beam | chain-0163-A890 28 | `Taken(Ice)`, location 50, with wave on | `09A6`, `09A8` 1 → 3, `D876` bit 2 | C-beam-other |
| R-charge-beam | item-charge-beam 902 | `Taken(Charge)`, location 23 | `09A7`, `09A9` 0 → 10h, `D872` bit 7 | C-beam-other |
| R-wave-beam | item-wave-beam 37 | `Taken(Wave)`, location 68 | `09A6`, `09A8` 0 → 1, `D878` bit 4 | C-beam-other |
| R-energy-tank | item-energy-tank 240 | `Taken(EnergyTank)`, location 8, amount 100 | `09C4` 99 → 199, `09C2` 69 → 199, `D871` bit 0 | C-energy-tank |
| R-reserve-tank | item-reserve-tank 1212 | `Taken(ReserveTank)`, location 17, amount 100 | `09D4` 0 → 100, `09C0` 0 → 1, `09D6` unchanged, `D872` bit 1 | C-reserve-first |
| R-power-bomb-tank | chain-0043-9AD9 417 | `Taken(PowerBombTank)`, location 13, amount 5 | `09D0`, `09CE` 0 → 5, `D871` bit 5 | C-power-bomb-tank |

The item, beam and tank cases, the three items-placed cases and R-door-a are heard. The seven
from R-plasma-beam on are in the rows added since the cases were named (the cache-v2 rows), where
the harness already matches them frame by frame (`coverage.txt`: `TakingBeams 4`, `Tanks 7` of
8); the integrator names them as cases. The cache's other missile tanks (chain-0234-D51E 153,
item-charge-beam 296, item-reserve-tank 112 and 658) carry the same rule as R-missile-tank.

`coverage.txt`'s `Tanks 7` of 8: the frame left unverified is item-reserve-tank 112, not a named
case. Record N-1 holds two uncollected missile tank PLMs (room arguments 13h and 12h) whose lists
both use the missile pickup instruction that ran, so the hit alone can't say which was taken;
the harness hears none and the frame falls to the stand-in. At 658 the first is emptied and one
candidate remains, so it is heard; `frames.py` matches all eight because it takes the location
from the newly set bit, the stand-in's own source. A harness option for the integrator (the
root's call, it needs a re-record): a watched peek of X, or `$1DC7,x`, at `$84:8899` would hear
it.

The harness hears a door at `$84:8A91` when record N holds exactly one door PLM not yet open
whose hit counter
(`$7E:DF0C,x`) is one short of its count, and takes its room argument (`$1DC7,x`) as the door
id; `coverage.txt` has `DoorsStayOpen 2 0 5 0`: r04:280 (door 51) and boss-phantoon:1985 (door
134, no case of its own) are matched. R-door-b stays stood in (`Source::Copied`) and doesn't
count: a super missile raises the counter before the instruction counts, so its door, short by
more than one, can't be told from one that stays shut. boss-kraid-defeated:1074 (two doors in
a frame) is stood in too.

**Room states.** Each case is the state a room holds at a record (the row's first, or the load
on the frame `07B5` changes; §The harness), chosen from the room's state list as the importer
decodes it from the room header (`$8F:E5D2`'s format), against the progress decoded from that
record. Compared: the state pointer `07BB`'s state, as its index in the list.

| Case | Row and record | Room (header) and its state list | State | Constructed vector |
| --- | --- | --- | --- | --- |
| R-landing-power-bombs | samus-mockball 0 | Landing Site (`$91F8`): event Eh, power bombs, event 0, default | 1 (power bombs, over the planet awake) | C-state-earlier-wins |
| R-landing-default | r01 0 | Landing Site (`$91F8`), the same list | 3 (default) | C-state-none-hold |
| R-morph-room-default | item-morph-ball 0 | Morph Ball Room (`$9E9F`): event 0, default | 1 | C-state-event-not-yet |
| R-no-morph-ball | r03 267 (the load through door `$8B62`) | `$975C`: morph ball and missiles, default | 1 | C-state-morph-no-missiles, C-state-missiles-only |
| R-ceres-alive | samus-ceres-push 0 | `$E0B5` (Ceres): boss bit 1, default | 1 | C-state-boss-alive |
| R-torizo-alive | samus-bomb-jump-vertical 0 | Bomb Torizo's room (`$9804`): event Eh, boss bit 4, default | 2 | C-state-boss-not-yet |
| R-tunnel-whole | area-maridia 0 | Glass Tunnel (`$CEFB`): event Bh, default | 1 | C-state-other-event |

Six of the seven are rows' first records (every case but R-no-morph-ball): they are checked by
the harness test `progress_chooses_each_rows_standing_room_state` (44 of 44 standing states
on the scenes cache, null model 34), which is where the integrator names them, so that a fall
there is reported by its case. R-no-morph-ball (r03 267) is a load, compared on the frame (the
room state line of `coverage.txt`, `RoomStates 8 8 8 0`); every compared load takes the
default state, so the null model matches each one too, and this case doesn't tell the rule
from the default.

**No record: the full percentage.** C-percent-all carried the game's totals (energy 1499,
reserve 400, missiles 230, super missiles 50, power bombs 50; with the 11 items and 5 beams,
100). It becomes a harness test, not a floors row, needing only the ROM (`SM_ROM`) and no
record:
- **Every room.** No importer lists the rooms (the harness reads a room by its pointer), so
  the test walks the doors from the Landing Site (`$91F8`) and from Ceres's first room
  (`$DF45`), as `rooms.py` does: 261 rooms.
- **The population.** Every item PLM of every state of every room, deduplicated by location
  id (a location holds one kind across its room's states): 100 locations, 14 energy tanks,
  4 reserve tanks, 46 missile tanks, 10 super missile tanks and 10 power bomb tanks (amounts
  100, 100, 5, 5, 5), and 16 one-off items and beams. The test checks its walk against these
  counts.
- **The run.** From a new file (spec §1.1, The start of a run), each location's item fed to
  the model as one `Taken` report with its location and amount; then the percentage asked
  must be 100.

C-percent-many carries the rule with invented capacities.

**Constructed vectors changed with them.** The constructed vectors modelled on real rooms'
lists were rewritten over invented lists: C-state-event and C-state-first-holds (the Landing
Site's), C-state-boss (Bomb Torizo's room's), C-state-boss-other-area (a Brinstar boss room's,
boss bit 1 and default), C-state-morph-missiles and C-state-missiles-only (`$975C`'s). The
constructed location and door ids and tank amounts are invented (`edit_ids.py`). No list keeps
a real room's shape either: `real_lists.py` also compares each list's sequence of condition
kinds with the ROM's shapes of three states or more (the Landing Site's and the torizo rooms'
among them); 6 vectors had one, now 0. A two-state shape (one condition, then the default) is
the smallest that shows a condition, so it is not counted as a room's.

**Kept as vectors.** R-ceres-boss (samus-ceres-push 1586: `D82E` 00h → 01h, Ceres Ridley's
`Defeated(MainBoss)`), R-gunship-save (r01 610: `D8F8` bit 0, the gunship's station 0) and
R-planet-awake (e01 152: `D820` bit 0, from `$84:BE0B`, run by two PLMs in the
frame, so two reports) are RAM only: the boss's kind, the gunship's station and the event are
what those domains report, constants of their code, not a room's population or a table.

## Size baseline

**The comparison, by area** (adversary 44 finding 2, adopted by the root). The old code is
compared per area, each against the old code that is complete for it; neither whole figure is a
fair bar.

- Today's engine (spine `387640758`, 494.75 over all four cuts below) holds only slice 0's part of
  the domain: one item, no beams, tanks, doors, maps, stations or percentage. Only its rooms cut is
  complete: `Condition::holds` and `Room::select` handle all 9 condition kinds the ROM's list uses
  and the first-that-holds choice, like for like with `RoomStates`.
- The whole-game port (tag `archive/old-engine`, 2,421.25 over `progress.rs`,
  `gameplay_flag_state.rs`, `item_pickups.rs`) is no ceiling: `progress.rs` (1,350) is mostly RAM
  compatibility plumbing, with a 192 clump penalty; `item_pickups.rs` (844.25) carries W3's
  drawing and message table, a 172 depth penalty, and the energy domain's `restore`;
  `gameplay_flag_state.rs` (227) is a RAM byte layout, P2's. With a ceiling that high, "over the
  old code is a failure" could never fail. Only `item_pickups.rs`'s `collect` state arms are
  complete for an area: `TakingItems`, `TakingBeams` and `Tanks`.
- So rooms is capped against slice 0's state choice, equipment against the port's `collect`
  state arms, and happened, places and percentage, which no old code holds complete, are
  uncapped. The observer's reference model (`model.py`, about 170 Python lines) agrees that the
  earlier 1,200 was generous.

**Old code** (`tools size` of `5e2e4ee51`, the score v2 weights: lookups as data,
duplication priced, clumps off; through the DLL `$TEMP/collab/progress-r8/tools.dll`,
2026-11-11; code lines by `$TEMP/collab/scripts/code_lines.py`, unchanged):

| Area | Cut | Score | Logic | Types | Data | Depth | Code lines | Before (2026-11-08) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| equipment | `crates/sm_game/src/item_pickups.rs` at `archive/old-engine`: `PickupKind`, `Loadout`, `Supply`, `Inventory`, and `Inventory::collect`'s first `match` (the state arms), without the `VisualUpdate`/message match and `restore` (`$TEMP/collab/progress-spec/archive_cut/equipment_collect.rs`) | 231.75 | 211 | 143 | 12 | 40.75 | 72 | 253.5 |
| rooms | `data/src/room.rs` at spine `387640758`: `Item`, `Condition`, `Progress`, `Condition::holds`, `Room::select` (`$TEMP/collab/progress-spec/old/states.rs`) | 185.25 | 209 | 147 | 0 | 3.75 | 54 | 197.25 |

Both fell under the v2 weights: a match that maps one value to another is a lookup, counted as
data (equipment's pickup kinds, 12 tokens), and the depth term is lower. Neither cut has
duplication. The spec counter takes `old` in whole tokens, so the Budget section writes 232 and
185 (nearest).

### The other cuts (for the record)

`tools size` of `5e2e4ee51` (score v2), over cuts in `$TEMP/collab/progress-spec/old`, measured
together; the 2026-11-08 figures (the quality-terms counter built from spine `387640758`'s
`verify`) beside them; code lines by `$TEMP/collab/scripts/code_lines.py`:

| Area | Old code (today's engine, spine `387640758`) | Score | Logic | Types | Data | Duplication | Before |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| happened | `data/src/room.rs`: `Event`, `BossBits` and `intersects` | 15 | 32 | 63 | 6 | 0 | 26 |
| places | `game/src/state/mod.rs`: `COLLECTED_ITEM_WORDS`, `CollectedItems` and its `place`, `contains`, `insert`; `game/src/plm.rs`: `is_collected`, `NEVER_COLLECTED` | 128 | 143 | 41 | 2 | 12 | 123 |
| equipment | `game/src/state/samus.rs`: `Item`, `Items`, `Item::bit`, `Items::insert`/`contains`, `Gauge`, `Equipment`, `ReserveMode`; `game/src/plm.rs`: the two `insert`s in `resume` | 69.5 | 106 | 173 | 66 | 0 | 148.5 |
| rooms | `data/src/room.rs`: `Item`, `Condition`, `Progress`, `Condition::holds`, `Room::select` | 185.25 | 209 | 147 | 0 | 0 | 197.25 |
| percentage | none | 0 | | | | | 0 |
| total | 161 code lines | 397.75 | 490 | 424 | 74 | 12 | 494.75 |

Not counted: the state condition decoder `condition()` in `data/src/room.rs` (the importer's), the
HUD's and mini-map's reads (`hud.rs`, `mini_map.rs`: their domains').

The whole-game port as a whole, at the tag `archive/old-engine` (`crates/sm_game/src`, cut to
`$TEMP/collab/progress-spec/archive`): `progress.rs` 1,350, `gameplay_flag_state.rs` 227,
`item_pickups.rs` 844.25, 2,421.25 in all, 386 code lines (no ceiling, above); under score v2
(`5e2e4ee51`) 1,285, 247 and 808.25, 2,340.25 in all;
its room states and percentage are elsewhere in it and not counted (`inventory_domain.rs`, the
reserves, 349.75, is the energy domain's).

## Budget

Moved from the spec's header (MODEL §1 decision 10): the implementer is not told the score or
the budget. `tools spec-units` reads k and the areas here, and `tools report` the areas' old
figures and the commit.

Size is counted in logical tokens: each identifier, keyword, literal and operator is one, and so is
each bracket pair; comments, layout and tests are free. Two kinds of token are counted apart and
reported, not held against the target: the values of constants and statics (tables of the game's
data, such as which beam switches off which), and declarations (the `struct`, `enum` and `type`
items and attributes that define the model's shapes; define as many as the behaviour needs). The
target is on everything else, the logic.

**Baseline.** Each area is compared with the old code that is complete for that area (`tools
size`, score v2, re-measured 2026-11-11 at this round's base; §Size baseline). Rooms: today's
engine's state conditions and choice of state, which handle every condition and the first that
holds, 185.25 (197.25 under the earlier weights). Equipment: the earlier whole-game port's taking
of an item, a beam or a tank, without the drawing and the message, 231.75 (253.5). Happened,
places and percentage have no complete old code. The counter takes the old code in whole tokens:
185 and 232.

**Budget (MODEL §1 decision 7).** k = 37.5 weighted tokens per spec unit, frozen on samus-movement
and tested on the camera. An area's budget is k × its units; its old code (`old`), where it has
one, is for reference and doesn't bound it. The budget is an indicator: the fit is score ÷ budget, and being over it
costs in proportion to how far over, never as a wall; the old-code comparison is a guideline assessed under MODEL §1 decision 19.

- Area happened: EventsHappen, ZebetitesCounted, BossesDefeated
- Area places: EmptiedLocations, DoorsStayOpen, MapsByArea, MapOnArrival, StationsSavedAt,
  ItemsPlacedCount
- Area equipment (old 232): TakingItems, TakingBeams, Tanks
- Area rooms (old 185): RoomStates
- Area percentage: ItemPercentage

**Adjustable entries.** The two `Parameter` entries, each with vectors at a value other than
the game's, carried by the vector's `rules` (the vector header gained the key in round 8):
`TakingBeams` (C-switches-off-adjusted, C-switches-off-adjusted-other,
C-switches-off-adjusted-pair: taking ice switches charge and wave off) and `ItemPercentage`
(C-percent-adjusted: divisors 50, 200, 10, 2, 3).

**Units, round 8.** 30.3 → 28.3: `EmptiedLocations`' and `DoorsStayOpen`'s arms for an item or
door placed without an id became consequences, since their outcome is that nothing changes (the
counter's null-outcome rule) and the question without an id is gone. No arm was added: the
order within a frame, the direction of the switch-off table and the adjusted values are stated
as consequences and params.

Budget: 37.5 × 28.3 units = 1,061 (happened 125, places 312, equipment 287, rooms 262, percentage 75)
