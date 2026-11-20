use super::*;
use canonical::model::Model;
use core::num::Wrapping;
use game::production::frame::{
    camera_stage::{Previous, Stage},
    movement::{Movement, ReceiptsPin, Work, WorkPin},
};
use types::Pair;

struct Air;
impl types::World for Air {
    fn block(&self, _: types::Cell) -> types::Block {
        types::Block::Air
    }
    fn lets_through(&self, _: types::Cell, _: types::Way) -> bool {
        false
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        &[]
    }
}

fn samus() -> samus_movement::Samus {
    samus_movement::Samus {
        pose: types::Pose {
            body: types::Body::Upright,
            action: types::Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        },
        position: Pair {
            x: types::px(Wrapping(200)),
            y: types::px(Wrapping(300)),
        },
        run: samus_movement::Run {
            speed: Wrapping(0),
            way: types::Side::Right,
            carry: samus_movement::Carry::FollowsPad,
            dash: Wrapping(0),
            dashing: samus_movement::Dashing::Off,
        },
        vertical: samus_movement::Vertical {
            speed: Wrapping(0),
            phase: samus_movement::Phase::Ground,
            bounces: samus_movement::Bounces::None,
        },
        spin: samus_movement::Spin::Off,
        seat: samus_movement::Seat::Unseated,
        jump_held: samus_movement::JumpHeld::NONE,
    }
}

fn selected() -> (Content, Stage) {
    let samus = samus();
    let frame = samus_movement::Frame {
        sprite_before_movement: types::Sprite::Plain,
        cinematic_active: false,
        pad: types::Buttons::default(),
        pressed: types::Buttons::default(),
        items: types::Set::default(),
        spin_turned_far: false,
        elevator_moving: false,
        parity: types::Parity::Even,
        animation_ends: None,
        body: &|_| Pair {
            x: Wrapping(5),
            y: Wrapping(21),
        },
        cannon_heading: &|_| None,
    };
    let mut work = samus_movement::suspended::SuspendedMovement::new(samus).unwrap();
    let mut decisions = Vec::new();
    while let Some(decision) = work
        .offer(&frame, &Air, &samus_movement::Rules::default())
        .unwrap()
        .cloned()
    {
        decisions.push(decision);
        work.complete(&frame, &Air, &samus_movement::Rules::default())
            .unwrap();
    }
    work.state.position = Pair {
        x: types::px(Wrapping(220)),
        y: samus.position.y,
    };
    work.state.pose = samus.pose;
    work.outputs.skidding = false;
    work.outputs.placed = None;
    let movement = Movement::Work(Work {
        model: Model::<WorkPin>(Box::new(work)),
        completed: Model::<ReceiptsPin>(decisions),
    });
    let previous = Previous::capture(&Movement::new(samus));
    let camera = Model(world_camera::Camera::initialise(
        world_camera::ViewPosition(Pair {
            x: types::ZERO,
            y: types::ZERO,
        }),
        world_camera::RememberedPosition(samus.position),
    ));
    let inputs = Inputs {
        room: Room {
            identity: data::id::RoomId(99),
            screens: vec![Screen::Open; 20],
            size: [4, 4],
            heights: [91, 153],
        },
        lead: Lead::Ahead,
        time_stopped: false,
        tuning: Tuning::capture(&world_camera::Rules::REFERENCE),
        grapple: Activity::Inactive,
        knockback: Activity::Inactive,
        hook: Activity::Inactive,
    };
    let content = Content {
        inputs: inputs.clone(),
        fields: vec![
            field("CurrentCentreX", 220),
            field("PreviousCentreX", 200),
            field("CurrentMovementKind", 0xab01),
            field("CurrentHeading", 0xcd08),
            field("HorizontalAccelerationMode", 0),
            field("PreviousCameraX", 0),
            field("CurrentCameraY", 0),
            field("RoomWidthScreens", 4),
            field("RoomHeightScreens", 4),
            field("CurrentCentreSubX", 0),
            field("CurrentCentreY", 300),
            field("CurrentCentreSubY", 0),
            field("PreviousCentreSubX", 0),
            field("PreviousCentreY", 300),
            field("PreviousCentreSubY", 0),
            field("PreviousCameraSubX", 0),
            field("CurrentCameraYScreen", 0),
            field("FreezeLow", 0),
            field("FreezeHigh", 0),
            field("GrappleScrollMode", 0),
            field("KnockbackDirection", 0),
            field("ScrollFinishedHook", 0),
        ],
    };
    let mut stage = Stage::Idle;
    stage
        .choose(&camera, &previous, &movement, &inputs)
        .unwrap();
    (content, stage)
}

fn field(name: &str, expected: u16) -> Field {
    Field {
        name: name.to_owned(),
        private_wram_offset: 0,
        quantity_width: 2,
        private_source_encoding: expected,
        actual_model_join_required: true,
    }
}

fn profile() -> super::super::frame_origin::profiles::Profile {
    use super::super::frame_origin::profiles::{ArithmeticTask, Publication, Run};
    use data::timing::{Cycle, ExposureKind, PublicationPort};
    let mut runs = Vec::new();
    for name in ["VerticalScrollZonePrimary", "VerticalScrollZoneSecondary"] {
        let task = ArithmeticTask {
            name: name.to_owned(),
            quantities: vec!["ViewportYScreen".to_owned(), "RoomWidthScreens".to_owned()],
        };
        for effect in [
            Effect::Publication(Publication {
                port: PublicationPort(0x4202),
                device: "Multiplicand".to_owned(),
                producer: name.to_owned(),
                arithmetic: Some(task.clone()),
            }),
            Effect::Publication(Publication {
                port: PublicationPort(0x4203),
                device: "Multiplier".to_owned(),
                producer: name.to_owned(),
                arithmetic: Some(task.clone()),
            }),
            Effect::Sample(Sample::ProductLow, Some(task.clone())),
            Effect::Sample(Sample::ProductHigh, Some(task.clone())),
        ] {
            let kind = match &effect {
                Effect::Publication(_) => ExposureKind::Write,
                Effect::Sample(..) => ExposureKind::Read,
                Effect::None => ExposureKind::Internal,
            };
            runs.push(Run {
                count: 1,
                cycle: Cycle::Six,
                kind,
                poll_before: false,
                effect,
            });
        }
    }
    super::super::frame_origin::profiles::Profile {
        name: ProfileName::PreWait,
        runs,
        exposures: 8,
    }
}

#[test]
fn complete_arithmetic_namespaces_and_actual_request_operands_are_required() {
    use data::timing::{DeviceEffect, Publication};
    let mut profile = profile();
    assert_eq!(arithmetic(&profile), Ok(()));
    let requests = game::production::frame::camera_stage::Requests { row: 3, width: 7 };
    let bound = requests.bind(&profile.template().unwrap()).unwrap();
    let writes = bound
        .runs
        .iter()
        .filter_map(|run| match &run.exposure.effect {
            DeviceEffect::Publish(write) => Some(write.effect.clone()),
            DeviceEffect::None | DeviceEffect::Sample(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        writes,
        [
            Publication::Multiplicand(3),
            Publication::Multiplier(7),
            Publication::Multiplicand(3),
            Publication::Multiplier(7)
        ]
    );
    let mut reordered = profile.template().unwrap();
    reordered.runs.swap(2, 4);
    assert_eq!(
        requests.bind(&reordered),
        Err(game::production::frame::camera_stage::Fault::Profile)
    );
    let Effect::Publication(publication) = &mut profile.runs.first_mut().unwrap().effect else {
        panic!();
    };
    publication.arithmetic.as_mut().unwrap().name = "OtherTask".to_owned();
    assert_eq!(arithmetic(&profile), Err(Fault::Profile));
}

#[test]
fn exact_current_centre_join_and_source_certified_pose_masks_are_required() {
    let (content, mut stage) = selected();
    let contract = PhysicalContract::new(
        content.dependencies().unwrap(),
        profile().template().unwrap(),
    );
    assert!(stage.physical(&contract).is_ok());
    let Stage::Offered { receipt, .. } = &mut stage else {
        panic!();
    };
    let Movement::Work(work) = &mut receipt.movement else {
        panic!();
    };
    work.model.0.state.position.x = types::px(Wrapping(221));
    assert!(matches!(
        stage.physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
}

#[test]
fn actual_initial_camera_and_complete_current_neighbours_cannot_be_substituted() {
    let (content, stage) = selected();
    let contract = PhysicalContract::new(
        content.dependencies().unwrap(),
        profile().template().unwrap(),
    );
    let Stage::Offered { receipt, next } = stage else {
        panic!();
    };
    let mut changed = receipt.clone();
    changed.initial.0 = changed
        .initial
        .0
        .with_view(world_camera::ViewPosition(Pair {
            x: types::px(Wrapping(1)),
            y: types::ZERO,
        }));
    assert!(matches!(
        Stage::Offered {
            receipt: changed,
            next
        }
        .physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
    let mut changed = receipt.clone();
    changed.inputs.room.screens.fill(Screen::Closed);
    assert!(matches!(
        Stage::Offered {
            receipt: changed,
            next
        }
        .physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
    let mut changed = receipt.clone();
    let Movement::Work(work) = &mut changed.movement else {
        panic!();
    };
    work.model.0.state.run.carry = samus_movement::Carry::CarriesOn;
    assert!(matches!(
        Stage::Offered {
            receipt: changed,
            next
        }
        .physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
    let mut changed = receipt.clone();
    changed.previous.0.0.position.x = types::px(Wrapping(199));
    assert!(matches!(
        Stage::Offered {
            receipt: changed,
            next
        }
        .physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
    let changed_next = Model(next.0.with_view(world_camera::ViewPosition(Pair {
        x: next.0.position().x,
        y: types::px(Wrapping(1)),
    })));
    assert!(matches!(
        Stage::Offered {
            receipt: receipt.clone(),
            next: changed_next
        }
        .physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
    let mut changed = receipt;
    changed.inputs.knockback = Activity::Active;
    assert!(matches!(
        Stage::Offered {
            receipt: changed,
            next
        }
        .physical(&contract),
        Err(game::production::frame::camera_stage::Fault::Dependencies)
    ));
}

fn native_profile() -> Profile {
    let directory = std::path::PathBuf::from(
        std::env::var_os("SM_CAMERA_PROFILES").expect("explicit Camera profile directory required"),
    );
    let texts = [
        "profiles.toml",
        "portions.toml",
        "publication-bindings.toml",
        "sample-bindings.toml",
        "materialization-bindings.toml",
        "arithmetic-receipts.toml",
    ]
    .map(|name| std::fs::read_to_string(directory.join(name)).unwrap());
    let templates =
        super::super::frame_origin::profiles::decode(texts.each_ref().map(String::as_str)).unwrap();
    camera_profile(
        &templates,
        &envelope(
            &std::fs::read_to_string(
                std::env::var_os("SM_CAMERA_ENVELOPE")
                    .expect("explicit Camera whole envelope required"),
            )
            .unwrap(),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
#[ignore = "requires explicit pinned ROM, accepted Camera contract and independent origin"]
fn actual_rom_tuning_and_unchanged_neighbour_projection_are_checked() {
    let rom = Rom::new(
        std::fs::read(std::env::var_os("SM_ROM").expect("explicit SM_ROM required")).unwrap(),
    )
    .unwrap();
    let text = std::fs::read_to_string(
        std::env::var_os("SM_CAMERA_CONTRACT").expect("explicit Camera contract required"),
    )
    .unwrap();
    let room_text = std::fs::read_to_string(
        std::env::var_os("SM_CAMERA_ROOM_PROJECTION")
            .expect("explicit Camera room projection required"),
    )
    .unwrap();
    let mut origin = std::fs::read(
        std::env::var_os("SM_CAMERA_ORIGIN").expect("explicit Camera origin required"),
    )
    .unwrap();
    let content =
        Content::import(&origin, &rom, &text, &room_text).expect("actual accepted Camera import");
    let profile = native_profile();
    assert_eq!(arithmetic(&profile), Ok(()));
    assert!(profile.template().is_ok());
    assert!(content.dependencies().is_ok());
    assert_eq!(content.fields.len(), 45);
    assert_eq!(content.inputs().room.size, [1, 2]);
    assert_eq!(
        content.inputs().tuning.ahead,
        Tuning::capture(&world_camera::Rules::REFERENCE).ahead
    );
    let source: Source = toml::from_str(&text).unwrap();
    assert_eq!(parameter(&rom, &source, "RightLeadDistance"), Ok(96));
    let joins = source
        .camera_current_input
        .iter()
        .filter(|field| field.actual_model_join_required)
        .count();
    let runtime = source
        .camera_current_input
        .iter()
        .filter(|field| !field.actual_model_join_required && field.name != "CameraDistanceIndex")
        .count();
    assert_eq!(
        (joins, runtime, source.checked_camera_tuning.len()),
        (24, 20, 3)
    );
    if let Some(path) = std::env::var_os("SM_CAMERA_BINDING_REPORT") {
        std::fs::write(path, format!("format = \"checked-camera-importer-edge-v1\"\nsource_sha256 = \"{}\"\nchecked_fields = {}\nmodel_join_fields = {joins}\nunchanged_runtime_fields = {runtime}\nchecked_parameters = {}\n", Sha256::of(text.as_bytes()), content.fields.len(), source.checked_camera_tuning.len())).unwrap();
    }
    let mut wrong = source;
    wrong
        .checked_camera_tuning
        .iter_mut()
        .find(|parameter| parameter.name == "RightLeadDistance")
        .unwrap()
        .value = 95;
    assert_eq!(
        parameter(&rom, &wrong, "RightLeadDistance"),
        Err(Fault::Tuning)
    );
    let byte = origin.get_mut(0x07a9).unwrap();
    *byte ^= 1;
    assert!(matches!(
        Content::import(&origin, &rom, &text, &room_text),
        Err(Fault::Source)
    ));
    let old = text.replace(
        "bounded-current-camera-owner-v4",
        "bounded-current-camera-owner-v3",
    );
    assert!(matches!(
        Content::import(&origin, &rom, &old, &room_text),
        Err(Fault::Source)
    ));
}
