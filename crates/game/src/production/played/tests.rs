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
        room_video: super::room_video::RoomVideo::default(),
        shaft_sound: super::shaft_sound::State::default(),
        play_time: crate::production::played::play_time::ZERO,
        clock_picture: super::clock_picture::ClockPicture::default(),
        room_tiles: super::room_tiles::ShaftBackground::Pending,
        samus: samus::State {
            last_draw_jump: Model(samus_movement::JumpButton::Up),
            travelled: Model(samus_movement::TravelledDistances::default()),
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
        progress: run_progress::Progress::default(),
        passage: passage::State {
            doors: Doors::Play,
            doorway: None,
            awaiting_actors: false,
            station: types::StationId(0),
            room: passage::Room {
                id: data::assets::world::rooms::CERES_ELEVATOR_ROOM,
                area: data::room::Area::Ceres,
                entered_through: None,
                tileset: None,
                entry: crate::production::shaft_entry::State::default(),
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
            slots: crate::production::room_loading::ActorSlots::default(),
            doors,
            vents: Vec::new(),
            pad_and_cover: crate::production::shaft::GONE,
            shaft_overlay: false,
            escape_shaft: None,
            ridley: None,
            warning: None,
            stage: Stage::Calm,
        },
        quake: Model(world_quake::State {
            phase: world_quake::QuakePhase(0),
            shake: None,
        }),
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
        countdown_starts: None,
    }
}

pub(super) fn content() -> Content {
    content_with(&data::assets::samus::palettes::SamusPalettes::default())
}

/// The fixture's content, with her pulse colours taken from `palettes`.
pub(super) fn content_with(palettes: &data::assets::samus::palettes::SamusPalettes) -> Content {
    let poses = data::assets::samus::map::MAP.iter().map(|entry| entry.key);
    Content {
        samus_body_art: None,
        samus_tiles: None,
        clock_figures: None,
        standing: None,
        shaft_track: None,
        shaft_overlay_art: None,
        shaft_effects: Vec::new(),
        shaft_raster: None,
        arrival_graphics: None,
        arrival_beam_tiles: None,
        arrival_overlay: None,
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
        opening_hud: None,
        map: HerMap::default(),
        start_x: Wrapping(0x80),
        ridley: ridley_content(),
        warning: None,
        // The worth the player's ROM gives his tail's hit.
        tail_worth: samus_health::WholeEnergy::new(15).unwrap(),
    }
}

fn ridley_content() -> boss_ceres_ridley::RidleyContent {
    boss_ceres_ridley::RidleyContent {
        flyby: boss_ceres_ridley::FlybyContent {
            path: boss_ceres_ridley::FlybyPath {
                first: boss_ceres_ridley::FlybyFrame {
                    shift: types::Pair { x: 0, y: 0 },
                    zoom: boss_ceres_ridley::FlybyZoom::new(0).unwrap(),
                },
                rest: Vec::new(),
            },
            body_shades: [[types::Colour::BLACK; 15]; 9],
            baby_looks: enum_map::EnumMap::default(),
            wing_looks: enum_map::EnumMap::default(),
        },
        getaway: boss_ceres_ridley::GetawayContent {
            left_wall: types::Pair {
                x: Wrapping(8),
                y: Wrapping(127),
            },
            right_wall: types::Pair {
                x: Wrapping(248),
                y: Wrapping(127),
            },
            body_colours: [types::Colour::BLACK; 15],
            wall_colours: [types::Colour::BLACK; 8],
        },
        eye_steps: boss_ceres_ridley::FadeSets::new(vec![boss_ceres_ridley::EyeColours(
            [types::Colour::BLACK; 3],
        )])
        .unwrap(),
        body_sets: boss_ceres_ridley::FadeSets::new(vec![boss_ceres_ridley::BodyColours(
            [types::Colour::BLACK; 11],
        )])
        .unwrap(),
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
    state.station.stage = ceres;
    state.step(&inputs(held), &passing(), &hud_frame(), content, &rules())
}

fn passing() -> Passing {
    Passing {
        actor_fetch: crate::production::played::passage::ActorFetch::Place,
        enemy_tiles: super::passage::EnemyTileTransfer::Waiting,
        shaft_background: super::room_tiles::ShaftBackground::Pending,
        shaft_raster: crate::production::played::passage::ShaftRaster::Pending,
        work_begun: true,
        work_done: true,
        publishes: true,
        audio: passage::PassageAudio {
            shaft_reply: None,
            sounds_sent: true,
            music_ready: true,
        },
        random: None,
        start: arrival::Readiness::NotYet,
    }
}

#[test]
fn first_active_draw_reads_the_last_draw_after_a_poll_gap() {
    use samus_movement::JumpButton;
    use types::{Buttons, button::JUMP};

    let mut state = state(Vec::new());
    state.passage.doors = Doors::Passing(world_doors::Passage {
        phase: world_doors::Phase::Brightening(world_doors::FadeTime(14)),
        arrival: None,
        slide_frames: 0,
    });
    let content = content();
    let rules = rules();
    let mut passing = passing();

    passing.work_done = false;
    passing.publishes = false;
    state
        .step(
            &inputs(Buttons::default()),
            &passing,
            &hud_frame(),
            &content,
            &rules,
        )
        .unwrap();
    assert!(matches!(state.passage.doors, Doors::Passing(_)));
    assert_eq!(state.samus.last_draw_jump.0, JumpButton::Up);

    passing.work_done = true;
    passing.publishes = true;
    passing.work_begun = false;
    state
        .step(&inputs(JUMP), &passing, &hud_frame(), &content, &rules)
        .unwrap();
    assert_eq!(state.passage.doors, Doors::Play);
    assert_eq!(state.samus.last_draw_jump.0, JumpButton::Up);
    assert_eq!(state.samus.movement.0.jump_held.frames.0, 0);

    passing.work_begun = true;
    let mut input = inputs(JUMP);
    input.pressed = Buttons::default();
    state
        .step(&input, &passing, &hud_frame(), &content, &rules)
        .unwrap();
    assert_eq!(state.samus.last_draw_jump.0, JumpButton::Held);
    assert_eq!(state.samus.movement.0.jump_held.frames.0, 0);
}

fn grounded() -> State {
    use samus_movement::{Choices, JumpButton, Phase};
    use types::{Action, Block};

    let mut state = state(Vec::new());
    let samus = &mut state.samus.movement.0;
    samus.pose.action = Action::Still;
    samus.vertical.phase = Phase::Ground;
    samus.position.y = Wrapping(108 * 65536 - 1);
    samus.choices = Choices::starting_in(samus.pose);
    state.samus.figure.0.pose = samus.pose;
    state.samus.last_draw_jump.0 = JumpButton::Held;
    let room = &mut state.passage.room;
    let cells = (0..16 * 16)
        .map(|index| {
            if index < 16 * 8 {
                Block::Air
            } else {
                Block::Solid
            }
        })
        .collect();
    room.geometry = crate::production::world::MovementWorld::new(
        16,
        cells,
        vec![false; 16 * 16],
        Vec::new(),
        crate::production::world::Boundary::Air,
    )
    .unwrap();

    state
}

#[test]
fn fresh_jump_still_selects_takeoff_when_the_last_draw_held_jump() {
    use types::{Action, button::JUMP};

    let mut state = grounded();
    state
        .step(
            &inputs(JUMP),
            &passing(),
            &hud_frame(),
            &content(),
            &rules(),
        )
        .unwrap();
    let samus = &state.samus.movement.0;
    assert_eq!(samus.pose.action, Action::TakingOff);
    assert_eq!(samus.jump_held.frames.0, 1);
}

#[test]
fn armed_jump_is_spent_before_the_active_draw_counts() {
    use samus_movement::{FrameBefore, JumpHeld};
    use types::{Action, Buttons, button::JUMP};

    let mut state = grounded();
    state.samus.movement.0.jump_held = JumpHeld {
        frames: Wrapping(5),
        before: FrameBefore::LandedOrEnded,
    };
    let mut input = inputs(JUMP);
    input.pressed = Buttons::default();
    state
        .step(&input, &passing(), &hud_frame(), &content(), &rules())
        .unwrap();

    let samus = &state.samus.movement.0;
    assert_eq!(samus.pose.action, Action::TakingOff);
    assert_eq!(
        samus.jump_held,
        JumpHeld {
            frames: Wrapping(1),
            before: FrameBefore::Other,
        }
    );
}
