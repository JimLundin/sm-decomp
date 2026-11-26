# Save-file decode and replay

[Behavior](save-files.md). Base is `b7dc44c8e`. Frames below name the local
lean-record index. `O` is an overrun. `R` is a requested NMI return. The
movie segment table uses zero-based sample numbers while the escape survey
counts some completed cuts from one. Use the row and index to select the
record.

## Sources by entry

| Entry | Source | Boundary |
| --- | --- | --- |
| ReadSlot | [Bank 81](https://patrickjohnston.org/bank/81/) `$81:8085..812A` and `$81:812B` | An invalid record copies its payload into WRAM staging and clears its cartridge payload. It leaves both checks in the image. Failure resets area and station to zero and does not publish profile or working explored maps. Success publishes the profile and expands the saved map. The checksum tests cannot mix words from different copies. |
| BootFiles | [Bank 80](https://patrickjohnston.org/bank/80/) `$80:85F6..86E2`, `$80:8261..82AB` | On the supported cartridge the diagnostic backs up the image, clears it, verifies the mirror using a descending word pattern, and restores the backup. This leaves no durable change. The boot probes files in menu order until a valid record is found. A successful probe tests the existing completion mark for four demos. If every probe fails the incomplete mark replaces the old mark. The region and faulty-mirror stop screens lie outside the supported power-on. |
| MenuFiles | [Bank 81](https://patrickjohnston.org/bank/81/) `$81:9ED6..A057` | The slot word must be 0 through 2. The stored second word only has to pass the AND test. The menu presents the result of each read in its own slot. |
| ChooseFile | [Bank 81](https://patrickjohnston.org/bank/81/) `$81:A219..A259`, `$81:B2CB..B3C4`. [Bank 82](https://patrickjohnston.org/bank/82/) `$82:EEC3..EEF5` | The slot and complement are stored on confirmation. A successful read also selects the current area's explored map through `$80:858C`. The new-file defaults set the fresh profile, clear ordinary explored maps and reset the named progress fields. A fresh file only becomes a cartridge record on a later save. The start route follows the loaded destination. |
| WriteFile | [Bank 81](https://patrickjohnston.org/bank/81/) `$81:8000..8084`, map packing `$81:834B..839E` and expansion `$81:82E4..834A`. [Bank 8B](https://patrickjohnston.org/bank/8B/) `$8B:C100..C11A`. [Bank 82](https://patrickjohnston.org/bank/82/) `$82:83A7..8410` and `$82:8032..803F` | Arrival captures the selected file after setting the Ceres destination, area and station. Escape changes the destination before capture and keeps the runtime Ceres area and station. The later landing branch selects Crateria's station for the landing sequence and loads its live explored map. It does not reread or rewrite the battery. |

## RAM and cartridge mapping

The existing `game::state::sram::SaveRam` decodes and encodes the full image
and implements the additive check. It has no live production owner at the
base. Its `Unused` currently includes the completion and selection fields.
Keep it as an exact codec if it can represent the new state losslessly.
`OwnedState` currently carries no cartridge. The file-select played runner
reads the remembered selection directly from the record and reads the occupancy mask from WRAM. Replace that dependency in an integrated run with
the same typed cartridge owner used by the scoped battery cases.

| Quantity | Original location | Exact representation |
| --- | --- | --- |
| primary checks | `$70:0000 + 2 * slot` and `$70:0008 + 2 * slot` | checksum and complement, little-endian words. Preserve the spare words of both columns |
| three payloads | `$70:0010`, `$70:066C`, `$70:0CC8` | each 1,628 bytes, through end-exclusive `$70:1324` for the last |
| retained bytes | `$70:1324..1FDF` | retain all 3,260 bytes on ordinary writes. The diagnostic restores the original values |
| completion mark | `$70:1FE0..1FEB` | 12 bytes. The completed and incomplete format words are given by Bank 80 at `$80:82B9` and `$80:82AD`. Preserve unrecognized marks on a successful startup probe |
| remembered file selection | `$70:1FEC..1FEF` | selected-slot word and its complement word. Preserve invalid starting values until a player confirms a slot |
| mirrored checks | `$70:1FF0 + 2 * slot` and `$70:1FF8 + 2 * slot` | separate checksum and complement. Preserve both spare words |
| active slot | `$7E:0952` when in a slot and thereafter | follow the confirmed choice, even if the slot was empty |
| menu and startup answers | `$7E:0954`, `$7E:0952`, `$7E:1F59` | occupancy bits, cursor, demo count |
| player profile | `$7E:09A2..0A01` projected at save to `$7E:D7C0..D81F` | 96 bytes. Equipment, capacities and reserves follow [run progress](run-progress.decode.md#state). Bindings are eleven words at `09AA..09BF`. HUD item is `09D2`, reserve ammo is `09D8`, play clock frames/seconds/minutes/hours are `09DA..09E1`, Japanese/moonwalk/weapon reset are `09E2/09E4/09EA`. Retain other profile words. Fresh sets `09E6` and `09E8` to one and leaves the profile's unassigned words as they were |
| events and recorded progress | `$7E:D820..D913` and `$7E:D91A..D91B` | 244 progress bytes plus two item-count bytes. Use [run progress](run-progress.decode.md#state) for typed fields. The fresh initializer clears `D830..D90F`. The menu's preceding failed reads clear the cartridge records, so selecting that slot also starts with no event, boss or item-count bits |
| restart point | `$7E:D914..D919` | destination, station, area in three words. In scope the destination is `0` (intro), `1Fh` (Ceres arrival) or `22h` (explosion and flight to Zebes). A normal Zebes resume can be represented without simulating its downstream route. `$7E:078B` and `$7E:079F` are the live station and area |
| explored map | live `$7E:07F7..08F6`, per area `$7E:CD52..D451`, packed `$7E:D91C..DA62` | Each map has 256 byte cells, eight tile bits per byte. At save replace the active area's remembered map with its live map. Project the named byte cells for Crateria, Brinstar, Norfair, Wrecked Ship, Maridia and Tourian into 327 bytes. The encoding uses lengths and map starts at `$81:8131..8145` and mapping pointers `$81:82D6..82E3`. At read expand over blank maps, including Ceres. Leave the live map to the caller. Ceres' working cells do not enter the packed image |
| remainder of payload | `$7E:DA63..DE1B` | 953 bytes. They are part of the checksum and return on load. On the fresh movie run they are zero after the cleared file is read. Carry them losslessly for arbitrary starting images |

For a checksummed image with unrecognized destination, area, option words or
bits, the checksum can still pass. Preserve the original representation at the
codec boundary even where the modeled interpretation is absent. The captured
route is recognized by the caller only when it has a corresponding supported
handoff. A failed read leaves the already-copied raw staging bytes in the
original. It is observable on boot before the menu reloads the cleared files.
Keep pending and copied payload only where a future comparison or consumer
reads it. The completed cartridge contains the **cleared** payload on failure.

## Exact-adapter contract

The confined `SaveFormat` tests run over the invented fixture. The production
adapter establishes the actual image layout in the table above. Its battery
projection must round-trip every original byte, including the four spare
words, unrecognized marks and both selection words. Both check pairs belong
to their own slot. A raw or malformed payload stays round-trippable even
when some values have no gameplay interpretation.

The adapter supplies the two marks as twelve-byte values obtained from the
cited exact format source at the ROM edge. It projects the typed snapshot
into the profile, progress, route, map and tail fields in the table. Use the
start route, area and station words as three independent values. Raw words
outside the named profile fields take their values from the last successful
read or the live profile. The two fresh defaults previously set by
`$81:B366..B369` become unsigned words equal to one at `$7E:09E6` and
`$7E:09E8`; these are the placement-disabled and spare startup values. Mark
the current map uncollected at fresh start, following `$81:A259`. Copy the
working profile and all carried words into the next saved payload after
applying any intervening settings changes.

The adapter supplies a per-area map projection using the cited format
lengths and offset lists at the ROM edge. Each entry identifies one byte of the
area's 256-byte map. Preserve the order and any repeated byte indices. On
load, zero the destination maps and apply each stored byte in projection
order. Use the live map rather than its older remembered copy when packing
its active area. Do not import either algorithm from the ROM. The imported
data provides the format inputs for the independently implemented encoder
and decoder. Both movie saves have a blank encoded map, so an actual-image
adapter test must use nonzero known selected and unselected map cells and
prove the projection and expansion. The fixture also proves a nonzero
Crateria cell survives and a nonzero Ceres working cell does not become a
Zebes cell.

An inspection's read receipt captures the whole pre-clear payload for the
staging projection. On valid reads, publish the decoded profile, maps and
location. On a failed read, the prior working profile and maps still apply.
Prepare the fresh profile from the prior working profile plus the initializer's
named overrides and an already inspected empty slot. The raw failed load
before inspection can carry nonzero events, bosses, item count, route and tail.
Keep those values in its receipt. The precondition for fresh start prevents
that payload from becoming a fresh run. A stale checksum accepting the zero
payload still returns a valid load.

A picture of physical diagnostic SRAM can be derived from the initial
cartridge and progress inputs. For the image of 4,096 little-endian words,
the mirrored test writes `4095 - word_index` at each word it has reached.
The clear and restore sweep from the end toward the beginning. The first
bad-file erase makes progress toward the end of its slot, with prior slot
erasures already reflected in storage. Those progress positions are
supplied by the scoped timing witness. Neither it nor the pending writes
accept SRAM bytes from a later record. At the selection interruption, the
exact output has the new selection and the old working profile. At each
save interruption, it has the new payload and the old checks. Restoration
carries the published image and the pending slot and inputs needed to
finish. The required indexed adapter proof compares each full reconstructed
image, checks unassigned and opaque bytes, then repeats the next completed
cut from the checkpoint. Cover wrong-slot, missing-mirror and omitted
route or map serialization with production faults.

## Indexed witness plan

The lean codec reconstructs SRAM along with WRAM, VRAM, CGRAM and OAM. The
records contain all 8,192 bytes and can be compared byte for byte without
pictures or new captures. Fold-only digests attest whole spans and cannot
identify the first differing cut or byte. Stream availability does not mean
the current indexed production runner compares it. `verify::fold` can take
an optional SRAM image and the existing file-select played probe reads oracle
SRAM as input. The new observation needs a required output at the named
boundary, failing explicitly if the owner or bytes are missing.

| Row and cuts | Independent input and output proof | Production obligation |
| --- | --- | --- |
| `any-power-on:0..271` | Cut 0 supplies the movie battery. Cuts 263–267 expose the mapping diagnostic. At 268 the backup has been restored and the first invalid slot is partly cleared. Clears span 268–271. At 271 all three payloads are zero, all checks, spare words and the selected-slot record come from the initial image, and the incomplete mark is present. Demo count is three | Carry the initialized cartridge into title. Capture the boot completion receipt and the full persistent image. At busy cuts the timing component supplies diagnostic and erase progress to reconstruct the interrupted image. Declare any durable-only startup comparison explicitly and keep full-cut comparison as the milestone obligation |
| `any-power-on:396..400,431..433` | At 400 all slot answers are empty and the menu cursor is A. At 432 only two bytes change, both in the chosen-slot word. At 433 the working profile shows the fresh defaults, all named progress and ordinary maps are empty. The cartridge is unchanged from 432 through 567 | Drive the menu from the live battery. Confirm through the file-select controller path. Compare selection bytes independently of the returned active slot and fresh profile. Keep settings and slot through the options handoff |
| `any-intro-0568:0` through `any-fly-to-ceres:802` | The battery is carried with no writes. The chosen slot persists. Fresh profile plus options changes supply the arrival's inputs. The options case observes moonwalk on at `any-power-on:519`. The earlier default is off | The options-to-flight route must preserve active slot and profile. Movie-mode intro/demos must leave the actual run ready for arrival |
| `any-fly-to-ceres:802`, `any-ceres-arrival:0..1` | The preceding cut provides profile and progress. Capture the new destination, Ceres area and station 0. At 0 (`O`, movie sample 8,315) the payload is visible. At 1 (`R`) exactly the eight check bytes change. Arrival saves full energy, empty acquired gear and zero play time, with the options settings retained | On the flight handoff produce the snapshot and the pending write, then finish both checks with no oracle reseed. Ceres Arrival consumes the continued live fresh run. Compare all selected and unselected image bytes on both cuts |
| `chain-any-0011-DF45:239..241` | The preceding cut provides live progress, the current Ceres area and station, energy 24 of 99, and elapsed time 0:00:51.55. The escape changes destination to Zebes Flight while leaving Ceres/station 0 in the saved location. At 240 (`O`, movie sample 13,312) five payload bytes change and both old checks persist. At 241 (`R`) eight check bytes change. Five nonzero bytes in the Ceres working map do not enter the packed six-area map | Capture the escape state on the blackout handoff and carry the pending check over the interruption. Derive the payload from the live owners and compare the resulting full image. Building the payload from the staged save copy, or leaving the arrival image in place, cannot establish this witness |
| `chain-any-0011-DF45:242..2123`, `any-zebes-arrival:0` | Every cartridge byte retains the escape output across the explosion and flight and at the start of landing setup. Live energy is 99 by the flight's handoff and the setup changes the live location to Crateria, station 18. Neither becomes the stored restart point | Carry the active slot and cartridge to the landing handoff. Compare the retained image independently of live location and live health. Room loading and the actual Landing Site save are the next milestone |

Use [the replay transition](../cleanroom/PROCESS.md#replay-transition)
when adding these cases. The scoped save witness can seed an earlier snapshot
and supply causal inputs for the other owners. Its comparison reads the
independent output records. Integrated replay must use live [Health](samus-health.md),
[Run Progress](run-progress.md), [HUD](interface-hud.md), [Play Time](game-time.md)
and [Options](screens-options.md) state, the live maps and confirmed slot.
Those producers and their handoffs must be available at capture. No case
loads an oracle image at each write, reconstructs the run from the already
written file, or calls an alternate verifier-only encoder in lieu of the
production path. Full-movie acceptance still needs the platform to produce
its own progress for the interrupted writes and diagnostic.

Use the initial battery and checkpoint continuation to prove preservation
of unselected payloads, spare checks and unrelated bytes. Restore after the
new payload is visible and before the checks are written. Recheck the same
result as continuous play even if the caller's live state changes during
the interruption. Candidate planted faults include a wrong slot, an AND
test replaced by a complement test, one missing check copy, clearing spare
bytes, capturing after health restoration, passing Ceres explored bits into
the packed map, and losing pending checks at restore. The movie's first
different bytes for a missing write are 432 for the selection, arrival cut
0 for the first save and escape cut 240 for the second. Use the constructed
non-A slots and weak cursor test to cover paths that the movie misses.

## Reference

The local `save-obs-notes/reference.py` runs the constructed vectors and
carries one cartridge image across 18 lean rows, seeded with the power-on
image once. It reconstructs the two save payloads using the preceding
live WRAM input, the scene's destination and station rules, the format's
map projection read at the local ROM edge, and the independent checksum
arithmetic. It compares against the reconstructed cartridge bytes on
each record. It does not use `D7C0..DE1B` at or after either save as the
snapshot input. It checks the fresh profile separately against the earlier
working profile and the default values.

All 19 constructed vectors agree. The synthetic format accounts for typed
destination, area and station in the encoded checksum. The invented cases
match 13 complete image digests computed with explicit expected layouts,
including the mark update, fresh profile's later write, untouched fields, unknown codes and checkpoint continuation. Across the 18 rows the reference
matches 15,196 of 15,196 complete images, versus 263 for the fixed initial
image. A second check with the project's Rust lean codec independently decoded 3,950
records across the five boundary rows. It confirmed the same change cuts and
byte counts, including the first landing record. The eight diagnostic/erase
lag cuts use observed timing positions,
with bytes generated independently. The other 15,188 cuts all match,
versus 263 for the fixed initial image. The two save opportunities and their
interrupted completion points are supplied by the named scoped witness.
Integrated production replay and authored timing remain open.

## Budget

The legacy branch at `b7dc44c8e` has only the exact codec and scoped decode
at this boundary. There is no attributable portable behavior producer.
The codec is retained at the exact edge, outside this model's baseline.

k = 37.5 weighted tokens per spec unit.

- Area cartridge (old 0): Memory and boundary, ReadSlot, BootFiles, MenuFiles
- Area run (old 0): ChooseFile, WriteFile

The exact codec scores 533 weighted tokens at this base, measured with
`tools size crates/game/src/state/sram.rs`. The spec counter reports five
entries, eleven arms, one table term and eight operators.
Budget: 37.5 × 14.0 units = 524 (cartridge 337, run 187).
