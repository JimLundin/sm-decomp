//! Synthetic contract regressions; no source packet or native selected state is used.
use super::super::{hud::Hud, movement::Movement, movement_stage};
use super::*;
use crate::production::timing::WorkId;
use core::{cell::Cell, num::Wrapping};
use types::{Pair, Set};

fn movement_owner() -> Movement {
    Movement::new(samus_movement::Samus {
        position: Pair {
            x: Wrapping(100 * 65536),
            y: Wrapping(100 * 65536),
        },
        pose: types::Pose {
            body: types::Body::Upright,
            action: types::Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
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
    })
}

fn content_with_width(width: u16) -> MovementContent {
    MovementContent::new(
        data::assets::samus::map::MAP
            .iter()
            .map(|entry| crate::production::PoseContent {
                pose: entry.key,
                radius: Pair {
                    x: Wrapping(width),
                    y: Wrapping(21),
                },
                cannon_heading: None,
            })
            .collect(),
    )
    .unwrap()
}

fn frame() -> samus_movement::Frame<'static> {
    samus_movement::Frame {
        sprite_before_movement: types::Sprite::Plain,
        cinematic_active: false,
        pad: types::Buttons::default(),
        pressed: types::Buttons::default(),
        items: Set::default(),
        spin_turned_far: false,
        elevator_moving: false,
        parity: types::Parity::Even,
        animation_ends: None,
        body: &|_| Pair {
            x: Wrapping(5),
            y: Wrapping(21),
        },
        cannon_heading: &|_| None,
    }
}

struct World {
    queries: Cell<u32>,
    blocked: Cell<bool>,
}
impl types::World for World {
    fn block(&self, _: types::Cell) -> types::Block {
        self.queries.set(self.queries.get().checked_add(1).unwrap());
        if self.blocked.get() {
            types::Block::Solid
        } else {
            types::Block::Air
        }
    }
    fn lets_through(&self, _: types::Cell, _: types::Way) -> bool {
        false
    }
    fn solid_enemies(&self) -> &[types::Hitbox] {
        self.queries.set(self.queries.get().checked_add(1).unwrap());
        &[]
    }
}

struct Fixture {
    samus: samus_movement::Samus,
    content: MovementContent,
    camera_view: Pair<u16>,
    inside: movement_stage::Completion,
    controls: super::super::hud_stage::ControlsCompletion,
    motion: super::super::MotionResetCompletion,
    membership: ActorEligibilityReceipt,
    interaction: SamusInteractionCompletion,
    offer: DeferredActorUpdateReceipt,
}

fn completed_controls() -> super::super::hud_stage::ControlsCompletion {
    use interface_hud::{
        Ammunition, CancelButton, Equipped, HudGrid, HudState, MapCellSet, MiniMapPower,
        SelectButton, Selection, Shown, Stock,
    };
    let mut hud = Hud::new(HudState {
        countdown: interface_hud::Countdown::Dormant,
        grid: HudGrid::filled(interface_hud::HudPictures::default().blank_cell()),
        shown: Shown::default(),
        selection: Selection::None,
        explored: MapCellSet::default(),
        mini_map: MiniMapPower::On,
    });
    super::super::hud_stage::choose_controls(
        &mut hud,
        interface_hud::ItemControlInputs {
            select: SelectButton::NotPressed,
            cancel: CancelButton::NotPressed,
            can_switch: true,
            ammunition: Ammunition::new(
                enum_map::enum_map! { _ => Stock { count: 0, capacity: 11 } },
            )
            .unwrap(),
            equipped: Equipped {
                grapple_beam: false,
                x_ray_scope: false,
            },
        },
    )
    .unwrap();
    super::super::hud_stage::complete_controls_receipt(&mut hud, WorkId(13)).unwrap()
}

fn actor_membership() -> ActorEligibilityReceipt {
    let mut slots = [super::super::borrowed::ActorSlot::Empty; 32];
    for (index, intangible) in [(3, true), (9, false)] {
        slots[index] = super::super::borrowed::ActorSlot::Occupied {
            deleted: false,
            process_offscreen: true,
            intangible,
        };
    }
    ActorEligibilityReceipt {
        dependency: super::super::borrowed::ActorEligibilityOrigin {
            permit_marked_offscreen: true,
            slots,
        },
        active: vec![ActorSlotIdentity(3), ActorSlotIdentity(9)],
        interactive: vec![ActorSlotIdentity(9)],
    }
}

fn actor_offer(activity: SamusActivity) -> DeferredActorUpdateReceipt {
    let seed = |touch| ActorSeed {
        content: ActorContentDisposition::OrdinaryCeresDoor,
        touch,
        solidity: ActorSolidity::Solid,
        geometry: ActorGeometry {
            position: Pair {
                x: Wrapping(200 * 65536),
                y: Wrapping(if touch == ActorTouch::Intangible {
                    100 * 65536
                } else {
                    400 * 65536
                }),
            },
            radius: Pair {
                x: Wrapping(5),
                y: Wrapping(21),
            },
        },
        frame_counter: AdministrativeCounter(u16::MAX),
        instruction_timer: 8,
        graphics_updated: GraphicsUpdated::Set,
        flash_timer: 0,
    };
    let dependency = ActorUnselectedOrigin {
        actors: [seed(ActorTouch::Intangible), seed(ActorTouch::Tangible)],
        clocks: CollectionClocks {
            simulation: ClockDisposition::Running,
            enemy: ClockDisposition::Running,
        },
        scene: ActorScene::PreEscape,
        shake_requested: false,
        contact: OrdinaryActorContact {
            actor_invincibility: 0,
            samus_invincibility: 0,
            additional_gate: AdditionalContactGate::None,
            damage_mode: ContactDamageDisposition::Ordinary,
            sprite_interaction: InteractionDisposition::Enabled,
            activity,
        },
    };
    let effect = |actor: &ActorSeed| ActorEffects {
        geometry: actor.geometry,
        frame_counter: CounterEffect::WrappingIncrementOne,
        instruction_timer: 7,
        graphics_updated: GraphicsUpdated::Clear,
    };
    DeferredActorUpdateReceipt {
        required: ActorRequiredDependencies {
            samus: SamusContactGeometry {
                centre_x: Wrapping(100),
                radius_x: Wrapping(5),
            },
            camera_view: Pair { x: 0, y: 0 },
            active_order: [ActorSlotIdentity(3), ActorSlotIdentity(9)],
            interactive_order: vec![ActorSlotIdentity(9)],
            views: [
                ActorViewDisposition::WithinPaddedViewButInvisible,
                ActorViewDisposition::BelowPaddedView,
            ],
        },
        terminal: ActorUpdateTerminal {
            actors: [effect(&dependency.actors[0]), effect(&dependency.actors[1])],
        },
        dependency,
    }
}

impl Fixture {
    fn new() -> Self {
        let mut movement = movement_owner();
        let samus = *movement.state();
        let content = content_with_width(5);
        let rules = samus_movement::Rules::default();
        let frame = frame();
        let world = World {
            queries: Cell::new(0),
            blocked: Cell::new(false),
        };
        let inputs = movement_stage::Inputs {
            content: &content,
            rules: &rules,
            frame: &frame,
            world: &world,
        };
        movement.begin().unwrap();
        let mut stage = movement_stage::Stage::default();
        stage
            .choose(movement_stage::Group::Inside, &mut movement, &inputs)
            .unwrap();
        stage.bind(WorkId(12)).unwrap();
        let inside = stage.complete(WorkId(12), &mut movement, &inputs).unwrap();
        let controls = completed_controls();
        // Synthetic current-only receipts, separate from the actual Inside/HUD completion APIs.
        let motion = super::super::MotionResetCompletion {
            clip_start: super::super::motion_reset_contract::ClipStartPreparation::Cleared,
            work: WorkId(14),
            previous: canonical::model::Model(samus),
        };
        let membership = actor_membership();
        let activity = SamusActivity {
            projectiles: 0,
            bombs: 0,
        };
        let interaction = SamusInteractionCompletion {
            dependency: SamusInteractionOrigin {
                sprite_interaction: InteractionDisposition::Enabled,
                projectile_interaction: InteractionDisposition::Enabled,
                activity,
            },
            terminal: SamusInteractionTerminal {
                sprite_interaction: InteractionDisposition::Enabled,
                projectile_interaction: InteractionDisposition::Enabled,
                activity,
            },
        };
        let offer = actor_offer(activity);
        Self {
            samus,
            content,
            camera_view: Pair { x: 0, y: 0 },
            inside,
            controls,
            motion,
            membership,
            interaction,
            offer,
        }
    }

    fn selected(&self) -> CompletedPlayerSelection<'_> {
        CompletedPlayerSelection {
            inside_hud: CompletedInsideAndHudWeapons {
                inside: &self.inside,
                controls: &self.controls,
            },
            motion_reset: &self.motion,
        }
    }

    fn view(&self) -> ActorView<'_> {
        ActorView {
            samus: &self.samus,
            content: &self.content,
            camera: self.camera_view,
        }
    }

    fn validate(&self) -> Result<(), PlayerPreparationFault> {
        self.offer.validate_dependencies(
            &self.offer.dependency,
            &self.selected(),
            self.view(),
            &self.membership,
            &self.interaction,
        )
    }
}

#[test]
fn actor_admission_validates_without_publishing_then_completion_binds_actual_membership() {
    let fixture = Fixture::new();
    let retained = fixture.offer.clone();
    assert!(fixture.validate().is_ok());
    assert!(fixture.offer.eq(&retained));
    let completed = fixture
        .offer
        .clone()
        .complete(
            &fixture.offer.dependency,
            &fixture.selected(),
            fixture.view(),
            &fixture.membership,
            &fixture.interaction,
        )
        .unwrap();
    assert!(
        completed
            .actors
            .iter()
            .map(|actor| actor.identity)
            .collect::<Vec<_>>()
            .eq(&fixture.membership.active)
    );
    assert!(completed.interactive.eq(&fixture.membership.interactive));
    assert!(
        completed
            .actors
            .iter()
            .all(|actor| actor.frame_counter.0 == 0
                && actor.instruction_timer == 7
                && actor.graphics_updated == GraphicsUpdated::Clear)
    );
    assert_eq!(completed.solid_boxes().unwrap().len(), 1);
}

#[test]
fn actor_admission_rejects_late_selected_centre_body_view_and_membership_changes() {
    let reject = |fixture: Fixture| {
        assert!(
            fixture
                .validate()
                .is_err_and(|fault| fault == PlayerPreparationFault::Dependency)
        );
        assert!(
            fixture
                .offer
                .clone()
                .complete(
                    &fixture.offer.dependency,
                    &fixture.selected(),
                    fixture.view(),
                    &fixture.membership,
                    &fixture.interaction
                )
                .is_err_and(|fault| fault == PlayerPreparationFault::Dependency)
        );
    };
    let mut fixture = Fixture::new();
    fixture.samus.position.x += Wrapping(65536);
    fixture.motion.previous.0 = fixture.samus;
    reject(fixture);
    let mut fixture = Fixture::new();
    fixture.content = content_with_width(6);
    reject(fixture);
    let mut fixture = Fixture::new();
    fixture.camera_view.x = 1;
    reject(fixture);
    let mut fixture = Fixture::new();
    fixture.offer.required.views.swap(0, 1);
    reject(fixture);
    let mut fixture = Fixture::new();
    fixture.membership.active[0] = ActorSlotIdentity(4);
    reject(fixture);
    let mut fixture = Fixture::new();
    fixture.offer.required.active_order.swap(0, 1);
    reject(fixture);
    let mut fixture = Fixture::new();
    fixture.membership.interactive = vec![ActorSlotIdentity(3)];
    reject(fixture);
}

#[test]
fn carried_doors_poll_after_restore_and_reject_stale_position_or_timer() {
    use super::super::actor_doors::Doors;
    use boss_ceres_ridley::{CeresDoor, DoorKind, Phase, Wait};
    use types::{Parts, Side};

    let fixture = Fixture::new();
    let mut current = fixture.offer.dependency.clone();
    let [first, second] = &mut current.actors;
    first.instruction_timer = 2;
    second.instruction_timer = 2;
    second.geometry.position.x = Wrapping(600 * 65536);
    let wait = Wait::new(2).unwrap();
    let door = |phase, actor: &ActorSeed| CeresDoor {
        kind: DoorKind::Door,
        facing: Side::Left,
        position: Pair {
            x: actor.geometry.position.x.pixel(),
            y: actor.geometry.position.y.pixel(),
        },
        phase,
    };
    let [a, b] = &current.actors;
    let doors = Doors::new(
        [
            (ActorSlotIdentity(3), door(Phase::Open { wait }, a)),
            (ActorSlotIdentity(9), door(Phase::Closed { wait }, b)),
        ],
        &current.actors,
        [true, false],
    )
    .unwrap();
    let views = fixture.offer.required.views;
    let membership = &fixture.membership;
    let near = Pair {
        x: Wrapping(210),
        y: Wrapping(100),
    };
    let far = Pair {
        x: Wrapping(400),
        y: Wrapping(100),
    };
    let (waited, first_window) = doors.run(&current, views, membership, far, 123).unwrap();
    assert!(waited.matches(&first_window));
    assert!(first_window.actors.iter().all(|actor| {
        actor.instruction_timer == 1 && actor.graphics_updated == GraphicsUpdated::Clear
    }));
    let restored: Doors = canonical::from_bytes(&canonical::to_bytes(&waited)).unwrap();
    assert_eq!(restored, waited);
    assert!(
        restored
            .run(&current, views, membership, far, 456)
            .is_none()
    );
    let (polled, second_window) = restored
        .run(&current, views, membership, near, 456)
        .unwrap();
    assert!(polled.matches(&second_window));
    assert!(second_window.actors.iter().all(|actor| {
        actor.instruction_timer == 2
            && actor.graphics_updated == GraphicsUpdated::Set
            && actor.frame_counter == AdministrativeCounter(1)
    }));
    assert!(!doors.matches(&first_window));
    assert!(!restored.matches(&second_window));
}
