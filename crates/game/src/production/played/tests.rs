//! The State every owner's tests start from, and the frame they play it through.

use super::*;
use crate::production::animation::AppearanceContent;
use crate::production::connected::inputs::{CameraInputs, Lead, Screen};
use crate::production::content::MovementContent;
use crate::production::room_loading::Rooms;
use crate::production::tests::{empty_world, fixtures};
use crate::production::{FigureClip, PoseContent};
use boss_ceres_ridley::{CeresDoor, Stage};
use core::num::Wrapping;
use interface_hud as hud;
use interface_hud::{
    Ammunition, CancelButton, CountdownRuns, FrameCount, HerMap, HudPictures, Reserve,
    SelectButton, StockAtStart,
};
use types::{Pair, Set};
use world_camera::{RememberedPosition, ViewPosition};

/// Her figure in the fixture's pose.
pub(super) fn figure() -> Figure {
    Figure {
        pose: fixtures::samus().pose,
        sprite: types::Sprite::Plain,
    }
}

pub(super) fn state(doors: Vec<CeresDoor>) -> State {
    let her = fixtures::samus().position;
    State {
        samus: samus::State {
            movement: Model(fixtures::samus()),
            appearance: Model(samus_appearance::AppearanceState::INITIAL),
            figure: Model(figure()),
        },
        camera: Model(world_camera::Camera::restore(
            ViewPosition(Pair {
                x: Wrapping(0),
                y: Wrapping(0),
            }),
            RememberedPosition(her),
            types::VerticalFraction(0),
        )),
        passage: passage::State {
            doors: Doors::Play,
            doorway: None,
            room: passage::Room {
                geometry: empty_world(),
                doorways: Vec::new(),
                camera: inputs(types::Buttons::default()).camera,
            },
            // A room no block across, so the mini-map never reads her map.
            map_room: hud::Room {
                cell: Pair { x: 0, y: 0 },
                blocks: Pair { x: 0, y: 0 },
            },
        },
        palette: Palette {
            screen: [types::Colour::BLACK; COLOURS],
            goal: [types::Colour::BLACK; COLOURS],
        },
        health: Model(samus_health::State {
            energy: samus_health::EnergyAmount {
                whole: samus_health::WholeEnergy::new(99).unwrap(),
                fraction: 0,
            },
            pending_loss: samus_health::EnergyLoss {
                whole: samus_health::WholeEnergy::ZERO,
                fraction: 0,
            },
            warning: samus_health::Warning::Off,
            protection: samus_health::ProtectionFrames::ZERO,
        }),
        hud: Model(hud::HudState {
            countdown: hud::Countdown::Dormant,
            grid: hud::HudGrid::filled(hud::Tile::new(
                types::Look::default(),
                hud::ColourSet::new(0).unwrap(),
            )),
            shown: hud::Shown::default(),
            selection: hud::Selection::None,
            explored: hud::MapCellSet::default(),
            mini_map: hud::MiniMapPower::On,
        }),
        station: station::State {
            doors,
            pad_and_cover: crate::production::shaft::GONE,
            shaft_overlay: false,
            ridley: None,
            stage: Stage::Calm,
        },
        frame: frame::State {
            count: FrameCount(0),
            finished_frames: 0,
            random: types::RandomSequence {
                number: types::RandomNumber(0),
            },
        },
        arrival: arrival::State::PLAYING,
    }
}

pub(super) fn inputs(held: types::Buttons) -> Inputs {
    Inputs {
        held,
        pressed: held,
        items: Set::default(),
        parity: types::Parity::Even,
        energy: 99,
        liquid: samus_appearance::Liquid::NONE,
        sounds_waiting: samus_appearance::SoundsWaiting::new(0).unwrap(),
        activities: Set::default(),
        charge: samus_appearance::Charge::try_from(0).unwrap(),
        geometry: empty_world(),
        camera: CameraInputs {
            screens: vec![Screen::Open; 4],
            size: [2, 2],
            heights: [112, 128],
            lead: Lead::Ahead,
        },
        activity: crate::production::connected::inputs::Activity::Inactive,
    }
}

pub(super) fn hud_frame() -> HudFrame {
    HudFrame {
        buttons: (SelectButton::NotPressed, CancelButton::NotPressed),
        stock_at_start: StockAtStart {
            counts: enum_map::EnumMap::from_fn(|_| 0),
            equipped: hud::Equipped::default(),
        },
        capacity: 99,
        reserve: Reserve {
            energy: 0,
            mode: hud::ReserveMode::Manual,
        },
        ammunition: Ammunition::new(enum_map::EnumMap::from_fn(|_| hud::Stock::default())).unwrap(),
        map_collected: false,
        countdown: CountdownRuns::MaintainCountdown,
    }
}

pub(super) fn content() -> Content {
    content_with(&data::assets::samus::palettes::SamusPalettes::default())
}

/// The fixture's content, with her pulse colours taken from `palettes`.
pub(super) fn content_with(palettes: &data::assets::samus::palettes::SamusPalettes) -> Content {
    let poses = data::assets::samus::map::MAP.iter().map(|entry| entry.key);
    Content {
        movement: MovementContent::new(
            poses
                .clone()
                .map(|pose| PoseContent {
                    pose,
                    radius: Pair {
                        x: Wrapping(5),
                        y: Wrapping(21),
                    },
                    cannon_heading: None,
                })
                .collect(),
        )
        .unwrap(),
        appearance: AppearanceContent::new(
            poses
                .map(|pose| FigureClip {
                    figure: Figure {
                        pose,
                        sprite: types::Sprite::Plain,
                    },
                    clip: samus_appearance::Clip {
                        instructions: vec![samus_appearance::Instruction::Show(
                            samus_appearance::Hold::new(4).unwrap(),
                        )],
                        kick: None,
                        fast_fall: None,
                        rising: None,
                        contacts: std::collections::BTreeSet::new(),
                    },
                })
                .collect(),
            samus_appearance::Paces {
                plain: samus_appearance::Pace(Vec::new()),
                boosted: enum_map::EnumMap::from_fn(|_| samus_appearance::Pace(Vec::new())),
            },
            crate::production::pulse_colours(palettes).unwrap(),
        )
        .unwrap(),
        colour_groups: [ColourGroup::Other; COLOURS],
        rooms: Rooms {
            world: data::assets::world::World::default(),
            tilesets: data::assets::tilesets::Tilesets::default(),
            actors: data::assets::actors::Actors::default(),
            samus: data::assets::samus::palettes::SamusPalettes::default(),
            slopes: data::level::slope::Slopes::default(),
        },
        hud: HudPictures::default(),
        map: HerMap::default(),
        start_x: Wrapping(0x80),
        ridley: boss_ceres_ridley::RidleyContent {
            eye_steps: boss_ceres_ridley::FadeSets::new(vec![boss_ceres_ridley::EyeColours(
                [types::Colour::BLACK; 3],
            )])
            .unwrap(),
            body_sets: boss_ceres_ridley::FadeSets::new(vec![boss_ceres_ridley::BodyColours(
                [types::Colour::BLACK; 11],
            )])
            .unwrap(),
        },
    }
}

pub(super) fn rules() -> Rules {
    Rules {
        movement: samus_movement::Rules::default(),
        appearance: samus_appearance::Rules::VANILLA,
        camera: CameraRules::new(world_camera::Rules::REFERENCE),
        doors: world_doors::Rules::default(),
        health: samus_health::Rules::REFERENCE,
        hud: interface_hud::Rules::default(),
        ceres: boss_ceres_ridley::Rules::default(),
        arrival: screens_arrival::Rules::REFERENCE,
    }
}

pub(super) fn play(
    state: &mut State,
    held: types::Buttons,
    ceres: Stage,
) -> Result<Outputs, Fault> {
    play_with(state, &content(), held, ceres)
}

pub(super) fn play_with(
    state: &mut State,
    content: &Content,
    held: types::Buttons,
    ceres: Stage,
) -> Result<Outputs, Fault> {
    let progress = run_progress::Progress::default();
    let passing = Passing {
        work_begun: true,
        work_done: true,
        audio: passage::PassageAudio {
            sounds_sent: true,
            music_ready: true,
        },
        timer_running: false,
        progress: &progress,
        random: None,
        start: arrival::Readiness::NotYet,
    };
    state.station.stage = ceres;
    state.step(&inputs(held), &passing, &hud_frame(), content, &rules())
}
