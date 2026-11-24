use super::*;
use crate::production::tests::{empty_world, fixtures};
use crate::production::{FigureClip, PoseContent};
use boss_ceres_ridley::{DoorKind, Phase, Quarter, Wait};
use core::num::Wrapping;
use interface_hud as hud;
use types::Side;
use world_doors::FadeTime;

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
        room: super::Room {
            geometry: empty_world(),
            doorways: Vec::new(),
            camera: inputs(types::Buttons::default()).camera,
        },
        ceres_doors: doors,
        frame: FrameCount(0),
        finished_frames: 0,
        pad_and_cover: super::shaft::GONE,
        shaft_overlay: false,
        ridley: None,
        stage: Stage::Calm,
        random: types::RandomSequence {
            number: types::RandomNumber(0),
        },
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
            slopes: data::level::slope::Slopes::default(),
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
    play_with(state, &content(), held, ceres)
}

fn play_with(
    state: &mut State,
    content: &Content,
    held: types::Buttons,
    ceres: Stage,
) -> Result<Outputs, Fault> {
    let progress = run_progress::Progress::default();
    let passing = Passing {
        work_begun: true,
        work_done: true,
        audio: PassageAudio {
            sounds_sent: true,
            music_ready: true,
        },
        timer_running: false,
        progress: &progress,
        random: None,
    };
    state.stage = ceres;
    state.step(&inputs(held), &passing, &hud_frame(), content, &rules())
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
    escaping.stage = Stage::Escape;
    let before = escaping.clone();
    assert_eq!(
        play(&mut escaping, types::Buttons::default(), Stage::Escape),
        Err(Fault::Escape)
    );
    assert_eq!(escaping, before);
}

/// The shaft's pad, under her with its wait `wait`, and the cover below.
fn on_the_pad(wait: u16) -> PadAndCover {
    let her = fixtures::samus().position;
    let mut placed = PadAndCover::arrival(
        Pair {
            x: her.x.pixel(),
            y: her.y.pixel(),
        },
        &boss_ceres_ridley::Rules::default(),
    )
    .pad_and_cover;
    if let Some(pad) = placed.pad.as_mut() {
        pad.wait = boss_ceres_ridley::PadWait::new(wait).unwrap();
    }
    placed
}

#[test]
fn while_the_pad_holds_her_her_moves_and_animation_dont_run_and_it_sets_her_height() {
    let mut held = state(Vec::new());
    held.pad_and_cover = on_the_pad(1);
    // A pixel above the floor, so the pad's run lowers her to it.
    held.movement.0.position.y = types::fixed(Wrapping(71), 0);
    let before = held.clone();
    let outputs = play(&mut held, types::button::RIGHT, Stage::Calm).unwrap();
    assert_eq!(outputs.movement, None);
    assert_eq!(held.movement.0.position.x, before.movement.0.position.x);
    assert_eq!(held.movement.0.position.y, types::fixed(Wrapping(72), 0));
    assert_eq!(held.appearance, before.appearance);
    assert!(held.pad_and_cover.holds_her(), "she reaches the floor held");
    // The next run lets her go after her frame, still held; the frame after, she moves.
    play(&mut held, types::button::RIGHT, Stage::Calm).unwrap();
    assert!(!held.pad_and_cover.holds_her());
    assert_eq!(held.movement.0.position.x, before.movement.0.position.x);
    play(&mut held, types::button::RIGHT, Stage::Calm).unwrap();
    assert!(held.movement.0.position.x.0 > before.movement.0.position.x.0);
}

#[test]
fn with_the_shafts_overlay_the_beacon_sets_its_colours_from_the_count_of_finished_frames() {
    let mut content = content();
    let mut beacon = data::assets::actors::BeaconColours::default();
    for (step, set) in beacon.sets.iter_mut().enumerate() {
        *set = [data::assets::art::Colour(u16::try_from(step).unwrap() + 1); 6];
    }
    content.rooms.actors.beacon = beacon;
    let mut shaft = state(Vec::new());
    shaft.shaft_overlay = true;
    // Step (24 / 8) mod 8 = 3 of the cycle.
    shaft.finished_frames = 24;
    let outputs = play_with(&mut shaft, &content, types::Buttons::default(), Stage::Calm).unwrap();
    assert_eq!(
        outputs
            .lights
            .and_then(|lights| lights.beacon)
            .map(boss_ceres_ridley::BeaconStep::place),
        Some(3)
    );
    let first = data::assets::actors::BeaconColours::FIRST;
    assert_eq!(shaft.palette.screen[first - 1], types::Colour::BLACK);
    assert!(
        shaft.palette.screen[first..first + 6]
            .iter()
            .all(|colour| colour.bits() == 4)
    );
    assert_eq!(shaft.palette.screen[first + 6], types::Colour::BLACK);
    assert_eq!(shaft.finished_frames, 25);
    // In the escape a later piece owns the overlay.
    assert_eq!(
        play_with(
            &mut shaft,
            &content,
            types::Buttons::default(),
            Stage::Escape
        ),
        Err(Fault::Escape)
    );
}

/// As the colours come up, the lights run before the fade's step, so on its first step the
/// beacon still shows; as they go dark the fade's step comes first, so the beacon shows none.
#[test]
fn the_beacon_shows_before_the_fade_in_takes_its_step_and_not_once_the_fade_out_has() {
    let mut content = content();
    content.rooms.actors.beacon = data::assets::actors::BeaconColours {
        sets: [[data::assets::art::Colour(7); 6]; 8],
    };
    let fading = |phase| {
        let mut shaft = state(Vec::new());
        shaft.shaft_overlay = true;
        shaft.doors = Doors::Passing(Passage {
            phase,
            arrival: None,
            slide_frames: 0,
        });
        let outputs =
            play_with(&mut shaft, &content, types::Buttons::default(), Stage::Calm).unwrap();
        (outputs.lights.and_then(|lights| lights.beacon), shaft)
    };
    let first = data::assets::actors::BeaconColours::FIRST;
    let (shown, brightening) = fading(world_doors::Phase::Brightening(FadeTime::START));
    assert!(shown.is_some());
    assert_eq!(brightening.palette.screen[first].bits(), 7);
    let (shown, darkening) = fading(world_doors::Phase::Darkening(FadeTime::START));
    assert_eq!(shown, None);
    assert_eq!(darkening.palette.screen[first], types::Colour::BLACK);
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

/// Each report of her moves gives the dash the clip reads, decided first by whether she is
/// dashing after them.
#[test]
fn dash_follows_her_dashing_after_moves() {
    let cases = [
        (Dashing::Off, false, true, Dash::Stopped),
        (Dashing::Off, true, false, Dash::Off),
        (Dashing::Off, false, false, Dash::Off),
        (Dashing::On, true, true, Dash::BeganAndLost),
        (Dashing::On, true, false, Dash::Began),
        (Dashing::On, false, true, Dash::Lost),
        (Dashing::On, false, false, Dash::On),
    ];
    for (dashing, began, lost, expected) in cases {
        assert_eq!(
            dash(dashing, began, lost),
            expected,
            "{dashing:?}, began {began}, lost {lost}"
        );
    }
}

/// A run into a wall that starts her dash leaves her not dashing after her moves: stopped, so
/// the clip holds its own pace.
#[test]
fn dash_begun_into_a_wall_is_stopped() {
    assert_eq!(dash(Dashing::Off, true, true), Dash::Stopped);
}
