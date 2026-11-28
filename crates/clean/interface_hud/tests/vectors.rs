//! The HUD's vectors (`scenarios/vectors/interface-hud.toml`) run through the model, and the catalogue
//! checked against them.
//!
//! The file is read with `toml` and `serde` into the shape declared once below, every table of
//! it closed to keys the file's header doesn't define, every name an enum of the names it does,
//! and every "column,row" key a place on the grid it names; that shape is then turned into the
//! model's own types. Each vector runs under its rules (the game's, with each value the vector
//! gives in its place), as the step it names, which refuses an input the header allows only in
//! another step.

#[cfg(test)]
mod shape {
    //! The vector file's shape, as its header defines it.

    use std::collections::BTreeMap;

    use interface_hud::{
        ColourSet, Flip, HudEvent, Look, Pair, Picture, Place, SelectableItem, Tile,
    };
    use serde::Deserialize;

    /// The HUD's columns and rows (Notation).
    pub type HudPlaceIn = PlaceIn<32, 3>;
    /// Her map's columns and rows (Notation).
    pub type MapPlaceIn = PlaceIn<64, 32>;
    /// The beyond pictures' columns and her map's rows (spec: `LastColumnsShift`).
    pub type BeyondPlaceIn = PlaceIn<2, 32>;

    /// The whole file.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct VectorFile {
        pub pictures: PicturesIn,
        pub vector: Vec<VectorIn>,
        pub countdown: Vec<CountdownIn>,
        pub startup_observed: Vec<StartupObservedIn>,
        pub startup: Vec<StartupIn>,
    }

    /// The opening frame's columns and rows: the upper row, then the HUD's three.
    pub type OpeningPlaceIn = PlaceIn<32, 4>;
    /// The opening map preview's columns and rows, the mini-map's window.
    pub type PreviewPlaceIn = PlaceIn<5, 3>;

    /// One `[[startup]]`: a new file's setup, from its opening art, preview, frame count and
    /// fresh energy to the HUD it puts up.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct StartupIn {
        pub id: String,
        #[serde(rename = "source")]
        pub _source: Source,
        pub entries: Vec<String>,
        pub rules: Option<RulesIn>,
        pub frame: u8,
        pub energy: [u16; 2],
        pub opening_fill: TileIn,
        pub opening: BTreeMap<OpeningPlaceIn, TileIn>,
        pub preview: BTreeMap<PreviewPlaceIn, PictureIn>,
        pub after: StartupAfterIn,
    }

    /// A startup's after: the lower-row cells drawn over the opening art, whether the upper row
    /// published is the one supplied, and the HUD's State.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct StartupAfterIn {
        pub upper_row_equal: bool,
        pub shown: ShownIn,
        pub selection: ItemName,
        pub one_shot: bool,
        pub mini_map_off: bool,
        pub explored: Vec<[i32; 2]>,
        pub countdown: CurrentCountdownIn,
        pub replaced: BTreeMap<HudPlaceIn, TileIn>,
    }

    /// One `[[startup_observed]]`: the HUD's State the oracle recorded at a cut of a new file's
    /// start, and the colour sets of the mini-map's window, the top row first.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct StartupObservedIn {
        pub id: String,
        /// The recordings whose cuts it names, which the independent replay reads.
        #[serde(rename = "row")]
        pub _row: String,
        pub before: u32,
        pub after: u32,
        pub held_through: Option<u32>,
        pub shown: ShownIn,
        pub selection: ItemName,
        pub one_shot: bool,
        pub mini_map_off: bool,
        pub explored: Vec<[i32; 2]>,
        pub countdown: CurrentCountdownIn,
        pub preview_colours: [[ColourSet; 5]; 3],
    }

    /// Whether a countdown case runs the model, or checks only the HUD's refusal of what the
    /// spec leaves out.
    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CountdownScope {
        Model,
        IntegrationAdmission,
    }

    /// A current countdown: `"Dormant"`, or a begun one's table.
    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum CurrentCountdownIn {
        Dormant(DormantIn),
        Begun(BegunIn),
    }

    /// `"Dormant"`: no countdown has begun.
    #[derive(Deserialize)]
    pub enum DormantIn {
        Dormant,
    }

    /// A begun countdown: its time left `[minutes, seconds, hundredths]`, its age, and its
    /// place, each axis `[whole pixels, 1/256ths]`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct BegunIn {
        pub time: [u8; 3],
        pub age: u8,
        pub place: [[u8; 2]; 2],
    }

    /// A frame's countdown picture: `"Absent"`, or its place in whole pixels and its time.
    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum CountdownPictureIn {
        Absent(AbsentIn),
        Shown(ShownPictureIn),
    }

    /// `"Absent"`: no countdown picture.
    #[derive(Deserialize)]
    pub enum AbsentIn {
        Absent,
    }

    /// A countdown picture shown: `place = [x, y]`, `time = [minutes, seconds, hundredths]`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ShownPictureIn {
        pub place: [u16; 2],
        pub time: [u8; 3],
    }

    /// The scene's choice for the countdown, as the file names it.
    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SceneIn {
        MaintainCountdown,
        PresentationOnly,
        Unknown,
    }

    /// What a countdown case expects of the frames.
    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CountdownExpected {
        Pass,
        Unsupported,
    }

    /// Where a countdown case comes from.
    #[derive(Deserialize)]
    pub enum CountdownSource {
        #[serde(rename = "recorded")]
        Recorded,
        #[serde(rename = "constructed")]
        Constructed,
    }

    /// One `[[countdown]]`: frames of play, in order, from the current countdown before them
    /// to the one after them.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CountdownIn {
        pub id: String,
        pub scope: CountdownScope,
        #[serde(rename = "source")]
        pub _source: CountdownSource,
        pub entries: Vec<String>,
        pub before: CurrentCountdownIn,
        pub expected: CountdownExpected,
        pub after: CurrentCountdownIn,
        pub energy: Option<u16>,
        /// Her protection from hits, which no rule of the HUD reads: not an input of the model.
        #[serde(rename = "protection_frames")]
        pub _protection_frames: Option<u16>,
        pub frames: Vec<CountdownFrameIn>,
    }

    /// One `[[countdown.frames]]`: a frame of play's countdown inputs and, unless it is
    /// refused, its outputs.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CountdownFrameIn {
        pub scene: SceneIn,
        #[serde(default)]
        pub finished: u16,
        pub start: Option<[u8; 2]>,
        pub answer: Option<interface_hud::CountdownExpiry>,
        pub picture: Option<CountdownPictureIn>,
    }

    /// `[pictures]`: the HUD's pictures by name.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct PicturesIn {
        pub blank: PictureIn,
        pub digits: [PictureIn; 10],
        pub full_tank: PictureIn,
        pub empty_tank: PictureIn,
        pub reserve_sign: [[PictureIn; 2]; 3],
        pub icons: IconsIn,
        pub map_blank: PictureIn,
        pub diagonal: PictureIn,
    }

    /// `[pictures.icons]`: each item's two rows.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "PascalCase")]
    pub struct IconsIn {
        pub missiles: [[PictureIn; 3]; 2],
        pub super_missiles: [[PictureIn; 2]; 2],
        pub power_bombs: [[PictureIn; 2]; 2],
        pub grapple_beam: [[PictureIn; 2]; 2],
        pub x_ray_scope: [[PictureIn; 2]; 2],
    }

    /// One `[[vector]]`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct VectorIn {
        pub id: String,
        /// Read only to refuse a source the header doesn't name.
        #[serde(rename = "source")]
        pub _source: Source,
        pub entries: Vec<String>,
        pub rules: Option<RulesIn>,
        #[serde(default)]
        pub step: StepIn,
        pub frame: Option<u8>,
        #[serde(default)]
        pub before: BeforeIn,
        #[serde(default)]
        pub inputs: InputsIn,
        pub after: AfterIn,
    }

    /// Where a vector comes from.
    #[derive(Deserialize)]
    pub enum Source {
        #[serde(rename = "constructed")]
        Constructed,
    }

    /// The step a vector takes.
    #[derive(Deserialize, Default, Clone, Copy)]
    pub enum StepIn {
        #[default]
        Frame,
        BossEvent,
        OutOfPlay,
    }

    /// `rules`: the adjustable rules a vector runs under; a key left out is the game's.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct RulesIn {
        pub one_shot_flash: Option<u8>,
        pub her_blink: Option<u8>,
        pub boss_screens: Option<BossScreensIn>,
    }

    /// `boss_screens`: the screens each boss named explores, `[across, down]`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "PascalCase")]
    pub struct BossScreensIn {
        pub kraid: Option<Vec<[u16; 2]>>,
        pub draygon: Option<Vec<[u16; 2]>>,
        pub crocomire: Option<Vec<[u16; 2]>>,
        pub phantoon: Option<Vec<[u16; 2]>>,
        pub mother_brain: Option<Vec<[u16; 2]>>,
        pub ridley_in_norfair: Option<Vec<[u16; 2]>>,
        pub ridley_in_ceres: Option<Vec<[u16; 2]>>,
    }

    /// A vector's before.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    pub struct BeforeIn {
        pub hud: Option<[HudRowIn; 3]>,
        #[serde(default)]
        pub shown: ShownIn,
        #[serde(default)]
        pub selection: ItemName,
        #[serde(default)]
        pub one_shot: bool,
        #[serde(default)]
        pub mini_map_off: bool,
        #[serde(default)]
        pub explored: Vec<[i32; 2]>,
    }

    /// A vector's after: a field left out is as before.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct AfterIn {
        #[serde(default)]
        pub changed: BTreeMap<HudPlaceIn, TileIn>,
        pub shown: Option<ShownAfterIn>,
        pub selection: Option<ItemName>,
        pub one_shot: Option<bool>,
        pub mini_map_off: Option<bool>,
        pub explored: Option<Vec<[i32; 2]>>,
        #[serde(default)]
        pub events: Vec<EventIn>,
        pub controls: Option<ControlsAfterIn>,
    }

    /// `controls`: the selection after the controls, before any drop (a `Frame` step only).
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ControlsAfterIn {
        pub selection: ItemName,
        pub one_shot: bool,
    }

    /// `shown`: each value left out is 0 or none.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields, default)]
    pub struct ShownIn {
        pub energy: u16,
        pub missiles: u16,
        pub super_missiles: u16,
        pub power_bombs: u16,
        pub selection: ItemName,
    }

    /// An after's `shown`: each value left out is as before.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ShownAfterIn {
        pub energy: Option<u16>,
        pub missiles: Option<u16>,
        pub super_missiles: Option<u16>,
        pub power_bombs: Option<u16>,
        pub selection: Option<ItemName>,
    }

    /// A vector's inputs; which are allowed depends on the step.
    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    pub struct InputsIn {
        pub energy: Option<[u16; 2]>,
        pub reserve: Option<u16>,
        pub auto_reserve: Option<bool>,
        pub missiles: Option<[u16; 2]>,
        pub super_missiles: Option<[u16; 2]>,
        pub power_bombs: Option<[u16; 2]>,
        pub equipped: Option<Vec<Equipment>>,
        pub at_start: Option<AtStartIn>,
        pub select: Option<SelectIn>,
        pub cancel: Option<CancelIn>,
        pub can_switch: Option<bool>,
        pub muted_by: Option<MuteIn>,
        pub dropped: Option<DroppedIn>,
        pub new_area: Option<NewAreaIn>,
        pub collected: Option<ItemName>,
        pub mini_map: Option<MiniMapIn>,
    }

    /// `at_start`: her stock as the frame begins, each `[count, capacity]`; a key left out is
    /// the same as at the frame's end.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct AtStartIn {
        pub missiles: Option<[u16; 2]>,
        pub super_missiles: Option<[u16; 2]>,
        pub power_bombs: Option<[u16; 2]>,
    }

    /// `dropped`: what another domain drops, and (a `Frame` step only) when.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct DroppedIn {
        pub what: DropWhatIn,
        pub when: Option<DropWhenIn>,
    }

    /// What a drop takes.
    #[derive(Deserialize, Clone, Copy)]
    pub enum DropWhatIn {
        Selection,
        OneShot,
    }

    /// When a drop lands in a frame of play.
    #[derive(Deserialize, Clone, Copy)]
    pub enum DropWhenIn {
        BeforeStatusBar,
        AfterStatusBar,
    }

    /// `new_area`: the explored cells Progress keeps for the area the step leads into.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct NewAreaIn {
        pub explored: Vec<[i32; 2]>,
    }

    /// `mini_map`; which keys are allowed depends on the step.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MiniMapIn {
        pub runs: Option<bool>,
        pub turned_on: Option<bool>,
        pub mother_brain_falls: Option<bool>,
        pub position: Option<[u16; 2]>,
        pub room: Option<RoomIn>,
        pub map: Option<MapIn>,
        pub boss: Option<BossName>,
    }

    /// `room`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct RoomIn {
        pub cell: [i32; 2],
        pub blocks: Option<[u16; 2]>,
    }

    /// `map`.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct MapIn {
        #[serde(default)]
        pub collected: bool,
        #[serde(default)]
        pub pictures: BTreeMap<MapPlaceIn, PictureIn>,
        #[serde(default)]
        pub charted: Vec<[i32; 2]>,
        #[serde(default)]
        pub beyond: BTreeMap<BeyondPlaceIn, PictureIn>,
    }

    /// An item's name, or `"None"`.
    #[derive(Deserialize, Default, Clone, Copy)]
    pub enum ItemName {
        #[default]
        None,
        Missiles,
        SuperMissiles,
        PowerBombs,
        GrappleBeam,
        XRayScope,
    }

    /// What `equipped` names.
    #[derive(Deserialize, Clone, Copy)]
    pub enum Equipment {
        GrappleBeam,
        XRayScope,
    }

    /// Item select this frame.
    #[derive(Deserialize, Clone, Copy)]
    pub enum SelectIn {
        Pressed,
    }

    /// Item cancel this frame.
    #[derive(Deserialize, Clone, Copy)]
    pub enum CancelIn {
        Pressed,
        Held,
    }

    /// Why a switch makes no click.
    #[derive(Deserialize, Clone, Copy)]
    pub enum MuteIn {
        None,
        Grapple,
        SpinJump,
        TimeStopped,
        WallJump,
    }

    /// The boss whose room a `BossEvent` enters.
    #[derive(Deserialize, Clone, Copy)]
    pub enum BossName {
        Kraid,
        Draygon,
        Crocomire,
        Phantoon,
        RidleyInNorfair,
        RidleyInCeres,
    }

    /// A `"column,row"` key: a place, column first, on a grid `COLUMNS` across by `ROWS` down.
    #[derive(Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(try_from = "String")]
    pub struct PlaceIn<const COLUMNS: i32, const ROWS: i32> {
        column: i32,
        row: i32,
    }

    impl<const COLUMNS: i32, const ROWS: i32> PlaceIn<COLUMNS, ROWS> {
        /// The place it names.
        pub const fn place(self) -> Place {
            Pair {
                x: self.column,
                y: self.row,
            }
        }
    }

    impl<const COLUMNS: i32, const ROWS: i32> TryFrom<String> for PlaceIn<COLUMNS, ROWS> {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let bad = || format!("no place {text} on a grid of {COLUMNS} columns by {ROWS} rows");
            let (column, row) = text.split_once(',').ok_or_else(bad)?;
            let column: i32 = column.trim().parse().map_err(|_| bad())?;
            let row: i32 = row.trim().parse().map_err(|_| bad())?;
            if !(0..COLUMNS).contains(&column) || !(0..ROWS).contains(&row) {
                return Err(bad());
            }
            Ok(Self { column, row })
        }
    }

    /// A picture, `PPP` then `h` and `v` for its flips.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct PictureIn(pub Look);

    impl TryFrom<String> for PictureIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let split = text
                .find(|c: char| !c.is_ascii_hexdigit())
                .unwrap_or(text.len());
            let (number, flips) = text.split_at(split);
            Ok(Self(look(number, flips)?))
        }
    }

    /// A HUD cell, `PPP:c` then `h` and `v` for its flips.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct TileIn(pub Tile);

    impl TryFrom<String> for TileIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let bad = || format!("bad cell {text}");
            let (number, rest) = text.split_once(':').ok_or_else(bad)?;
            let mut chars = rest.chars();
            let colours = chars
                .next()
                .and_then(|c| c.to_digit(10))
                .and_then(|c| u8::try_from(c).ok())
                .and_then(ColourSet::new)
                .ok_or_else(bad)?;
            Ok(Self(Tile::new(look(number, chars.as_str())?, colours)))
        }
    }

    /// A picture's three hex digits and its flip letters.
    fn look(number: &str, flips: &str) -> Result<Look, String> {
        let picture = u16::from_str_radix(number, 16)
            .ok()
            .and_then(Picture::new)
            .ok_or_else(|| format!("no picture {number}"))?;
        Ok(Look {
            picture,
            flip: flip(flips)?,
        })
    }

    /// A HUD row: its 32 cells, left to right, separated by spaces.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct HudRowIn(pub [Tile; 32]);

    impl TryFrom<String> for HudRowIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            let cells = text
                .split_whitespace()
                .map(|cell| TileIn::try_from(cell.to_owned()).map(|tile| tile.0))
                .collect::<Result<Vec<_>, _>>()?;
            let count = cells.len();
            cells
                .try_into()
                .map(Self)
                .map_err(|_| format!("a HUD row of {count} cells"))
        }
    }

    /// An event, `Switched:<item>` or `Click`.
    #[derive(Deserialize, Clone, Copy)]
    #[serde(try_from = "String")]
    pub struct EventIn(pub HudEvent);

    impl TryFrom<String> for EventIn {
        type Error = String;
        fn try_from(text: String) -> Result<Self, String> {
            if text == "Click" {
                return Ok(Self(HudEvent::Click));
            }
            let item = text
                .strip_prefix("Switched:")
                .ok_or_else(|| format!("unknown event {text}"))?;
            let item: ItemName = toml::Value::String(item.to_owned())
                .try_into()
                .map_err(|_| format!("unknown item in {text}"))?;
            Ok(Self(HudEvent::Switched { item: item.item() }))
        }
    }

    impl ItemName {
        /// The item it names, or none.
        pub const fn item(self) -> Option<SelectableItem> {
            use interface_hud::Ammo;
            match self {
                Self::None => None,
                Self::Missiles => Some(SelectableItem::Ammo(Ammo::Missiles)),
                Self::SuperMissiles => Some(SelectableItem::Ammo(Ammo::SuperMissiles)),
                Self::PowerBombs => Some(SelectableItem::Ammo(Ammo::PowerBombs)),
                Self::GrappleBeam => Some(SelectableItem::GrappleBeam),
                Self::XRayScope => Some(SelectableItem::XRayScope),
            }
        }
    }

    /// The flip letters.
    fn flip(letters: &str) -> Result<Flip, String> {
        match letters {
            "" => Ok(Flip::None),
            "h" => Ok(Flip::Across),
            "v" => Ok(Flip::Down),
            "hv" => Ok(Flip::Both),
            other => Err(format!("bad flips {other}")),
        }
    }
}

#[cfg(test)]
mod convert {
    //! The vector file's shape turned into the model's types, and a vector run.

    use std::collections::BTreeMap;

    use enum_map::enum_map;
    use interface_hud::{
        Alternation, Ammo, Ammunition, BegunCountdown, Boss, CancelButton, ColourSet, Controls,
        Countdown, CountdownAge, CountdownAnswer, CountdownInputs, CountdownPicture,
        CountdownPlace, CountdownRuns, Dropped, Energy, Equipped, FinishedFrames, FrameCount,
        FrameInputs, FrameOutputs, HerMap, HudEvent, HudGrid, HudPictures, HudState, Icons, Look,
        MapCellSet, MapPictures, MiniMapInputs, MiniMapPower, Mute, NewArea, NewFileHud,
        OpeningFrame, OpeningPreview, OutOfPlayInputs, Pair, Place, PreviewCell, Reserve,
        ReserveMode, Room, Rules, Screen, SelectButton, SelectableItem, Selection, Shown,
        StartTime, Status, Stock, StockAtStart, Tile, TimeLeft,
    };
    use serde::de::DeserializeOwned;

    use super::shape::{
        AbsentIn, AfterIn, BeforeIn, BegunIn, BossName, BossScreensIn, CancelIn, CountdownExpected,
        CountdownFrameIn, CountdownIn, CountdownPictureIn, CountdownScope, CurrentCountdownIn,
        DormantIn, DropWhatIn, DropWhenIn, DroppedIn, Equipment, IconsIn, InputsIn, ItemName,
        MapIn, MiniMapIn, MuteIn, OpeningPlaceIn, PictureIn, PicturesIn, PlaceIn, PreviewPlaceIn,
        RoomIn, RulesIn, SelectIn, ShownIn, ShownPictureIn, StartupIn, StartupObservedIn, StepIn,
        TileIn, VectorFile, VectorIn,
    };

    /// The HUD's rows (Notation).
    const HUD_ROWS: i32 = 3;
    /// The HUD's columns.
    const HUD_COLUMNS: i32 = 32;
    /// Her map's columns: a beyond picture's column counts on from the last (Notation).
    const MAP_COLUMNS: i32 = 64;
    /// Her map's rows.
    const MAP_ROWS: i32 = 32;

    /// The step a vector takes, with its inputs: the one entry point the vectors run through,
    /// built here as the main loop would choose among the model's three.
    enum Step<'a> {
        /// A frame of play.
        FrameOfPlay(FrameInputs<'a>),
        /// A step outside play.
        OutOfPlay(OutOfPlayInputs<'a>),
        /// A boss met on entering its room, from her room's top-left cell.
        BossEncountered(Boss, Place),
    }

    impl Step<'_> {
        /// Takes the step on `state`, returning its events and, for a frame of play, the
        /// selection after the controls.
        fn take(
            &self,
            state: &mut HudState,
            rules: &Rules,
            pictures: &HudPictures,
        ) -> Result<(Vec<HudEvent>, Option<Selection>), String> {
            match self {
                Self::FrameOfPlay(inputs) => {
                    let FrameOutputs {
                        selection_after_controls,
                        events,
                        countdown: _,
                    } = state
                        .frame_of_play(rules, pictures, inputs)
                        .map_err(|refused| format!("frame refused: {refused:?}"))?;
                    Ok((events, Some(selection_after_controls)))
                }
                Self::OutOfPlay(inputs) => {
                    state.out_of_play(inputs);
                    Ok((Vec::new(), None))
                }
                &Self::BossEncountered(boss, room) => {
                    state.boss_blanks_the_map(rules, pictures, boss, room);
                    Ok((Vec::new(), None))
                }
            }
        }
    }

    /// The file, read.
    pub fn load() -> Result<VectorFile, String> {
        load_text(&vector_file::read("interface-hud")?)
    }

    /// The file's text with its first `from` replaced by `to`, read.
    pub fn load_changed(from: &str, to: &str) -> Result<VectorFile, String> {
        let text = vector_file::read("interface-hud")?;
        assert!(text.contains(from), "the file holds no {from}");
        load_text(&text.replacen(from, to, 1))
    }

    /// A vector file's text, read.
    fn load_text(text: &str) -> Result<VectorFile, String> {
        toml::from_str(text).map_err(|error| error.to_string())
    }

    /// The HUD's pictures. The blank's flips and the diagonal's are read by no rule (§1.4), so
    /// each is held as its picture alone.
    pub fn pictures(pictures: &PicturesIn) -> HudPictures {
        let IconsIn {
            missiles,
            super_missiles,
            power_bombs,
            grapple_beam,
            x_ray_scope,
        } = &pictures.icons;
        HudPictures {
            blank: pictures.blank.0.picture,
            digits: pictures.digits.map(|digit| digit.0),
            full_tank: pictures.full_tank.0,
            empty_tank: pictures.empty_tank.0,
            reserve_sign: rows(&pictures.reserve_sign),
            icons: Icons {
                missiles: rows(missiles),
                super_missiles: rows(super_missiles),
                power_bombs: rows(power_bombs),
                grapple_beam: rows(grapple_beam),
                x_ray_scope: rows(x_ray_scope),
            },
            map_blank: pictures.map_blank.0,
            diagonal: pictures.diagonal.0.picture,
        }
    }

    /// Rows of pictures, as the model holds them.
    fn rows<const ACROSS: usize, const DOWN: usize>(
        rows: &[[PictureIn; ACROSS]; DOWN],
    ) -> [[Look; ACROSS]; DOWN] {
        rows.map(|row| row.map(|picture| picture.0))
    }

    impl VectorIn {
        /// Runs the vector: `Ok` with each way the model's after differs from the vector's, or
        /// `Err` when the vector breaks the file's format.
        ///
        /// A `Frame` step that gives no frame count must hold at every one, so it runs at each.
        pub fn run(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
            match (self.step, self.frame) {
                (StepIn::Frame, None) => self.run_at_every_frame(pictures),
                (_, frame) => self.run_at(frame.map(FrameCount), pictures),
            }
        }

        /// Runs a `Frame` step that gives no frame count at each count, reporting the first at
        /// which it fails.
        fn run_at_every_frame(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
            for frame in 0..=u8::MAX {
                let problems = self.run_at(Some(FrameCount(frame)), pictures)?;
                if !problems.is_empty() {
                    return Ok(problems
                        .into_iter()
                        .map(|problem| format!("at frame {frame}: {problem}"))
                        .collect());
                }
            }
            Ok(Vec::new())
        }

        /// Runs the vector's step at the frame count `frame`, none outside a frame of play.
        fn run_at(
            &self,
            frame: Option<FrameCount>,
            pictures: &HudPictures,
        ) -> Result<Vec<String>, String> {
            let before = state(&self.before, pictures)?;
            let rules = self.rules.as_ref().map_or_else(Rules::reference, rules);
            let map = her_map(self.inputs.mini_map.as_ref(), pictures)?;
            let new_area = self
                .inputs
                .new_area
                .as_ref()
                .map(|area| cell_set(&area.explored))
                .transpose()?;
            let step = self.step(frame, &map, new_area.as_ref())?;
            let mut after = before.clone();
            let (events, after_controls) = step.take(&mut after, &rules, pictures)?;
            let expected = expected_state(&self.after, &before)?;
            let expected_events: Vec<HudEvent> = self.after.events.iter().map(|e| e.0).collect();
            let mut problems = differences(&expected, &after, &expected_events, &events);
            let expected_controls = match (after_controls, self.after.controls.as_ref()) {
                (None, Some(_)) => return Err("only a Frame step has controls".to_owned()),
                (None, None) => None,
                (Some(_), None) => Some(expected.selection),
                (Some(_), Some(given)) => Some(selection(given.selection.item(), given.one_shot)?),
            };
            if after_controls != expected_controls {
                problems.push(format!(
                    "selection after the controls: expected {expected_controls:?}, got \
                     {after_controls:?}"
                ));
            }
            Ok(problems)
        }

        /// The step the vector takes, with its inputs; only a `Frame` step has a frame count.
        fn step<'a>(
            &self,
            frame: Option<FrameCount>,
            map: &'a HerMap,
            new_area: Option<&'a MapCellSet>,
        ) -> Result<Step<'a>, String> {
            match (self.step, frame) {
                (StepIn::Frame, Some(frame)) => {
                    frame_inputs(frame, &self.inputs, map).map(Step::FrameOfPlay)
                }
                (StepIn::Frame, None) => Err("a Frame step has a frame".to_owned()),
                (StepIn::BossEvent, None) => boss_event(&self.inputs),
                (StepIn::OutOfPlay, None) => {
                    out_of_play(&self.inputs, new_area).map(Step::OutOfPlay)
                }
                (StepIn::BossEvent | StepIn::OutOfPlay, Some(_)) => {
                    Err("only a Frame step has a frame".to_owned())
                }
            }
        }
    }

    impl CountdownIn {
        /// Runs the case's frames of play, in order, through the model: `Ok` with each way the
        /// model differs from the case, or `Err` when the case breaks the file's format.
        ///
        /// Each frame's scene is read into the frame's input through the model's own serde: a
        /// scene the spec leaves out has no value there, and the frame is refused before any
        /// effect, as is a frame the model refuses. A refused frame must leave the State as it
        /// was, give no outputs, and be the case's last.
        pub fn run(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
            let map = HerMap {
                pictures: MapPictures::filled(pictures.map_blank),
                charted: MapCellSet::default(),
            };
            let energy_inputs = InputsIn {
                energy: Some([self.energy.unwrap_or(0), COUNTDOWN_CAPACITY]),
                can_switch: Some(false),
                ..InputsIn::default()
            };
            let mut inputs = frame_inputs(FrameCount(0), &energy_inputs, &map)?;
            let mut hud = countdown_fixture(pictures)?;
            hud.countdown = current_countdown(&self.before)?;
            let mut problems = Vec::new();
            let mut refused = false;
            for (index, frame) in self.frames.iter().enumerate() {
                if refused {
                    return Err(format!("frame {index} follows a refused frame"));
                }
                let played = play_frame(&mut hud, frame, &mut inputs, pictures)?;
                refused = played.is_none();
                let expected = frame_outputs(frame)?;
                if played != expected {
                    problems.push(format!(
                        "frame {index}: expected {expected:?}, got {played:?}"
                    ));
                }
            }
            if refused != (self.expected == CountdownExpected::Unsupported) {
                problems.push(format!("expected {:?}, refused: {refused}", self.expected));
            }
            let after = current_countdown(&self.after)?;
            if hud.countdown != after {
                problems.push(format!(
                    "countdown after: expected {after:?}, got {:?}",
                    hud.countdown
                ));
            }
            Ok(problems)
        }
    }

    impl StartupIn {
        /// Runs the new file's setup through the model: `Ok` with each way the model's HUD
        /// differs from the case's, or `Err` when the case breaks the file's format.
        pub fn run(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
            let rules = self.rules.as_ref().map_or_else(Rules::reference, rules);
            let opening = opening_frame(self.opening_fill.0, &self.opening)?;
            let preview = opening_preview(&self.preview)?;
            let [now, capacity] = self.energy;
            let NewFileHud { upper_row, state } = HudState::new_file(
                &rules,
                pictures,
                &opening,
                &preview,
                FrameCount(self.frame),
                Energy { now, capacity },
            );
            let after = &self.after;
            let mut grid = opening.lower_rows;
            for (place, tile) in &after.replaced {
                grid.set(place.place(), tile.0);
            }
            let expected = HudState {
                grid,
                shown: shown(&after.shown),
                selection: selection(after.selection.item(), after.one_shot)?,
                explored: cell_set(&after.explored)?,
                mini_map: power(after.mini_map_off),
                countdown: current_countdown(&after.countdown)?,
            };
            let mut problems = differences(&expected, &state, &[], &[]);
            if (upper_row == opening.upper_row) != after.upper_row_equal {
                problems.push(format!(
                    "upper row: expected equal {}, got {upper_row:?}",
                    after.upper_row_equal
                ));
            }
            Ok(problems)
        }
    }

    /// A new file's setup, then its first frame of play with her energy as setup drew it and
    /// nothing else to do (no buttons, the mini-map not running): the State setup hands on, and
    /// the State that frame leaves.
    pub fn setup_then_quiet_frame(
        case: &StartupIn,
        pictures: &HudPictures,
    ) -> Result<(HudState, HudState), String> {
        let opening = opening_frame(case.opening_fill.0, &case.opening)?;
        let preview = opening_preview(&case.preview)?;
        let [now, capacity] = case.energy;
        let rules = case.rules.as_ref().map_or_else(Rules::reference, rules);
        let setup = HudState::new_file(
            &rules,
            pictures,
            &opening,
            &preview,
            FrameCount(case.frame),
            Energy { now, capacity },
        )
        .state;
        let map = HerMap {
            pictures: MapPictures::filled(pictures.map_blank),
            charted: MapCellSet::default(),
        };
        let quiet = InputsIn {
            energy: Some(case.energy),
            ..InputsIn::default()
        };
        let mut played = setup.clone();
        played
            .frame_of_play(
                &rules,
                pictures,
                &frame_inputs(FrameCount(0), &quiet, &map)?,
            )
            .map_err(|refused| format!("frame refused: {refused:?}"))?;
        Ok((setup, played))
    }

    /// The opening frame: every cell of its four rows the fill, but those the case gives; row 0
    /// is the upper row, rows 1 to 3 the HUD's.
    fn opening_frame(
        fill: Tile,
        cells: &BTreeMap<OpeningPlaceIn, TileIn>,
    ) -> Result<OpeningFrame, String> {
        let mut opening = OpeningFrame {
            upper_row: [fill; 32],
            lower_rows: HudGrid::filled(fill),
        };
        for (place, tile) in cells {
            let Pair { x, y } = place.place();
            if y == 0 {
                let column = usize::try_from(x).map_err(|_| format!("no column {x}"))?;
                opening.upper_row[column] = tile.0;
            } else {
                let below_upper_row = Pair {
                    x,
                    y: y.saturating_sub(1),
                };
                opening.lower_rows.set(below_upper_row, tile.0);
            }
        }
        Ok(opening)
    }

    /// The opening map preview: each cell the case gives shows its picture, every other is
    /// empty.
    fn opening_preview(
        cells: &BTreeMap<PreviewPlaceIn, PictureIn>,
    ) -> Result<OpeningPreview, String> {
        let mut preview = [[PreviewCell::Empty; 5]; 3];
        for (place, picture) in cells {
            let Pair { x, y } = place.place();
            let index = |n: i32| usize::try_from(n).map_err(|_| format!("no preview cell {n}"));
            preview[index(y)?][index(x)?] = PreviewCell::Shows(picture.0);
        }
        Ok(preview)
    }

    impl StartupObservedIn {
        /// The HUD's State it records, all but the grid, which the record gives only as the
        /// window's colour sets.
        fn recorded(&self, grid: &HudGrid) -> Result<HudState, String> {
            Ok(HudState {
                grid: *grid,
                shown: shown(&self.shown),
                selection: selection(self.selection.item(), self.one_shot)?,
                explored: cell_set(&self.explored)?,
                mini_map: power(self.mini_map_off),
                countdown: current_countdown(&self.countdown)?,
            })
        }

        /// The window's colour sets on `grid`, the top row first.
        fn window_colours(grid: &HudGrid) -> [[Option<ColourSet>; 5]; 3] {
            std::array::from_fn(|row| {
                std::array::from_fn(|column| {
                    let x = i32::try_from(column).expect("a window column");
                    let y = i32::try_from(row).expect("a window row");
                    grid.get(Pair {
                        x: x.saturating_add(WINDOW_LEFT),
                        y,
                    })
                    .map(|tile| tile.colours)
                })
            })
        }
    }

    /// The window's first column on the HUD (§1.2).
    const WINDOW_LEFT: i32 = 26;
    /// The colour sets the mini-map draws in (§1.4): explored, charted, her blink.
    const EXPLORED_COLOURS: u8 = 2;
    const CHARTED_COLOURS: u8 = 3;
    const BLINK_COLOURS: u8 = 7;

    /// Checks the oracle's records of a new file's start, which the independent replay compares
    /// with the original content: here, without that content, each record is read in the
    /// file's format, and what the model says without it is checked.
    ///
    /// - The setup's record (the one held through to the first frame of play): the model's new
    ///   file, with her fresh energy the one recorded, leaves the State recorded; and a preview
    ///   whose pictures stand where the record shows the explored colours, at a frame count in
    ///   the blink's lit or unlit half as her cell shows, draws the window's colours recorded.
    /// - The first frame of play's record, which begins where the setup's is held to: the setup's
    ///   values carry into play unchanged but for the cells she explores.
    pub fn check_startup_observed(
        observed: &[StartupObservedIn],
        pictures: &HudPictures,
    ) -> Result<Vec<String>, String> {
        let mut problems = Vec::new();
        let setup = observed
            .iter()
            .find(|record| record.held_through.is_some())
            .ok_or("no record of the setup")?;
        for record in observed {
            if record.before >= record.after
                || record.held_through.is_some_and(|held| held < record.after)
            {
                return Err(format!("{}: its cuts are out of order", record.id));
            }
        }
        let model = setup_from_record(setup, pictures)?;
        let recorded = setup.recorded(&model.grid)?;
        if model != recorded {
            problems.extend(
                differences(&recorded, &model, &[], &[])
                    .into_iter()
                    .map(|problem| format!("{}: {problem}", setup.id)),
            );
        }
        let colours = StartupObservedIn::window_colours(&model.grid);
        let expected = setup.preview_colours.map(|row| row.map(Some));
        if colours != expected {
            problems.push(format!(
                "{}: window colours: expected {expected:?}, got {colours:?}",
                setup.id
            ));
        }
        for play in observed
            .iter()
            .filter(|record| record.held_through.is_none())
        {
            if Some(play.before) != setup.held_through {
                problems.push(format!(
                    "{}: doesn't begin where the setup is held to",
                    play.id
                ));
            }
            let mut carried = setup.recorded(&model.grid)?;
            carried.explored = cell_set(&play.explored)?;
            if play.recorded(&model.grid)? != carried {
                problems.push(format!(
                    "{}: the setup's State isn't carried into play",
                    play.id
                ));
            }
        }
        Ok(problems)
    }

    /// The model's new file for the setup's record: invented opening art (every cell the blank
    /// cell), a preview picture wherever the record shows the explored colours, and a frame
    /// count in the blink's lit half where her cell shows the blink colours, its unlit half
    /// where it doesn't.
    fn setup_from_record(
        setup: &StartupObservedIn,
        pictures: &HudPictures,
    ) -> Result<HudState, String> {
        let rules = Rules::reference();
        let mut preview = [[PreviewCell::Empty; 5]; 3];
        let mut lit = false;
        for (row, colours) in preview.iter_mut().zip(&setup.preview_colours) {
            for (cell, colour) in row.iter_mut().zip(colours) {
                match colour.number() {
                    EXPLORED_COLOURS => *cell = PreviewCell::Shows(pictures.map_blank),
                    CHARTED_COLOURS => {}
                    BLINK_COLOURS => lit = true,
                    other => return Err(format!("{}: no window colours {other}", setup.id)),
                }
            }
        }
        let frame = if lit {
            FrameCount(0)
        } else {
            FrameCount(rules.her_blink.frames_each_way)
        };
        let blank = pictures.blank_cell();
        let opening = OpeningFrame {
            upper_row: [blank; 32],
            lower_rows: HudGrid::filled(blank),
        };
        let energy = setup.shown.energy;
        let hud = HudState::new_file(
            &rules,
            pictures,
            &opening,
            &preview,
            frame,
            Energy {
                now: energy,
                capacity: energy,
            },
        );
        Ok(hud.state)
    }

    /// Plays one frame of a countdown case on `hud`: its countdown's answer, or `None` when the
    /// frame is refused, which must leave the State as it was.
    fn play_frame(
        hud: &mut HudState,
        frame: &CountdownFrameIn,
        inputs: &mut FrameInputs<'_>,
        pictures: &HudPictures,
    ) -> Result<Option<CountdownAnswer>, String> {
        let Some(runs) = model_value::<CountdownRuns>(format!("{:?}", frame.scene).into()) else {
            return Ok(None);
        };
        let start = frame
            .start
            .map(|[minutes, seconds]| {
                StartTime::new(minutes, seconds)
                    .ok_or_else(|| format!("no start at {minutes}:{seconds}"))
            })
            .transpose()?;
        inputs.countdown = CountdownInputs {
            runs,
            start,
            finished: FinishedFrames(frame.finished),
        };
        let before = hud.clone();
        let result = hud.frame_of_play(&Rules::reference(), pictures, inputs);
        if result.is_err() && *hud != before {
            return Err("a refused frame changed the State".to_owned());
        }
        Ok(result.ok().map(|outputs| outputs.countdown))
    }

    /// The outputs a countdown case's frame expects: both, or neither on a refused frame.
    fn frame_outputs(frame: &CountdownFrameIn) -> Result<Option<CountdownAnswer>, String> {
        match (frame.answer, frame.picture.as_ref()) {
            (None, None) => Ok(None),
            (Some(expiry), Some(picture)) => Ok(Some(CountdownAnswer {
                expiry,
                picture: countdown_picture(picture)?,
            })),
            (Some(_), None) | (None, Some(_)) => {
                Err("a frame gives both its answer and its picture, or neither".to_owned())
            }
        }
    }

    /// A countdown picture.
    fn countdown_picture(given: &CountdownPictureIn) -> Result<CountdownPicture, String> {
        match given {
            CountdownPictureIn::Absent(AbsentIn::Absent) => Ok(CountdownPicture::Absent),
            CountdownPictureIn::Shown(ShownPictureIn {
                place: [x, y],
                time,
            }) => Ok(CountdownPicture::Shown {
                place: Pair { x: *x, y: *y },
                time: time_left(*time)?,
            }),
        }
    }

    /// A current countdown, each part within the range the spec gives it.
    fn current_countdown(given: &CurrentCountdownIn) -> Result<Countdown, String> {
        let BegunIn { time, age, place } = match given {
            CurrentCountdownIn::Dormant(DormantIn::Dormant) => return Ok(Countdown::Dormant),
            CurrentCountdownIn::Begun(begun) => begun,
        };
        let [x, y] = place.map(u16::from_be_bytes);
        Ok(Countdown::Begun(BegunCountdown {
            time_left: time_left(*time)?,
            age: CountdownAge::new(*age).ok_or_else(|| format!("no countdown age {age}"))?,
            place: CountdownPlace::new(Pair { x, y })
                .ok_or_else(|| format!("no countdown place {place:?}"))?,
        }))
    }

    /// A time left, `[minutes, seconds, hundredths]`.
    fn time_left([minutes, seconds, hundredths]: [u8; 3]) -> Result<TimeLeft, String> {
        TimeLeft::new(minutes, seconds, hundredths)
            .ok_or_else(|| format!("no time left {minutes}'{seconds}\"{hundredths}"))
    }

    /// The energy capacity of the countdown cases' frames, which no countdown rule reads.
    const COUNTDOWN_CAPACITY: u16 = 1499;

    /// A value of the model's, read by its serde from `value`, or `None` if it has no such value.
    fn model_value<T: DeserializeOwned>(value: toml::Value) -> Option<T> {
        value.try_into().ok()
    }

    /// Whether a countdown case runs the model, rather than checking only the HUD's refusal.
    pub fn runs_the_model(case: &CountdownIn) -> bool {
        case.scope == CountdownScope::Model
    }

    /// A State with every part set apart from the defaults: varied cells, values last shown,
    /// a one-shot selection, explored cells and the mini-map off.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "invented grid numbers and palettes have known bounded ranges"
    )]
    fn countdown_fixture(pictures: &HudPictures) -> Result<HudState, String> {
        let mut hud = state(&BeforeIn::default(), pictures)?;
        for y in 0..HUD_ROWS {
            for x in 0..HUD_COLUMNS {
                let number = u16::try_from(y * HUD_COLUMNS + x).expect("invented cell");
                hud.grid.set(
                    Pair { x, y },
                    interface_hud::Tile::new(
                        Look {
                            picture: interface_hud::Picture::new(number).expect("invented picture"),
                            flip: interface_hud::Flip::Both,
                        },
                        interface_hud::ColourSet::new(u8::try_from(number % 8).expect("palette"))
                            .expect("invented colours"),
                    ),
                );
            }
        }
        hud.shown.energy = 299;
        hud.shown.ammo =
            enum_map! { Ammo::Missiles => 5, Ammo::SuperMissiles => 99, Ammo::PowerBombs => 7 };
        hud.shown.highlighted = Some(SelectableItem::XRayScope);
        hud.selection = Selection::OneShot(SelectableItem::GrappleBeam);
        hud.explored.insert(Pair { x: 31, y: 30 });
        hud.explored.insert(Pair { x: 32, y: 31 });
        hud.explored.insert(Pair { x: 63, y: 31 });
        hud.mini_map = MiniMapPower::Off;
        Ok(hud)
    }

    /// The rules a vector runs under: the game's, with each value the vector gives in its place.
    fn rules(given: &RulesIn) -> Rules {
        let mut rules = Rules::reference();
        if let Some(frames_each_way) = given.one_shot_flash {
            rules.one_shot_flash = Alternation { frames_each_way };
        }
        if let Some(frames_each_way) = given.her_blink {
            rules.her_blink = Alternation { frames_each_way };
        }
        for (boss, screens) in given.boss_screens.iter().flat_map(boss_screens) {
            rules.boss_screens[boss] = screens;
        }
        rules
    }

    /// Each boss `boss_screens` names, with its screens.
    fn boss_screens(given: &BossScreensIn) -> impl Iterator<Item = (Boss, Vec<Screen>)> + '_ {
        let BossScreensIn {
            kraid,
            draygon,
            crocomire,
            phantoon,
            mother_brain,
            ridley_in_norfair,
            ridley_in_ceres,
        } = given;
        [
            (Boss::Kraid, kraid),
            (Boss::Draygon, draygon),
            (Boss::Crocomire, crocomire),
            (Boss::Phantoon, phantoon),
            (Boss::MotherBrain, mother_brain),
            (Boss::RidleyInNorfair, ridley_in_norfair),
            (Boss::RidleyInCeres, ridley_in_ceres),
        ]
        .into_iter()
        .filter_map(|(boss, screens)| {
            let screens = screens.as_ref()?.iter().map(|&[x, y]| Screen { x, y });
            Some((boss, screens.collect()))
        })
    }

    /// A `BossEvent`'s inputs: her room's cell and the boss, and nothing else.
    fn boss_event(inputs: &InputsIn) -> Result<Step<'static>, String> {
        let InputsIn {
            energy: None,
            reserve: None,
            auto_reserve: None,
            missiles: None,
            super_missiles: None,
            power_bombs: None,
            equipped: None,
            at_start: None,
            select: None,
            cancel: None,
            can_switch: None,
            muted_by: None,
            dropped: None,
            new_area: None,
            collected: None,
            mini_map:
                Some(MiniMapIn {
                    runs: None,
                    turned_on: None,
                    mother_brain_falls: None,
                    position: None,
                    room: Some(RoomIn { cell, blocks: None }),
                    map: None,
                    boss: Some(boss),
                }),
        } = inputs
        else {
            return Err("a BossEvent's only inputs are mini_map's room cell and boss".to_owned());
        };
        let boss = match boss {
            BossName::Kraid => Boss::Kraid,
            BossName::Draygon => Boss::Draygon,
            BossName::Crocomire => Boss::Crocomire,
            BossName::Phantoon => Boss::Phantoon,
            BossName::RidleyInNorfair => Boss::RidleyInNorfair,
            BossName::RidleyInCeres => Boss::RidleyInCeres,
        };
        Ok(Step::BossEncountered(boss, point(*cell)))
    }

    /// An `OutOfPlay` step's inputs: `mini_map`'s `turned_on`, `new_area` and `dropped`
    /// without its `when`, and nothing else.
    fn out_of_play<'a>(
        inputs: &InputsIn,
        new_area: Option<&'a MapCellSet>,
    ) -> Result<OutOfPlayInputs<'a>, String> {
        let refused = || {
            "an OutOfPlay step's only inputs are mini_map's turned_on, new_area and dropped \
             without its when"
                .to_owned()
        };
        let InputsIn {
            energy: None,
            reserve: None,
            auto_reserve: None,
            missiles: None,
            super_missiles: None,
            power_bombs: None,
            equipped: None,
            at_start: None,
            select: None,
            cancel: None,
            can_switch: None,
            muted_by: None,
            dropped,
            new_area: _,
            collected: None,
            mini_map,
        } = inputs
        else {
            return Err(refused());
        };
        let mini_map_turned_on = match mini_map {
            None => false,
            Some(MiniMapIn {
                runs: None,
                turned_on,
                mother_brain_falls: None,
                position: None,
                room: None,
                map: None,
                boss: None,
            }) => turned_on.unwrap_or(false),
            Some(_) => return Err(refused()),
        };
        let dropped = match dropped {
            None => Dropped::Nothing,
            Some(DroppedIn { what, when: None }) => drop_of(*what),
            Some(DroppedIn { when: Some(_), .. }) => return Err(refused()),
        };
        Ok(OutOfPlayInputs {
            mini_map_turned_on,
            new_area: new_area.map_or(NewArea::SameArea, NewArea::Entered),
            dropped,
        })
    }

    /// The drop of what a `dropped` takes.
    const fn drop_of(what: DropWhatIn) -> Dropped {
        match what {
            DropWhatIn::Selection => Dropped::Selection,
            DropWhatIn::OneShot => Dropped::OneShotItemAlone,
        }
    }

    /// A `Frame` step's two drops, before the status bar and after it: its `dropped` lands at
    /// its `when`, before the status bar if the vector doesn't say, and nothing at the other.
    fn frame_drops(dropped: Option<&DroppedIn>) -> (Dropped, Dropped) {
        match dropped {
            None => (Dropped::Nothing, Dropped::Nothing),
            Some(DroppedIn {
                what,
                when: None | Some(DropWhenIn::BeforeStatusBar),
            }) => (drop_of(*what), Dropped::Nothing),
            Some(DroppedIn {
                what,
                when: Some(DropWhenIn::AfterStatusBar),
            }) => (Dropped::Nothing, drop_of(*what)),
        }
    }

    /// A `Frame` step's inputs.
    fn frame_inputs<'map>(
        frame: FrameCount,
        inputs: &InputsIn,
        map: &'map HerMap,
    ) -> Result<FrameInputs<'map>, String> {
        if inputs.new_area.is_some() {
            return Err("only an OutOfPlay step has a new_area".to_owned());
        }
        let [now, capacity] = inputs.energy.ok_or("a Frame step has an energy")?;
        let stock = |stock: Option<[u16; 2]>| {
            let [count, capacity] = stock.unwrap_or([0, 0]);
            Stock { count, capacity }
        };
        let ammunition = Ammunition::new(enum_map! {
            Ammo::Missiles => stock(inputs.missiles),
            Ammo::SuperMissiles => stock(inputs.super_missiles),
            Ammo::PowerBombs => stock(inputs.power_bombs),
        })
        .ok_or("ammunition outside the HUD's domain")?;
        let mode = if inputs.auto_reserve.unwrap_or(false) {
            ReserveMode::Auto
        } else {
            ReserveMode::Manual
        };
        let (mini_map, mother_brain_falls) = mini_map_inputs(inputs.mini_map.as_ref(), map)?;
        let (dropped_before_status_bar, dropped_after_status_bar) =
            frame_drops(inputs.dropped.as_ref());
        Ok(FrameInputs {
            frame,
            stock_at_start: stock_at_start(inputs, &ammunition),
            controls: controls(inputs),
            mini_map,
            mother_brain_falls,
            status: Status {
                energy: Energy { now, capacity },
                reserve: Reserve {
                    energy: inputs.reserve.unwrap_or(0),
                    mode,
                },
                ammunition,
            },
            collected: inputs.collected.and_then(ItemName::item),
            muted_by: inputs.muted_by.and_then(|mute| match mute {
                MuteIn::None => None,
                MuteIn::Grapple => Some(Mute::Grapple),
                MuteIn::SpinJump => Some(Mute::SpinJump),
                MuteIn::TimeStopped => Some(Mute::TimeStopped),
                MuteIn::WallJump => Some(Mute::WallJump),
            }),
            dropped_before_status_bar,
            dropped_after_status_bar,
            countdown: CountdownInputs {
                runs: CountdownRuns::MaintainCountdown,
                start: None,
                finished: FinishedFrames(0),
            },
        })
    }

    /// Her stock as the frame begins: `at_start`'s counts, each left out the same as at the
    /// frame's end, and the items she has on.
    fn stock_at_start(inputs: &InputsIn, at_end: &Ammunition) -> StockAtStart {
        let given = inputs.at_start.as_ref();
        let count = |ammo: Ammo, at_start: Option<[u16; 2]>| {
            at_start.map_or_else(|| at_end.of(ammo).count, |[count, _capacity]| count)
        };
        let mut equipped = Equipped::default();
        for equipment in inputs.equipped.iter().flatten() {
            match equipment {
                Equipment::GrappleBeam => equipped.grapple_beam = true,
                Equipment::XRayScope => equipped.x_ray_scope = true,
            }
        }
        StockAtStart {
            counts: enum_map! {
                Ammo::Missiles => count(Ammo::Missiles, given.and_then(|stock| stock.missiles)),
                Ammo::SuperMissiles =>
                    count(Ammo::SuperMissiles, given.and_then(|stock| stock.super_missiles)),
                Ammo::PowerBombs =>
                    count(Ammo::PowerBombs, given.and_then(|stock| stock.power_bombs)),
            },
            equipped,
        }
    }

    /// The buttons.
    fn controls(inputs: &InputsIn) -> Controls {
        Controls {
            select: match inputs.select {
                None => SelectButton::NotPressed,
                Some(SelectIn::Pressed) => SelectButton::Pressed,
            },
            cancel: match inputs.cancel {
                None => CancelButton::NotPressed,
                Some(CancelIn::Pressed) => CancelButton::Pressed,
                Some(CancelIn::Held) => CancelButton::Held,
            },
            can_switch: inputs.can_switch.unwrap_or(true),
        }
    }

    /// The mini-map's inputs and whether Mother Brain falls; with no `mini_map`, it has nothing
    /// to do: it doesn't run.
    ///
    /// A mini-map that runs reads where she is and her room's cell and blocks; her map left out
    /// is one whose every cell shows the map's blank, charted by none and not collected. Her
    /// fall reads her room's cell. An input no rule reads this step may be left out, and stands
    /// at 0.
    fn mini_map_inputs<'map>(
        mini_map: Option<&MiniMapIn>,
        map: &'map HerMap,
    ) -> Result<(MiniMapInputs<'map>, bool), String> {
        let idle = MiniMapIn {
            runs: Some(false),
            turned_on: None,
            mother_brain_falls: None,
            position: None,
            room: None,
            map: None,
            boss: None,
        };
        let mini_map = mini_map.unwrap_or(&idle);
        if mini_map.boss.is_some() {
            return Err("a Frame step names no boss".to_owned());
        }
        let runs = mini_map.runs.unwrap_or(true);
        let falls = mini_map.mother_brain_falls.unwrap_or(false);
        let room = mini_map.room.as_ref();
        let blocks = room.and_then(|room| room.blocks);
        if runs && (mini_map.position.is_none() || blocks.is_none()) {
            return Err("a running mini-map has a position and a room's blocks".to_owned());
        }
        if falls && room.is_none() {
            return Err("Mother Brain's fall reads her room's cell".to_owned());
        }
        let [x, y] = mini_map.position.unwrap_or([0, 0]);
        let [across, down] = blocks.unwrap_or([0, 0]);
        let inputs = MiniMapInputs {
            runs,
            turned_on: mini_map.turned_on.unwrap_or(false),
            position: Pair { x, y },
            room: Room {
                cell: point(room.map_or([0, 0], |room| room.cell)),
                blocks: Pair { x: across, y: down },
            },
            map,
            map_collected: mini_map.map.as_ref().is_some_and(|map| map.collected),
        };
        Ok((inputs, falls))
    }

    /// Her map: a cell not listed shows the map's blank, as does a beyond picture not listed;
    /// with no map given, every cell does.
    fn her_map(mini_map: Option<&MiniMapIn>, pictures: &HudPictures) -> Result<HerMap, String> {
        let mut map = HerMap {
            pictures: MapPictures::filled(pictures.map_blank),
            charted: MapCellSet::default(),
        };
        let Some(MapIn {
            collected: _,
            pictures: cells,
            charted,
            beyond,
        }) = mini_map.and_then(|mini_map| mini_map.map.as_ref())
        else {
            return Ok(map);
        };
        lay_pictures(&mut map.pictures, cells, 0);
        lay_pictures(&mut map.pictures, beyond, MAP_COLUMNS);
        map.charted = cell_set(charted)?;
        Ok(map)
    }

    /// Lays `"column,row" = picture` pictures into her map's pictures `shift` columns along: the
    /// beyond pictures' columns 0 and 1 are its columns 64 and 65.
    fn lay_pictures<const COLUMNS: i32, const ROWS: i32>(
        grid: &mut MapPictures,
        pictures: &BTreeMap<PlaceIn<COLUMNS, ROWS>, PictureIn>,
        shift: i32,
    ) {
        for (key, picture) in pictures {
            let at = key.place();
            grid.set(
                Pair {
                    x: at.x.saturating_add(shift),
                    y: at.y,
                },
                picture.0,
            );
        }
    }

    /// The state from a before; its HUD left out, every cell is the blank in its colours.
    fn state(before: &BeforeIn, pictures: &HudPictures) -> Result<HudState, String> {
        let mut grid = HudGrid::filled(pictures.blank_cell());
        for (y, row) in (0..).zip(before.hud.iter().flatten()) {
            for (x, &tile) in (0..).zip(&row.0) {
                grid.set(Pair { x, y }, tile);
            }
        }
        Ok(HudState {
            grid,
            shown: shown(&before.shown),
            selection: selection(before.selection.item(), before.one_shot)?,
            explored: cell_set(&before.explored)?,
            mini_map: power(before.mini_map_off),
            countdown: interface_hud::Countdown::Dormant,
        })
    }

    /// The state an after expects: the before's, with the after's changes.
    fn expected_state(after: &AfterIn, before: &HudState) -> Result<HudState, String> {
        let mut grid = before.grid;
        for (place, tile) in &after.changed {
            grid.set(place.place(), tile.0);
        }
        let item = after
            .selection
            .map_or_else(|| before.selection.item(), ItemName::item);
        let one_shot = after
            .one_shot
            .unwrap_or_else(|| before.selection.one_shot().is_some());
        Ok(HudState {
            grid,
            shown: after
                .shown
                .as_ref()
                .map_or(before.shown, |given| shown_after(given, before.shown)),
            selection: selection(item, one_shot)?,
            explored: after
                .explored
                .as_ref()
                .map_or(Ok(before.explored), |cells| cell_set(cells))?,
            mini_map: after.mini_map_off.map_or(before.mini_map, power),
            countdown: before.countdown,
        })
    }

    /// The selection from the selected item and whether it is the one-shot item.
    fn selection(item: Option<SelectableItem>, one_shot: bool) -> Result<Selection, String> {
        match (item, one_shot) {
            (None, false) => Ok(Selection::None),
            (None, true) => Err("one_shot with none selected".to_owned()),
            (Some(item), false) => Ok(Selection::Item(item)),
            (Some(item), true) => Ok(Selection::OneShot(item)),
        }
    }

    /// Whether the mini-map is off.
    const fn power(off: bool) -> MiniMapPower {
        if off {
            MiniMapPower::Off
        } else {
            MiniMapPower::On
        }
    }

    /// The values last shown.
    fn shown(shown: &ShownIn) -> Shown {
        Shown {
            energy: shown.energy,
            ammo: enum_map! {
                Ammo::Missiles => shown.missiles,
                Ammo::SuperMissiles => shown.super_missiles,
                Ammo::PowerBombs => shown.power_bombs,
            },
            highlighted: shown.selection.item(),
        }
    }

    /// The values last shown an after expects: each it gives, and every other as before.
    fn shown_after(given: &super::shape::ShownAfterIn, before: Shown) -> Shown {
        Shown {
            energy: given.energy.unwrap_or(before.energy),
            ammo: enum_map! {
                Ammo::Missiles => given.missiles.unwrap_or(before.ammo[Ammo::Missiles]),
                Ammo::SuperMissiles =>
                    given.super_missiles.unwrap_or(before.ammo[Ammo::SuperMissiles]),
                Ammo::PowerBombs => given.power_bombs.unwrap_or(before.ammo[Ammo::PowerBombs]),
            },
            highlighted: given.selection.map_or(before.highlighted, ItemName::item),
        }
    }

    /// A set of `[column, row]` cells, each on her map.
    fn cell_set(cells: &[[i32; 2]]) -> Result<MapCellSet, String> {
        let mut set = MapCellSet::default();
        for &cell in cells {
            let at = point(cell);
            if set.get(at).is_none() {
                return Err(format!("no map cell {cell:?}"));
            }
            set.insert(at);
        }
        Ok(set)
    }

    /// A `[column, row]` pair as a place.
    const fn point([x, y]: [i32; 2]) -> Place {
        Pair { x, y }
    }

    /// Every way the model's after differs from the vector's.
    fn differences(
        expected: &HudState,
        actual: &HudState,
        expected_events: &[HudEvent],
        actual_events: &[HudEvent],
    ) -> Vec<String> {
        let mut problems = grid_differences(&expected.grid, &actual.grid);
        if expected.countdown != actual.countdown {
            problems.push(format!(
                "countdown: expected {:?}, got {:?}",
                expected.countdown, actual.countdown
            ));
        }
        if expected.shown != actual.shown {
            problems.push(format!(
                "shown: expected {:?}, got {:?}",
                expected.shown, actual.shown
            ));
        }
        if expected.selection != actual.selection {
            problems.push(format!(
                "selection: expected {:?}, got {:?}",
                expected.selection, actual.selection
            ));
        }
        if expected.mini_map != actual.mini_map {
            problems.push(format!(
                "mini-map: expected {:?}, got {:?}",
                expected.mini_map, actual.mini_map
            ));
        }
        if expected.explored != actual.explored {
            problems.push(format!(
                "explored: expected {:?}, got {:?}",
                cell_list(&expected.explored),
                cell_list(&actual.explored)
            ));
        }
        if expected_events != actual_events {
            problems.push(format!(
                "events: expected {expected_events:?}, got {actual_events:?}"
            ));
        }
        problems
    }

    /// The cells that differ, each named "column,row" as the vector file names them.
    fn grid_differences(expected: &HudGrid, actual: &HudGrid) -> Vec<String> {
        let mut problems = Vec::new();
        for y in 0..HUD_ROWS {
            for x in 0..HUD_COLUMNS {
                let at = Pair { x, y };
                if expected.get(at) != actual.get(at) {
                    problems.push(format!(
                        "cell {x},{y}: expected {:?}, got {:?}",
                        expected.get(at),
                        actual.get(at)
                    ));
                }
            }
        }
        problems
    }

    /// A set's cells as `[column, row]` pairs.
    fn cell_list(cells: &MapCellSet) -> Vec<[i32; 2]> {
        (0..MAP_ROWS)
            .flat_map(|y| (0..MAP_COLUMNS).map(move |x| [x, y]))
            .filter(|&cell| cells.contains(point(cell)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use interface_hud::{BehaviourId, CATALOGUE, Checked, HudEvent};

    use crate::convert::{
        check_startup_observed, load, load_changed, pictures, runs_the_model,
        setup_then_quiet_frame,
    };
    use crate::shape::VectorIn;

    #[test]
    fn every_startup_passes() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        assert!(!file.startup.is_empty(), "the file holds no startups");
        let failures: Vec<String> = file
            .startup
            .iter()
            .filter_map(|case| match case.run(&pictures) {
                Ok(problems) if problems.is_empty() => None,
                Ok(problems) => Some(format!("{}: {}", case.id, problems.join("; "))),
                Err(error) => Some(format!("{}: unreadable: {error}", case.id)),
            })
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} startups fail:\n{}",
            failures.len(),
            file.startup.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn the_recorded_startup_agrees_with_the_model() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        let problems = check_startup_observed(&file.startup_observed, &pictures).unwrap();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    /// Spec: `NewFileStatusBar`, `NewFileMiniMap`. The State setup hands on is the one ordinary
    /// play reads: a first frame with her energy as setup drew it and nothing else to do
    /// redraws nothing.
    #[test]
    fn the_setup_state_carries_into_play() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        for case in &file.startup {
            let (setup, played) = setup_then_quiet_frame(case, &pictures).unwrap();
            assert_eq!(played, setup, "{}", case.id);
        }
    }

    /// Changes to the file that each add a key or a name its header doesn't define: a key in
    /// each table, a place off its grid or not written "column,row" in each table keyed by
    /// places, and each kind of name.
    const UNKNOWN: [(&str, &str); 48] = [
        // The new file's tables: a key in each, a place off its grid in each keyed by places,
        // and a colour set the format lacks.
        (
            "id = \"C-new-file-energy\"",
            "id = \"C-new-file-energy\"\nspeed = 1",
        ),
        (
            "[startup.after]\nupper_row_equal = true\nshown = { energy = 237",
            "[startup.after]\nspeed = 1\nupper_row_equal = true\nshown = { energy = 237",
        ),
        (
            "id = \"R-new-file-hud\"",
            "id = \"R-new-file-hud\"\nspeed = 1",
        ),
        ("\"31,3\" = \"302:0\"", "\"31,4\" = \"302:0\""),
        ("\"4,2\" = \"202hv\"", "\"5,2\" = \"202hv\""),
        ("\"30,2\" = \"202:2hv\"", "\"30,3\" = \"202:2hv\""),
        (
            "preview_colours = [[2, 3, 3, 3, 2]",
            "preview_colours = [[2, 3, 3, 3, 8]",
        ),
        ("opening_fill = \"300:6v\"", "opening_fill = \"300:6x\""),
        // A key in each table.
        ("[pictures]", "speed = 1\n[pictures]"),
        (
            "[vector.inputs.at_start]\nmissiles = [2, 10]",
            "[vector.inputs.at_start]\nmissiles = [2, 10]\nspeed = 1",
        ),
        (
            "controls = { selection = \"SuperMissiles\", one_shot = false }",
            "controls = { selection = \"SuperMissiles\", one_shot = false, speed = 1 }",
        ),
        (
            "controls = { selection = \"SuperMissiles\", one_shot = false }",
            "controls = { selection = \"Bombs\", one_shot = false }",
        ),
        (
            "rules = { one_shot_flash = 4 }",
            "rules = { one_shot_flash = 4, speed = 1 }",
        ),
        (
            "{ Kraid = [[2, 0], [0, 2]] }",
            "{ Kraid = [[2, 0], [0, 2]], speed = [] }",
        ),
        (
            "[vector.inputs.new_area]",
            "[vector.inputs.new_area]\nspeed = 1",
        ),
        (
            "dropped = { what = \"OneShot\" }",
            "dropped = { what = \"OneShot\", speed = 1 }",
        ),
        ("diagonal = \"2E4\"", "diagonal = \"2E4\"\nspeed = \"000\""),
        ("[pictures.icons]", "[pictures.icons]\nSpeedBooster = []"),
        ("id = \"C-R1\"", "id = \"C-R1\"\nspeed = 1"),
        ("[vector.before]", "[vector.before]\nspeed = 1"),
        (
            "shown = { energy = 333,",
            "shown = { speed = 1, energy = 333,",
        ),
        ("can_switch = true", "can_switch = true\nspeed = 1"),
        // The shot's cancel is no input of its own: it is the drop before the status bar.
        (
            "can_switch = true",
            "can_switch = true\nshot_cancels = true",
        ),
        ("runs = true", "runs = true\nspeed = 1"),
        ("blocks = [128, 32] }", "blocks = [128, 32], speed = 1 }"),
        ("collected = false", "collected = false\nspeed = 1"),
        (
            "changed = { \"7,2\" = \"055:3\" }",
            "changed = { \"7,2\" = \"055:3\" }\nspeed = 1",
        ),
        (
            "shown = { energy = 338,",
            "shown = { speed = 1, energy = 338,",
        ),
        // A place off its grid, or not a place, in each table keyed by places. Column 7, row 2
        // written row first is row 7, which the HUD lacks.
        ("\"7,2\" = \"055:3\"", "\"2,7\" = \"055:3\""),
        ("\"7,2\" = \"055:3\"", "\"speed\" = \"055:3\""),
        ("\"30,20\" = \"06Av\"", "\"64,20\" = \"06Av\""),
        ("\"30,20\" = \"06Av\"", "\"30\" = \"06Av\""),
        ("beyond = { \"0,9\"", "beyond = { \"2,9\""),
        // Each kind of name.
        ("boss = \"Kraid\"", "boss = \"Mario\""),
        ("muted_by = \"SpinJump\"", "muted_by = \"Somersault\""),
        ("source = \"constructed\"", "source = \"recorded\""),
        ("step = \"BossEvent\"", "step = \"Pause\""),
        (
            "\"Switched:Missiles\", \"Click\"",
            "\"Switched:Missiles\", \"Clack\"",
        ),
        ("selection = \"None\"", "selection = \"Nothing\""),
        (
            "equipped = [\"XRayScope\"]",
            "equipped = [\"SpeedBooster\"]",
        ),
        ("select = \"Pressed\"", "select = \"Held\""),
        ("cancel = \"Pressed\"", "cancel = \"Released\""),
        ("collected = \"Missiles\"", "collected = \"Bombs\""),
        ("what = \"OneShot\"", "what = \"Everything\""),
        ("when = \"AfterStatusBar\"", "when = \"Later\""),
        // A picture, a flip or a colour set the format lacks.
        ("full_tank = \"0A1\"", "full_tank = \"4A1\""),
        ("full_tank = \"0A1\"", "full_tank = \"0A1x\""),
        ("\"7,2\" = \"055:3\"", "\"7,2\" = \"055:8\""),
    ];

    #[test]
    fn an_unknown_key_or_name_fails_the_file() {
        for (from, to) in UNKNOWN {
            assert!(load_changed(from, to).is_err(), "{to} was read");
        }
    }

    /// Changes to the file that each give a step an input its header allows only in another
    /// step: a drop's when outside play, an input of a frame outside play, a mini-map that runs
    /// outside play, and a new area in a frame.
    const MISPLACED: [(&str, &str); 6] = [
        (
            "[vector.inputs]\ndropped = { what = \"Selection\" }",
            "[vector.inputs]\ndropped = { what = \"Selection\" }\nat_start = { missiles = [1, 1] }",
        ),
        (
            "[vector.inputs]\ndropped = { what = \"Selection\" }\n[vector.after]",
            "[vector.inputs]\ndropped = { what = \"Selection\" }\n[vector.after]\n\
             controls = { selection = \"None\", one_shot = false }",
        ),
        (
            "[vector.inputs]\ndropped = { what = \"Selection\" }",
            "[vector.inputs]\ndropped = { what = \"Selection\", when = \"AfterStatusBar\" }",
        ),
        (
            "[vector.inputs]\ndropped = { what = \"Selection\" }",
            "[vector.inputs]\nenergy = [1, 1]\ndropped = { what = \"Selection\" }",
        ),
        (
            "[vector.inputs.mini_map]\nturned_on = true\n[vector.after]",
            "[vector.inputs.mini_map]\nturned_on = true\nruns = true\n[vector.after]",
        ),
        (
            "muted_by = \"SpinJump\"\n[vector.inputs.mini_map]",
            "muted_by = \"SpinJump\"\nnew_area = { explored = [] }\n[vector.inputs.mini_map]",
        ),
    ];

    #[test]
    fn an_input_outside_its_step_fails_a_vector() {
        for (from, to) in MISPLACED {
            let file = load_changed(from, to).unwrap();
            let pictures = pictures(&file.pictures);
            assert!(
                file.vector.iter().any(|v| v.run(&pictures).is_err()),
                "{to} was run"
            );
        }
    }

    /// The entry's name in the spec: its variant's.
    fn name(id: BehaviourId) -> String {
        format!("{id:?}")
    }

    /// The entries a vector exercises: those it lists, and the outputs whose events it expects
    /// (spec: `Switching`, `SwitchClick`).
    fn exercised(vector: &VectorIn) -> impl Iterator<Item = String> + '_ {
        let outputs = vector.after.events.iter().map(|event| match event.0 {
            HudEvent::Switched { .. } => "Switching".to_owned(),
            HudEvent::Click => "SwitchClick".to_owned(),
        });
        vector.entries.iter().cloned().chain(outputs)
    }

    #[test]
    fn every_vector_passes() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        assert!(!file.vector.is_empty(), "the vector file holds no vectors");
        let failures: Vec<String> = file
            .vector
            .iter()
            .filter_map(|vector| match vector.run(&pictures) {
                Ok(problems) if problems.is_empty() => None,
                Ok(problems) => Some(format!("{}: {}", vector.id, problems.join("; "))),
                Err(error) => Some(format!("{}: unreadable: {error}", vector.id)),
            })
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} vectors fail:\n{}",
            failures.len(),
            file.vector.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn every_countdown_case_passes() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        assert!(
            !file.countdown.is_empty(),
            "the file holds no countdown cases"
        );
        let failures: Vec<String> = file
            .countdown
            .iter()
            .filter_map(|case| match case.run(&pictures) {
                Ok(problems) if problems.is_empty() => None,
                Ok(problems) => Some(format!("{}: {}", case.id, problems.join("; "))),
                Err(error) => Some(format!("{}: unreadable: {error}", case.id)),
            })
            .collect();
        assert!(
            failures.is_empty(),
            "{} of {} countdown cases fail:\n{}",
            failures.len(),
            file.countdown.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn countdown_unknown_fields_and_names_are_rejected() {
        for (from, to) in [
            (
                "id = \"R-countdown-dormant\"",
                "id = \"R-countdown-dormant\"\nspeed = 1",
            ),
            ("scope = \"Model\"\nsource", "scope = \"Anything\"\nsource"),
            ("before = \"Dormant\"", "before = \"Other\""),
            (
                "after = { time = [1, 0, 0], age = 0,",
                "after = { time = [1, 0, 0], speed = 0, age = 0,",
            ),
            (
                "after = { time = [1, 0, 0], age = 0,",
                "after = { time = [1, 0], age = 0,",
            ),
            (
                "scene = \"MaintainCountdown\"",
                "scene = \"InventedChoice\"",
            ),
            ("finished = 10764", "finished = 10764\nspeed = 1"),
            ("finished = 10764", "finished = \"one\""),
            ("start = [1, 0]", "start = [1]"),
            ("expected = \"Pass\"", "expected = \"Maybe\""),
            ("answer = \"NoExpiry\"", "answer = \"Expiry\""),
            ("picture = \"Absent\"", "picture = \"Digits\""),
            (
                "picture = { place = [128, 128], time = [1, 0, 0] }",
                "picture = { place = [128, 128], time = [1, 0, 0], speed = 1 }",
            ),
        ] {
            assert!(load_changed(from, to).is_err(), "accepted {to}");
        }
    }

    /// Changes that each give a countdown case a value outside the range the spec gives it: an
    /// age past 96, seconds past 59, a place past the corner, a start past 59 seconds; and a
    /// refused frame that gives outputs.
    const OUT_OF_RANGE: [(&str, &str); 5] = [
        ("age = 4,", "age = 97,"),
        ("time = [1, 0, 0], age = 4", "time = [1, 60, 0], age = 4"),
        ("place = [[220, 0], [48, 0]]", "place = [[220, 1], [48, 0]]"),
        ("start = [1, 0]", "start = [1, 60]"),
        (
            "scene = \"Unknown\"",
            "scene = \"Unknown\"\nanswer = \"NoExpiry\"",
        ),
    ];

    #[test]
    fn a_countdown_value_out_of_range_fails_its_case() {
        for (from, to) in OUT_OF_RANGE {
            let file = load_changed(from, to).unwrap();
            let pictures = pictures(&file.pictures);
            assert!(
                file.countdown.iter().any(
                    |case| !matches!(case.run(&pictures), Ok(problems) if problems.is_empty())
                ),
                "{to} passed"
            );
        }
    }

    #[test]
    fn ids_are_unique_and_entries_are_in_the_catalogue() {
        let file = load().unwrap();
        let mut ids = std::collections::BTreeSet::new();
        let rows = file
            .vector
            .iter()
            .map(|v| (&v.id, &v.entries))
            .chain(file.countdown.iter().map(|v| (&v.id, &v.entries)))
            .chain(file.startup.iter().map(|v| (&v.id, &v.entries)));
        for record in &file.startup_observed {
            assert!(ids.insert(&record.id), "{} appears twice", record.id);
        }
        for (id, entries) in rows {
            assert!(ids.insert(id), "{id} appears twice");
            for entry in entries {
                assert!(
                    CATALOGUE.iter().any(|b| name(b.id) == *entry),
                    "{id} lists {entry}, which the catalogue lacks"
                );
            }
        }
    }

    #[test]
    fn the_catalogue_lists_the_specs_29_entries() {
        assert_eq!(CATALOGUE.len(), 29);
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let file = load().unwrap();
        let checked = CATALOGUE.iter().filter(|b| b.checked == Checked::ByVectors);
        for behaviour in checked {
            let name = name(behaviour.id);
            assert!(
                file.vector.iter().any(|v| exercised(v).any(|e| e == name))
                    || file
                        .countdown
                        .iter()
                        .any(|case| runs_the_model(case) && case.entries.contains(&name))
                    || file.startup.iter().any(|case| case.entries.contains(&name)),
                "no vector exercises {name}"
            );
        }
    }
}
