# Legacy register

This register names current differences from the original and proof limits. Add or remove a
difference in the same change as the implementation. Replay case limits name narrower witness
claims; removed gameplay's former claims are in the
[replay transition](../scenarios/replay/transition.toml).

## Audio transport production inputs

Production audio owns the complete transport State and plays one frame at a time. Each frame's
queue offers and its audio cadence (effect turns around the sender service, and music turns) are
borrowed: in the native cases the offers are the cues the CPU newly presents and the cadence is
the capture's port order. This proves the transport's frame-level replies, request lines and
song, not CPU request generation, the CPU's frame timing, the audio clock or waveform synthesis.
An effect turn under way at a cut is completed in the next frame, so the replies it had already
published at that cut are not reproduced there. Required whole-engine producers remain
unsupported. See [audio-transport gaps](behaviours/audio-transport.gaps.md).

The played Ceres shaft produces its burst's CPU sound offers and carries the Ambient
sender through delivery and release. The room supplies sound permission. The
service opportunity and the reply it reads come from the replay platform inputs.
The shaft's resident SPC state, producer replies and effect playback are unproved.

## Comparison representation limits

Pending aim represented as `Option<FiringDirection>` preserves presence and direction but not
the original writers' high publication marker. Equality of that raw source word is unclaimed.
Physical audio reply aliases are comparison/import representations, not transport behaviour.
The five PPU fields lsnes does not expose remain unobserved as listed below. Scope-specific
excluded fields and consumers are explicit in each indexed case.

## The engine: original overflows it doesn't reproduce yet

The engine has no RAM image from which to invent out-of-range data. Selected production
content must either support the actual read or reject it explicitly. Quiet ordinary power
geometry answers actual Air, Solid and checked Floor/Surface slope queries; reaching reactive terrain or an unsupported
span fails the candidate advance. Appearance's five-sounds-waiting timer lookup lies outside its
specified domain, and retained scroll memory requires independently admitted complete content.
See [appearance gaps](behaviours/samus-appearance.gaps.md),
[camera gaps](behaviours/world-camera.gaps.md) and
[weapons gaps](behaviours/weapons-projectiles.gaps.md).

The following limits belong to retained clean models or content readers:

- **Scroll areas past the room's table.** The camera reads the supplied run of screens below
  the room, then treats absent screens as closed. The original can read neighbouring RAM
  beyond that run. Admission must supply any retained scroll memory needed by a case;
  missing room-loading history is not simulated.
- **Collision cells below the intro rooms' populated rows.** The original keeps earlier
  memory there. Typed intro collision ends at the populated rows. The movie keeps Samus and
  her shots inside them. Only a platform with the complete earlier memory can supply the backed
  grid. See
  [the intro's playable rooms](behaviours/world-intro-rooms.decode.md).
- **Samus spritemaps with a zero pointer or past the table.**
  `data::pose::spritemap` returns no map for these pointers. The original reads a pointer or
  count from neighbouring data or work RAM. No reachable vanilla animation selects one.
- **Tile numbers past the tile tables.** `data::assets::tilesets::Tilesets::block` returns no
  block outside the imported tables; it does not reproduce reads of neighbouring RAM.
- **Samus's animation timing spec's defined domain.** Five waiting sounds at boost start,
  a clip end without its required pose handover, and an `again` or `branch` target or a rest
  successor that is not a shown step are outside the clean animation contract. The original may continue
  through neighbouring bytes. The five-sound case can occur in vanilla; see
  [the animation input domain](behaviours/samus-appearance.md#15-the-inputs-domain).
- **Samus's motion in the demos.** The clean movement end-of-animation handover follows its
  play contract. The original demo handler can hand over regardless of a jump on the last
  frame. Demo integration and movement outside the model's poses remain unsupported.
- **The HUD spec's defined domain.** Missile counts and capacities are limited to 999,
  super missiles and power bombs to 99. An unselected one-shot item is not representable.
  Outside-map cells follow the stated blank, uncharted and unexplored rule, except the named
  `LastColumnsShift` beyond pictures. Exploring an outside-map cell changes nothing.
  The original can read adjoining bitmap or map data there. See
  [the HUD input domain](behaviours/interface-hud.md#15-the-inputs-domain).
  The model also applies a map turn-on before a boss blanks the map; the original's result
  follows actor order when both happen together. No vanilla room holds both actors.
- **A scroll cell with no scroll reaction.** The blocks contract requires a matching scroll
  placement in valid room content; the original can search forever without one. The clean
  model does not reproduce that hang or report the missing placement as an error. Required
  room-content validation and integrated handling of that invalid room remain unproved.
- **A stand-in chain that loops.** Level import and bounded movement reject extension cycles.
  The original can follow such a chain forever. This covers imported or independently
  admitted movement geometry; validation of cycles introduced by a running block reaction
  still needs the integrated Blocks-to-Movement path.
- **Reacting cells whose tables end before their mark.** The content importer accepts only
  its defined reaction tables and named tolerated entries. The original can read the next
  table or code bytes as a reaction. See the
  [blocks decode notes](behaviours/world-blocks.decode.md).
- **Reactions the original would misread.** Imported shows must last 1 to 32,767 frames,
  and a bomb block's broken list must start with its sound. The original gives a zero show
  a wrapped wait and can interpret a large wait or the skipped bytes as instructions.
- **Patterns past the room's edge.** The blocks contract requires patterns to stay inside
  the admitted room. The original's linear addressing can wrap them into another row or
  write beyond the room. Room-loading support for arbitrary custom patterns is unavailable.
- **A door block whose place is past its room's door list.** The movement observation adapter
  treats a door without an admitted destination as solid. It does not follow whatever
  pointer the original reads after the list.
- **The grapple on a bomb, super missile or power bomb block.** The clean blocks contract
  treats the grapple by its declared kind. The original can reuse the last projectile's
  weapon value and start one of these reactions. That stale-value behaviour is not modeled.

## Samus's aim after an isolated down press in a fall

On `samus-ceres-push`, the polling cut at 904 newly presses Down. The following game
frame ends in the downward-aimed falling pose at 905, which polls the release.
With Down released, the original keeps that pose at 906. The movement chart chooses
forward aim on the frame with no button held. The alternate played route first differs
at 906 in `movement.pose`. Its second tail hit is compared from a separate observed
origin at cut 926.

## The engine: memory a model keeps that the engine can't hold yet

Production holds the selected clean State itself. Some clean States lack complete serialization; their live entry remains usable while save or
restore reports unavailable codec support. No partial observed-field checkpoint is a complete
continuation claim.

## The engine: Drawing after a decrease in Health

In the scoped Ceres tail, Drawing commits its energy history from each frame's Health, but no
producer compares that history with the frame's energy for the hurt flash. The flash state,
its readers and the visible flash stay outside the production scope.

## The engine: checked ordinary power beam

The selected weapons/contact recipe holds full clean shot, trail and spacing State, advances
the existing clean weapons owner and applies ordered current Ceres-door contact opportunities.
Its quiet geometry capability is intentionally bounded. Borrowed control, aim, camera and actor
opportunities limit the proof. Required terrain reactions, general actor policy, passage,
sound delivery or downstream drawing remain unsupported or explicitly excluded. Indexed witnesses state the actual compared outputs.

## The engine: slots it carries but doesn't run

Production holds no enemy or PLM slots. Independently admitted solid geometry is
a scoped input, not proof of AI, population changes or lifecycle. Required missing owners fail
explicitly rather than carrying raw words forward as simulated behaviour.

## The engine: colours a room's enemies set as they are placed

A room's loading puts the Ceres doors' colour set for the station's stage into the doors' row
for every version that loads the Ceres door. The engine places no enemies yet, so this holds
for every Ceres door alike. Before Ridley's getaway, the door in Ridley's room puts the calm
set in the row its species loads to instead. No other species' placement changes a colour.

## Ceres escape boundaries

The escape shaft publishes the overlay figure and platform tiles through the
room's sprite buffer and low-byte tilemap transfer. After the first overlay
publication, other sprites can precede the figure. The full sprite order
depends on the particle, vent and Samus drawing owners. Physical mode changes
during scanout and the haze fade have their own owners.

## The engine: planned differences from the original

The engine executes no game code, including the uploaded sound driver. Driver upload bytes are
uncompared; their observable timing/replies require modeled producers. Audio waveform synthesis and its runtime backend
are unimplemented and have no exactness claim. See
[ARCHITECTURE](architecture/ARCHITECTURE.md#116-what-isnt-compared-by-name).

## Tolerated values kept as named variants

Imported content can retain named unknown variants where the original tolerates them.
`data::room::Door::HeaderInRam(u16)` retains a source pointer but world import rejects it;
`data::pose::PoseFacing::Unknown(u8)` preserves an unused pose's facing byte;
`data::level::ScrollArea::Unknown(u8)` preserves an unknown scroll byte. These representations
alone establish no successful production capability. Selected admission validates actual
content requirements.

## The oracle

- **`LEGACY(verifier)`** in `crates/oracle/src/streams.rs`: the `Definition::Legacy` streams, which
  compare with the old lock `scenarios/lock.toml` while `oracle corpus` still reads it, zero the WRAM
  that lock's `wram` digests zeroed (`LOCK_WRAM_MASK`, end-exclusive WRAM offsets): the direct page
  `$0000-$0051` and the stack `$1F5B-$2000`; `$00AD-$00B1`, `$05BC-$05C5`, `$05ED-$05F5`,
  `$060B-$0617`, `$077E-$0783`, `$078F-$0791`, `$07B7-$07B9`, `$0933-$093D`, `$0B24-$0B28`,
  `$0D82-$0D90`, `$0D9C-$0D9E`, `$0DD2-$0DD8`, `$0E02-$0E04`, `$0E20-$0E46`, `$0E4A-$0E4C`,
  `$0E54-$0E5A`, `$1784-$178C`, `$17A8-$17AC`, `$1834-$183C`, `$184A-$185C`, `$186E-$1886`,
  `$189A-$18A8`, `$1966-$196C`, `$EF74-$EF78`, `$F37A-$F380`; and the bug-fix counter `$00C2-$00C4`,
  which only the old patched oracle wrote. The V1 streams and the format-2 lock mask nothing. Remove both when `oracle corpus` reads
  `scenarios/locks/lsnes.toml` instead ([ORACLE §9](ORACLE.md#9-open-items)).
- **`LEGACY(seed)`** in `crates/oracle/src/seed.rs`: lsnes room captures (`Start::Capture`) hold no
  SPC700, DSP or timer state, so a migration seed resumes the SPC700 at the driver's idle loop (`$15BF`,
  SP `$CF`), the CPU-to-APU ports from the driver's copy at SPC `$00-$03`, the timers from their RAM
  shadows, and the DSP from reset with echo writes off. In some rows the resumed driver answers no sound
  request, so uploads, door transitions and message boxes stall. Native seeds retire this entry.
- **`LEGACY(seed-ppu)`** in `oracle_record::lsnes::read`: 531 of the relock bundle's 541 seed
  records lack the optional PPU tier. Reading one starts the committed PPU and DMA registers
  at their defaults, even where the native state retains other values. A first NMI overwrites
  most registers; retained Mode 7 matrix and DMA values can differ from frame 0 in a free run.
  The `any-boundary:3` seed carries its captured PPU tier. The remaining records need the
  same [seed recapture check](ORACLE.md#seed-records-with-the-ppu-registers-relock-9), preserving
  their original record prefixes and native states. Every PPU field covered by the contract
  stays compared.
- **lsnes's cut at line 241** without a previewed pad: an NMI whose writes pass that line shows a step
  later (`Cut::NmiStillRunning`). The format-2 lock is recorded from frame pads with preview, so it has
  no such cut ([ORACLE, MEASUREMENTS (g), (i)](ORACLE.md#appendix-a-measurements-measurements)).
- **The five `ppu` fields lsnes doesn't expose** (`nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel`) are
  not compared (ORACLE Q4).
- **A one-frame lag shift after an APU upload** between oracles is an oracle difference, not engine
  behaviour; the lag pattern is pinned to lsnes ([ORACLE appendix B](ORACLE.md#appendix-b-emulator-accuracy-accuracy)).
