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
        #[serde(default)]
        pub hud_work: Vec<WorkIn>,
        #[serde(default)]
        pub hud_resume: Vec<ResumeIn>,
        pub hud_observed: Vec<ObservedIn>,
        #[serde(default)]
        pub countdown: Vec<CountdownIn>,
    }

    /// Countdown scope distinguishes actual model vectors from neighboring producer obligations.
    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CountdownScope {
        Model,
        IntegrationAdmission,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CountdownCurrentIn {
        Dormant,
        Active,
        Expired,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum SceneIn {
        MaintainCountdown,
        PresentationOnly,
        Unknown,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CountdownBoundary {
        Unread,
        Chosen,
        Completed,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum CountdownExpected {
        Pass,
        Unsupported,
    }

    #[derive(Deserialize)]
    pub enum CountdownSource {
        #[serde(rename = "recorded")]
        Recorded,
        #[serde(rename = "constructed")]
        Constructed,
    }

    /// Strict vector shape only, not an integration admission implementation.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CountdownIn {
        pub id: String,
        pub scope: CountdownScope,
        #[serde(rename = "source")]
        pub _source: CountdownSource,
        pub entries: Vec<String>,
        pub before: CountdownCurrentIn,
        pub scene: SceneIn,
        pub opportunities: usize,
        pub boundary: CountdownBoundary,
        pub restore: bool,
        pub later_scene: SceneIn,
        pub expected: CountdownExpected,
        pub after: CountdownCurrentIn,
        pub answers: Vec<interface_hud::CountdownExpiry>,
        pub pictures: Vec<interface_hud::CountdownPicture>,
        pub energy: Option<u16>,
        /// Neighboring quantity, deliberately not an input of the model.
        #[serde(rename = "protection_frames")]
        pub _protection_frames: Option<u16>,
    }

    /// Additive staged-frame tables, recursively validated using the ordinary strict shapes.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct WorkIn {
        pub id: String,
        #[serde(rename = "source")]
        pub _source: Source,
        pub entries: Vec<String>,
        #[serde(default)]
        pub before: BeforeIn,
        #[serde(default)]
        pub controls: toml::Table,
        #[serde(default)]
        pub mini_map: toml::Table,
        #[serde(default)]
        pub status: toml::Table,
        #[serde(default)]
        pub frame: u8,
        #[serde(default)]
        pub controls_after: toml::Table,
        pub map_after: Option<toml::Table>,
        pub after: toml::Table,
        pub pose_after: Option<[u16; 2]>,
        pub energy_at_map: Option<[u16; 2]>,
        pub owner_at_map: Option<OwnerIn>,
        pub owner_at_status: Option<OwnerIn>,
    }

    /// A change at an offered or chosen observation of a base work case.
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ResumeIn {
        pub id: String,
        #[serde(rename = "source")]
        pub _source: Source,
        pub entries: Vec<String>,
        pub base: String,
        pub opportunity: Stage,
        pub cut: CutIn,
        pub patch: toml::Table,
        #[serde(default)]
        pub expect_controls_after: toml::Table,
        #[serde(default)]
        pub expect_map_after: toml::Table,
        pub expect_after: toml::Table,
    }

    #[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Stage {
        ItemControls,
        MiniMapUpdate,
        StatusRefresh,
    }

    #[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
    pub enum CutIn {
        BeforeAnswer,
        AfterAnswer,
        AfterEffect,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    pub struct OwnerIn {
        pub position: [u16; 2],
        pub energy: [u16; 2],
    }

    #[derive(Deserialize)]
    pub enum ObservedSource {
        #[serde(rename = "ordinary-play-pair")]
        OrdinaryPlayPair,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "independent recorded relations, not work state flags"
    )]
    pub struct ObservedIn {
        pub id: String,
        #[serde(rename = "source")]
        pub _source: ObservedSource,
        pub cut: u16,
        pub entries: Vec<String>,
        pub map_runs: bool,
        pub map_window_changed_by_movement: bool,
        pub map_retained_through_status: bool,
        pub energy_read_after_drawing: bool,
        pub gauge_retained_through_status: bool,
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
        pub shown: Option<ShownIn>,
        pub selection: Option<ItemName>,
        pub one_shot: Option<bool>,
        pub mini_map_off: Option<bool>,
        pub explored: Option<Vec<[i32; 2]>>,
        #[serde(default)]
        pub events: Vec<EventIn>,
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
        pub select: Option<SelectIn>,
        pub cancel: Option<CancelIn>,
        pub can_switch: Option<bool>,
        pub muted_by: Option<MuteIn>,
        pub dropped: Option<DroppedIn>,
        pub new_area: Option<NewAreaIn>,
        pub collected: Option<ItemName>,
        pub mini_map: Option<MiniMapIn>,
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
        Alternation, Ammo, Ammunition, Boss, CancelButton, Controls, DisplayInputs, Dropped,
        Energy, Equipped, FrameCount, FrameInputs, HerMap, HudEvent, HudGrid, HudPictures,
        HudState, HudWork, Icons, ItemControlInputs, Look, MapCellSet, MapPictures, MiniMapInputs,
        MiniMapPower, MovementInputs, Mute, NewArea, Observation, OutOfPlayInputs, Pair, Place,
        Reserve, ReserveMode, Room, Rules, Screen, SelectButton, SelectableItem, Selection, Shown,
        Status, Stock,
    };

    use super::shape::{
        AfterIn, BeforeIn, BossName, BossScreensIn, CancelIn, DropWhatIn, DropWhenIn, DroppedIn,
        Equipment, IconsIn, InputsIn, ItemName, MapIn, MiniMapIn, MuteIn, PictureIn, PicturesIn,
        PlaceIn, ResumeIn, RoomIn, RulesIn, SelectIn, ShownIn, StepIn, VectorFile, VectorIn,
        WorkIn,
    };

    /// The vector file.
    const VECTOR_TEXT: &str = include_str!("../../../../scenarios/vectors/interface-hud.toml");
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
        /// Takes the step on `state`, returning what it reports; only a frame of play reports
        /// anything.
        fn take(
            &self,
            state: &mut HudState,
            rules: &Rules,
            pictures: &HudPictures,
        ) -> Vec<HudEvent> {
            match self {
                Self::FrameOfPlay(inputs) => state.frame_of_play(rules, pictures, inputs),
                Self::OutOfPlay(inputs) => {
                    state.out_of_play(inputs);
                    Vec::new()
                }
                &Self::BossEncountered(boss, room) => {
                    state.boss_blanks_the_map(rules, pictures, boss, room);
                    Vec::new()
                }
            }
        }
    }

    /// The file, read.
    pub fn load() -> Result<VectorFile, String> {
        load_text(VECTOR_TEXT)
    }

    /// The file's text with its first `from` replaced by `to`, read.
    pub fn load_changed(from: &str, to: &str) -> Result<VectorFile, String> {
        assert!(VECTOR_TEXT.contains(from), "the file holds no {from}");
        load_text(&VECTOR_TEXT.replacen(from, to, 1))
    }

    /// A vector file's text, read.
    fn load_text(text: &str) -> Result<VectorFile, String> {
        let file: VectorFile = toml::from_str(text).map_err(|error| error.to_string())?;
        validate_work_tables(&file)?;
        Ok(file)
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
        pub fn run(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
            let before = state(&self.before, pictures)?;
            let rules = self.rules.as_ref().map_or_else(Rules::reference, rules);
            let map = her_map(self.inputs.mini_map.as_ref(), pictures)?;
            let new_area = self
                .inputs
                .new_area
                .as_ref()
                .map(|area| cell_set(&area.explored))
                .transpose()?;
            let step = self.step(&map, new_area.as_ref())?;
            let mut after = before.clone();
            let events = step.take(&mut after, &rules, pictures);
            if let Step::FrameOfPlay(inputs) = &step {
                let complete = complete_staged(
                    before.clone(),
                    ItemControlInputs::from_frame(inputs),
                    &DisplayInputs::from_frame(inputs),
                    &rules,
                    pictures,
                );
                assert_eq!(
                    complete.state(),
                    &after,
                    "{} full ordinary/staged State",
                    self.id
                );
                assert_eq!(
                    complete.events(),
                    events,
                    "{} complete ordered events",
                    self.id
                );
            }
            let expected = expected_state(&self.after, &before)?;
            let expected_events: Vec<HudEvent> = self.after.events.iter().map(|e| e.0).collect();
            Ok(differences(&expected, &after, &expected_events, &events))
        }

        /// The step the vector takes, with its inputs; only a `Frame` step has a frame count.
        fn step<'a>(
            &self,
            map: &'a HerMap,
            new_area: Option<&'a MapCellSet>,
        ) -> Result<Step<'a>, String> {
            match (self.step, self.frame) {
                (StepIn::Frame, Some(frame)) => {
                    frame_inputs(FrameCount(frame), &self.inputs, map).map(Step::FrameOfPlay)
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

    /// Execute only admitted Model cases through the public countdown model. This harness
    /// has no scene admission decision: the producer's qualified input is supplied directly.
    pub fn run_countdown(vector: &super::shape::CountdownIn, pictures: &HudPictures) {
        use interface_hud::{Countdown, CountdownOpportunity, CountdownWork};

        use super::shape::{
            CountdownBoundary, CountdownCurrentIn, CountdownExpected, CountdownScope, SceneIn,
        };

        assert_eq!(vector.scope, CountdownScope::Model);
        assert_eq!(vector.before, CountdownCurrentIn::Dormant);
        assert_eq!(vector.after, CountdownCurrentIn::Dormant);
        assert_eq!(vector.scene, SceneIn::MaintainCountdown);
        assert_eq!(vector.expected, CountdownExpected::Pass);
        // Later scene input is deliberately not supplied to already chosen work. Qualification
        // of an unread PresentationOnly contrast is outside this model and is not tested here.
        assert!(matches!(
            vector.later_scene,
            SceneIn::MaintainCountdown | SceneIn::PresentationOnly
        ));
        assert_ne!(vector.boundary, CountdownBoundary::Unread);
        let mut current = countdown_fixture(pictures);
        if let Some(energy) = vector.energy {
            refresh_countdown_fixture(&mut current, pictures, energy);
        }
        // Protection is external to this domain and deliberately not supplied to the model.
        current.countdown = Countdown::Dormant;
        let initial = current.clone();
        let mut answers = Vec::new();
        let mut contributions = Vec::new();
        for _ in 0..vector.opportunities {
            let mut work = CountdownWork::new(current);
            assert_eq!(work.answer(), &Observation::Unread);
            assert!(work.publication().is_none());
            assert!(!work.apply());
            assert!(work.choose(CountdownOpportunity::MaintainCountdown));
            let Observation::Chosen(chosen) = work.answer() else {
                panic!("{}: no actual chosen model answer", vector.id);
            };
            assert_eq!(chosen.expiry, interface_hud::CountdownExpiry::NoExpiry);
            assert_eq!(chosen.picture, interface_hud::CountdownPicture::Absent);
            assert_eq!(chosen.opportunity, CountdownOpportunity::MaintainCountdown);
            assert_eq!(work.state(), &initial);
            assert!(work.publication().is_none());
            assert!(!work.choose(CountdownOpportunity::MaintainCountdown));
            if vector.boundary == CountdownBoundary::Completed {
                assert!(work.apply());
            }
            if vector.restore {
                work = restore_countdown(work);
            }
            let completed_now = work.apply();
            assert_eq!(completed_now, vector.boundary == CountdownBoundary::Chosen);
            assert!(!work.apply());
            let (state, answer) = work.publication().expect("completed countdown publication");
            assert_eq!(work.answer(), &Observation::Completed(*answer));
            assert_eq!(state, &initial, "{}: complete HUD preserved", vector.id);
            answers.push(answer.expiry);
            contributions.push(answer.picture);
            current = state.clone();
        }
        assert_eq!(
            answers, vector.answers,
            "{}: ordered expiry answers",
            vector.id
        );
        assert_eq!(
            contributions, vector.pictures,
            "{}: ordered picture contributions",
            vector.id
        );
        assert_eq!(current, initial);
    }

    /// Independently varied grid, caches, selection, explored map and map power.
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "invented grid numbers and palettes have known bounded ranges"
    )]
    fn countdown_fixture(pictures: &HudPictures) -> HudState {
        let mut hud = state(&BeforeIn::default(), pictures).expect("countdown fixture");
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
        hud
    }

    /// The independent-health vector supplies actual status inputs to the ordinary HUD first.
    fn refresh_countdown_fixture(hud: &mut HudState, pictures: &HudPictures, energy: u16) {
        let inputs = InputsIn {
            energy: Some([energy, 1499]),
            can_switch: Some(false),
            ..InputsIn::default()
        };
        let map = HerMap {
            pictures: MapPictures::filled(pictures.map_blank),
            charted: MapCellSet::default(),
        };
        let frame = frame_inputs(FrameCount(0), &inputs, &map).expect("fixture frame inputs");
        assert_eq!(
            hud.frame_of_play(&Rules::reference(), pictures, &frame),
            [HudEvent::Click]
        );
        assert_eq!(hud.shown.energy, energy);
        assert_eq!(hud.countdown, interface_hud::Countdown::Dormant);
    }

    /// Whole-value fresh restoration: discard the work, not just its State projection.
    #[expect(
        clippy::drop_non_drop,
        reason = "whole-work serde restoration explicitly discards its original"
    )]
    fn restore_countdown(original: interface_hud::CountdownWork) -> interface_hud::CountdownWork {
        let text = toml::to_string(&original).expect("serialize countdown work");
        drop(original);
        toml::from_str(&text).expect("restore fresh countdown work")
    }

    /// Recursively apply a table patch; arrays and scalar answers replace whole values.
    fn merge(target: &mut toml::Table, patch: &toml::Table) {
        for (key, value) in patch {
            match (target.get_mut(key), value) {
                (Some(toml::Value::Table(old)), toml::Value::Table(new)) => merge(old, new),
                _ => {
                    target.insert(key.clone(), value.clone());
                }
            }
        }
    }

    /// Additive defaults only. The ordinary parser's requirements remain unchanged.
    fn work_inputs(table: &toml::Table) -> Result<(u8, InputsIn), String> {
        let mut table = table.clone();
        let frame = table
            .remove("frame")
            .map_or(Ok(0), toml::Value::try_into)
            .map_err(|error: toml::de::Error| error.to_string())?;
        table
            .entry("energy")
            .or_insert(toml::Value::Array(vec![0.into(), 0.into()]));
        if let Some(toml::Value::Table(mini_map)) = table.get_mut("mini_map") {
            mini_map
                .entry("map")
                .or_insert(toml::Value::Table(toml::Table::new()));
        }
        let inputs = toml::Value::Table(table)
            .try_into()
            .map_err(|error: toml::de::Error| error.to_string())?;
        Ok((frame, inputs))
    }

    fn after_table(table: &toml::Table) -> Result<AfterIn, String> {
        toml::Value::Table(table.clone())
            .try_into()
            .map_err(|error| error.to_string())
    }

    /// Even recursive patches are checked against the appropriate stage's shape at load.
    fn validate_work_tables(file: &VectorFile) -> Result<(), String> {
        let pictures = pictures(&file.pictures);
        for work in &file.hud_work {
            WorkCase::load(work, None, &pictures)?;
        }
        for resume in &file.hud_resume {
            WorkCase::load(resume.base(file)?, Some(resume), &pictures)?;
        }
        Ok(())
    }

    /// The public staged path for inputs already known to describe a complete frame.
    fn complete_staged(
        before: HudState,
        controls: ItemControlInputs,
        display: &DisplayInputs,
        rules: &Rules,
        pictures: &HudPictures,
    ) -> HudWork {
        let mut work = HudWork::begin(before, display.frame);
        assert!(work.choose_controls(controls));
        assert!(work.apply());
        let movement = MovementInputs::capture(&display.mini_map(), display.mother_brain_falls);
        assert!(work.choose_mini_map(rules, pictures, movement));
        assert!(work.apply());
        assert!(work.choose_status(rules, pictures, display.status_inputs()));
        assert!(work.apply());
        work
    }

    /// Additive after tables leave omitted shown values independent, as they were before.
    fn work_expected(
        table: &toml::Table,
        before: &HudState,
    ) -> Result<(HudState, Vec<HudEvent>), String> {
        let after = after_table(table)?;
        let mut expected = expected_state(&after, before)?;
        if let Some(toml::Value::Table(shown_patch)) = table.get("shown") {
            if !shown_patch.contains_key("energy") {
                expected.shown.energy = before.shown.energy;
            }
            if !shown_patch.contains_key("selection") {
                expected.shown.highlighted = before.shown.highlighted;
            }
            for (key, ammo) in [
                ("missiles", Ammo::Missiles),
                ("super_missiles", Ammo::SuperMissiles),
                ("power_bombs", Ammo::PowerBombs),
            ] {
                if !shown_patch.contains_key(key) {
                    expected.shown.ammo[ammo] = before.shown.ammo[ammo];
                }
            }
        }
        Ok((expected, after.events.iter().map(|event| event.0).collect()))
    }

    fn chosen_inputs(table: &toml::Table, pictures: &HudPictures) -> Result<DisplayInputs, String> {
        let (frame, inputs) = work_inputs(table)?;
        let map = her_map(inputs.mini_map.as_ref(), pictures)?;
        Ok(DisplayInputs::from_frame(&frame_inputs(
            FrameCount(frame),
            &inputs,
            &map,
        )?))
    }

    fn chosen_controls(
        table: &toml::Table,
        pictures: &HudPictures,
    ) -> Result<ItemControlInputs, String> {
        let (frame, inputs) = work_inputs(table)?;
        let map = her_map(inputs.mini_map.as_ref(), pictures)?;
        Ok(ItemControlInputs::from_frame(&frame_inputs(
            FrameCount(frame),
            &inputs,
            &map,
        )?))
    }

    mod staged_vectors;
    use staged_vectors::WorkCase;

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
        let mut equipped = Equipped::default();
        for equipment in inputs.equipped.iter().flatten() {
            match equipment {
                Equipment::GrappleBeam => equipped.grapple_beam = true,
                Equipment::XRayScope => equipped.x_ray_scope = true,
            }
        }
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
            status: Status {
                energy: Energy { now, capacity },
                reserve: Reserve {
                    energy: inputs.reserve.unwrap_or(0),
                    mode,
                },
                ammunition,
                equipped,
            },
            controls: controls(inputs),
            dropped_before_status_bar,
            dropped_after_status_bar,
            collected: inputs.collected.and_then(ItemName::item),
            mother_brain_falls,
            mini_map,
        })
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
            muted_by: inputs.muted_by.and_then(|mute| match mute {
                MuteIn::None => None,
                MuteIn::Grapple => Some(Mute::Grapple),
                MuteIn::SpinJump => Some(Mute::SpinJump),
                MuteIn::TimeStopped => Some(Mute::TimeStopped),
                MuteIn::WallJump => Some(Mute::WallJump),
            }),
        }
    }

    /// The mini-map's inputs and whether Mother Brain falls; with no `mini_map`, it has nothing
    /// to do: it doesn't run.
    ///
    /// A mini-map that runs reads where she is, her room's cell and blocks, and her map; her
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
        let reads_all = mini_map.position.is_some() && blocks.is_some() && mini_map.map.is_some();
        if runs && !reads_all {
            return Err("a running mini-map has a position, a room's blocks and a map".to_owned());
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
            shown: after.shown.as_ref().map_or(before.shown, shown),
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

    use crate::convert::{load, load_changed, pictures};
    use crate::shape::VectorIn;

    /// Changes to the file that each add a key or a name its header doesn't define: a key in
    /// each table, a place off its grid or not written "column,row" in each table keyed by
    /// places, and each kind of name.
    const UNKNOWN: [(&str, &str); 37] = [
        // A key in each table.
        ("[pictures]", "speed = 1\n[pictures]"),
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
    const MISPLACED: [(&str, &str); 4] = [
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
    fn five_countdown_model_vectors_pass() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        let model: Vec<_> = file
            .countdown
            .iter()
            .filter(|v| v.scope == super::shape::CountdownScope::Model)
            .collect();
        assert_eq!(model.len(), 5);
        for vector in model {
            super::convert::run_countdown(vector, &pictures);
        }
        let neighboring: Vec<_> = file
            .countdown
            .iter()
            .filter(|v| v.scope == super::shape::CountdownScope::IntegrationAdmission)
            .collect();
        assert_eq!(neighboring.len(), 5);
        // Descriptions only: no admission evaluator runs and none earn model coverage.
        for vector in neighboring {
            assert_eq!(
                vector.expected,
                super::shape::CountdownExpected::Unsupported
            );
            let no_answers: [interface_hud::CountdownExpiry; 0] = [];
            let no_pictures: [interface_hud::CountdownPicture; 0] = [];
            assert_eq!(vector.answers, no_answers);
            assert_eq!(vector.pictures, no_pictures);
        }
    }

    #[test]
    fn countdown_catalogue_metadata_matches_spec() {
        for (id, kind) in [
            (BehaviourId::EscapeCountdown, interface_hud::Kind::Mechanic),
            (
                BehaviourId::CountdownContinuation,
                interface_hud::Kind::Timing,
            ),
        ] {
            let entry = CATALOGUE.iter().find(|b| b.id == id).unwrap();
            assert_eq!(entry.kind, kind);
            assert_eq!(entry.switch, interface_hud::Switch::Fixed);
            assert_eq!(entry.checked, Checked::ByVectors);
        }
    }

    #[test]
    fn countdown_unknown_fields_and_names_are_rejected() {
        for (from, to) in [
            (
                "id = \"R-countdown-dormant\"",
                "id = \"R-countdown-dormant\"\nspeed = 1",
            ),
            ("scope = \"Model\"", "scope = \"Anything\""),
            ("before = \"Dormant\"", "before = \"Other\""),
            (
                "scene = \"MaintainCountdown\"",
                "scene = \"InventedChoice\"",
            ),
            ("boundary = \"Completed\"", "boundary = \"AppliedTwice\""),
            ("answers = [\"NoExpiry\"]", "answers = [\"Expiry\"]"),
            ("pictures = [\"Absent\"]", "pictures = [\"Digits\"]"),
        ] {
            assert!(load_changed(from, to).is_err(), "accepted {to}");
        }
    }

    #[test]
    fn additive_work_and_resume_vectors_pass() {
        let file = load().unwrap();
        let pictures = pictures(&file.pictures);
        assert_eq!(file.hud_work.len(), 8);
        assert_eq!(file.hud_resume.len(), 11);
        for work in &file.hud_work {
            let problems = work.run(&pictures).unwrap();
            assert!(problems.is_empty(), "{}: {}", work.id, problems.join("; "));
        }
        for resume in &file.hud_resume {
            let problems = resume.run(&file, &pictures).unwrap();
            assert!(
                problems.is_empty(),
                "{}: {}",
                resume.id,
                problems.join("; ")
            );
        }
    }

    #[test]
    fn additive_unknown_fields_and_names_are_rejected() {
        for (from, to) in [
            (
                "id = \"H-controls-drop\"",
                "id = \"H-controls-drop\"\nspeed = 1",
            ),
            ("[hud_work.controls]", "[hud_work.controls]\nspeed = 1"),
            ("[hud_work.status]", "[hud_work.status]\nspeed = 1"),
            (
                "[hud_work.controls]",
                "[hud_work.controls]\nenergy = [0, 0]",
            ),
            ("[hud_work.map_after]", "[hud_work.map_after]\nspeed = 1"),
            ("[hud_observed]]", "[hud_observed]]\nspeed = 1"),
            (
                "[hud_work.controls_after]",
                "[hud_work.controls_after]\nspeed = 1",
            ),
            ("[hud_work.after]", "[hud_work.after]\nspeed = 1"),
            ("[hud_work.before]", "[hud_work.before]\nspeed = 1"),
            ("[hud_work.mini_map]", "[hud_work.mini_map]\nspeed = 1"),
            (
                "id = \"H-controls-before\"",
                "id = \"H-controls-before\"\nspeed = 1",
            ),
            (
                "patch = { missiles = [0, 10],",
                "patch = { speed = 1, missiles = [0, 10],",
            ),
            ("expect_after = {}", "expect_after = { speed = 1 }"),
            (
                "owner_at_map = { position = [256, 16], energy = [119, 199] }",
                "owner_at_map = { position = [256, 16], energy = [119, 199], speed = 1 }",
            ),
            ("\ncut = \"BeforeAnswer\"", "\ncut = \"BeforeReading\""),
            ("opportunity = \"ItemControls\"", "opportunity = \"Move\""),
        ] {
            assert!(load_changed(from, to).is_err(), "accepted {to}");
        }
    }

    #[test]
    fn recorded_pair_checks_only_the_exported_relations() {
        let file = load().unwrap();
        let [first, second] = file.hud_observed.as_slice() else {
            panic!("successive recorded frames")
        };
        assert_eq!((first.cut, second.cut), (127, 128));
        assert!(!first.map_window_changed_by_movement);
        assert!(second.map_window_changed_by_movement);
        for observation in [first, second] {
            assert!(observation.map_runs);
            assert!(observation.map_retained_through_status);
            assert!(observation.energy_read_after_drawing);
            assert!(observation.gauge_retained_through_status);
        }
        // No native state, energy or picture numbers are supplied by these witnesses.
        // The H vectors separately execute the model with invented content and inputs.
    }

    #[test]
    fn staged_catalogue_metadata_matches_spec() {
        for id in [
            BehaviourId::ItemControls,
            BehaviourId::MiniMapUpdate,
            BehaviourId::StatusRefresh,
            BehaviourId::RetainedHudWork,
        ] {
            let entry = CATALOGUE.iter().find(|b| b.id == id).unwrap();
            assert_eq!(entry.kind, interface_hud::Kind::Timing);
            assert_eq!(entry.switch, interface_hud::Switch::Fixed);
            assert_eq!(entry.checked, Checked::ByVectors);
        }
    }

    #[test]
    fn additive_ids_and_catalogue_entries_are_checked() {
        let file = load().unwrap();
        let mut ids = std::collections::BTreeSet::new();
        let rows = file
            .vector
            .iter()
            .map(|v| (&v.id, &v.entries))
            .chain(file.hud_work.iter().map(|v| (&v.id, &v.entries)))
            .chain(file.hud_resume.iter().map(|v| (&v.id, &v.entries)))
            .chain(file.countdown.iter().map(|v| (&v.id, &v.entries)))
            .chain(file.hud_observed.iter().map(|v| (&v.id, &v.entries)));
        for (id, entries) in rows {
            assert!(ids.insert(id), "duplicate {id}");
            for entry in entries {
                assert!(
                    CATALOGUE.iter().any(|b| name(b.id) == *entry),
                    "{id}: unknown {entry}"
                );
            }
        }
    }

    #[test]
    fn vector_ids_are_unique() {
        let file = load().unwrap();
        for (at, vector) in file.vector.iter().enumerate() {
            let again = file
                .vector
                .iter()
                .skip(at)
                .skip(1)
                .any(|v| v.id == vector.id);
            assert!(!again, "vector {} appears twice", vector.id);
        }
    }

    #[test]
    fn every_listed_entry_is_in_the_catalogue() {
        for vector in load().unwrap().vector {
            for entry in &vector.entries {
                assert!(
                    CATALOGUE.iter().any(|b| name(b.id) == *entry),
                    "vector {} lists {entry}, which the catalogue lacks",
                    vector.id
                );
            }
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let file = load().unwrap();
        let checked = CATALOGUE.iter().filter(|b| b.checked == Checked::ByVectors);
        for behaviour in checked {
            let name = name(behaviour.id);
            assert!(
                file.vector.iter().any(|v| exercised(v).any(|e| e == name))
                    || file.hud_work.iter().any(|v| v.entries.contains(&name))
                    || file.hud_resume.iter().any(|v| v.entries.contains(&name))
                    || file.countdown.iter().any(|v| {
                        v.scope == super::shape::CountdownScope::Model && v.entries.contains(&name)
                    }),
                "no vector exercises {name}"
            );
        }
    }
}
