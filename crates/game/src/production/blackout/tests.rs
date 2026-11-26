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
        },
        Content {
            format,
            graphics: data::pose::tiles::Graphics::default(),
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
        assert!(state.validate().is_ok());
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
                    publish: true
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
        assert_eq!(state.validate(), Ok(()));
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
        state.advance(Cut { work, publish }, &content).unwrap();
        assert_eq!(state.validate(), Ok(()));
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
    let (mut state, _) = pending();
    state.owners.0.scene.pass = WorkPass::Finished;
    assert_eq!(state.validate(), Err(Fault::State));
}
