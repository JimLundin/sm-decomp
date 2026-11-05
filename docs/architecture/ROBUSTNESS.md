# Robustness review

Architecture lane, `research/architecture`, 2026-11-05. This is a companion to
[ARCHITECTURE.md](ARCHITECTURE.md). It cites its decisions (D1-D24) and the
[principles](PRINCIPLES.md).

For each future feature, this review asks four things: how the feature is added, what it
touches, whether gameplay changes, and whether the foundation must change. Where a feature
needed a foundation change, the change has been made in ARCHITECTURE, and the decision that
carries it is named. The features are Jim's list (widescreen, audio upgrades, quick
transitions and other timing options, native lag, custom content, multiple oracles, Map
Rando), followed by others that are likely.

**Summary.** None of the listed features needs a foundation change *after* this draft. Six
of them needed one compared with FOUNDATION, and each is now a decision:

| Feature | What FOUNDATION lacked | Now carried by |
| --- | --- | --- |
| Widescreen | meaning beyond SNES-layout bytes; one gameplay window | D16 intent output; the `Viewport` in D19 |
| Native lag | a place for overrun timing | D8 timing input; D9 cost model |
| Multiple oracles | anything outside the old stepper's rules | D6, D11, D17, D19's `OracleRules`; D22's locks keyed by oracle |
| Audio upgrades | a backend seam that can't change gameplay | D18: driver model in the Session, synthesis behind `SoundBackend` |
| Quick transitions | timings as data | D19's resolved `Rules` tables |
| Custom content and saves | stable identities apart from engine indices | D15 dense IDs plus stable keys, overlays |

One plausible feature would still need a foundation change: two Samuses in one world
(section 9). It isn't on Jim's list, so it is recorded here rather than designed for.

## 1. Widescreen (the hard case)

### What the original does, checked in the bank logs

- **Level tiles are streamed only about one column past the screen.** When the screen
  scrolls, `$80:A3DF` ("Update BG graphics when scrolling") draws one block column: "If
  scrolled rightwards: X = 10h, else X = 0", then "Blocks to update X block = [layer 1 X
  block] + [X]". Scrolling right draws block column +16 from the camera's left edge (the
  column just past the 256 px edge). Scrolling left draws column +0. Rows work the same way
  ("If scrolled down: X = Fh, else X = 1"). Layer 2 uses the same offsets. So outside the
  visible 256 px plus one 16 px column, the BG tilemap in VRAM holds **stale tiles** from
  wherever the camera was before. A wider view of the tilemap shows garbage. bsnes-hd's
  widescreen mode for this game shows exactly that.
- **Enemies off screen aren't processed.** `$A0:8EB6` ("Determine which enemies to process")
  skips an enemy unless it overlaps a window around the camera: "If (enemy right boundary) +
  1 < [layer 1 X position]: next", "If [layer 1 X position] + 100h < (enemy left boundary):
  next", and vertically "[layer 1 Y position] - 8 <= [enemy Y position] <= [layer 1 Y
  position] + F8h". The exceptions are property `$0800` (processed off screen), frozen
  enemies, and a global off-screen flag. An enemy outside the window runs no AI and no
  instructions, and it is **not drawn**: the draw pass draws an enemy only "if (enemy is on
  screen or enemy uses extended spritemap format)".
- **Other screen-bound rules.** Samus projectiles are deleted when too far off screen
  (`$90:B16A`, "Delete projectile if too far off-screen"; its bounds are confirmed when the
  beam family is built). Sprite X positions are 9-bit in OAM (-256 to 255), so a sprite past
  256 px can't be expressed in OAM. HDMA window positions are 8-bit (0-255), and that is how
  the power bomb, the x-ray cone and the crystal flash are drawn. The camera is clamped to
  the room's bounds and stops at hidden scroll regions.

So the exact output, the SNES-layout image, **cannot** supply a wider picture. The extra
columns must come from somewhere else, and enemies near the edge have a gameplay question
attached.

### The design

**Two separate things, at two seams.**

1. **Wide view: a presentation profile.** Gameplay is unchanged, and so are the fingerprint
   and every compared stream. This is the default meaning of "widescreen".
2. **Wide gameplay: a gameplay option.** The processing window and the other screen-bound
   rules widen. This changes behaviour, so it isn't vanilla and it is fingerprinted. Whether
   to offer it is [QUESTIONS Q1](QUESTIONS.md#q1-may-widescreen-change-gameplay).

**What renders the extra columns.** The `runtime` **wide renderer** composes a picture of
`W` columns (for example 400 for 16:9 at 224 lines) from the intent view (D16) and the video
memory mirror:

| Layer | Source for the wide picture |
| --- | --- |
| BG1 and BG2 level layers | the room's block grid in the State (`View.level`), through the tileset's tile table, into tiles drawn with the character data and palettes from the video memory mirror. Every block of the room is in the State, and the tileset's graphics are fully loaded in VRAM for the room, so every column is correct. Blocks that PLMs change (door caps, crumbling blocks) change in the level grid, so they appear in the margins too. |
| BG2 static backgrounds (not level data) | the tilemap in video memory, wrapped the way the PPU wraps it. That is what the console would show if it were wider. |
| BG3 (FX layers, HUD) | FX: the tilemap with wrap, plus the HDMA scroll curves per line. HUD: drawn at native 256 px and centred, with the sides extended by the presentation profile's HUD style. |
| Sprites | `View.sprites`: full-precision entries, so nothing is clipped at 256. The per-line sprite limits are applied to the original's 256 px entries exactly as the exact compositor applies them. Margin entries are extra. |
| Enemies outside the processing window | **margin draws**: the same draw functions run on the actors' current State, with no AI and no instructions run. The presentation profile chooses `MarginActors::{Hide, ShowFrozen}`. |
| Window effects (power bomb, x-ray, crystal flash) | recomputed at width `W` from `View.effects` parameters (centre, radius, angle), not from the 8-bit tables |
| Outside the room, and hidden scroll regions | black. The margins don't reveal what the original's camera limits hide. That covers secrets behind hidden scroll regions, which `View.level`'s scroll map identifies. |
| Non-gameplay screens (title, menus, pause, map, Ceres Mode 7, cinematics, credits) and door transitions | pillarboxed: the native 256 px, centred. A screen can later opt in through a per-mode flag in the presentation profile. |

**What gameplay may and may not change.**

- **Wide view:** nothing. The wide renderer receives `&State` and the `View` (K4, P18). It is
  checked on every corpus row: at `W = 256` its picture must equal the exact picture (D16's
  check). So the wide renderer's interpretation of the State is proven against the oracle on
  every row, and the margins are an extension of a proven function.
- **Wide gameplay:** every "is it on screen" rule in gameplay reads one `Viewport` value from
  `Rules` (D19). The vanilla `Viewport` is 256 × 224 with the original's margins (+1 px left,
  `100h` right, -8 to `F8h` vertical). That covers the enemy processing window, the draw cull,
  the projectile deletion bounds and the camera clamps. So a wide-gameplay option changes one
  value, not a hunt through the code. A source check enforces it: there is no literal 256 or
  224 in gameplay modules, and those values come from `Viewport` (K1's spirit).

**Foundation change made in this draft.** The intent output (D16), with one drawing source
for exact and intent sprites, and the `Viewport` in `Rules` (D19). Without them, widescreen
would need either a second drawing path or a change to the exact path.

**Residual limits (presentation only).** With `ShowFrozen`, an enemy in the margin sits still
until it enters the vanilla window. With `Hide`, it appears at the vanilla edge. Both are
honest consequences of vanilla gameplay, and only wide gameplay removes them.

## 2. Audio upgrades

| Upgrade | How it is added | What it touches |
| --- | --- | --- |
| Higher-quality resampling, interpolation (cubic, sinc instead of the DSP's Gaussian), output rate, filters | a `SoundBackend` setting in the presentation profile | `runtime` only |
| Replacement instrument samples | a media pack keyed by `(SongSetId, instrument)`. The sequencer plays the same notes with other samples. | media pack; backend |
| MSU-1-style music replacement | `MusicSource::Stream(map)`, mapping each cue `(SongSetId, track)` to an audio file with loop points. The N-SPC music voice is silenced. | presentation profile; backend |
| Replacement sound effects | a media pack keyed by `(library, SoundId)` | media pack; backend |

**Why gameplay can't change.** Everything the game can observe about sound (the music queue,
the library handshakes, the driver's echoes and their timing) is in the Session's CPU
protocol and driver model (D18). Backends receive intent only and return nothing. If the game
ever waits on a driver-reported state, such as a song upload finishing, that state is part of
the driver model, so a stream backend can't shorten or lengthen it.

**Verification.** The vanilla backend is checked sample-exact against the oracle's audio tier
nightly. Replacement backends are outside exactness and are tested for coverage: every cue
the corpus plays maps to a file, or to a recorded fallback to N-SPC.

**Foundation change:** none after this draft. Compared with FOUNDATION, the driver model's
place is made explicit (D18).

## 3. Quick transitions and other timing options

Examples: faster door transitions (Map Rando ships them), skipping the "wait for music"
step, fast elevators, a shorter item fanfare or message box, faster pause fades, skipping
the intro or Ceres, and instant save confirmation.

**How it is added.** Every one of these is a value in a typed `Rules` table that one module
reads (D19). Examples:

- `rules.doors: DoorTiming { fade, scroll_speed, wait_for_music, load_cost }`;
- `rules.messages: MessageTiming`;
- `rules.elevators`;
- `rules.start: StartDefinition`, where the new-game strand can be skipped.

Vanilla's tables are decoded from the ROM or written from the bank logs. An option is a
different table. With `TimingSource::Modeled`, an "instant load" option scales or zeroes the
loading costs that D9 charges.

**What it touches:** `data` profiles, and the one module that reads each table. The Tick,
strands and State are unchanged.

**Gameplay:** these options are gameplay (fewer Ticks, so fewer RNG calls, different
in-game time). They are fingerprinted, with no oracle. Two checks cover them:

1. The option's own scenarios, with facts.
2. **A carried-state invariant.** The same route, run under vanilla and under the option,
   must arrive with the same carried State (everything outside `RoomScoped` except the clock,
   RNG and timers). This uses vanilla as the oracle for everything the option shouldn't
   change.

**Foundation change:** none, as long as no frame count is folded into code. That rule is
FOUNDATION §5.3's and D19's, and review enforces it: a frame count in a step function is a
finding.

## 4. Native lag

**How it is added.** D8 and D9 already provide the seam. The timing input says whether an NMI
arrived during work. The `Modeled` source computes it from the cost model, which counts
named work against the frame's budget. Native lag for busy rooms means:

1. Add cost-table rows for gameplay work: enemy AI by species, projectiles, collisions,
   sprites drawn, VRAM queued. Each pass charges its work, the way the loading routines
   already do. This charging is part of the gameplay model, because lag is behaviour. It is
   not a verification probe (P6).
2. Measure the costs on the oracle, which has a real beam, over the corpus.
3. Gate the result with the `timing` stream: the Modeled schedule against the oracle's, on
   every row.

**What it touches:** the cost table (data), one `charge` call per pass phase, and nothing else.
Gameplay verification keeps `Recorded` timing, so the model's accuracy never masks or causes
a gameplay divergence.

**What it can't do without more work.** Tearing inside an overrun frame needs work placed at
lines. The same cost model can provide that later through beam-sync points inside frames
(D7). Until then, those frames are LEGACY rows.

**Foundation change:** none after this draft. FOUNDATION had none (J1 and J18).

## 5. Custom content

| Content | How it is added | Code needed? |
| --- | --- | --- |
| New or edited rooms, layouts, collision, scrolls, FX | an overlay of `Room` records (D15) | no |
| Doors, placements, a new world map | `Placement`, `Doorway` and `Topology` overlays (FOUNDATION §7.3) | no |
| Enemy and PLM placements, item placements | room-state records | no |
| New instruction lists for existing families (a new enemy animation, a new PLM made of existing ops) | `Programs<F>` in the overlay, validated at load | no |
| Palettes, tiles, Samus sheets, HUD art | a media pack keyed by stable asset keys | no |
| Songs and sound effects | N-SPC data in a pack, or a stream source (section 2) | no |
| Text | message records | no |
| A new enemy behaviour, PLM op, item or mechanic | a new enum variant with its Rust handler, and its option in `Rules` if it is optional | yes, additive |
| A new boss | a phase table (D14), plus Rust motion functions if its motion is new | data, sometimes code |

**Identities.** Overlays use stable keys (`Pack(pack, name)`), which the content set maps to
dense IDs at load (D15). Saves and replays carry keys and the content fingerprint, so a pack
change is detected, not misread.

**Foundation change:** the dense-ID and stable-key split (D15). Without it, custom content
would need to invent ROM-address-like identities, and saves would break whenever content was
reordered.

## 6. Multiple oracles

**How it is added.** A new emulator is an `Oracle` adapter in `oracle` that emits the same
`FrameRecord` (D20). It touches nothing in the engine.

- **Locks** are keyed by (row, oracle, version) (D22). The gate's primary oracle is named in
  the `ReferenceProfile`.
- **Oracle-specific rules** are `OracleRules` data (D19): IRQ write dots, beam-sync
  measurements, cost table, driver constants.
- **Disagreements.** A cross-oracle report steps two oracles with the same pads and diffs the
  records. A disagreement is a dispute for a hardware capture.
- **Hardware captures** (#250) act as a recorded oracle: video and input with lag, but no
  machine state. They are compared on the `video` and `timing` streams only.

**Why this works now.** The Tick follows hardware order (D6, D7). Interrupt write positions
are dots taken from data (D11). Pixels are compared in the PPU domain, with no emulator's
colour conversion (D17). Comparisons sit at the NMI return, which every oracle has (D20).
None of the old stepper's artefacts are structure (P4).

**Foundation change:** this draft's D6, D7, D11, D17 and D20 replace FOUNDATION's
stepper-shaped Tick.

## 7. Map Rando integration

The plan is FOUNDATION §7.3 and §7.5 and the Map Rando requirements report: rooms apart from
placements, doorways that own their arrival data, the topology, locks with shared identity,
typed items and events, `StartDefinition`, `GameplayProfile` presets, the importer in
`tools`, and constructed arrivals (`Start::Arrival`). This draft adds:

- **Stable keys** (D15) for the importer's ID mapping (J21: regenerate the mapping at import
  from pinned upstream commits).
- **Map Rando's patches** become `Rules` options (D19). For example, fast doors are a
  `DoorTiming`, and the quick reload is a Session command.
- **An oracle for "Map Rando base"** is a Map Rando-patched ROM on the emulator: a different
  content base plus its own locks under the same contract (section 6).
- **Load timing for rearranged rooms** comes from the cost model applied to the content
  actually loaded (D9). That is the report's §5 rule ("load effects from content, not from
  neighbours").

**Foundation change:** none after this draft. It doesn't rule out Map Rando, as Jim asked.

## 8. Other likely features

| Feature | How | Touches | Foundation change |
| --- | --- | --- | --- |
| Save states, rewind, practice-hack tools (room timers, overlays, input display) | the Checkpoint (canonical and fast) in a ring buffer in `app`; overlays read `&State` and the `View` | `app`, presentation | no |
| Rollback netplay | a deterministic Tick (P8) plus cheap Checkpoints | `app` | no |
| Multiworld randomisers (items sent between separate games) | a typed `SessionCommand::Receive(item)`, recorded as input | one command variant and the item module | no |
| HD texture packs | each `Upload` carries its `source: AssetRef` (FOUNDATION §6.1's media references), so the renderer's mirror knows which asset each VRAM tile came from, and the pack replaces it by key | `runtime`, media packs | no (keep `source` on `Upload`) |
| High-refresh interpolation | presentation interpolates intent positions between Ticks k-1 and k. Not vanilla, presentation only. | presentation | no |
| Accessibility (fewer flashes, less shake, palettes, remapping, spin lock) | shake and flash intent in the `View`; the presentation and input profiles | presentation, input | no |
| Other ROM revisions (JP, PAL) | decoder tables per revision, keyed by ROM hash. PAL's cadence and line count are in the `ReferenceProfile`, and the visible line count is a rule, not a constant. | `data`, profiles | no, if line counts stay data |
| TAS movies from other emulators | converted to pads plus the movie's lag marks, run as `Recorded` timing, or replayed on the oracle to record a lock | `tools` | no |
| Speedrun autosplitters | events derived from projections (room entered, boss defeated) | `app` | no |
| Live content editing (hot reload) | reopen at the current room with `Start::Arrival` and the carried State, keyed by stable keys (CONTRACTS' gameplay reload) | tools | no |

## 8a. Adjusted and new behaviours (the thesis, Jim 2026-11-05)

Jim's thesis (PRINCIPLES): the engine models the game's behaviour, so behaviours can be
reproduced, adjusted, added or removed. His examples are the space jump timing window and
respin. Map Rando ships both (`infinite_space_jump`, `respin`), along with
`momentum_conservation` and others (map-rando-requirements.md).

| Behaviour | Kind | How | Foundation change |
| --- | --- | --- | --- |
| Space jump timing window | adjust | The window is a named value in Samus's movement rules (`rules.movement.space_jump`). Vanilla's value comes from the original. A lenient window is a different value. | no |
| Respin (spin again in mid-air after the spin is broken) | add | A new transition in Samus's movement state machine, enabled by a `GameplayProfile` field. With the field off, the transition doesn't exist and vanilla digests are unchanged. | no |
| Momentum conservation, lag off, quick transitions | adjust or remove | A modelled behaviour with a profile value (D8, D19) | no |

**What this requires of each domain's design.** When a domain's cluster starts, its design
lists its behaviours as a catalogue before code. Samus is the first and largest. Each entry
has a name, the routine it models, and the parameters it exposes as rule values. An
adjustable behaviour must be one unit: one transition, one rule value or one table entry. It
must not be a condition spread through several routines. Review question: "could a player
option change or remove this behaviour by changing one value or one entry?" When the answer
is no for a behaviour players care about, the model is reshaped before it lands. This is
what keeps the thesis true as the engine grows (P20, P17, P16).

## 9. A feature that would need a foundation change

**Two Samuses in one world (local co-op).** The State has one Samus, and many rules read "the"
Samus: enemy targeting, camera, doors, HUD and collision priorities. Supporting two would
change the State schema and most domain signatures. It isn't on Jim's list, and nothing in
this design makes it harder than it would be in any faithful engine. It is recorded here so
the limit is known. Multiworld, which is separate games exchanging items, is supported
(section 8).
