//! Projectile launch records, animation and trail markers on the owned Session.
#![forbid(unsafe_code)]
use super::{Host, content};
use crate::projectile_initialization::Initialization;
use crate::shot;

pub fn table(kind: Initialization, flags: u16) -> (u32, isize) {
    match kind {
        Initialization::SuperMissile => (0x93842b, ((flags >> 8) & 15) as isize),
        Initialization::Bomb => (0x9383f1, ((flags >> 8) & 15) as isize),
        Initialization::SpecialAttack => (0x938413, (flags & 15) as isize),
        Initialization::Echo => (0x938403, isize::from(flags.to_le_bytes()[0]) - 34),
        Initialization::Projectile if flags & super::kinds::TYPE_MASK != 0 => {
            (0x9383f1, isize::from(flags.to_ne_bytes()[1] & 15))
        }
        Initialization::Projectile => (
            if flags & 0x10 != 0 {
                0x9383d9
            } else {
                0x9383c1
            },
            (flags & 15) as isize,
        ),
    }
}
pub fn initialize(h: &mut impl Host, slot: u16, kind: Initialization) {
    let direction = matches!(kind, Initialization::Projectile | Initialization::Echo)
        .then(|| usize::from(shot!(h, slot, direction) & 15));
    let (address, index) = table(kind, shot!(h, slot, kind));
    let pointer = content::table(h, address, index);
    let launch = h.record(0x93, pointer);
    let damage = launch.word(0);
    if kind != Initialization::Projectile {
        shot!(h, slot, damage = damage);
    }
    let checked = if kind == Initialization::Echo {
        launch.word(0)
    } else {
        damage
    };
    if checked & 0x8000 != 0 {
        h.invalid_damage();
    }
    if kind == Initialization::Projectile {
        shot!(h, slot, damage = launch.word(0));
    }
    let field = direction.map_or(2, |direction| {
        [2, 4, 6, 8, 10, 12, 14, 16, 18, 20][direction]
    });
    let cursor = launch.word(field);
    shot!(h, slot, cursor = cursor);
    if kind == Initialization::Projectile {
        let frame = h.record(0x93, cursor);
        shot!(h, slot, x_radius = u16::from(frame.byte(4)));
        shot!(h, slot, y_radius = u16::from(frame.byte(5)));
    }
    shot!(h, slot, remaining = 1);
}

pub fn advance(h: &mut impl Host) {
    let selected = h.session().projectiles.selected;
    let prior = shot!(h, selected, remaining);
    shot!(h, selected, remaining = prior.wrapping_sub(1));
    if prior != 1 {
        return;
    }
    let mut cursor = shot!(h, selected, cursor);
    loop {
        let frame = h.record(0x93, cursor);
        match frame.word(0) {
            0x822f => {
                super::motion::clear(h, selected);
                return;
            }
            0x8239 => {
                let target = h.record(0x93, cursor.wrapping_add(2)).word(0);
                if target == 0 {
                    return;
                }
                cursor = target;
            }
            duration if duration & 0x8000 == 0 => {
                // Keep this captured record and publish each field before the next read.
                shot!(h, selected, remaining = duration);
                shot!(h, selected, sprite = frame.word(2));
                shot!(h, selected, x_radius = u16::from(frame.byte(4)));
                shot!(h, selected, y_radius = u16::from(frame.byte(5)));
                shot!(h, selected, cursor = cursor.wrapping_add(8));
                h.probe("player_projectiles.playback.frame");
                return;
            }
            _ => {
                if !h.unknown() {
                    return;
                }
                cursor = 1;
            }
        }
    }
}
pub fn trail_frame(h: &mut impl Host, slot: u16) -> u16 {
    let cursor = shot!(h, slot, cursor);
    let upcoming = shot!(h, slot, remaining) == 1 && h.record(0x93, cursor).word(0) & 0x8000 == 0;
    let cursor = if upcoming {
        cursor
    } else {
        cursor.wrapping_sub(8)
    };
    // A diagnostic on the first lookup can replace the loaded allocation.
    h.record(0x93, cursor).word(6)
}
