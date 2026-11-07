use clean_types::{Aim, Body, Facing};
use data::pose::PoseId;

use super::{
    Action, Dress, Gun, LandedFrom, Legs, MAP, Pose, Shown, SpinSprite, Suit, TurnKind, id_of,
    pose_of,
};

/// The engine's pose ids: `00h` to `FCh`.
const IDS: u16 = 0xFD;

const DRESSES: [Dress; 6] = {
    use {SpinSprite as S, Suit as U};
    [
        Dress {
            spin: S::Plain,
            suit: U::First,
        },
        Dress {
            spin: S::SpaceJump,
            suit: U::First,
        },
        Dress {
            spin: S::ScrewAttack,
            suit: U::First,
        },
        Dress {
            spin: S::Plain,
            suit: U::VariaOrGravity,
        },
        Dress {
            spin: S::SpaceJump,
            suit: U::VariaOrGravity,
        },
        Dress {
            spin: S::ScrewAttack,
            suit: U::VariaOrGravity,
        },
    ]
};

/// Every dress the id is shown in.
fn shown_in(shown: Shown) -> impl Iterator<Item = Dress> {
    DRESSES.into_iter().filter(move |&dress| shown.holds(dress))
}

#[test]
fn the_ids_are_in_order_and_164_of_253_are_a_pose() {
    assert!(MAP.windows(2).all(|pair| pair[0].id.0 < pair[1].id.0));
    let mapped = (0..IDS).filter(|&id| pose_of(PoseId(id)).is_some()).count();
    assert_eq!((mapped, usize::from(IDS) - mapped), (164, 89));
    // Another domain's and the unused ids have no Pose.
    for id in [0x0D, 0x20, 0x4F, 0x53, 0xA8, 0xBA, 0xC9, 0xD5, 0xE8, 0xF0] {
        assert_eq!(pose_of(PoseId(id)), None, "{id:#04X}");
    }
}

#[test]
fn every_mapped_id_comes_back_from_its_pose_in_each_dress_it_is_shown_in() {
    for entry in MAP {
        assert_eq!(pose_of(entry.id), Some(entry.pose));
        for dress in shown_in(entry.shown) {
            assert_eq!(
                id_of(entry.pose, dress),
                Some(entry.id),
                "{:#04X}",
                entry.id.0
            );
        }
    }
}

#[test]
fn every_mapped_pose_is_shown_in_every_dress_by_an_id_of_that_pose() {
    for entry in MAP {
        for dress in DRESSES {
            let id = id_of(entry.pose, dress);
            assert_eq!(
                id.and_then(pose_of),
                Some(entry.pose),
                "{:#04X}",
                entry.id.0
            );
        }
    }
}

#[test]
fn the_equipment_draws_the_spin_and_the_suit_facing_the_screen() {
    let spin = pose_of(PoseId(0x19)).unwrap();
    let dressed = |spin_sprite| Dress {
        spin: spin_sprite,
        suit: Suit::First,
    };
    assert_eq!(id_of(spin, dressed(SpinSprite::Plain)), Some(PoseId(0x19)));
    assert_eq!(
        id_of(spin, dressed(SpinSprite::SpaceJump)),
        Some(PoseId(0x1B))
    );
    assert_eq!(
        id_of(spin, dressed(SpinSprite::ScrewAttack)),
        Some(PoseId(0x81))
    );
    let screen = pose_of(PoseId(0x00)).unwrap();
    let suited = Dress {
        spin: SpinSprite::Plain,
        suit: Suit::VariaOrGravity,
    };
    assert_eq!(id_of(screen, suited), Some(PoseId(0x9B)));
}

#[test]
fn the_new_parts_tell_apart_the_sprites_one_action_shares() {
    let pose = |action, aim| Pose {
        body: Body::Upright,
        action,
        facing: Facing::Right,
        aim,
        gun: Gun::Stowed,
        legs: Legs::Together,
    };
    let plain = Dress {
        spin: SpinSprite::Plain,
        suit: Suit::First,
    };
    let jump = pose(Action::Jumping, Aim::Forward);
    let firing = Pose {
        gun: Gun::Drawn,
        ..pose(Action::Landing(LandedFrom::Jump), Aim::Forward)
    };
    let striding = Pose {
        legs: Legs::Striding,
        ..jump
    };
    assert_eq!(id_of(jump, plain), Some(PoseId(0x4D)));
    assert_eq!(id_of(striding, plain), Some(PoseId(0x51)));
    assert_eq!(id_of(firing, plain), Some(PoseId(0xE6)));
    let spin_landing = pose(Action::Landing(LandedFrom::Spin), Aim::Forward);
    assert_eq!(id_of(spin_landing, plain), Some(PoseId(0xA6)));
    let moonwalk_turn = pose(Action::Turning(TurnKind::MoonwalkJump), Aim::Forward);
    assert_eq!(id_of(moonwalk_turn, plain), Some(PoseId(0xC0)));
}

#[test]
fn a_pose_the_spec_never_produces_is_shown_by_its_nearest_sprite() {
    let plain = Dress {
        spin: SpinSprite::Plain,
        suit: Suit::First,
    };
    let turn = |body, kind, aim| Pose {
        body,
        action: Action::Turning(kind),
        facing: Facing::Left,
        aim,
        gun: Gun::Stowed,
        legs: Legs::Together,
    };
    let air = turn(Body::Upright, TurnKind::Jump, Aim::DownForward);
    assert_eq!(id_of(air, plain), Some(PoseId(0x91)));
    let fall = turn(Body::Upright, TurnKind::Fall, Aim::DownForward);
    assert_eq!(id_of(fall, plain), Some(PoseId(0x95)));
    let crouched = turn(Body::Crouched, TurnKind::Ground, Aim::Down);
    assert_eq!(id_of(crouched, plain), Some(PoseId(0x99)));
    // No sprite aims a running pose down.
    let running = Pose {
        action: Action::Running,
        ..turn(Body::Upright, TurnKind::Ground, Aim::Down)
    };
    assert_eq!(id_of(running, plain), None);
}
