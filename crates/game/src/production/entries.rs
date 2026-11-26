//! Concrete input and output boundaries for existing clean production entries.

use super::{
    AdmissionFault, AdvanceInput, ProductionContent, ProductionFault, ProductionOutputs,
    ProductionRules, ProductionSeed, Unsupported,
};
use crate::state::owned::OwnedState;
use canonical::model::Model;

/// Complete passage state, including the scene the passage retains and moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PassageState {
    /// Play or the complete retained passage phase.
    pub doors: world_doors::Doors,
    /// Carried player/view positions and all current and target colours.
    pub scene: world_doors::Scene,
}

/// One independently admitted block entry.
#[derive(Clone, Copy)]
pub enum BlocksInput<'a> {
    /// Exact ordered room placements from imported room content.
    StartRoom(&'a [world_blocks::Placement]),
    /// Actual ordered collision requests and the selected run permission.
    Frame {
        /// Requests from the selected producer or admitted causal source.
        requests: &'a [world_blocks::Request],
        /// Actual movement modes for this frame.
        moving: types::Set<types::Motion>,
        /// Whether this frame runs the scripts.
        run: world_blocks::Run,
    },
}

/// Complete block publications; state remains the clean model's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlocksOutputs {
    /// Every admitted placement was applied successfully.
    RoomStarted,
    /// Every ordered answer, tile/screen write and sound request from the clean step.
    Frame(world_blocks::Report),
}

/// A selected clean HUD entry; ordinary play runs only once.
#[derive(Clone, Copy)]
pub enum HudInput<'a> {
    /// Every physical controller, progress, map, frame and drop input for play.
    Play(&'a interface_hud::FrameInputs<'a>),
    /// An actual passage/load/cinematic input from its selected producer or causal source.
    OutOfPlay(&'a interface_hud::OutOfPlayInputs<'a>),
    /// A boss met on entering its room.
    Boss {
        /// The admitted boss kind.
        boss: interface_hud::Boss,
        /// Its room's mini-map origin.
        room_cell: interface_hud::Place,
    },
}

pub(super) fn admit(
    seed: ProductionSeed,
    content: &ProductionContent,
    rules: &ProductionRules,
) -> Result<OwnedState, AdmissionFault> {
    Ok(match (seed, content, rules) {
        (
            ProductionSeed::Departure(run),
            ProductionContent::Departure(content),
            ProductionRules::Departure(_),
        ) => {
            run.validate(content).map_err(AdmissionFault::Departure)?;
            OwnedState::Departure(run)
        }
        (
            ProductionSeed::Explosion(state),
            ProductionContent::Explosion(content),
            ProductionRules::Explosion(_),
        ) => {
            state.validate(content).map_err(AdmissionFault::Explosion)?;
            OwnedState::Explosion(state)
        }
        (
            ProductionSeed::Escape(state),
            ProductionContent::Escape(content),
            ProductionRules::Departure(_),
        ) => {
            state.validate(content)?;
            OwnedState::Escape(state)
        }
        (
            ProductionSeed::Blackout(state),
            ProductionContent::Blackout(_),
            ProductionRules::Blackout,
        ) => {
            state.validate().map_err(AdmissionFault::Blackout)?;
            OwnedState::Blackout(state)
        }
        (seed, content, rules) => admit_standalone(seed, content, rules)?,
    })
}

fn admit_standalone(
    seed: ProductionSeed,
    content: &ProductionContent,
    rules: &ProductionRules,
) -> Result<OwnedState, AdmissionFault> {
    Ok(match (seed, content, rules) {
        (ProductionSeed::Intro(state), ProductionContent::Intro(_), ProductionRules::Intro(_)) => {
            OwnedState::Intro(state)
        }
        (
            ProductionSeed::IntroRooms(state),
            ProductionContent::IntroRooms(content),
            ProductionRules::IntroRooms(_),
        ) => {
            content.movement.validate()?;
            OwnedState::IntroRooms(state)
        }
        (
            ProductionSeed::Discovery(state),
            ProductionContent::Discovery(_),
            ProductionRules::Discovery,
        ) => OwnedState::Discovery(Model(state)),
        (
            ProductionSeed::LandingSite(state),
            ProductionContent::LandingSite(_),
            ProductionRules::LandingSite,
        ) => OwnedState::LandingSite(state),
        (
            ProductionSeed::Zebes(state),
            ProductionContent::Zebes(content),
            ProductionRules::Zebes(_),
        ) => {
            state.validate(content).map_err(AdmissionFault::Zebes)?;
            OwnedState::Zebes(state)
        }
        (
            ProductionSeed::HealthTail(state),
            ProductionContent::HealthTail(content),
            ProductionRules::HealthTail(_),
        ) => {
            state.validate(content)?;
            OwnedState::HealthTail(state)
        }
        (
            ProductionSeed::Audio(state),
            ProductionContent::Audio(content),
            ProductionRules::Audio,
        ) => {
            super::audio::validate(&state, content)?;
            OwnedState::Audio(Model(state))
        }
        (ProductionSeed::Blocks(state), ProductionContent::Blocks, ProductionRules::Blocks(_)) => {
            OwnedState::Blocks(state)
        }
        (ProductionSeed::Camera(state), ProductionContent::Camera, ProductionRules::Camera(_)) => {
            OwnedState::Camera(Model(state))
        }
        (
            ProductionSeed::Appearance(state),
            ProductionContent::Appearance(_),
            ProductionRules::Appearance(_),
        ) => OwnedState::Appearance(Model(state)),
        (
            ProductionSeed::Progress(state),
            ProductionContent::Progress,
            ProductionRules::Progress(_),
        ) => OwnedState::Progress(state),
        (ProductionSeed::Hud(state), ProductionContent::Hud(_), ProductionRules::Hud(_)) => {
            OwnedState::Hud(state)
        }
        (ProductionSeed::Ceres(state), ProductionContent::Ceres, ProductionRules::Ceres(_)) => {
            OwnedState::Ceres(state)
        }
        (ProductionSeed::Doors(state), ProductionContent::Doors, ProductionRules::Doors(_)) => {
            OwnedState::Doors(state)
        }
        (seed, content, rules) => admit_room(seed, content, rules)?,
    })
}

fn admit_room(
    seed: ProductionSeed,
    content: &ProductionContent,
    rules: &ProductionRules,
) -> Result<OwnedState, AdmissionFault> {
    Ok(match (seed, content, rules) {
        (
            ProductionSeed::ShaftEntry(state),
            ProductionContent::ShaftEntry(_),
            ProductionRules::ShaftEntry,
        ) => {
            if !state.valid() {
                return Err(AdmissionFault::ShaftLevel);
            }
            OwnedState::ShaftEntry(state)
        }
        (
            ProductionSeed::JumpHistory(state),
            ProductionContent::JumpHistory,
            ProductionRules::JumpHistory,
        ) => OwnedState::JumpHistory(Model(state)),
        _ => return Err(AdmissionFault::WrongScope),
    })
}

pub(super) fn advance_scene(
    state: &mut OwnedState,
    content: &ProductionContent,
    rules: &ProductionRules,
    input: AdvanceInput<'_>,
) -> Result<ProductionOutputs, ProductionFault> {
    Ok(match (state, content, rules, input) {
        (
            OwnedState::Explosion(state),
            ProductionContent::Explosion(content),
            ProductionRules::Explosion(rules),
            AdvanceInput::Explosion(cut),
        ) => ProductionOutputs::Explosion(Box::new(state.advance(content, *rules, cut).map_err(
            |fault| ProductionFault::Unsupported(super::Unsupported::Explosion(fault)),
        )?)),
        (
            OwnedState::Escape(state),
            ProductionContent::Escape(content),
            ProductionRules::Departure(rules),
            AdvanceInput::Departure(input),
        ) => state.advance(content, *rules, input)?,
        (
            OwnedState::IntroMissiles(state),
            ProductionContent::IntroMissiles(content),
            ProductionRules::IntroMissiles(rules),
            AdvanceInput::IntroMissiles(input),
        ) => ProductionOutputs::IntroMissiles(Box::new(super::intro_missile_scope::advance(
            state, input, content, *rules,
        )?)),
        (
            OwnedState::Intro(state),
            ProductionContent::Intro(content),
            ProductionRules::Intro(rules),
            AdvanceInput::Intro(frame),
        ) => ProductionOutputs::Intro(
            super::played::intro::played(state, frame, content, *rules)
                .map_err(|fault| ProductionFault::Unsupported(Unsupported::Intro(fault)))?
                .map(Box::new),
        ),
        (
            OwnedState::Discovery(state),
            ProductionContent::Discovery(art),
            ProductionRules::Discovery,
            AdvanceInput::Discovery(cut),
        ) => ProductionOutputs::Discovery(art.scoped(&mut state.0, cut)),
        (
            OwnedState::Zebes(state),
            ProductionContent::Zebes(content),
            ProductionRules::Zebes(rules),
            AdvanceInput::Zebes(cut),
        ) => {
            ProductionOutputs::Zebes(Box::new(state.advance(content, *rules, cut).map_err(
                |fault| ProductionFault::Unsupported(super::Unsupported::Zebes(fault)),
            )?))
        }
        (
            OwnedState::LandingSite(state),
            ProductionContent::LandingSite(content),
            ProductionRules::LandingSite,
            AdvanceInput::LandingSite(work),
        ) => {
            state
                .advance(content, work)
                .map_err(|fault| ProductionFault::Unsupported(Unsupported::LandingSite(fault)))?;
            ProductionOutputs::LandingSite
        }
        (
            OwnedState::Departure(state),
            ProductionContent::Departure(content),
            ProductionRules::Departure(rules),
            AdvanceInput::Departure(input),
        ) => state
            .advance(content, *rules, input)
            .map_err(|fault| ProductionFault::Unsupported(Unsupported::Departure(fault)))?,
        (
            OwnedState::ShaftEntry(state),
            ProductionContent::ShaftEntry(setup),
            ProductionRules::ShaftEntry,
            AdvanceInput::ShaftEntry(opportunity),
        ) => {
            state
                .advance(*setup, opportunity)
                .map_err(ProductionFault::RoomLoad)?;
            ProductionOutputs::ShaftEntry
        }
        (state, content, rules, input) => advance(state, content, rules, input)?,
    })
}

fn advance(
    state: &mut OwnedState,
    content: &ProductionContent,
    rules: &ProductionRules,
    input: AdvanceInput<'_>,
) -> Result<ProductionOutputs, ProductionFault> {
    Ok(match (state, content, rules, input) {
        (
            OwnedState::JumpHistory(state),
            ProductionContent::JumpHistory,
            ProductionRules::JumpHistory,
            AdvanceInput::JumpHistory(sample),
        ) => {
            super::jump_history::advance(&mut state.0, sample);
            ProductionOutputs::JumpHistory
        }
        (
            OwnedState::Blackout(state),
            ProductionContent::Blackout(content),
            ProductionRules::Blackout,
            AdvanceInput::Blackout(cut),
        ) => state.output(content, cut)?,
        (
            OwnedState::HealthTail(state),
            ProductionContent::HealthTail(content),
            ProductionRules::HealthTail(rules),
            AdvanceInput::HealthTail(input),
        ) => ProductionOutputs::HealthTail(state.advance(content, *rules, input)?),
        (
            OwnedState::WeaponsContact(state),
            ProductionContent::WeaponsContact(content),
            ProductionRules::WeaponsContact(rules),
            AdvanceInput::WeaponsContactDrawing { shot, drawing },
        ) => ProductionOutputs::WeaponsContactDrawing(Box::new(
            super::weapons::advance_with_drawing(&mut state.0, shot, content, *rules, drawing)?,
        )),

        (
            OwnedState::Blocks(state),
            ProductionContent::Blocks,
            ProductionRules::Blocks(rules),
            AdvanceInput::Blocks(input),
        ) => ProductionOutputs::Blocks(advance_blocks(state, rules.get(), input)?),
        (
            OwnedState::Camera(state),
            ProductionContent::Camera,
            ProductionRules::Camera(rules),
            AdvanceInput::Camera(frame),
        ) => {
            super::camera_semantic::step(&mut state.0, rules.get(), frame);
            ProductionOutputs::Camera(state.0)
        }
        (
            OwnedState::Appearance(state),
            ProductionContent::Appearance(content),
            ProductionRules::Appearance(rules),
            AdvanceInput::Appearance(frame),
        ) => ProductionOutputs::Appearance(
            super::animation::play(&mut state.0, content, rules, frame)
                .map_err(ProductionFault::Appearance)?,
        ),
        (
            OwnedState::Progress(state),
            ProductionContent::Progress,
            ProductionRules::Progress(rules),
            AdvanceInput::Progress(reports),
        ) => {
            state.apply(rules, reports);
            ProductionOutputs::Progress
        }
        (
            OwnedState::Hud(state),
            ProductionContent::Hud(pictures),
            ProductionRules::Hud(rules),
            AdvanceInput::Hud(input),
        ) => ProductionOutputs::Hud(advance_hud(state, rules.get(), pictures, input)?),
        (
            OwnedState::Ceres(state),
            ProductionContent::Ceres,
            ProductionRules::Ceres(rules),
            AdvanceInput::Ceres(inputs),
        ) => {
            let output = ceres_advance(state, inputs, rules);
            ProductionOutputs::Ceres(output)
        }
        (
            OwnedState::Doors(state),
            ProductionContent::Doors,
            ProductionRules::Doors(rules),
            AdvanceInput::Doors(inputs),
        ) => {
            let output = state.doors.step(state.scene, inputs, rules);
            state.doors = output.doors;
            state.scene = output.scene;
            ProductionOutputs::Doors(Box::new(output))
        }
        (
            OwnedState::Audio(state),
            ProductionContent::Audio(content),
            ProductionRules::Audio,
            AdvanceInput::Audio { offers, cadence },
        ) => {
            super::audio::play(&mut state.0, content, offers, cadence);
            ProductionOutputs::Audio
        }
        _ => return Err(ProductionFault::WrongScope),
    })
}

fn advance_blocks(
    state: &mut world_blocks::State,
    rules: &world_blocks::Rules,
    input: BlocksInput<'_>,
) -> Result<BlocksOutputs, ProductionFault> {
    Ok(match input {
        BlocksInput::StartRoom(placements) => {
            state
                .start_room(placements.iter().cloned())
                .map_err(ProductionFault::Blocks)?;
            BlocksOutputs::RoomStarted
        }
        BlocksInput::Frame {
            requests,
            moving,
            run,
        } => BlocksOutputs::Frame(
            state
                .step(rules, requests, moving, run)
                .map_err(ProductionFault::Blocks)?,
        ),
    })
}

fn advance_hud(
    state: &mut interface_hud::HudState,
    rules: &interface_hud::Rules,
    pictures: &interface_hud::HudPictures,
    input: HudInput<'_>,
) -> Result<Option<interface_hud::FrameOutputs>, ProductionFault> {
    Ok(match input {
        HudInput::Play(inputs) => Some(
            state
                .frame_of_play(rules, pictures, inputs)
                .map_err(|frame| ProductionFault::Unsupported(Unsupported::Hud(frame)))?,
        ),
        HudInput::OutOfPlay(inputs) => {
            state.out_of_play(inputs);
            None
        }
        HudInput::Boss { boss, room_cell } => {
            state.boss_blanks_the_map(rules, pictures, boss, room_cell);
            None
        }
    })
}

/// Shared complete Ceres adapter entry.
pub(super) fn ceres_advance(
    state: &mut boss_ceres_ridley::CeresDoor,
    inputs: &boss_ceres_ridley::Inputs,
    rules: &boss_ceres_ridley::Rules,
) -> boss_ceres_ridley::Run {
    let output = state.run(inputs, rules);
    *state = output.door;
    output
}
