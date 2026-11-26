# Save files

The cartridge remembers three files, the last file chosen and whether the game has
been completed. Choosing a file starts a run or resumes one. The game records a
restart point at the Ceres arrival and again at the escape.
[Vectors](../../scenarios/vectors/save-files.toml).

## Header

**Scope.** The cartridge check and startup file reads, file selection, starting a
fresh file, and both automatic saves through the flight to Zebes. The cartridge
needs a live owner. The existing battery image codec already represents the three
records, both checksum copies and the remaining bytes.

**Reads.** Boot requests the cartridge check and startup read. File select
requests a menu scan and reports the confirmed slot. Options supplies changes to
settings and controls. At an automatic save, the scene gives the restart point
and the live owners supply equipment and persistent progress, current energy and
ammunition, the selected weapon, settings and controls, explored maps, and play time.

**Reports.** The complete battery image, the startup demo count, the menu's
initial cursor and file summaries, and the active slot and start data. Fresh and
resumed data go to the live owners that supply the next save. Each file summary
reports whether the slot held a file and, on success, its energy capacity, energy
and play time.

**Leaves to others.** Scenes choose when to make a request and consume its
result. The live owners run gameplay after receiving start data. Cartridge bus
timing, CPU work within a frame and the timing of an interrupted transfer belong
to the platform. File copying, clearing by player choice, save stations and
actual gameplay after the Landing Site handoff are outside this scope.

## Memory and boundary

Cells: quantities and readers at the save boundary.

| Quantity | Units and domain type | Read by |
| --- | --- | --- |
| cartridge | `Cartridge` with three `SlotRecord` values, `RememberedSelection`, `CompletionMark` and `RetainedCartridgeData` | BootFiles, MenuFiles, ChooseFile, WriteFile, output |
| slot record | `EncodedRun`, 814 unsigned 16-bit words, and two independent `CheckPair` values, each two unsigned 16-bit words | ReadSlot, WriteFile, output |
| remembered selection | two unsigned 16-bit words, including unrecognized values | MenuFiles, ChooseFile, output |
| completion mark | `CompletionMark`, 12 opaque bytes | BootFiles, output |
| retained cartridge data | 3,260 opaque bytes and four spare unsigned 16-bit words | output |
| active slot | optional `FileSlot`, one of the three labelled slots | ChooseFile, WriteFile, scene handoffs |
| run data | `RunSnapshot` with `PlayerProfile`, `Progress`, `RememberedMaps`, `RestartPoint` and `RetainedRunData` | ReadSlot, ChooseFile, WriteFile, options and gameplay owners |
| working profile | `PlayerProfile` with retained profile values, carried from the last successful read or current run | BootFiles, MenuFiles, ChooseFile |
| file read result | `ReadResult`, copied `EncodedRun` plus either a decoded run or empty with the default location | BootFiles, MenuFiles, ChooseFile, output at the exact boundary |
| inspected menu | a completed `MenuFiles` result for this cartridge session | ChooseFile |
| restart point | `RestartPoint` with a destination, an area and an unsigned 16-bit station number | ChooseFile, WriteFile, options and the next scene |
| demo count | `DemoCount`, three or four sets | the title screen |
| pending choice | optional selected slot and prior working profile, with the selection words published | ChooseFile |
| pending write | optional selected slot and the new pair, with the captured payload published | WriteFile, output |

State can be carried, inspected and restored, including the pending actions.
No write updates a different slot, the spare check words or the retained
battery bytes. Start with the cartridge supplied at power-on and advance it
with requests and live owner values.

### The exact format boundary

The model depends on a supplied `SaveFormat`. It has three responsibilities.

1. Supply the fixed completed and incomplete marks as exact byte values. The
   model compares the current mark byte for byte and can assign the incomplete
   mark. The supplied marks are distinct. The model carries all other mark values through a read.
2. Encode a complete `RunSnapshot` into all 814 words of an `EncodedRun`.
   Decode those words back into a typed snapshot. An unrecognized field remains
   representable as an opaque encoded value, with its exact width and contents.
   The format supplies any pass-through values needed to preserve it when other
   fields change. Decode also exposes the stored area, station and destination
   independently. For unsupported destinations the caller receives the
   unrecognized route and decides which scenes it can start. The checksum
   validates words even when a field has no recognized meaning.
3. Map between the 8,192-byte image and the complete `Cartridge`, including
   the spare words and untouched bytes. The exact adapter uses this conversion
   at its input and output edges. The model operates on the typed values and
   requests encoding at capture. Round trips of an arbitrary battery image
   preserve each byte. For edited snapshots, all words covered by the checksum
   come from the format's output, including route, location and opaque values.

The actual format is supplied at integration. Its explored-map projection
comes from the exact edge. The clean model tests use the invented format
in the vectors. That format assigns its own byte positions and codes. The
adapter must prove the real mark, the full typed payload encoding, the
six-area projection, and lossless battery bytes with independent observations.

The player profile includes equipment owned and in use, control bindings,
settings, energy and capacity, reserves, ammunition and capacities, the
selected weapon, the four components of play time, a placement-disabled value,
a spare startup word, and other retained words. Progress includes events,
defeated bosses, acquired items and their count, opened doors, map and save
stations and elevator progress. A typed snapshot also holds pass-through
progress data and the retained tail. Only the reads and initialization that
write those values replace them. The map value has 256 bytes per area, each
byte covering eight cells. At a save, use the current area's live map in place
of its remembered map. The supplied format packs its named cells for the six
areas on Zebes. On read, expand onto blank maps and apply the packed cells in
format order. The explored cells of Ceres stay empty on expansion.

### Publication and continuation

The caller identifies an admitted request and tells the model how far it
finished. For a choice, the two milestones are `SelectionWritten` and
`ChoiceLoaded`. Start by storing the selection words and slot, with the
working profile retained for the load. On `ChoiceLoaded`, read the selected
slot as it was left by the menu scan, apply its result and clear the pending
choice. A finished request may reach both milestones in the same frame. The
active run becomes available only on `ChoiceLoaded`.

For a save the milestones are `PayloadWritten` and `ChecksWritten`. Capture
once, encode once, publish the selected payload and retain its checksum with
the pending slot. On `ChecksWritten`, publish both pairs and clear the pending
write. A finished request may reach both milestones in the same frame. Resume
a pending action with its completion input. No new choice, read or save runs
between these milestones. Restoring a checkpoint after the first milestone
carries the complete image, the pending action and the prior data needed at its
completion. Caller values supplied for a later capture have no effect on a
pending save. The platform owns finer staging progress within each milestone.

## Rules

### ReadSlot

- kind: mechanic
- summary: accept either complete matching checksum pair, or clear the damaged file.
- params: none
- switch: fixed
- Either complete pair matches → return the copied payload and its decoded
  run. Keep both original pairs in the cartridge. Compute
  `sum = sum(words) mod 65536` and `complement = 65535 - sum`. A pair passes
  only if both values match [C-primary, C-mirror, C-opaque-encoding].
- Neither pair matches → report empty, with Crateria and station zero.
  Return the original payload in the read receipt and clear the cartridge's
  payload. Preserve both pairs and the existing working profile and maps.
  The original copied payload remains available in the receipt after the clear
  [R-boot, R-menu, C-split-check, C-stale-check, C-failed-copy].
- checked: oracle (R-boot, R-menu). Matching and stale checks are constructed.

### BootFiles

- kind: mechanic
- summary: check the cartridge and find out how many demos are available.
- params: none
- switch: fixed
- Valid cartridge layout → restore the original cartridge bytes when the check
  finishes. The diagnostic can temporarily clear the image and write its test
  pattern. Its interrupted progress is a platform input [R-boot, C-boot-stops].
- The startup reads find a valid file → stop at the first valid slot in menu
  order. Report four demos if the completion mark says completed and three
  otherwise [C-boot-stops, C-boot-unmarked].
- All three startup reads fail → mark the game incomplete and report three
  demos [R-boot, C-boot-empty].
- checked: oracle (R-boot). The startup reads with a valid record are constructed.

### MenuFiles

- kind: mechanic
- summary: inspect all three slots and recover the last cursor choice.
- params: none
- switch: fixed
- File menu loads → read all three slots and return their summaries. Carry
  each successful decoded working profile into the next read. A failed
  read keeps that profile. Use the
  remembered slot for the initial cursor if `0 <= slot <= 2` and
  `(slot & selection_check) == 0`. Otherwise begin at A
  [R-menu, C-cursor-weak, C-cursor-overlap, C-cursor-range, C-fresh-keeps-profile].
- checked: oracle (R-menu). A remembered choice is constructed.

### ChooseFile

- kind: mechanic
- summary: record the player's choice and provide the run to start or resume.
- params: none
- switch: fixed
- Player confirms a slot after a completed menu scan → at
  `SelectionWritten` set the active slot and record the slot number
  (A is 0, B is 1, C is 2) and its 16-bit bitwise complement. Retain the
  prior working profile and publish the run at `ChoiceLoaded`
  [R-choice, C-select-mirror, C-select-resumes].
- The chosen slot loads → restore the saved profile, progress, location and
  maps. The current area uses its restored map. The explored map of Ceres is
  empty. Options receives the start route from the saved destination
  [C-select-mirror].
- The chosen slot still fails when the load finishes → start with 99
  current and maximum energy, zero ammunition and reserves, no equipment
  and no weapon selected. Clear acquired items, changed locations, door and
  station progress, elevator progress, map collection, remembered maps and
  play time. Leave the unassigned working profile values from the menu
  session. Set placement disabled and the spare startup word to unsigned
  one. The inspected slot's copied progress, route and retained tail are
  zero because the scan already cleared it. Start at the intro with no
  events, boss results or acquired item count. Move with directions, jump
  with A, run with B, shoot with X, cancel items with Y, select items with
  Select, aim up with R and aim down with L. Japanese text, moonwalk and
  weapon reset start off. The choice words are the only persistent write for
  an already-cleared slot [R-choice, C-select-fresh, C-fresh-keeps-profile,
  C-select-resumes].
- checked: oracle (R-choice). Resuming a populated slot is constructed.

### WriteFile

- kind: mechanic
- summary: save the active run at the two cutscene transitions.
- params: none
- switch: fixed
- The trip to Ceres ends → capture live state with the Ceres Arrival destination,
  Ceres area and station zero. Publish the active slot's payload and
  then its checks [R-arrival, C-save-B, C-save-resumes, C-fresh-keeps-profile].
- The Ceres escape fade ends → capture live state with the Zebes Flight
  destination and the current area and station. Publish the active slot's
  payload and then its checks [R-escape, C-save-C].
- checked: oracle (R-arrival, R-escape). Saves in other slots are constructed.

## Order that matters

Boot and menu inspection publish the read receipts in menu order. A successful
receipt supplies the working profile and the remembered maps. A failed
receipt carries the read payload and default location, with the previous
working profile and maps still in use. Choosing a slot after inspection uses
the latest working profile and reads the selected slot again.

The boot check restores the battery before the startup reads can clear damaged
files. The file menu reads the three slots in menu order. A failed read preserves
both checks, so the next read may accept the cleared payload if one retained pair
matches zero. The menu scan must finish before a choice. A failed read of an uninspected
slot can copy damaged run data into its receipt. It cannot start a new file on
this path. The slot choice is visible
on the cartridge before the active run is ready. Options can change the active
run before the flight to Ceres captures it.

Each automatic save publishes its payload before the check pairs. Both saves in
the movie have an interrupted cut with the new payload and the preceding checks.
The next requested cut has both new check pairs. Reach the Landing Site handoff
with the escape save intact. Starting that room sets the runtime location to
the Landing Site and reads the live map for that area. The cartridge still
holds the restart point for the flight.
