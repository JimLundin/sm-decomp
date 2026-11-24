use super::*;
use crate::production::tests::{empty_world, fixtures};
use crate::production::{FigureClip, PoseContent};
use boss_ceres_ridley::{DoorKind, Phase, Quarter, Wait};
use core::num::Wrapping;
use interface_hud as hud;
use types::Side;

/// Her figure in the fixture's pose.
fn figure() -> Figure {
    Figure {
        pose: fixtures::samus().pose,
        sprite: types::Sprite::Plain,
    }
}

/// A Ceres door touching her right side: her box reaches 5 px right of her, the door's 8 px
/// left of its centre.
fn door(phase: Phase) -> CeresDoor {
    let her = fixtures::samus().position;
    CeresDoor {
        kind: DoorKind::Door,
        facing: Side::Left,
        position: Pair {
            x: Wrapping(her.x.pixel().0.wrapping_add(13)),
            y: her.y.pixel(),
        },
        phase,
    }
}

fn state(doors: Vec<CeresDoor>) -> State {
    let her = fixtures::samus().position;
    State {
        movement: Model(fixtures::samus()),
        appearance: Model(samus_appearance::AppearanceState::INITIAL),
        figure: Model(figure()),
        camera: Model(world_camera::Camera::restore(
            ViewPosition(Pair {
                x: Wrapping(0),
                y: Wrapping(0),
            }),
            RememberedPosition(her),
            types::VerticalFraction(0),
        )),
        doors: Doors::Play,
        doorway: None,
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
        // A room no block across, so the mini-map never reads her map.
        map_room: hud::Room {
            cell: Pair { x: 0, y: 0 },
            blocks: Pair { x: 0, y: 0 },
        },
        ceres_doors: doors,
        frame: FrameCount(0),
    }
}

fn inputs(held: types::Buttons) -> Inputs {
    Inputs {
        held,
        pressed: held,
        items: Set::default(),
        parity: types::Parity::Even,
        energy: 99,
        liquid: samus_appearance::Liquid::NONE,
        sounds_waiting: samus_appearance::SoundsWaiting::NONE,
        activities: Set::default(),
        charge: samus_appearance::Charge::try_from(0).unwrap(),
        geometry: empty_world(),
        camera: CameraInputs {
            screens: vec![Screen::Open; 4],
            size: [2, 2],
            heights: [112, 128],
            lead: Lead::Ahead,
        },
        activity: super::super::connected::inputs::Activity::Inactive,
    }
}

fn hud_frame() -> HudFrame {
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

fn content() -> Content {
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
        )
        .unwrap(),
        colour_groups: [ColourGroup::Other; COLOURS],
        rooms: Rooms {
            world: data::assets::world::World::default(),
            tilesets: data::assets::tilesets::Tilesets::default(),
            actors: data::assets::actors::Actors::default(),
            samus: data::assets::samus::palettes::SamusPalettes::default(),
        },
        hud: HudPictures::default(),
        map: HerMap::default(),
    }
}

fn rules() -> Rules {
    Rules {
        movement: samus_movement::Rules::default(),
        appearance: samus_appearance::Rules::VANILLA,
        camera: super::super::CameraRules::new(world_camera::Rules::REFERENCE),
        doors: world_doors::Rules::default(),
        health: samus_health::Rules::REFERENCE,
        hud: interface_hud::Rules::default(),
        ceres: boss_ceres_ridley::Rules::default(),
    }
}

fn play(state: &mut State, held: types::Buttons, ceres: Stage) -> Result<Outputs, Fault> {
    let progress = run_progress::Progress::default();
    let passing = Passing {
        doorways: &[],
        work_done: true,
        audio: PassageAudio {
            sounds_sent: true,
            music_ready: true,
        },
        timer_running: false,
        progress: &progress,
        ceres,
    };
    state.step(&inputs(held), &passing, &hud_frame(), &content(), &rules())
}

/// A door on the last frame of its opening, which its run this frame leaves open.
fn opening_ends() -> Phase {
    Phase::Opening {
        quarter: Quarter::Fourth,
        wait: Wait::new(1).unwrap(),
    }
}

#[test]
fn a_door_that_opens_this_frame_still_blocks_her_moves() {
    let mut blocked = state(vec![door(opening_ends())]);
    play(&mut blocked, types::button::RIGHT, Stage::Calm).unwrap();
    assert!(!blocked.ceres_doors[0].phase.solid());
    let mut open = state(vec![door(Phase::Open {
        wait: Wait::new(2).unwrap(),
    })]);
    play(&mut open, types::button::RIGHT, Stage::Calm).unwrap();
    let start = fixtures::samus().position.x;
    assert!(open.movement.0.position.x.0 > start.0);
    assert!(blocked.movement.0.position.x.0 < open.movement.0.position.x.0);
}

#[test]
fn the_escape_refuses_a_room_with_ceres_doors_and_leaves_state_as_it_was() {
    let mut escaping = state(vec![door(opening_ends())]);
    let before = escaping.clone();
    assert_eq!(
        play(&mut escaping, types::Buttons::default(), Stage::Escape),
        Err(Fault::Escape)
    );
    assert_eq!(escaping, before);
}

#[test]
fn a_finished_frame_counts_and_a_restarting_one_counts_from_zero() {
    let last = FrameCount(0xff);
    assert_eq!(counted(last, Finishing::Overrun), last);
    assert_eq!(counted(last, Finishing::Finished), FrameCount(0));
    assert_eq!(
        counted(last, Finishing::Restarts(world_doors::SlideScreen::Sprites)),
        FrameCount(1)
    );
}
