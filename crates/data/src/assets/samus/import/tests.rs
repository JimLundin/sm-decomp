use super::map::{MAP, id_of, key_of};
use super::*;
use crate::assets::samus::ClipEnd;
use crate::assets::samus::key::{
    Action, Aim, Body, Dress, Facing, Gun, LandedFrom, Legs, Pose, SpinSprite, Suit, Turn,
};
use crate::rom::Rom;

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

/// Every dress she can wear.
fn dresses() -> Vec<Dress> {
    [
        SpinSprite::Plain,
        SpinSprite::SpaceJump,
        SpinSprite::ScrewAttack,
    ]
    .into_iter()
    .flat_map(|spin| {
        [Suit::First, Suit::VariaOrGravity]
            .into_iter()
            .map(move |suit| Dress { spin, suit })
    })
    .collect()
}

#[test]
fn every_pose_of_hers_is_one_id_and_back_in_its_dress() {
    for entry in MAP {
        assert_eq!(key_of(entry.id), Some(entry.key), "{:?}", entry.id);
        for dress in dresses() {
            let id = id_of(entry.key, dress);
            assert!(id.is_some(), "{:?} in {dress:?}", entry.id);
            if entry.worn.holds(dress) {
                assert_eq!(id, Some(entry.id), "{dress:?}");
            }
        }
    }
    assert_eq!(key_of(PoseId(0xFD)), None);
}

#[test]
fn every_pose_of_hers_imports_with_its_frames() {
    let Some(cartridge) = rom() else {
        return;
    };
    let assets = import(cartridge.lorom()).unwrap();
    assert_eq!(assets.clips.len(), MAP.len());
    for (asset, entry) in assets.clips.iter().zip(MAP) {
        assert_eq!(asset.key, entry.key);
        assert!(!asset.frames.is_empty(), "{:?}", entry.id);
        assert!(
            asset.frames.iter().any(|frame| frame.top.is_some()),
            "{:?}",
            entry.id
        );
    }
    assert!(matches!(assets.running.first(), Some(Hold::Shows(_))));
    // A sprite list or an upload the frames share is kept once.
    let frames: usize = assets.clips.iter().map(|asset| asset.frames.len()).sum();
    assert!(frames > assets.sheet.sprites.len());
    assert!(frames > assets.sheet.uploads.len());
    // A loop plays on from one of its frames; a landing hands over to standing.
    for clip in &assets.clips {
        if let ClipEnd::Loop { from } = clip.end {
            assert!(usize::from(from.0) < clip.frames.len(), "{:?}", clip.key);
        }
    }
    let landing = MAP
        .iter()
        .position(|entry| entry.id == PoseId(0xA4))
        .unwrap();
    assert_eq!(assets.clips[landing].end, ClipEnd::HandOver);
}

/// The original's pose ids: `00h` to `FCh`.
const IDS: u16 = 0xFD;

const PLAIN: Dress = Dress {
    spin: SpinSprite::Plain,
    suit: Suit::First,
};

/// A pose facing `facing`, gun stowed and legs together.
const fn pose(body: Body, action: Action, facing: Facing, aim: Aim) -> Pose {
    Pose {
        body,
        action,
        facing,
        aim,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

#[test]
fn the_ids_are_in_order_and_164_of_253_are_hers() {
    assert!(MAP.windows(2).all(|pair| pair[0].id.0 < pair[1].id.0));
    let mapped = (0..IDS).filter(|&id| key_of(PoseId(id)).is_some()).count();
    assert_eq!((mapped, usize::from(IDS) - mapped), (164, 89));
    // Another domain's and the unused ids have no pose.
    for id in [0x0D, 0x20, 0x4F, 0x53, 0xA8, 0xBA, 0xC9, 0xD5, 0xE8, 0xF0] {
        assert_eq!(key_of(PoseId(id)), None, "{id:#04X}");
    }
}

#[test]
fn the_equipment_draws_the_spin_and_the_suit_facing_the_screen() {
    let spin = key_of(PoseId(0x19)).unwrap();
    let dressed = |spin| Dress {
        spin,
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
    let screen = key_of(PoseId(0x00)).unwrap();
    let suited = Dress {
        spin: SpinSprite::Plain,
        suit: Suit::VariaOrGravity,
    };
    assert_eq!(id_of(screen, suited), Some(PoseId(0x9B)));
}

#[test]
fn the_landing_turn_gun_and_legs_tell_apart_the_sprites_one_action_shares() {
    let jump = pose(Body::Upright, Action::Jumping, Facing::Right, Aim::Forward);
    let landing = |from| {
        pose(
            Body::Upright,
            Action::Landing(from),
            Facing::Right,
            Aim::Forward,
        )
    };
    let firing = Pose {
        gun: Gun::Drawn,
        ..landing(LandedFrom::Jump)
    };
    let striding = Pose {
        legs: Legs::Striding,
        ..jump
    };
    assert_eq!(id_of(jump, PLAIN), Some(PoseId(0x4D)));
    assert_eq!(id_of(striding, PLAIN), Some(PoseId(0x51)));
    assert_eq!(id_of(firing, PLAIN), Some(PoseId(0xE6)));
    assert_eq!(id_of(landing(LandedFrom::Spin), PLAIN), Some(PoseId(0xA6)));
    let moonwalk_turn = pose(
        Body::Upright,
        Action::Turning(Turn::MoonwalkJump),
        Facing::Right,
        Aim::Forward,
    );
    assert_eq!(id_of(moonwalk_turn, PLAIN), Some(PoseId(0xC0)));
}

#[test]
fn a_pose_no_sprite_of_its_own_shows_is_shown_by_its_nearest() {
    let turn = |body, kind, aim| pose(body, Action::Turning(kind), Facing::Left, aim);
    let air = turn(Body::Upright, Turn::Jump, Aim::DownForward);
    assert_eq!(id_of(air, PLAIN), Some(PoseId(0x91)));
    let fall = turn(Body::Upright, Turn::Fall, Aim::DownForward);
    assert_eq!(id_of(fall, PLAIN), Some(PoseId(0x95)));
    let crouched = turn(Body::Crouched, Turn::Ground, Aim::Down);
    assert_eq!(id_of(crouched, PLAIN), Some(PoseId(0x99)));
    // No sprite aims a running pose down.
    let running = pose(Body::Upright, Action::Running, Facing::Left, Aim::Down);
    assert_eq!(id_of(running, PLAIN), None);
}
