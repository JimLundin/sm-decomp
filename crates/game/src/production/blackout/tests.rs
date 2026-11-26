use super::*;
use ::save_files::SaveFormat;
use types::ceres_blackout::{
    BlackoutTimer, FinishedPasses, PreparedPicture, PublishedPicture, TimerSprite, WaitPasses,
};
use types::save_files::{BatteryImage, EncodedRun};

fn pending() -> (State, Content) {
    let format = Format::test_format();
    let mut files = save_files::power_on(&format, &BatteryImage::zeroed());
    files.active = Some(types::file_select::Slot::Second);
    let o = Owners {
        scene: BlackoutState {
            phase: BlackoutPhase::AwaitBlank,
            light: BlackoutLight::ForcedBlank,
            published: PublishedPicture {
                light: BlackoutLight::FULL,
                timer_sprites: vec![TimerSprite(29)],
            },
            pass: WorkPass::Prepared(PreparedPicture {
                timer_sprites: vec![TimerSprite(31)],
            }),
            timer: BlackoutTimer {
                time: TimerCentiseconds(0),
                activity: TimerActivity::Active,
            },
            shaft: ShaftStage::Rotating,
            finished: FinishedPasses(19),
        },
        countdown: Countdown::Begun(interface_hud::BegunCountdown {
            time_left: interface_hud::TimeLeft::RUN_OUT,
            age: interface_hud::CountdownAge::BEGUN,
            place: interface_hud::CountdownPlace::new(types::Pair {
                x: 128 * 256,
                y: 128 * 256,
            })
            .unwrap(),
        }),
        files: save_files::State::new(files),
        run: format.decode(&EncodedRun::zeroed()),
        live_map: ExploredCells::zeroed(),
        location: (Area::Ceres, StationId(0)),
        health: samus_health::State {
            energy: samus_health::EnergyAmount::ZERO,
            pending_loss: samus_health::EnergyLoss::ZERO,
            warning: samus_health::Warning::Off,
            protection: samus_health::ProtectionFrames::ZERO,
        },
        random: types::RandomSequence {
            number: types::RandomNumber(17),
        },
    };
    (
        State {
            owners: Model(o),
            video: Box::default(),
            platform: None,
            hud: None,
        },
        Content {
            format,
            graphics: data::pose::tiles::Graphics::default(),
            standing: None,
            hud: None,
            figures: Figures {
                frame: data::spritemap::Spritemap::default(),
                digits: std::array::from_fn(|_| data::spritemap::Spritemap::default()),
            },
        },
    )
}

#[test]
fn a_restored_save_publishes_the_room_before_dispatching_once() {
    let (mut state, content) = pending();
    let new = Cut {
        work: Work::Fresh { reached: false },
        publish: false,
        standing: None,
    };
    for cut in [
        Cut {
            work: Work::VerticalEnded,
            ..new
        },
        Cut {
            work: Work::SaveCompleted,
            ..new
        },
        new,
        Cut {
            publish: true,
            ..new
        },
    ] {
        let report = state.advance(cut, &content).unwrap();
        assert!(report.dispatch.is_none());
        assert!(state.validate(&content).is_ok());
        let bytes = canonical::to_bytes(&state);
        assert_eq!(canonical::from_bytes::<State>(&bytes).unwrap(), state);
    }
    assert_eq!(
        state.owners.0.scene.published.timer_sprites,
        [TimerSprite(31)]
    );
    assert!(
        state
            .advance(
                Cut {
                    work: Work::Interrupted,
                    publish: true,
                    standing: None,
                },
                &content
            )
            .unwrap()
            .dispatch
            .is_none()
    );
    assert!(state.advance(new, &content).unwrap().dispatch.is_some());
    assert!(state.advance(new, &content).unwrap().dispatch.is_none());
}

fn lit_room(state: &mut State, phase: BlackoutPhase) {
    let scene = &mut state.owners.0.scene;
    scene.phase = phase;
    scene.light = BlackoutLight::FULL;
    scene.pass = WorkPass::Finished;
}

#[test]
fn checkpoint_shape_covers_the_scene_and_the_carried_owners() {
    let (mut state, content) = pending();
    let blank = state.owners.0.scene.clone();
    let mut wire = Vec::new();
    for phase in [
        BlackoutPhase::Shaft,
        BlackoutPhase::Hold(WaitPasses::new(37).unwrap()),
        BlackoutPhase::Fade,
    ] {
        lit_room(&mut state, phase);
        assert_eq!(state.validate(&content), Ok(()));
        wire.extend(canonical::to_bytes(&state.owners));
    }
    state.owners.0.scene = blank;
    wire.extend(canonical::to_bytes(&state.owners));
    for (work, publish) in [
        (Work::VerticalEnded, false),
        (Work::SaveCompleted, false),
        (Work::Interrupted, true),
        (Work::Fresh { reached: false }, false),
    ] {
        state
            .advance(
                Cut {
                    work,
                    publish,
                    standing: None,
                },
                &content,
            )
            .unwrap();
        assert_eq!(state.validate(&content), Ok(()));
        wire.extend(canonical::to_bytes(&state.owners));
    }
    let digest = data::rom::sha256(&wire);
    let shape = canonical::SchemaHasher::new()
        .schema(canonical::model::shape::<BlackoutState>().unwrap())
        .schema(u64::from_le_bytes(*digest.first_chunk().unwrap()))
        .finish();
    assert_eq!(shape, OwnersPin::SHAPE);
}

#[test]
fn stored_shaft_and_fade_tags_restore_distinct_room_phases() {
    let (mut state, _) = pending();
    lit_room(&mut state, BlackoutPhase::Shaft);
    let mut wire = canonical::to_bytes(&state.owners);
    // The four-byte model length precedes the scene's one-byte phase tag.
    for (stored_tag, phase) in [(0, BlackoutPhase::Shaft), (2, BlackoutPhase::Fade)] {
        wire[size_of::<u32>()] = stored_tag;
        let restored = canonical::from_bytes::<Model<OwnersPin>>(&wire).unwrap();
        assert_eq!(restored.0.scene.phase, phase);
    }
}

#[test]
fn an_unpublished_departure_cannot_be_admitted_as_finished() {
    let (mut state, content) = pending();
    state.owners.0.scene.pass = WorkPass::Finished;
    assert_eq!(state.validate(&content), Err(Fault::State));
}

fn standing_art() -> crate::samus_appearance_bridge::elevator::Art {
    use data::pose::tiles::{Graphics, GraphicsRef};
    use samus_appearance::{
        ArtDisplacement, BodyHalf, BodyPiece, ElevatorArt, ObjectSize, ObjectStyle,
        StandingPoseArt, StandingStepArt, Step,
    };
    use types::Pair;

    use crate::samus_appearance_bridge::elevator::Art;
    use crate::state::video::RomWords;

    let source = |offset| RomWords {
        from: GraphicsRef { bank: 0xa0, offset },
        words: 1,
    };
    let piece = BodyPiece {
        position: Pair { x: -3, y: -4 },
        style: ObjectStyle {
            tile: 7,
            palette: 3,
            size: ObjectSize::Large,
            priority: 2,
            flips: Pair { x: true, y: false },
        },
    };
    let standing = StandingPoseArt {
        displacement: ArtDisplacement(-2),
        steps: [(
            Step(0),
            StandingStepArt {
                upper: BodyHalf {
                    pieces: vec![piece],
                    first_strip: source(0x8000),
                    second_strip: None,
                },
                lower: BodyHalf {
                    pieces: vec![piece],
                    first_strip: source(0x8002),
                    second_strip: Some(source(0x8004)),
                },
            },
        )]
        .into(),
    };
    Art {
        pictures: ElevatorArt {
            left: standing.clone(),
            right: standing,
        },
        graphics: Graphics::new(vec![(0xa0, vec![1, 0, 2, 0, 3, 0])]),
    }
}

fn standing_frame() -> StandingFrame {
    use samus_appearance::{ElevatorProjection, ShaftView, Step};
    use types::{Action, Aim, Body, Facing, Gun, Legs, Pair, Pose};

    StandingFrame {
        pose: Pose {
            body: Body::Upright,
            action: Action::Still,
            facing: Facing::Right,
            aim: Aim::Forward,
            gun: Gun::Stowed,
            legs: Legs::Together,
        },
        step: Step(0),
        projection: ElevatorProjection {
            centre: Pair { x: 2, y: 28 },
            pivot: Pair { x: 0, y: 0 },
            camera_origin: Pair { x: 0, y: 0 },
            carried_view: ShaftView {
                diagonal: 256,
                cross: 0,
            },
        },
    }
}

#[test]
fn standing_rows_and_oam_wait_for_publication_across_a_checkpoint() {
    use crate::state::video::{OamEntry, SpriteCount};

    let (mut state, mut content) = pending();
    lit_room(
        &mut state,
        BlackoutPhase::Hold(WaitPasses::new(37).unwrap()),
    );
    content.standing = Some(standing_art());
    state.video.memory.vram[0x6100] = 0x55;
    state.video.batch.oam.low[2] = OamEntry {
        x: 11,
        y: 18,
        tile: 29,
        attributes: 33,
    };
    let live = state.video.memory.clone();
    let prepared = state
        .advance(
            Cut {
                work: Work::Fresh { reached: false },
                publish: false,
                standing: Some(standing_frame()),
            },
            &content,
        )
        .unwrap();
    assert_eq!(state.video.memory, live);
    assert_eq!(prepared.standing_publication, StandingStrips::default());
    let saved = canonical::to_bytes(&state);
    let restored: State = canonical::from_bytes(&saved).unwrap();
    assert_eq!(restored, state);
    let mut state = restored;
    let strips = state.video.batch.standing_strips;
    assert_eq!(strips.upper_second, None);
    let pending_oam = state.video.batch.oam;
    assert_eq!(state.video.batch.sprites, SpriteCount(2));
    assert_ne!(pending_oam, live.oam);

    let suspended = Cut {
        work: Work::Interrupted,
        publish: false,
        standing: None,
    };
    let held = state.advance(suspended, &content).unwrap();
    assert_eq!(state.video.memory, live);
    assert_eq!(state.video.batch.oam, pending_oam);
    assert_eq!(state.video.batch.sprites, SpriteCount(2));
    assert_eq!(state.video.batch.standing_strips, strips);
    assert_eq!(held.standing_publication, StandingStrips::default());

    let published = state
        .advance(
            Cut {
                publish: true,
                ..suspended
            },
            &content,
        )
        .unwrap();
    let mut expected_oam = pending_oam;
    for unused in &mut expected_oam.low[2..] {
        unused.y = 240;
    }
    assert_eq!(state.video.memory.oam, expected_oam);
    assert_eq!(state.video.batch.sprites, SpriteCount(0));
    assert_eq!(published.standing_publication, strips);
    for (word, expected) in [(0x6000, 1), (0x6100, 0x55), (0x6080, 2), (0x6180, 3)] {
        assert_eq!(state.video.memory.vram[word], expected);
    }
}

#[test]
fn unsupported_standing_looks_leave_the_room_unchanged() {
    use samus_appearance::Step;
    use types::{Aim, Pose};

    let (mut state, mut content) = pending();
    lit_room(
        &mut state,
        BlackoutPhase::Hold(WaitPasses::new(37).unwrap()),
    );
    content.standing = Some(standing_art());
    let before = state.clone();
    let frame = standing_frame();
    let wrong_aim = StandingFrame {
        pose: Pose {
            aim: Aim::Up,
            ..frame.pose
        },
        ..frame
    };
    let missing_step = StandingFrame {
        step: Step(1),
        ..frame
    };
    for (standing, expected) in [
        (wrong_aim, Fault::StandingPose),
        (missing_step, Fault::StandingArt),
    ] {
        let result = state.advance(
            Cut {
                work: Work::Fresh { reached: false },
                publish: true,
                standing: Some(standing),
            },
            &content,
        );
        assert_eq!(result, Err(expected));
        assert_eq!(state, before);
    }
}
