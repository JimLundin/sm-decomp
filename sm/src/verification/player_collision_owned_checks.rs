//! Candidate-owned invariants. Register under crate::game for real owner access.
//! No Game, RAM facade or copied collision algorithm is used by generic checks.
#![forbid(unsafe_code)]
use crate::game::{
    actors::access::ActorKey,
    launch_motion::MovementDirection,
    player_collision::{
        effects::Host,
        operations::{self, Direction, DisplacementRequest, Obstruction},
    },
    visual_host,
    visual_state::PaletteCarry,
};
use std::cell::RefCell;

std::thread_local! {
    static LAUNCHES: RefCell<Option<Vec<(u8, i32)>>> = const { RefCell::new(None) };
}

// Called only by a cfg-gated hook in the actual LaunchMotion writer.
// Dormant unless a check owns a recording scope; never changes gameplay state.
pub(crate) fn record_launch(direction: MovementDirection, displacement: i32) {
    LAUNCHES.with(|slot| {
        if let Some(events) = slot.borrow_mut().as_mut() {
            events.push((direction as u8, displacement));
        }
    });
}
struct Recording;
impl Drop for Recording {
    fn drop(&mut self) {
        LAUNCHES.with(|slot| *slot.borrow_mut() = None);
    }
}
pub(crate) fn observed<R>(action: impl FnOnce() -> R) -> (R, Vec<(u8, i32)>) {
    LAUNCHES.with(|slot| {
        let mut active = slot.borrow_mut();
        assert!(active.is_none(), "collision.owned.observer_not_nested");
        *active = Some(Vec::new());
    });
    let scope = Recording;
    let result = action();
    let events = LAUNCHES.with(|slot| slot.borrow_mut().take().unwrap());
    drop(scope);
    (result, events)
}

// One owner, four actual operations; sentinel directions expose extra writes.
pub(crate) fn unchecked_launch<H: Host>(h: &mut H) {
    let cases = [
        (
            Direction::Right,
            1u8,
            0x0001_8001i32,
            (0x0102u16, 0x0000u16),
        ),
        (Direction::Left, 0, -0x0001_8001, (0x00fe, 0xfffe)),
        (Direction::Down, 3, 0x0001_8001, (0x0102, 0x0000)),
        (Direction::Up, 2, -0x0001_8001, (0x00fe, 0xfffe)),
    ];
    for (direction, lane, signed, expected) in cases {
        {
            let p = &mut h.session_mut().player;
            p.x = 0x0100;
            p.y = 0x0100;
            p.x_fraction = 0x7fff;
            p.y_fraction = 0x7fff;
            p.collision.flags = 0xa55a;
        }
        let (result, events) = observed(|| {
            operations::displace(
                h,
                DisplacementRequest::Unchecked {
                    direction,
                    amount: 0x0001_8001,
                },
            )
        });
        assert_eq!(
            events,
            [(lane, signed)],
            "collision.owned.unchecked_launch_exact_once"
        );
        assert_eq!(
            result.obstruction,
            Obstruction::Unchecked,
            "collision.owned.unchecked_policy"
        );
        let p = &h.session().player;
        let (moved, untouched) = if lane < 2 {
            ((p.x, p.x_fraction), (p.y, p.y_fraction))
        } else {
            ((p.y, p.y_fraction), (p.x, p.x_fraction))
        };
        assert_eq!(moved, expected, "collision.owned.unchecked_fixed_point");
        assert_eq!(
            untouched,
            (0x0100, 0x7fff),
            "collision.owned.unchecked_other_axis"
        );
        assert_eq!(
            p.collision.flags, 0xa55a,
            "collision.owned.unchecked_preserves_contact"
        );
        assert_eq!(
            h.session().projectiles.launch.displacements[usize::from(lane)],
            signed,
            "collision.owned.launch_value_matches_event"
        );
    }
}

// Caller supplies one prepared actual actor at (120,100), radius (2,2).
// Narrow/wide/narrow checks all use the same owner and never refresh inputs.
#[cfg(sm_verify_dll)]
pub(crate) fn same_interval_radius<H: sm_game::actors::access::ActorAccess>(
    h: &mut H,
    actor: ActorKey,
) {
    {
        let p = &mut h.session_mut().player;
        p.x = 100;
        p.y = 100;
        p.contact.radius = 2;
    }
    for (radius, expected) in [(1, false), (18, true), (1, false)] {
        h.session_mut().player.contact.x_radius = radius;
        assert_eq!(
            crate::game::actors::queries::overlaps_player_with_padding(h, actor),
            expected,
            "collision.owned.radius_live_actor_consumer"
        );
    }
}

// Actual palette phase consumes the live clock, without content reads here.
pub(crate) fn same_interval_frame<H: visual_host::Host>(h: &mut H) {
    {
        let s = h.session_mut();
        s.presentation.palette_mut().suit_index = 4;
        s.player.movement_and_facing = 3 << 8;
        s.inventory.equipment.active |= 8;
    }
    for (frame, expected_carry, expected_palette) in [
        (0, PaletteCarry::Set, 0),
        (27, PaletteCarry::Clear, 8),
        (0, PaletteCarry::Set, 0),
    ] {
        h.session_mut().player.animation_clock.frame = frame;
        h.session_mut().player.palette_frame = 8;
        assert_eq!(
            sm_game::samus_palette::screw_speed(h),
            expected_carry,
            "collision.owned.frame_live_palette_decision"
        );
        assert_eq!(
            h.session().player.palette_frame,
            expected_palette,
            "collision.owned.frame_live_palette_publication"
        );
    }
}

// Probe-only caller must remain responsible for position publication.
// The fixture supplies valid authored empty terrain and no interactive actors.
pub(crate) fn pose_probe_does_not_commit<H: Host>(h: &mut H) {
    let before = {
        let p = &h.session().player;
        (p.x, p.x_fraction, p.y, p.y_fraction)
    };
    let (_result, events) = observed(|| {
        operations::displace(
            h,
            DisplacementRequest::PoseClearanceProbe {
                amount: 0x0001_0000,
            },
        )
    });
    let p = &h.session().player;
    assert_eq!(
        (p.x, p.x_fraction, p.y, p.y_fraction),
        before,
        "collision.owned.pose_probe_no_coordinate_commit"
    );
    assert_eq!(events, [], "collision.owned.pose_probe_no_launch");
}

// Same host interval, actual actor-first horizontal core. Fixture actor is solid,
// at (120,100), radii (2,2), and the actual interactive list contains its key.
pub(crate) fn same_interval_collision_radius<H: Host>(h: &mut H, actor: ActorKey) {
    use operations::{HorizontalDirection, HorizontalRequest};
    for (radius, blocked) in [(1, false), (18, true), (1, false)] {
        {
            let p = &mut h.session_mut().player;
            p.x = 100;
            p.y = 100;
            p.x_fraction = 0;
            p.y_fraction = 0;
            p.contact.x_radius = radius;
            p.contact.radius = 2;
            p.contact.base_run = Default::default();
            p.contact.extra_run = Default::default();
            p.contact.external_x = Default::default();
            p.contact.acceleration_mode = 0;
            p.contact.momentum = 0;
            p.contact.speed_boost = 0;
            p.collision.horizontal_slope_flags = 0;
        }
        let result = operations::horizontal(
            h,
            HorizontalRequest {
                base: 4 << 16,
                direction: HorizontalDirection::Right,
            },
        );
        assert_eq!(
            result.obstruction,
            if blocked {
                Obstruction::SolidActor(actor)
            } else {
                Obstruction::Clear
            },
            "collision.owned.radius_live_collision_consumer"
        );
    }
}

// Actual shinespark Y consumes the same shared sequence on successive calls.
// Fixture supplies valid empty terrain and an empty actual interactive list.
pub(crate) fn shared_sequence_move_y<H: Host>(h: &mut H) {
    use operations::Axis;
    {
        let s = h.session_mut();
        s.shared_sequence.stage = 0;
        s.shared_sequence.value = 0x4000;
        s.player.x = 100;
        s.player.y = 100;
        s.player.y_fraction = 0;
        s.player.previous_position.y.whole = 100;
        s.player.previous_position.y.fraction = 0;
        s.player.contact.y_acceleration = Default::default();
        s.player.contact.y_speed = Default::default();
        s.player.contact.external_y = Default::default();
    }
    for (speed, position) in [(0x4000, (99, 0xc000)), (0x8000, (99, 0x4000))] {
        operations::shinespark(h, Axis::Y);
        let s = h.session();
        assert_eq!(
            (s.shared_sequence.stage, s.shared_sequence.value),
            (0, 0x4000),
            "collision.owned.sequence_shared_after_move"
        );
        assert_eq!(
            (
                s.player.contact.y_speed.whole,
                s.player.contact.y_speed.fraction
            ),
            (0, speed),
            "collision.owned.sequence_live_accumulation"
        );
        assert_eq!(
            (s.player.y, s.player.y_fraction),
            position,
            "collision.owned.sequence_real_move"
        );
    }
}

#[cfg(sm_verify_dll)]
pub(crate) fn foreign_sequence_then_move(game: &mut crate::game_state::Game) {
    use crate::game::session_runtime::Runtime;
    use operations::Axis;
    let mut rt = Runtime::new(game);
    {
        let s = rt.session_mut();
        s.shared_sequence.stage = 0;
        s.shared_sequence.value = 0x805a;
        s.player.x = 100;
        s.player.y = 100;
        s.player.y_fraction = 0;
        s.player.previous_position.y.whole = 100;
        s.player.previous_position.y.fraction = 0;
        s.player.contact.y_acceleration = Default::default();
        s.player.contact.y_speed = Default::default();
        s.player.contact.external_y = Default::default();
    }
    rt.foreign(|g| {
        assert_eq!(
            &{
                let mut bytes = [0; 2];
                g.ram().copy_out(0xdee, &mut bytes);
                bytes
            },
            &[0x5a, 0x80],
            "collision.owned.foreign_published_sequence"
        );
        g.ram().write_byte(0xdef, 0x40);
    });
    operations::shinespark(&mut rt, Axis::Y);
    let s = rt.session();
    assert_eq!(
        s.shared_sequence.value, 0x405a,
        "collision.owned.foreign_partial_sequence_import"
    );
    assert_eq!(
        (
            s.player.contact.y_speed.whole,
            s.player.contact.y_speed.fraction
        ),
        (0, 0x405a),
        "collision.owned.foreign_partial_real_consumer"
    );
    assert_eq!(
        (s.player.y, s.player.y_fraction),
        (99, 0xbfa6),
        "collision.owned.foreign_partial_real_move"
    );
}

// Actual owned momentum-action witness. Invocation calls the production action
// through Runtime so collision and transition state share one owner.
// Initialized fixture supplies empty room terrain; native pose radius is authored.
#[cfg(sm_verify_dll)]
pub(crate) fn transition_a7_single_commit(
    g: &mut crate::game_state::Game,
    invoke: impl FnOnce(&mut crate::game_state::Game),
) {
    use crate::rom::{RomImage, lorom_offset};
    let image = RomImage::zeroed(0x200000);
    image.write(lorom_offset(0x91b629) as usize + 0xf1 * 8 + 6, 8);
    g.register_native_rom(image);
    for (at, value) in [
        (0xa1c, 0x00f1u16),
        (0xaf6, 100),
        (0xafa, 100),
        (0xafc, 0x7fff),
        (0xb00, 3),
        (0xb14, 77),
        (0xb16, 0x55aa),
        (0xb20, 1),
        (0xb2c, 0x1234),
        (0xb2e, 7),
        (0xb36, 2),
    ] {
        g.ram().copy_in(at, &value.to_le_bytes());
    }
    let (_, launches) = observed(|| invoke(g));
    let word = |at: usize| {
        u16::from_le_bytes({
            let mut bytes = [0; 2];
            g.ram().copy_out(at, &mut bytes);
            bytes
        })
    };
    assert_eq!(
        (word(0xafa), word(0xafc)),
        (105, 0x7fff),
        "collision.owned.transition_a7_single_coordinate_commit"
    );
    assert_eq!(
        (word(0xb14), word(0xb16)),
        (105, 0x55aa),
        "collision.owned.transition_a7_previous_y"
    );
    assert_eq!(
        word(0xb00),
        8,
        "collision.owned.transition_a7_native_radius"
    );
    assert_eq!(
        (word(0xb20), word(0xb2c), word(0xb2e), word(0xb36)),
        (0, 0, 0, 0),
        "collision.owned.transition_a7_bounce_reset"
    );
    assert_eq!(launches, [], "collision.owned.transition_a7_no_launch");
}

pub(crate) fn launches_so_far() -> Vec<(u8, i32)> {
    LAUNCHES.with(|slot| {
        slot.borrow()
            .as_ref()
            .expect("collision launch observation must be active")
            .clone()
    })
}

// Root verification cannot name game-local ActorKey. Keep that construction
// here without widening the production actor API or refreshing external inputs.
#[cfg(sm_verify_dll)]
pub(crate) fn same_interval_radius_runtime(g: &mut crate::game_state::Game) {
    let mut runtime = crate::game::session_runtime::Runtime::new(g);
    same_interval_radius(&mut runtime, ActorKey::from_legacy(0));
}
