use super::key::{Action, Aim, Body, Facing, Gun, Legs, SpinSprite, Suit, TurnKind};
use super::*;

fn key(body: Body, action: Action, aim: Aim) -> PoseKey {
    PoseKey {
        body,
        action,
        facing: Facing::Right,
        aim,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

fn asset(key: PoseKey, worn: Worn) -> Clip {
    Clip {
        key,
        worn,
        frames: Vec::new(),
        end: ClipEnd::HandOver,
    }
}

fn assets(clips: Vec<Clip>) -> SamusAssets {
    SamusAssets {
        sheet: SpriteSheet::default(),
        clips,
        running: Vec::new(),
        chest: Chest {
            left: 0,
            up: 0,
            attributes: SpriteAttributes::from_word(0),
        },
    }
}

const PLAIN: Dress = Dress {
    spin: SpinSprite::Plain,
    suit: Suit::First,
};

#[test]
fn a_turn_without_a_sprite_of_its_own_shows_its_nearest() {
    let air = Action::Turning(TurnKind::Fall);
    let ground = Action::Turning(TurnKind::Ground);
    let assets = assets(vec![
        asset(key(Body::Upright, air, Aim::Down), Worn::Always),
        asset(key(Body::Crouched, ground, Aim::DownForward), Worn::Always),
    ]);
    let shown = |key| assets.clip(key, PLAIN).map(|asset| asset.key);
    assert_eq!(
        shown(key(Body::Upright, air, Aim::DownForward)),
        Some(key(Body::Upright, air, Aim::Down))
    );
    assert_eq!(
        shown(key(Body::Crouched, ground, Aim::Down)),
        Some(key(Body::Crouched, ground, Aim::DownForward))
    );
    assert_eq!(shown(key(Body::Upright, ground, Aim::Down)), None);
}

#[test]
fn the_dress_picks_between_the_sprites_of_one_pose() {
    let spin = key(Body::Upright, Action::Spinning, Aim::Forward);
    let assets = assets(vec![
        asset(spin, Worn::Spin(SpinSprite::Plain)),
        asset(spin, Worn::Spin(SpinSprite::ScrewAttack)),
    ]);
    let screw = Dress {
        spin: SpinSprite::ScrewAttack,
        ..PLAIN
    };
    let worn = |dress| assets.clip(spin, dress).map(|asset| asset.worn);
    assert_eq!(worn(PLAIN), Some(Worn::Spin(SpinSprite::Plain)));
    assert_eq!(worn(screw), Some(Worn::Spin(SpinSprite::ScrewAttack)));
}
