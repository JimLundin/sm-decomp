use super::super::*;
use crate::production::weapons::ActorSlotIdentity;
use crate::production::weapons::{ActorOccurrence, DoorOpportunity, QuietBeamWorld};
use crate::state::owned::{OwnedState, PowerPin};
use crate::weapons_projectiles_bridge::PowerContent;
use data::assets::{projectile as asset, samus};
use weapons_projectiles as beam;

fn content() -> PowerContent {
    let clip = asset::Clip {
        frames: vec![asset::Frame {
            hold: asset::Hold::try_from(3).unwrap(),
            picture: samus::SpriteId(0),
            radius: asset::Radius { x: 2, y: 2 },
            trail_appearance: asset::TrailAppearanceId(0),
        }],
        end: asset::ClipEnd::RepeatFrom(samus::FrameNumber(0)),
    };
    let flight = asset::Headings {
        up_facing_right: clip.clone(),
        up_right: clip.clone(),
        right: clip.clone(),
        down_right: clip.clone(),
        down_facing_right: clip.clone(),
        down_facing_left: clip.clone(),
        down_left: clip.clone(),
        left: clip.clone(),
        up_left: clip.clone(),
        up_facing_left: clip.clone(),
    };
    PowerContent::new(asset::Content {
        power: asset::PowerBeam {
            sheet: samus::SpriteSheet {
                sprites: vec![data::spritemap::Spritemap {
                    entries: vec![data::spritemap::SpritemapEntry::default()],
                }],
                ..Default::default()
            },
            flight,
            impact: asset::Clip {
                end: asset::ClipEnd::Remove,
                ..clip
            },
            damage: asset::Damage::try_from(8).unwrap(),
            trails: vec![asset::TrailAppearance::Inactive],
            trail_presentation: asset::TrailPresentation::EndsImmediately,
        },
        muzzles: vec![],
    })
    .unwrap()
}

fn state(x: u16, radius: u8) -> beam::State {
    let mut state = beam::State::reset(enum_map::EnumMap::from_fn(|_| beam::Axes {
        x: beam::PixelFraction(0),
        y: beam::PixelFraction(0),
    }));
    state.count = beam::ShotCount(1);
    state.slots[beam::ShotSlot::Slot0] = beam::ShotState::Flying {
        consumed: false,
        body: beam::ShotBody {
            kind: beam::ShotKind::Beam,
            position: beam::Axes {
                x: beam::FixedPosition::from_parts(x, beam::PixelFraction(0)),
                y: beam::FixedPosition::from_parts(128, beam::PixelFraction(0)),
            },
            heading: beam::Heading::Right,
            speed: beam::Axes {
                x: beam::BeamSpeed(1024),
                y: beam::BeamSpeed(0),
            },
            radius: beam::Axes {
                x: beam::Radius(radius),
                y: beam::Radius(radius),
            },
            damage: beam::Damage::new(8).unwrap(),
            trail: beam::FrameWord(1),
            clip: crate::weapons_projectiles_bridge::clip_id(asset::ClipId::Flight(
                asset::Heading::Right,
            )),
            next_frame: beam::ClipFrame::new(0, 1).unwrap(),
            remaining_hold: beam::FrameWord(1),
            picture: Some(crate::weapons_projectiles_bridge::picture(0)),
            trail_appearance: None,
        },
    };
    state
}

fn input() -> beam::UpdateInput {
    beam::UpdateInput {
        mode: beam::UpdateMode::ShotsOnly,
        time_stopped: false,
        cannon: beam::CannonInput {
            shoot: false,
            new_press: false,
            previous_new_press: false,
            muzzle: beam::Muzzle {
                aim: beam::MuzzleAim::Forbidden,
                lift: 0,
                stationary: enum_map::EnumMap::from_fn(|_| beam::Offset { x: 0, y: 0 }),
                moving: enum_map::EnumMap::from_fn(|_| beam::Offset { x: 0, y: 0 }),
            },
            previous_muzzle: beam::MuzzleAim::Forbidden,
            moving_muzzle: false,
            pending_aim: None,
            centre: beam::SamusCentre(beam::Axes { x: 128, y: 128 }),
        },
        inheritance: beam::Inheritance::stationary(),
        camera: beam::ScreenOrigin(beam::Axes { x: 0, y: 0 }),
        room: beam::RoomScreens(beam::Axes { x: 1, y: 1 }),
    }
}

fn world() -> QuietBeamWorld {
    QuietBeamWorld::new(16, 16, &vec![data::level::BlockKind::default(); 256]).unwrap()
}

fn ordinary_world(bottom: data::level::BlockKind) -> QuietBeamWorld {
    use data::level::{BlockKind, Bts, DoorIndex};
    let cells = (0..256)
        .map(|index| match index {
            0 => BlockKind::Door(DoorIndex(0)),
            120 => BlockKind::Solid(Bts(0)),
            136 => bottom,
            _ => BlockKind::default(),
        })
        .collect::<Vec<_>>();
    QuietBeamWorld::new(16, 16, &cells).unwrap()
}

#[test]
fn actual_solid_cells_block_only_when_every_contact_cell_blocks() {
    use data::level::{BlockKind, Bts};
    for (bottom, impact) in [
        (BlockKind::default(), false),
        (BlockKind::Solid(Bts(0)), true),
    ] {
        let world = ordinary_world(bottom);
        let input = input();
        let (mut production, _, _) = admit(state(128, 2));
        let ProductionOutputs::WeaponsContact(output) = production
            .advance(AdvanceInput::WeaponsContact {
                input: &input,
                world: &world,
                contacts: &[],
            })
            .unwrap()
        else {
            panic!("weapons output")
        };
        let OwnedState::WeaponsContact(held) = production.state() else {
            panic!("weapons scope")
        };
        assert_eq!(
            matches!(
                held.0.slots[beam::ShotSlot::Slot0],
                beam::ShotState::Impact { .. }
            ),
            impact
        );
        if impact {
            assert!(matches!(
                output.events.as_slice(),
                [
                    beam::Event::Trail { .. },
                    beam::Event::ImpactSound(beam::ShotSlot::Slot0)
                ]
            ));
        } else {
            assert!(matches!(
                output.events.as_slice(),
                [beam::Event::Trail { .. }]
            ));
        }
    }
}

#[test]
fn reached_unsupported_cell_discards_prior_query_motion_and_trail() {
    let world = ordinary_world(data::level::BlockKind::Door(data::level::DoorIndex(0)));
    let input = input();
    let (mut production, _, _) = admit(state(128, 2));
    let saved = production.save().unwrap();
    assert_eq!(
        production.advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &[]
        }),
        Err(ProductionFault::Unsupported(Unsupported::ReactiveBeamWorld))
    );
    assert_eq!(production.save().unwrap(), saved);
}

fn admit(state: beam::State) -> (Production, ProductionContent, ProductionRules) {
    let content = ProductionContent::WeaponsContact(Box::new(content()));
    let rules = ProductionRules::WeaponsContact(beam::Rules::REFERENCE);
    (
        Production::admit(
            ProductionSeed::WeaponsContact(Box::new(state)),
            content.clone(),
            rules.clone(),
        )
        .unwrap(),
        content,
        rules,
    )
}

fn opportunity(occurrence: usize, x: Option<u16>) -> DoorOpportunity {
    DoorOpportunity {
        occurrence: ActorOccurrence(occurrence),
        actor: ActorSlotIdentity(u8::try_from(occurrence).unwrap()),
        conditions: enemy_interactions::ContactConditions {
            actor_updates_enabled: true,
            interactions_enabled: true,
            samus_time_stopped: false,
            enemies_stopped_in_time: false,
            processing_membership: true,
            protection_remaining: enemy_interactions::ProtectionFrames(0),
        },
        door: x.map_or(enemy_interactions::DoorContact::Passable, |x| {
            enemy_interactions::DoorContact::Box(enemy_interactions::ContactBox {
                centre: enemy_interactions::Centre { x, y: 128 },
                radii: enemy_interactions::Radii { x: 16, y: 16 },
            })
        }),
    }
}

#[test]
fn power_state_shape_remains_pinned() {
    assert_eq!(canonical::model::pin_holds::<PowerPin>(), Ok(()));
}

#[test]
fn late_contacts_use_actual_moved_shot_and_keep_complete_events_and_bodies() {
    let (mut production, content, rules) = admit(state(128, 2));
    let input = input();
    let world = world();
    let contacts = [
        opportunity(0, None),
        opportunity(1, Some(132)),
        opportunity(2, Some(200)),
        opportunity(3, Some(132)),
    ];
    let ProductionOutputs::WeaponsContact(output) = production
        .advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &contacts,
        })
        .unwrap()
    else {
        panic!("weapons output");
    };
    assert_eq!(
        output
            .contacts
            .iter()
            .map(|receipt| receipt.opportunity)
            .collect::<Vec<_>>(),
        contacts
    );
    assert_eq!(
        output
            .contacts
            .iter()
            .map(|receipt| receipt.consumed)
            .collect::<Vec<_>>(),
        [
            None,
            Some(beam::ShotSlot::Slot0),
            None,
            Some(beam::ShotSlot::Slot0)
        ]
    );
    assert_eq!(
        output.events,
        vec![beam::Event::Trail {
            slot: beam::ShotSlot::Slot0,
            position: beam::Axes {
                x: beam::FixedPosition::from_parts(128, beam::PixelFraction(0)),
                y: beam::FixedPosition::from_parts(128, beam::PixelFraction(0))
            },
            picture: Some(crate::weapons_projectiles_bridge::picture(0)),
            appearance: None
        }]
    );
    let OwnedState::WeaponsContact(held) = production.state() else {
        panic!("weapons scope");
    };
    let beam::ShotState::Flying { body, consumed } = &held.0.slots[beam::ShotSlot::Slot0] else {
        panic!("retained shot");
    };
    assert!(*consumed);
    assert_eq!(body.position.x.integer(), 132);
    let mut restored = Production::restore(&production.save().unwrap(), content, rules).unwrap();
    let next = production
        .advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &[],
        })
        .unwrap();
    assert_eq!(
        restored.advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &[]
        }),
        Ok(next)
    );
    assert_eq!(production.state(), restored.state());
    let OwnedState::WeaponsContact(held) = production.state() else {
        panic!("weapons scope");
    };
    assert!(matches!(
        held.0.slots[beam::ShotSlot::Slot0],
        beam::ShotState::Free { .. }
    ));
    assert_eq!(held.0.count.0, 0);
}

#[test]
fn callback_suppressed_outside_and_oversized_spans_refuse_before_any_publication() {
    let input = input();
    let world = world();
    for (x, radius, reason) in [
        (260, 2, Unsupported::BeamContinuation),
        (128, 200, Unsupported::BeamSpan),
    ] {
        assert!(
            !beam::edge_geometry(
                beam::Axes { x, y: 128 },
                beam::Axes {
                    x: beam::Radius(radius),
                    y: beam::Radius(radius)
                },
                beam::BeamSpeed(1024),
                beam::Axis::Horizontal,
                1,
            )
            .requested
        );
        let (mut production, _, _) = admit(state(x, radius));
        let saved = production.save().unwrap();
        assert_eq!(
            production.advance(AdvanceInput::WeaponsContact {
                input: &input,
                world: &world,
                contacts: &[]
            }),
            Err(ProductionFault::Unsupported(reason))
        );
        assert_eq!(production.save().unwrap(), saved);
    }
}

#[test]
fn post_update_continuation_failure_discards_movement_and_trail_publication() {
    let (mut production, _, _) = admit(state(244, 2));
    let input = input();
    let world = world();
    let saved = production.save().unwrap();
    assert_eq!(
        production.advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &[],
        }),
        Err(ProductionFault::Unsupported(Unsupported::BeamContinuation))
    );
    assert_eq!(production.save().unwrap(), saved);
}

#[test]
fn room_mismatch_and_outside_muzzle_cannot_create_shots_or_requests() {
    let (mut production, _, _) = admit(state(128, 2));
    let world = world();
    let mut input = input();
    input.room.0.x = 2;
    let saved = production.save().unwrap();
    assert_eq!(
        production.advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &[]
        }),
        Err(ProductionFault::Unsupported(Unsupported::BeamWorld))
    );
    assert_eq!(production.save().unwrap(), saved);
    input.room.0.x = 1;
    input.mode = beam::UpdateMode::CannonAndShots;
    input.cannon.shoot = true;
    input.cannon.muzzle.aim = beam::MuzzleAim::Aimed(beam::Heading::Right);
    input.cannon.centre.0.x = 255;
    assert_eq!(
        production.advance(AdvanceInput::WeaponsContact {
            input: &input,
            world: &world,
            contacts: &[]
        }),
        Err(ProductionFault::Unsupported(Unsupported::BeamContinuation))
    );
    assert_eq!(production.save().unwrap(), saved);
}

#[test]
fn wrapping_flying_picture_refuses_the_entire_opportunity() {
    use crate::production::weapons::{FlyingDraw, Opportunity};
    use crate::state::video::{NmiBatch, SpriteCount};

    let (mut production, _, _) = admit(state(128, 2));
    let previous = production.save().unwrap();
    let batch = NmiBatch {
        sprites: SpriteCount(127),
        ..NmiBatch::default()
    };
    let result = production.advance(AdvanceInput::WeaponsContactDrawing {
        shot: Opportunity {
            input: &input(),
            world: &world(),
            contacts: &[opportunity(0, Some(132))],
        },
        drawing: FlyingDraw {
            phase: beam::FlickerPhase::Odd,
            view: &beam::DrawView::Normal(beam::ScreenOrigin(beam::Axes { x: 0, y: 0 })),
            batch: &batch,
        },
    });
    assert_eq!(
        result,
        Err(ProductionFault::Unsupported(Unsupported::BeamDrawing))
    );
    assert_eq!(production.save().unwrap(), previous);
}
