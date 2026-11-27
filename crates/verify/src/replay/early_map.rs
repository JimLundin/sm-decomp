//! Reviewed external Movement inputs for the early mini-map opportunities.

use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::{appearance, camera, compare::Observation, hud};
use crate::{
    cache::Cache,
    lock::{Lock, RowId},
};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed,
    connected::{self as production, early_map as map},
};
use game::state::owned::OwnedState;

use super::{
    Sources, connected, data_failure,
    index::{Case, FixtureFile},
    movement,
    result::Outcome,
};
use crate::lock::Sha256;
use game::production::connected::inputs::{Activity, CameraInputs, Inputs};
use serde::Deserialize;
use std::{num::Wrapping, path::Path};
use types::{Buttons, Hitbox, Item, Pair, Parity, Parts};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Borrowed {
    format: String,
    repository: String,
    row: String,
    initial_cut: u64,
    terrain_source_cut: u64,
    initial_wram_sha256: String,
    opportunity: [Opportunity; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Opportunity {
    end_cut: u64,
    locked_held_index: u64,
    locked_edge_before_index: u64,
    held: u16,
    pressed: u16,
    equipped_items: Vec<Item>,
    parity: Parity,
    energy: u16,
    liquid: Dry,
    sounds_waiting: u8,
    contact_suppressors: Vec<ContactSuppressor>,
    cinematic: bool,
    elevator_moving: bool,
    ordinary_guards_clear: bool,
    solid_actor: Vec<Actor>,
    camera: CameraInputs,
    map: SourceMap,
}

#[derive(Deserialize)]
enum Dry {
    None,
}

/// An owner active at her stride, as the fixture names it.
#[derive(Clone, Copy, Deserialize)]
enum ContactSuppressor {
    Cinematic,
    Boss,
    SpecialColours,
}

impl ContactSuppressor {
    const fn activity(self) -> samus_appearance::Activity {
        match self {
            Self::Cinematic => samus_appearance::Activity::Cinematic,
            Self::Boss => samus_appearance::Activity::Boss,
            Self::SpecialColours => samus_appearance::Activity::SpecialColourTime,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Actor {
    centre: [u16; 2],
    radius: [u16; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent source map activity and events"
)]
struct SourceMap {
    enabled: bool,
    room_origin: [u16; 2],
    room_size_blocks: [u16; 2],
    area: u8,
    map_collected: bool,
    frame_count: u8,
    turned_on: bool,
    mother_brain_falls: bool,
}

fn inputs(
    native: &Path,
    file: &FixtureFile,
    seed: &[u8],
    content: &connected::Content,
    [prior, first_pad, second_pad]: [u16; 3],
) -> Result<[Current; 2], Outcome> {
    let source: Borrowed = file.read(native)?;
    if source.format != "scoped-movement-map-input-v1"
        || source.repository != "15a04113d80468046849b65703c3ccde8f0cb6f6"
        || source.row != "chain-any-0003-DFD7"
        || source.initial_cut != 78
        || source.terrain_source_cut != 78
        || source.initial_wram_sha256 != Sha256::of(seed).to_string()
    {
        return Err(data_failure(
            Some(78),
            "early Map source and seed alignment",
        ));
    }
    let [first, second] = source.opportunity;
    Ok([
        first.inputs(FIRST_MAP, seed, content, [prior, first_pad])?,
        second.inputs(SECOND_MAP, seed, content, [first_pad, second_pad])?,
    ])
}

impl Opportunity {
    fn inputs(
        self,
        cut: u64,
        seed: &[u8],
        content: &connected::Content,
        [previous, held]: [u16; 2],
    ) -> Result<Current, Outcome> {
        let pressed = held & !previous;
        if self.end_cut != cut
            || self.locked_held_index != cut.saturating_sub(1)
            || self.locked_edge_before_index != cut.saturating_sub(2)
            || movement::pad(held).0 != self.held
            || movement::pad(pressed).0 != self.pressed
            || held & 0x4000 != 0
            || pressed & 0x2000 != 0
            || self.map.turned_on
            || self.map.mother_brain_falls
            || self.elevator_moving
            || !self.ordinary_guards_clear
        {
            return Err(data_failure(Some(cut), "early Map ordinary ingress"));
        }
        let Dry::None = self.liquid;
        let pair = |[x, y]: [u16; 2]| Pair {
            x: Wrapping(x),
            y: Wrapping(y),
        };
        let actors = self
            .solid_actor
            .into_iter()
            .map(|actor| Hitbox {
                centre: pair(actor.centre),
                radius: pair(actor.radius),
            })
            .collect();
        let mut items = types::Set::default();
        for item in self.equipped_items {
            items[item] = true;
        }
        let mut activities = types::Set::default();
        for suppressor in self.contact_suppressors {
            activities[suppressor.activity()] = true;
        }
        // The fixture reads the cinematic at her stride and at her moves; the frame's one input
        // holds both readings when they agree.
        if activities[samus_appearance::Activity::Cinematic] != self.cinematic {
            return Err(data_failure(Some(cut), "early Map cinematic lifetime"));
        }
        let movement = Inputs {
            held: Buttons(self.held),
            pressed: Buttons(self.pressed),
            items,
            parity: self.parity,
            energy: self.energy,
            liquid: samus_appearance::Liquid::NONE,
            sounds_waiting: samus_appearance::SoundsWaiting::new(self.sounds_waiting)
                .ok_or_else(|| data_failure(Some(cut), "early Map queue occupancy"))?,
            activities,
            // Held from the origin: only a passage's first entry reads it, and neither
            // opportunity has one.
            charge: super::appearance::charge(seed)?,
            geometry: movement::level_world(seed, &content.movement, actors)?,
            camera: self.camera,
            activity: Activity::Inactive,
        };
        Ok(Current {
            movement,
            map: self.map,
        })
    }
}

struct Current {
    movement: Inputs,
    map: SourceMap,
}

/// The HUD's neighbours in a frame, from its record pair; no selected owner supplies them.
#[derive(Clone, Copy)]
struct HudNeighbours {
    stock_at_start: interface_hud::StockAtStart,
    status: interface_hud::Status,
    muted_by: Option<interface_hud::Mute>,
    can_switch: bool,
    countdown: interface_hud::CountdownInputs,
}

impl HudNeighbours {
    fn read(cache: &Cache, cut: u64) -> Result<Self, Outcome> {
        let start = record(cache, cut.saturating_sub(1))?;
        let end = record(cache, cut)?;
        Ok(Self {
            stock_at_start: hud::stock_at_start(&start.wram)?,
            status: hud::status(&end.wram)?,
            muted_by: hud::muted_by(&end.wram)?,
            can_switch: hud::can_switch(&end.wram)?,
            countdown: hud::countdown_inputs(&start.wram, &end.wram)?,
        })
    }
}

pub(super) struct InputTape {
    current: [Current; 2],
    neighbours: [HudNeighbours; 2],
    hud: hud::Content,
    rules: interface_hud::Rules,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    format: String,
    row: String,
    required_carried_owner_cut: u64,
    second_consumer_cut: u64,
    opportunity: [CapturedMap; 2],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedMap {
    end_cut: u64,
    early_whole: [u16; 2],
    early_fraction: [u16; 2],
    explored: String,
    map_words: [u16; 15],
    carried: Option<Carried>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Carried {
    late_whole: [u16; 2],
    late_fraction: [u16; 2],
    complete_movement_appearance_camera_expected_source_cut: u64,
}

pub(super) struct Expectations {
    captured: Expected,
    carry: Vec<Observation>,
}

const ROW: &str = "chain-any-0003-DFD7";
const ORIGIN: u64 = 78;
const FIRST_MAP: u64 = ORIGIN + 1;
const SECOND_MAP: u64 = FIRST_MAP + 1;
const EARLY_FIELDS: usize = 4;

// Only the first map frame's owners are compared beside the map.
pub(super) const fn field_count(case: &super::index::Case, cuts: usize) -> Option<usize> {
    match cuts {
        0 => Some(0),
        1 => Some(case.expectations.fields.len()),
        2 => case.expectations.fields.len().checked_add(EARLY_FIELDS),
        _ => None,
    }
}

fn index(cut: u64) -> Result<usize, Outcome> {
    match cut {
        FIRST_MAP => Ok(0),
        SECOND_MAP => Ok(1),
        _ => Err(data_failure(Some(cut), "early Map cut")),
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let lock = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "early Map record lock"))?;
    let lock = Lock::parse(&lock).map_err(|_| data_failure(None, "early Map lock schema"))?;
    let row = lock
        .row(&RowId(ROW.to_owned()))
        .ok_or_else(|| data_failure(None, "early Map locked row"))?;
    let cache = Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "early Map record cache"))?;
    let origin = record(&cache, ORIGIN)?;
    let content = connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: appearance::Content::load(&sources.rom)?,
    };
    // The record's pad field is the authenticated physical input at the same lock index.
    // No selected owner state from the later record enters the input tape.
    let pads = [
        record(&cache, ORIGIN - 1)?.pad.0,
        origin.pad.0,
        record(&cache, FIRST_MAP)?.pad.0,
    ];
    let read_word = |offset: usize| {
        origin
            .wram
            .get(offset..offset.saturating_add(2))
            .and_then(|bytes| bytes.try_into().ok())
            .map(u16::from_le_bytes)
    };
    if read_word(0x09b8) != Some(0x4000) || read_word(0x09ba) != Some(0x2000) {
        return Err(data_failure(
            Some(ORIGIN),
            "early Map locked HUD control bindings",
        ));
    }
    let files = case.fixture_pair()?;
    let current = inputs(&sources.native, &files.inputs, &origin.wram, &content, pads)?;
    let neighbours = [
        HudNeighbours::read(&cache, FIRST_MAP)?,
        HudNeighbours::read(&cache, SECOND_MAP)?,
    ];
    let hud = hud::Content::load(&sources.rom)?;
    let seed_hud = hud::decode_state(&origin.wram)?;
    let expected = Expectations::load(&sources.native, &files.expected, &cache, &content)?;
    let (appearance, playing) = appearance::running_owner(&origin.wram, &content.appearance)?;
    let seed = production::State::new(
        movement::seed(&origin.wram)?,
        appearance,
        playing,
        camera::seed(&origin.wram, &camera::Content)?,
    )
    .and_then(|state| state.with_early_map(seed_hud))
    .map_err(|_| data_failure(Some(ORIGIN), "early Map admitted owner state"))?;
    let content = ProductionContent::Connected(Box::new(production::Content {
        movement: content.movement.movement,
        appearance: content.appearance.appearance,
    }));
    let rules = ProductionRules::Connected(Box::new(production::Rules {
        movement: samus_movement::Rules::default(),
        appearance: samus_appearance::Rules::VANILLA,
        camera: game::production::CameraRules::new(world_camera::Rules::REFERENCE),
    }));
    let production = Production::admit(
        ProductionSeed::Connected(Box::new(seed)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(ORIGIN), "early Map production admission"))?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::EarlyMap(Box::new(InputTape {
            current,
            neighbours,
            hud,
            rules: interface_hud::Rules::default(),
        })),
        expected: ExpectedReader::EarlyMap(Box::new(expected)),
        actual: ActualReader::EarlyMap,
        schedule: vec![FIRST_MAP, SECOND_MAP],
    })
}

impl InputTape {
    fn input(&self, cut: u64) -> Result<AdvanceInput<'_>, Outcome> {
        let index = index(cut)?;
        let current = self
            .current
            .get(index)
            .ok_or_else(|| data_failure(Some(cut), "early Map input opportunity"))?;
        let neighbours = self
            .neighbours
            .get(index)
            .ok_or_else(|| data_failure(Some(cut), "early Map HUD neighbours"))?;
        let map = &current.map;
        Ok(AdvanceInput::ConnectedMap {
            inputs: &current.movement,
            map: map::Inputs {
                enabled: map.enabled,
                room: hud::room_from_words(map.room_origin, map.room_size_blocks),
                frame: interface_hud::FrameCount(map.frame_count),
                controls: (
                    interface_hud::SelectButton::NotPressed,
                    interface_hud::CancelButton::NotPressed,
                ),
                can_switch: neighbours.can_switch,
                stock_at_start: neighbours.stock_at_start,
                status: neighbours.status,
                muted_by: neighbours.muted_by,
                countdown: neighbours.countdown,
                map_collected: map.map_collected,
                map: self.hud.map_area(map.area)?,
                pictures: &self.hud.pictures,
                rules: &self.rules,
            },
        })
    }

    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        production
            .advance(self.input(cut)?)
            .map_err(|fault| super::production_failure(cut, fault))
    }
}

impl Expectations {
    fn load(
        native: &Path,
        file: &FixtureFile,
        cache: &Cache,
        content: &connected::Content,
    ) -> Result<Self, Outcome> {
        let captured: Expected = file.read(native)?;
        let [first, second] = &captured.opportunity;
        let Some(carried) = &first.carried else {
            return Err(data_failure(
                Some(FIRST_MAP),
                "early Map carry record locator",
            ));
        };
        if captured.format != "scoped-movement-map-expected-v1"
            || captured.row != ROW
            || captured.required_carried_owner_cut != FIRST_MAP
            || captured.second_consumer_cut != SECOND_MAP
            || first.end_cut != FIRST_MAP
            || second.end_cut != SECOND_MAP
            || second.carried.is_some()
            || carried.complete_movement_appearance_camera_expected_source_cut != FIRST_MAP
        {
            return Err(data_failure(
                None,
                "early Map expected opportunity alignment",
            ));
        }
        let record = record(cache, FIRST_MAP)?;
        let movement = movement::seed(&record.wram)?;
        let position = movement.position;
        if [position.x.words(), position.y.words()] != {
            let [x, y] = carried.late_whole;
            let [sub_x, sub_y] = carried.late_fraction;
            [(x, sub_x), (y, sub_y)]
        } {
            return Err(data_failure(
                Some(FIRST_MAP),
                "early Map independent carry alignment",
            ));
        }
        let (state, playing) = appearance::running_animation(&record.wram, &content.appearance)?;
        let camera = camera::seed(&record.wram, &camera::Content)?;
        let carry = carry(CompletedOwners {
            movement: &movement,
            appearance: state,
            playing,
            camera,
        });
        Ok(Self { captured, carry })
    }

    pub fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        let each = self
            .captured
            .opportunity
            .get(index(cut)?)
            .ok_or_else(|| data_failure(Some(cut), "early Map comparison opportunity"))?;
        let [x, y] = each.early_whole;
        let [sub_x, sub_y] = each.early_fraction;
        let position = [
            types::fixed(Wrapping(x), sub_x).0,
            types::fixed(Wrapping(y), sub_y).0,
        ];
        let explore_hex = each.explored.as_bytes();
        if explore_hex.len() != 512 {
            return Err(data_failure(Some(cut), "early Map explored extent"));
        }
        let explored = explore_hex
            .as_chunks::<2>()
            .0
            .iter()
            .map(|digits| {
                let hex = core::str::from_utf8(digits).ok()?;
                u8::from_str_radix(hex, 16).ok()
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| data_failure(Some(cut), "early Map bitmap encoding"))?;
        let mut fields = Vec::from(map_observations(
            position,
            explored,
            each.map_words
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect(),
        ));
        if cut == FIRST_MAP {
            fields.extend(self.carry.iter().cloned());
        }
        Ok(ExpectedCut {
            semantic_count: fields.len(),
            semantics: fields.clone(),
            fields,
            output_count: 0,
            replies: 0,
        })
    }
}

fn map_observations(
    position: [u32; 2],
    explored: Vec<u8>,
    window: Vec<u8>,
) -> [Observation; EARLY_FIELDS] {
    let [x, y] = position;
    [
        Observation {
            field: "hud.explored".to_owned(),
            bytes: explored,
        },
        Observation {
            field: "hud.map-window".to_owned(),
            bytes: window,
        },
        Observation::quantity("movement.position.x", i64::from(x)),
        Observation::quantity("movement.position.y", i64::from(y)),
    ]
}

/// Her animation's four quantities, which the completed carry compares. The cannon delay and
/// the colour sequences have no producer in this loop, and her kept action can be decoded only
/// from its movement type, so the rest of her appearance State is carried, not compared.
struct AnimationPin;
impl canonical::model::Pinned for AnimationPin {
    type State = samus_appearance::Animation;
    const NAME: &'static str = "samus-appearance-animation";
    const SHAPE: u64 = 0xF21C_717E_58BA_7D4D;
}

struct CompletedOwners<'a> {
    movement: &'a samus_movement::Samus,
    appearance: samus_appearance::Animation,
    playing: samus_appearance::Figure,
    camera: world_camera::Camera,
}

fn carry(
    CompletedOwners {
        movement,
        appearance,
        playing,
        camera,
    }: CompletedOwners<'_>,
) -> Vec<Observation> {
    use canonical::{model::Model, to_bytes};
    let movement = connected::state_observation(movement).bytes;
    let appearance = to_bytes(&vec![
        to_bytes(&Model::<AnimationPin>(appearance)),
        to_bytes(&Model::<production::PlayingPin>(playing)),
    ]);
    let camera = to_bytes(&Model::<game::state::owned::CameraPin>(camera));
    [
        ("movement.ordinary-completion", movement),
        ("appearance.ordinary-completion", appearance),
        ("camera.ordinary-completion", camera),
    ]
    .into_iter()
    .map(|(field, bytes)| Observation {
        field: field.to_owned(),
        bytes,
    })
    .collect()
}

pub(super) fn actual(
    production: &Production,
    cut: Option<u64>,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Connected(state) = production.state() else {
        return Err(data_failure(None, "early Map production scope"));
    };
    let (hud, position) = state
        .early_map()
        .ok_or_else(|| data_failure(None, "early Map owner"))?;
    let position = match (position, cut) {
        (Some(position), _) => position,
        (None, None) => [state.movement.0.position.x.0, state.movement.0.position.y.0],
        (None, Some(cut)) => return Err(data_failure(Some(cut), "early Map publication")),
    };
    let grid = game::production::hud::tilemap(hud)
        .map_err(|_| data_failure(cut, "early Map tile encoding"))?;
    let window = grid
        .as_chunks::<64>()
        .0
        .iter()
        .flat_map(|row| row.iter().skip(52).take(10).copied())
        .collect();
    let mut fields = Vec::from(map_observations(
        position,
        hud::explored_bitmap(hud),
        window,
    ));
    // The second map frame's owners have no independent capture, so only its map is compared.
    if cut != Some(SECOND_MAP) {
        fields.extend(carry(CompletedOwners {
            movement: &state.movement.0,
            appearance: state.appearance.0.animation,
            playing: state.playing.0,
            camera: state.camera.0,
        }));
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_compared_animation_keeps_its_shape() {
        assert_eq!(canonical::model::pin_holds::<AnimationPin>(), Ok(()));
    }

    #[test]
    #[ignore = "requires the pinned record cache, ROM and early Map fixture"]
    fn early_map_restores_between_frames_and_refuses_a_missing_map() {
        let sources = Sources {
            cache: std::env::var_os("SM_CACHE").unwrap().into(),
            rom: std::env::var_os("SM_ROM").unwrap().into(),
            native: std::env::var_os("SM_CONNECTED_SOURCE").unwrap().into(),
            lock: std::env::var_os("SM_LOCK").unwrap().into(),
            ..Default::default()
        };
        let index = super::super::index::Index::parse(include_str!(
            "../../../../scenarios/replay/index.toml"
        ))
        .unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.inputs == super::super::index::InputProvider::EarlyMapInputs)
            .unwrap();
        let AdmittedCase {
            production,
            content,
            rules,
            inputs,
            expected,
            ..
        } = admit(case, &sources).expect("admit reviewed early Map");
        let InputReader::EarlyMap(inputs) = inputs else {
            panic!("early Map input tape")
        };
        let ExpectedReader::EarlyMap(expected) = expected else {
            panic!("early Map expectations")
        };
        let restore = |state: &Production| {
            Production::restore(&state.save().unwrap(), content.clone(), rules.clone()).unwrap()
        };
        let mut direct = production.clone();
        let mut restored = restore(&production);
        let AdvanceInput::ConnectedMap {
            inputs: current, ..
        } = inputs.input(FIRST_MAP).unwrap()
        else {
            panic!("early Map advance")
        };
        let before = restored.clone();
        assert!(matches!(
            restored.advance(AdvanceInput::Connected { inputs: current }),
            Err(game::production::ProductionFault::Connected(
                production::Fault::HudMap
            )),
        ));
        assert!(PartialEq::eq(&restored, &before));

        for cut in [FIRST_MAP, SECOND_MAP] {
            inputs.advance(&mut direct, cut).unwrap();
            inputs.advance(&mut restored, cut).unwrap();
            restored = restore(&restored);
            let check = expected.read(cut).unwrap();
            let direct_fields = actual(&direct, Some(cut)).unwrap();
            let restored_fields = actual(&restored, Some(cut)).unwrap();
            assert!(super::super::compare::compare(&check.fields, &direct_fields).is_ok());
            assert!(super::super::compare::compare(&check.fields, &restored_fields).is_ok());
            assert!(PartialEq::eq(&direct, &restored));
        }
    }
}
