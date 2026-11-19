//! Concrete input and output boundaries for existing clean production entries.

use super::{
    AdmissionFault, AdvanceInput, ProductionContent, ProductionFault, ProductionOutputs,
    ProductionRules, ProductionSeed,
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
            ProductionSeed::AudioSender(state),
            ProductionContent::AudioSender(group),
            ProductionRules::AudioSender,
        ) => {
            super::sender::validate(&state, *group)?;
            OwnedState::AudioSender(Model(state))
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
        _ => return Err(AdmissionFault::WrongScope),
    })
}

pub(super) fn advance(
    state: &mut OwnedState,
    content: &ProductionContent,
    rules: &ProductionRules,
    input: AdvanceInput<'_>,
) -> Result<ProductionOutputs, ProductionFault> {
    Ok(match (state, content, rules, input) {
        (
            OwnedState::AudioSender(state),
            ProductionContent::AudioSender(group),
            ProductionRules::AudioSender,
            AdvanceInput::AudioSender(input),
        ) => ProductionOutputs::AudioSender(super::sender::advance(&mut state.0, *group, input)?),
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
            AdvanceInput::Appearance(inputs),
        ) => ProductionOutputs::Appearance(
            super::animation::advance(&mut state.0, content, rules, inputs)
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
        ) => ProductionOutputs::Hud(advance_hud(state, rules.get(), pictures, input)),
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
) -> Vec<interface_hud::HudEvent> {
    match input {
        HudInput::Play(inputs) => state.frame_of_play(rules, pictures, inputs),
        HudInput::OutOfPlay(inputs) => {
            state.out_of_play(inputs);
            Vec::new()
        }
        HudInput::Boss { boss, room_cell } => {
            state.boss_blanks_the_map(rules, pictures, boss, room_cell);
            Vec::new()
        }
    }
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
