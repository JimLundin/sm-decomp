//! The air movements against values worked by hand from the bank $90 log's constants.

use data::movement::{MOVEMENT_TYPES, SpeedTable};
use data::pose::MovementType;

use super::*;
use crate::collision::{Surroundings, Touches};
use crate::movement::run;
use crate::state::samus::Fixed;
use crate::state::samus::Point;
use crate::state::slots::FlagSet;

/// The log's air column (`$90:9EA1`, `$90:9EB9` on) and a spin jump row of acceleration
/// 0.1000 and cap 1.0000 (a made-up row, the rest zero).
fn content() -> Content {
    let mut words = vec![0_u16; MOVEMENT_TYPES * 6];
    if let Some(entry) = words.as_chunks_mut::<6>().0.get_mut(3) {
        entry.copy_from_slice(&[0x0000, 0x1000, 0x0001, 0x0000, 0x0000, 0x8000]);
    }
    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let mut content = Content {
        vertical: VerticalSpeeds {
            gravity: Speed::new(0, 0x1C00),
            jump: Speed::new(4, 0xE000),
            hi_jump: Speed::new(6, 0),
            wall_jump: Speed::new(4, 0xA000),
            wall_hi_jump: Speed::new(5, 0x8000),
        },
        ..Content::default()
    };
    content.speeds.normal = SpeedTable::decode(&bytes);
    content
}

fn holding(buttons: &[Button]) -> Controller {
    Controller {
        held: FlagSet {
            bits: buttons.iter().fold(0, |bits, &button| bits | button.bit()),
            ..FlagSet::default()
        },
        ..Controller::default()
    }
}

const fn at(pixels: u16, subpixels: u16) -> Fixed {
    Fixed { pixels, subpixels }
}

/// Facing right in a normal jump past its take-off (pose $4D), at `speed` vertically.
fn jumping(speed: Speed, direction: YDirection) -> Samus {
    Samus {
        pose: PoseId(0x4D),
        facing: Facing::Right,
        movement: Movement::Jumping {
            jump: Jump::Normal,
            speed: Horizontal::default(),
            vertical: Vertical { speed, direction },
        },
        position: Point {
            x: at(0x0080, 0),
            y: at(0x0100, 0),
        },
        ..Samus::default()
    }
}

fn vertical(samus: &Samus) -> Vertical {
    samus.movement.vertical().unwrap_or_default()
}

fn equipping(item: Item) -> crate::state::samus::Items {
    FlagSet {
        bits: item.bit(),
        ..FlagSet::default()
    }
}

#[test]
fn a_jump_takes_its_initial_speed_from_the_items() {
    let speeds = content().vertical;
    let mut samus = jumping(Speed::ZERO, YDirection::None);
    make_jump(&mut samus, Takeoff::Jump, &speeds);
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(4, 0xE000),
            direction: YDirection::Up,
        }
    );
    samus.equipment.equipped = equipping(Item::HiJumpBoots);
    make_jump(&mut samus, Takeoff::WallJump, &speeds);
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0x8000));
}

#[test]
fn the_speed_booster_adds_half_the_run_speed_without_a_carry() {
    let speeds = content().vertical;
    let mut samus = jumping(Speed::ZERO, YDirection::None);
    samus.equipment.equipped = equipping(Item::SpeedBooster);
    if let Some(speed) = samus.movement.horizontal_mut() {
        speed.extra_run = Speed::new(3, 0x2000);
    }
    make_jump(&mut samus, Takeoff::Jump, &speeds);
    // Pixels 4 + 3 / 2 = 5; subpixels E000 + 2000 wrap to 0000 and carry nothing.
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0));
}

#[test]
fn a_rise_moves_by_the_frames_speed_then_gravity_slows_it() {
    let content = content();
    let jump = holding(&[Button::A]);
    let mut samus = jumping(Speed::new(4, 0xE000), YDirection::Up);
    run(
        &mut samus,
        &jump,
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    // $100.0000 - 4.E000 = $FB.2000; 4.E000 - 0.1C00 = 4.C400.
    assert_eq!(samus.position.y, at(0x00FB, 0x2000));
    assert_eq!(vertical(&samus).speed, Speed::new(4, 0xC400));
    run(
        &mut samus,
        &jump,
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(samus.position.y, at(0x00F6, 0x5C00));
    assert_eq!(vertical(&samus).speed, Speed::new(4, 0xA800));
}

#[test]
fn letting_go_of_jump_ends_the_rise() {
    let content = content();
    let mut samus = jumping(Speed::new(4, 0xE000), YDirection::Up);
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    // She falls from rest: no move this frame, then gravity's 0.1C00.
    assert_eq!(samus.position.y, at(0x0100, 0));
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(0, 0x1C00),
            direction: YDirection::Down,
        }
    );
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(samus.position.y, at(0x0100, 0x1C00));
}

#[test]
fn the_rise_ends_at_its_apex() {
    let content = content();
    let jump = holding(&[Button::A]);
    let mut samus = jumping(Speed::new(0, 0x1000), YDirection::Up);
    run(
        &mut samus,
        &jump,
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    // 0.1000 - 0.1C00 goes below zero for the rest of the frame...
    assert_eq!(vertical(&samus).speed, Speed(0xFFFF_F400));
    assert_eq!(samus.position.y, at(0x00FF, 0xF000));
    // ... and the next frame's check starts the fall from rest.
    run(
        &mut samus,
        &jump,
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(0, 0x1C00),
            direction: YDirection::Down,
        }
    );
}

#[test]
fn gravity_stops_adding_once_the_pixel_word_is_5() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0x29),
        movement: Movement::Falling {
            speed: Horizontal::default(),
            vertical: Vertical {
                speed: Speed::new(4, 0xF000),
                direction: YDirection::Down,
            },
        },
        ..jumping(Speed::ZERO, YDirection::None)
    };
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    // 4.F000 + 0.1C00 = 5.0C00: past 5, as the original tests the pixel word alone.
    assert_eq!(samus.position.y, at(0x0104, 0xF000));
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0x0C00));
    // At terminal speed the falling pose's animation goes to its frame 5.
    assert_eq!(samus.animation.frame, TERMINAL_FRAME);
    assert_eq!(samus.animation.timer, TERMINAL_DELAY);
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(vertical(&samus).speed, Speed::new(5, 0x0C00));
    assert_eq!(samus.position.y, at(0x0109, 0xFC00));
}

/// Facing right in a spin jump (pose $19), with no vertical direction, at base speed `base`.
fn spinning(base: Speed) -> Samus {
    Samus {
        pose: PoseId(0x19),
        movement: Movement::Jumping {
            jump: Jump::Spin,
            speed: Horizontal {
                base,
                ..Horizontal::default()
            },
            vertical: Vertical::default(),
        },
        ..jumping(Speed::ZERO, YDirection::None)
    }
}

#[test]
fn a_spin_jump_keeps_its_speed_only_forwards_or_at_the_cap() {
    let content = content();
    // Not steering: she loses her base speed and stays where she is.
    let mut samus = spinning(Speed::new(0, 0x8000));
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    let speed = samus.movement.horizontal().unwrap_or_default();
    assert_eq!(
        (speed.base, speed.acceleration),
        (Speed::ZERO, Acceleration::Accelerating)
    );
    assert_eq!(samus.position.x, at(0x0080, 0));
    // Steering forwards: 0.8000 + 0.1000, and the mode turns to decelerating.
    let mut samus = spinning(Speed::new(0, 0x8000));
    run(
        &mut samus,
        &holding(&[Button::Right]),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    let speed = samus.movement.horizontal().unwrap_or_default();
    assert_eq!(
        (speed.base, speed.acceleration),
        (Speed::new(0, 0x9000), Acceleration::Decelerating)
    );
    assert_eq!(samus.position.x, at(0x0080, 0x9000));
    // At the cap she keeps moving without steering: 0.F800 + 0.1000 caps to 1.0000.
    let mut samus = spinning(Speed::new(0, 0xF800));
    run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(samus.position.x, at(0x0081, 0));
    // With no direction the rise rule runs: it moves her by nothing and gravity takes from
    // it (the moonfall glitch's start).
    assert_eq!(vertical(&samus).speed, Speed(0xFFFF_E400));
}

#[test]
fn the_take_off_moves_her_by_her_run_speed_alone() {
    let content = content();
    let mut samus = Samus {
        pose: PoseId(0x4B),
        ..jumping(Speed::new(4, 0xE000), YDirection::Up)
    };
    if let Some(speed) = samus.movement.horizontal_mut() {
        speed.base = Speed::new(1, 0);
        speed.extra_run = Speed::new(0, 0x4000);
        speed.momentum = true;
    }
    run(
        &mut samus,
        &holding(&[Button::A]),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(
        samus.position,
        Point {
            x: at(0x0080, 0x4000),
            y: at(0x0100, 0),
        }
    );
    assert_eq!(vertical(&samus).speed, Speed::new(4, 0xE000));
    assert_eq!(samus.movement.kind(), MovementType::NormalJumping);
}

#[test]
fn a_fall_lands_her_on_the_floor() {
    use data::level::{BlockKind, Bts};
    let content = content();
    let mut blocks = vec![crate::state::room::Block::default(); 16 * 32];
    // The floor is row $11 under her column 8: its top is $110.
    if let Some(block) = blocks.get_mut(0x11 * 16 + 8) {
        block.kind = BlockKind::Solid(Bts(0));
    }
    let surroundings = Surroundings {
        room: crate::collision::Room {
            blocks: &blocks,
            width: crate::state::room::BlockColumns(16),
            slopes: None,
        },
        ..Surroundings::default()
    };
    let mut samus = jumping(Speed::new(4, 0), YDirection::Down);
    samus.movement = Movement::Falling {
        speed: Horizontal::default(),
        vertical: vertical(&samus),
    };
    samus.position.y = at(0x010D, 0);
    let contacts = run(
        &mut samus,
        &Controller::default(),
        &content,
        surroundings,
        &mut Touches::default(),
    );
    // With no Y radius her bottom is her Y less one: she stops at $110, on the floor's top.
    assert_eq!(contacts.vertical, crate::collision::VerticalContact::Landed);
    assert_eq!(samus.position.y, at(0x0110, 0xFFFF));
    let contacts = run(
        &mut samus,
        &Controller::default(),
        &content,
        Surroundings::default(),
        &mut Touches::default(),
    );
    assert_eq!(
        contacts.vertical,
        crate::collision::VerticalContact::Falling
    );
}

/// A level eight blocks across and 24 down of air, with `kind` at column 7 of row 16.
fn level_with(kind: data::level::BlockKind) -> Vec<crate::state::room::Block> {
    let mut blocks = vec![crate::state::room::Block::default(); 8 * 24];
    if let Some(block) = blocks.get_mut(16 * 8 + 7) {
        block.kind = kind;
    }
    blocks
}

fn room(blocks: &[crate::state::room::Block]) -> Surroundings<'_> {
    Surroundings {
        room: crate::collision::Room {
            blocks,
            width: crate::state::room::BlockColumns(8),
            slopes: None,
        },
        ..Surroundings::default()
    }
}

/// [`spinning`] with no base speed, rows 15 and 16 tall, from a spin jump's pose, its
/// animation on `frame`.
fn spun(frame: u16) -> Samus {
    let mut samus = spinning(Speed::ZERO);
    samus.y_radius = 0x0C;
    samus.last_different.movement = MovementType::SpinJumping;
    samus.animation.frame = AnimationFrame(frame);
    samus
}

#[test]
fn the_wall_jump_check_touches_an_item_eight_pixels_behind_her() {
    use crate::collision::Touch;
    use crate::state::room::BlockIndex;
    use data::level::{BlockKind, PlmChoice};
    let content = content();
    let blocks = level_with(BlockKind::Special(PlmChoice::Common(0x45)));
    // Holding right she moves to $80.1000; the check looks left from $80 - 8 - 5 = $73, in
    // column 7, which her move didn't reach.
    let mut samus = spun(3);
    let mut touches = Touches::default();
    let contacts = run(
        &mut samus,
        &holding(&[Button::Right]),
        &content,
        room(&blocks),
        &mut touches,
    );
    assert_eq!(touches.touched(), [Touch::Item(BlockIndex(16 * 8 + 7))]);
    assert_eq!(samus.position.x, at(0x0080, 0x1000));
    assert_eq!(samus.animation.frame, AnimationFrame(3));
    assert_ne!(contacts.vertical, VerticalContact::WallJumpTriggered);
    // Not from a spin, or holding no direction, she doesn't look.
    for (mut samus, controller) in [
        (
            Samus {
                last_different: Samus::default().last_different,
                ..spun(3)
            },
            holding(&[Button::Right]),
        ),
        (spun(3), Controller::default()),
    ] {
        let mut touches = Touches::default();
        run(
            &mut samus,
            &controller,
            &content,
            room(&blocks),
            &mut touches,
        );
        assert_eq!(touches.touched(), []);
    }
}

#[test]
fn a_wall_behind_her_readies_the_spin_then_triggers_the_wall_jump() {
    use data::level::{BlockKind, Bts};
    let content = content();
    let blocks = level_with(BlockKind::Solid(Bts(0)));
    // Still spinning: the animation goes to the frame before the wall jump's.
    let mut samus = spun(3);
    run(
        &mut samus,
        &holding(&[Button::Right]),
        &content,
        room(&blocks),
        &mut Touches::default(),
    );
    assert_eq!(
        (samus.animation.frame, samus.animation.timer),
        (AnimationFrame(0x0A), AnimationTimer(1))
    );
    // Eligible and newly pressing jump: the wall, 0 pixels away once the check puts her X
    // subpixels at the block's edge, triggers the wall jump and she doesn't move vertically.
    let mut samus = spun(0x0B);
    if let Some(vertical) = samus.movement.vertical_mut() {
        vertical.speed = Speed::new(1, 0);
    }
    let mut jump = holding(&[Button::Right, Button::A]);
    jump.pressed.bits = Button::A.bit();
    let contacts = run(
        &mut samus,
        &jump,
        &content,
        room(&blocks),
        &mut Touches::default(),
    );
    assert_eq!(contacts.vertical, VerticalContact::WallJumpTriggered);
    assert_eq!(samus.position.y, at(0x0100, 0));
    // Without the press she only moves on.
    let mut samus = spun(0x0B);
    let contacts = run(
        &mut samus,
        &holding(&[Button::Right]),
        &content,
        room(&blocks),
        &mut Touches::default(),
    );
    assert_ne!(contacts.vertical, VerticalContact::WallJumpTriggered);
}

/// Falling at 4.0 a frame from Y `y`.`subpixels`, radius $10, over an enemy centred at
/// ($80, $120): its top is $118.
fn falling_onto(y: u16, subpixels: u16, solid: bool) -> (Samus, crate::collision::Contacts) {
    use crate::collision::solid_enemy::{FIRST, SolidEnemies, one_enemy};
    let content = content();
    let slots = one_enemy(0x80, 0x120, solid);
    let surroundings = Surroundings {
        enemies: SolidEnemies {
            interactive: &FIRST,
            slots: &slots,
        },
        ..Surroundings::default()
    };
    let mut samus = jumping(Speed::new(4, 0), YDirection::Down);
    samus.movement = Movement::Falling {
        speed: Horizontal::default(),
        vertical: vertical(&samus),
    };
    samus.y_radius = 0x10;
    samus.position.y = at(y, subpixels);
    let contacts = run(
        &mut samus,
        &Controller::default(),
        &content,
        surroundings,
        &mut Touches::default(),
    );
    (samus, contacts)
}

#[test]
fn a_fall_onto_a_solid_enemy_is_cut_to_its_top_and_lands_her() {
    // Her bottom, $105 + $10, is 3 pixels above the enemy's top: the 4-pixel fall is cut to
    // 3, with no fraction, and no block is looked for.
    let (samus, contacts) = falling_onto(0x0105, 0x4000, true);
    assert_eq!(contacts.vertical, VerticalContact::Landed);
    assert_eq!(samus.position.y, at(0x0108, 0x4000));
    // An enemy that isn't solid lets her fall the whole way.
    let (samus, contacts) = falling_onto(0x0105, 0x4000, false);
    assert_eq!(contacts.vertical, VerticalContact::Falling);
    assert_eq!(samus.position.y, at(0x0109, 0x4000));
}

#[test]
fn touching_a_solid_enemy_below_lands_her_where_she_is_and_zeroes_her_y_subpixels() {
    let (samus, contacts) = falling_onto(0x0108, 0x8000, true);
    assert_eq!(contacts.vertical, VerticalContact::Landed);
    assert_eq!(samus.position.y, at(0x0108, 0));
}

#[test]
fn a_rise_into_a_solid_enemy_is_cut_to_its_bottom_and_hits_the_ceiling() {
    use crate::collision::solid_enemy::{FIRST, SolidEnemies, one_enemy};
    let content = content();
    // The enemy's bottom is $E0 + 8 = $E8; her top, $100 - $10 = $F0, is 8 below it.
    let slots = one_enemy(0x80, 0xE0, true);
    let surroundings = Surroundings {
        enemies: SolidEnemies {
            interactive: &FIRST,
            slots: &slots,
        },
        ..Surroundings::default()
    };
    let mut samus = jumping(Speed::new(0x0A, 0), YDirection::Up);
    samus.y_radius = 0x10;
    let contacts = run(
        &mut samus,
        &holding(&[Button::A]),
        &content,
        surroundings,
        &mut Touches::default(),
    );
    assert_eq!(contacts.vertical, VerticalContact::HitCeiling);
    assert_eq!(samus.position.y, at(0x00F8, 0));
}

#[test]
fn a_solid_enemy_behind_her_is_the_wall_the_wall_jump_check_finds() {
    use crate::collision::solid_enemy::{FIRST, SolidEnemies, one_enemy};
    let content = content();
    // Holding right she looks left: the enemy's right edge, $70 + 8 = $78, is 3 pixels from
    // her left side at $80 - 5 = $7B, with no block in the room.
    let slots = one_enemy(0x70, 0x100, true);
    let surroundings = Surroundings {
        enemies: SolidEnemies {
            interactive: &FIRST,
            slots: &slots,
        },
        ..Surroundings::default()
    };
    let mut samus = spun(3);
    run(
        &mut samus,
        &holding(&[Button::Right]),
        &content,
        surroundings,
        &mut Touches::default(),
    );
    assert_eq!(
        (samus.animation.frame, samus.animation.timer),
        (AnimationFrame(0x0A), AnimationTimer(1))
    );
    let mut samus = spun(0x0B);
    let mut jump = holding(&[Button::Right, Button::A]);
    jump.pressed.bits = Button::A.bit();
    let contacts = run(
        &mut samus,
        &jump,
        &content,
        surroundings,
        &mut Touches::default(),
    );
    assert_eq!(contacts.vertical, VerticalContact::WallJumpTriggered);
}

fn morphing(pose: u16, speed: Speed, direction: YDirection) -> Samus {
    Samus {
        pose: PoseId(pose),
        movement: Movement::Transition {
            speed: Horizontal::default(),
            vertical: Vertical { speed, direction },
        },
        position: Point {
            x: at(0x0080, 0),
            y: at(0x0100, 0),
        },
        ..Samus::default()
    }
}

fn step(samus: &mut Samus, surroundings: Surroundings<'_>) -> crate::collision::Contacts {
    run(
        samus,
        &Controller::default(),
        &content(),
        surroundings,
        &mut Touches::default(),
    )
}

#[test]
fn a_transition_keeps_falling_by_its_y_speed() {
    // `$90:A61C`: the morphing transition (pose $37) runs the simple Y movement, so the fall
    // goes on: this frame's speed moves her, then gravity adds to it.
    let mut samus = morphing(0x37, Speed::new(2, 0x4C00), YDirection::Down);
    step(&mut samus, Surroundings::default());
    assert_eq!(samus.position.y, at(0x0102, 0x4C00));
    assert_eq!(
        vertical(&samus),
        Vertical {
            speed: Speed::new(2, 0x6800),
            direction: YDirection::Down,
        }
    );
}

#[test]
fn a_transition_with_no_y_direction_does_not_move_vertically() {
    let mut samus = morphing(0x37, Speed::new(2, 0), YDirection::None);
    step(&mut samus, Surroundings::default());
    assert_eq!(samus.position.y, at(0x0100, 0));
}

#[test]
fn the_unused_transition_poses_have_no_y_movement() {
    let mut samus = morphing(0xDB, Speed::new(2, 0), YDirection::Down);
    step(&mut samus, Surroundings::default());
    assert_eq!(samus.position.y, at(0x0100, 0));
    assert_eq!(vertical(&samus).speed, Speed::new(2, 0));
}

#[test]
fn landing_in_a_transition_stops_the_y_speed() {
    use data::level::{BlockKind, Bts};
    let mut blocks = vec![crate::state::room::Block::default(); 16 * 32];
    if let Some(block) = blocks.get_mut(0x11 * 16 + 8) {
        block.kind = BlockKind::Solid(Bts(0));
    }
    let surroundings = Surroundings {
        room: crate::collision::Room {
            blocks: &blocks,
            width: crate::state::room::BlockColumns(16),
            slopes: None,
        },
        ..Surroundings::default()
    };
    let mut samus = morphing(0x37, Speed::new(4, 0), YDirection::Down);
    samus.position.y = at(0x010D, 0);
    let contacts = step(&mut samus, surroundings);
    assert_eq!(contacts.vertical, crate::collision::VerticalContact::Landed);
    assert_eq!(vertical(&samus), Vertical::default());
}
