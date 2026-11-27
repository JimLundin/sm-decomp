//! Movement and consumer stimuli, admitted independently of the later touchdown observations.

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use audio_transport::{CueId, OfferPermission, SenderProgress};
use canonical::model::Model;
use game::production::{
    AdvanceInput, MovementInputs, Production, ProductionContent, ProductionOutputs,
    ProductionRules, ProductionSeed,
    appearance::TouchdownArea,
    movement_touchdown::{Input, State},
    sender::OfferCurrent,
    world::MovementWorld,
};
use samus_appearance::Figure;
use serde::Deserialize;
use std::num::Wrapping;
use types::{Action, Buttons, Facing, Hitbox, Pair, Sprite, button};

use super::{
    Sources, data_failure,
    index::{Case, FixtureFile, Start},
    movement, production_failure,
    readers::{ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record},
    result::Outcome,
    sender_offer,
};
use crate::{
    cache::Cache,
    lock::{Lock, RowId},
    source,
};

mod observations;
#[cfg(test)]
mod tests;
pub(super) use observations::{actual, initial};

#[derive(Deserialize)]
struct Fixture<T> {
    schema: u8,
    row: String,
    window: Vec<T>,
}

impl<T: serde::de::DeserializeOwned> Fixture<T> {
    fn window(
        path: &Path,
        file: &FixtureFile,
        row: &str,
        cut: u64,
        at: impl Fn(&T) -> u64,
    ) -> Result<T, Outcome> {
        let fixture: Self = file.read(path)?;
        if fixture.schema != 1 || fixture.row != row {
            return Err(data_failure(None, "touchdown fixture origin"));
        }
        let mut windows = fixture
            .window
            .into_iter()
            .filter(|window| at(window) == cut);
        match (windows.next(), windows.next()) {
            (Some(window), None) => Ok(window),
            _ => Err(data_failure(Some(cut), "touchdown fixture window")),
        }
    }
}

#[derive(Deserialize)]
struct Borrowed {
    cut: u64,
    movement_seed_cut: u64,
    room_terrain_source_cut: u64,
    held: Vec<Control>,
    pressed: Vec<Control>,
    facing: Facing,
    current_action: ObservedAction,
    previous_action: ObservedAction,
    prior_selected_sprite: Sprite,
    movement_entry_x_fp16: u32,
    movement_entry_y_fp16: u32,
    movement_entry_downward_speed_fp16: u32,
    cinematic_active: bool,
    contact_offer_permission: String,
    contact_seed_queue: VecDeque<CueId>,
    contact_seed_progress: SenderProgress,
    contact_seed_master_ticks: u64,
    current_solid_actors: [SolidActor; 2],
}

#[derive(Deserialize)]
enum Control {
    Run,
    Left,
    Right,
}

fn controls(controls: Vec<Control>) -> Buttons {
    controls
        .into_iter()
        .fold(Buttons::default(), |held, control| {
            let button = match control {
                Control::Run => button::RUN,
                Control::Left => button::LEFT,
                Control::Right => button::RIGHT,
            };
            Buttons(held.0 | button.0)
        })
}

#[derive(Deserialize)]
enum ObservedAction {
    NormalJumping,
}

impl ObservedAction {
    const fn action(self) -> Action {
        match self {
            Self::NormalJumping => Action::Jumping,
        }
    }
}

#[derive(Deserialize)]
struct SolidActor {
    roster_slot: u16,
    centre_px: [u16; 2],
    radius_px: [u16; 2],
}

impl SolidActor {
    const fn geometry(self) -> Hitbox {
        let [x, y] = self.centre_px;
        let [rx, ry] = self.radius_px;
        Hitbox {
            centre: Pair {
                x: Wrapping(x),
                y: Wrapping(y),
            },
            radius: Pair {
                x: Wrapping(rx),
                y: Wrapping(ry),
            },
        }
    }
}

pub(super) struct Inputs {
    cut: u64,
    movement: MovementInputs,
    world: MovementWorld,
    suppression: OfferCurrent,
}

impl Inputs {
    pub(super) fn advance(
        &self,
        owner: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        if cut != self.cut {
            return Err(data_failure(Some(cut), "touchdown opportunity schedule"));
        }
        owner
            .advance(AdvanceInput::MovementTouchdown(Input {
                movement: self.movement,
                world: &self.world,
                suppression: self.suppression,
                audio: sender_offer::offers_only(),
                graphics_area: TouchdownArea::DryClear,
            }))
            .map_err(|fault| production_failure(cut, fault))
    }
}

pub(super) struct Expected {
    path: PathBuf,
    observations: FixtureFile,
    sender: sender_offer::SenderPacket,
    cut: u64,
    source_id: usize,
}

impl Expected {
    pub(super) fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        if cut != self.cut {
            return Err(data_failure(Some(cut), "touchdown comparison schedule"));
        }
        let captured = Fixture::<observations::Captured>::window(
            &self.path,
            &self.observations,
            &self.sender.row,
            cut,
            |window| window.cut,
        )?;
        let fifo = sender_offer::contact_fifo(&self.path, &self.sender, self.source_id, cut)?;
        captured.compare_with(&fifo)
    }
}

fn seed_record(
    sources: &Sources,
    row: &str,
    cut: u64,
) -> Result<oracle_record::FrameRecord, Outcome> {
    let record_lock = std::fs::read_to_string(&sources.lock)
        .map_err(|_| data_failure(None, "record lock file"))?;
    let record_lock =
        Lock::parse(&record_lock).map_err(|_| data_failure(None, "record lock schema"))?;
    let row = record_lock
        .row(&RowId(row.to_owned()))
        .ok_or_else(|| data_failure(None, "touchdown locked row"))?;
    let cache =
        Cache::open(&sources.cache, row).map_err(|_| data_failure(None, "touchdown cache pins"))?;
    record(&cache, cut)
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "touchdown seed cut"));
    };
    let cut = case.end.cut_inclusive;
    let before = seed_record(sources, &case.row, start)?;
    let wram = &before.wram;
    let files = case.fixture_pair()?;
    let sender = case.sender_packet()?;
    let observed =
        Fixture::<Borrowed>::window(&sources.native, &files.inputs, &case.row, cut, |window| {
            window.cut
        })?;
    let content = movement::Content::load(&sources.rom)?;
    let samus = movement::seed(wram)?;
    let preceding = movement::input(wram, samus.pose, &content)?;
    let prior_pose = source::SAMUS_PREV_POSE
        .read(wram)
        .ok()
        .and_then(|id| data::assets::samus::map::key_of(data::pose::PoseId(id)));
    let owned_contact = sender_offer::contact_ingress(
        &sources.native,
        sender,
        cut,
        observed.contact_seed_master_ticks,
    )?;
    let [first, second] = observed.current_solid_actors;
    if observed.movement_seed_cut != start
        || observed.room_terrain_source_cut != start
        || samus.position.x.0 != observed.movement_entry_x_fp16
        || samus.position.y.0 != observed.movement_entry_y_fp16
        || samus.vertical.speed.0 != observed.movement_entry_downward_speed_fp16
        || samus.pose.action != observed.current_action.action()
        || prior_pose.is_none_or(|pose| pose.action != observed.previous_action.action())
        || samus.pose.facing != observed.facing
        || preceding.sprite != observed.prior_selected_sprite
        || preceding.cinematic != observed.cinematic_active
        || preceding.pad != controls(observed.held)
        || preceding.pressed != controls(observed.pressed)
        || first.roster_slot >= 32
        || second.roster_slot >= 32
        || first.roster_slot == second.roster_slot
        || owned_contact.seed.queue.cues() != &observed.contact_seed_queue
        || owned_contact.seed.progress != observed.contact_seed_progress
        || observed.contact_offer_permission != "Enabled"
        || owned_contact.current.transport_permission() != OfferPermission::Enabled
    {
        return Err(data_failure(Some(cut), "touchdown live starting premises"));
    }
    let world = movement::level_world(wram, &content, vec![first.geometry(), second.geometry()])?;
    let inputs = Inputs {
        cut,
        movement: preceding.stimuli(),
        world,
        suppression: owned_contact.current,
    };
    let state = State {
        movement: Model(samus),
        figure: Model(Figure {
            pose: samus.pose,
            sprite: preceding.sprite,
        }),
        audio: Model(Box::new(sender_offer::silent_transport(
            audio_transport::SoundGroup::Contact,
            owned_contact.seed,
        ))),
    };
    let content = ProductionContent::MovementTouchdown(
        content.movement,
        Box::new(game::production::audio::Content::silent()),
    );
    let rules = ProductionRules::MovementTouchdown(samus_movement::Rules::default());
    let production = Production::admit(
        ProductionSeed::MovementTouchdown(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(start), "touchdown production admission"))?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::MovementTouchdown(Box::new(inputs)),
        expected: ExpectedReader::MovementTouchdown(Expected {
            path: sources.native.clone(),
            observations: files.expected.clone(),
            sender: sender.clone(),
            cut,
            source_id: owned_contact.source_id,
        }),
        actual: ActualReader::MovementTouchdown(None),
        schedule: vec![cut],
    })
}
